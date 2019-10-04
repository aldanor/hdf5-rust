use std::borrow::Cow;
use std::fmt::{self, Display};
use std::mem;
use std::ops::{Deref, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::slice;

use ndarray::{self, s, Array1, Array2, ArrayView1, ArrayView2};

use hdf5_sys::h5s::{
    H5S_sel_type, H5Sget_select_elem_npoints, H5Sget_select_elem_pointlist, H5Sget_select_type,
    H5Sget_simple_extent_ndims, H5Sselect_all, H5Sselect_elements, H5Sselect_hyperslab,
    H5Sselect_none, H5S_SELECT_SET, H5S_UNLIMITED,
};
#[cfg(hdf5_1_10_0)]
use hdf5_sys::h5s::{H5Sget_regular_hyperslab, H5Sis_regular_hyperslab};
#[cfg(not(hdf5_1_10_0))]
use hdf5_sys::h5s::{H5Sget_select_hyper_blocklist, H5Sget_select_hyper_nblocks};

use crate::hl::extents::Ix;
use crate::internal_prelude::*;

unsafe fn get_points_selection(space_id: hid_t) -> Result<Array2<Ix>> {
    let npoints = h5check(H5Sget_select_elem_npoints(space_id))? as usize;
    let ndim = h5check(H5Sget_simple_extent_ndims(space_id))? as usize;
    let mut coords = vec![0; npoints * ndim];
    h5check(H5Sget_select_elem_pointlist(space_id, 0, npoints as _, coords.as_mut_ptr()))?;
    let coords = if mem::size_of::<hsize_t>() == mem::size_of::<Ix>() {
        mem::transmute(coords)
    } else {
        coords.iter().map(|&x| x as _).collect()
    };
    Ok(Array2::from_shape_vec_unchecked((npoints, ndim), coords))
}

unsafe fn set_points_selection(space_id: hid_t, coords: ArrayView2<Ix>) -> Result<()> {
    let nelem = coords.shape()[0] as _;
    let same_size = mem::size_of::<hsize_t>() == mem::size_of::<Ix>();
    let coords = match (coords.as_slice(), same_size) {
        (Some(coords), true) => {
            Cow::Borrowed(slice::from_raw_parts(coords.as_ptr() as *const _, coords.len()))
        }
        _ => Cow::Owned(coords.iter().map(|&x| x as _).collect()),
    };
    h5check(H5Sselect_elements(space_id, H5S_SELECT_SET, nelem, coords.as_ptr()))?;
    Ok(())
}

unsafe fn get_regular_hyperslab(space_id: hid_t) -> Result<Option<RawHyperslab>> {
    #[cfg(hdf5_1_10_0)]
    {
        if h5check(H5Sis_regular_hyperslab(space_id))? <= 0 {
            return Ok(None);
        }
        let ndim = h5check(H5Sget_simple_extent_ndims(space_id))? as usize;
        let (mut start, mut stride, mut count, mut block) =
            (vec![0; ndim], vec![0; ndim], vec![0; ndim], vec![0; ndim]);
        h5check(H5Sget_regular_hyperslab(
            space_id,
            start.as_mut_ptr(),
            stride.as_mut_ptr(),
            count.as_mut_ptr(),
            block.as_mut_ptr(),
        ))?;
        let mut hyper = vec![];
        for i in 0..ndim {
            hyper.push(RawSlice {
                start: start[i] as _,
                step: stride[i] as _,
                count: if count[i] == H5S_UNLIMITED { None } else { Some(count[i] as _) },
                block: block[i] as _,
            });
        }
        return Ok(Some(hyper.into()));
    }
    #[allow(unreachable_code)]
    Ok(None)
}

unsafe fn set_regular_hyperslab(space_id: hid_t, hyper: &RawHyperslab) -> Result<()> {
    let (mut start, mut stride, mut count, mut block) = (vec![], vec![], vec![], vec![]);
    for slice_info in hyper.iter() {
        start.push(slice_info.start as _);
        stride.push(slice_info.step as _);
        count.push(slice_info.count.map_or(H5S_UNLIMITED, |x| x as _));
        block.push(slice_info.block as _);
    }
    h5check(H5Sselect_hyperslab(
        space_id,
        H5S_SELECT_SET,
        start.as_ptr(),
        stride.as_ptr(),
        count.as_ptr(),
        block.as_ptr(),
    ))?;
    Ok(())
}

fn check_coords(coords: &Array2<Ix>, shape: &[Ix]) -> Result<()> {
    if coords.shape() == &[0, 0] {
        return Ok(());
    }
    let ndim = coords.shape()[1];
    ensure!(ndim == shape.len(), "Slice ndim ({}) != shape ndim ({})", ndim, shape.len());
    for i in 0..ndim {
        let dim = shape[i];
        for &d in coords.slice(s![.., i]).iter() {
            ensure!(d < dim, "Index {} out of bounds for axis {} with size {}", d, i, dim);
        }
    }
    Ok(())
}

#[inline]
fn abs_index(len: usize, index: isize) -> isize {
    if index < 0 {
        (len as isize) + index
    } else {
        index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawSlice {
    pub start: Ix,
    pub step: Ix,
    pub count: Option<Ix>,
    pub block: Ix,
}

impl RawSlice {
    pub fn new(start: Ix, step: Ix, count: Option<Ix>, block: Ix) -> Self {
        Self { start, step, count, block }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawHyperslab {
    dims: Vec<RawSlice>,
}

impl Deref for RawHyperslab {
    type Target = [RawSlice];

    fn deref(&self) -> &Self::Target {
        &self.dims
    }
}

impl RawHyperslab {
    fn is_none(&self) -> bool {
        self.iter().any(|s| s.count == Some(0))
    }

    fn is_all(&self, shape: &[Ix]) -> bool {
        if self.is_empty() {
            return true;
        }
        for (slice, &dim) in self.iter().zip(shape) {
            let count = match slice.count {
                Some(count) => count,
                None => return false,
            };
            if slice.start != 0 || slice.step != slice.block || count * slice.block != dim {
                return false;
            }
        }
        true
    }
}

impl From<Vec<RawSlice>> for RawHyperslab {
    fn from(dims: Vec<RawSlice>) -> Self {
        Self { dims }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RawSelection {
    None,
    All,
    Points(Array2<Ix>),
    RegularHyperslab(RawHyperslab),
    ComplexHyperslab,
}

impl Default for RawSelection {
    fn default() -> Self {
        RawSelection::All
    }
}

impl From<RawHyperslab> for RawSelection {
    fn from(hyper: RawHyperslab) -> Self {
        RawSelection::RegularHyperslab(hyper)
    }
}

impl From<Vec<RawSlice>> for RawSelection {
    fn from(dims: Vec<RawSlice>) -> Self {
        RawSelection::RegularHyperslab(dims.into())
    }
}

impl RawSelection {
    pub unsafe fn apply_to_dataspace(&self, space_id: hid_t) -> Result<()> {
        match self {
            RawSelection::None => drop(h5check(H5Sselect_none(space_id))?),
            RawSelection::All => drop(h5check(H5Sselect_all(space_id))?),
            RawSelection::Points(ref coords) => set_points_selection(space_id, coords.view())?,
            RawSelection::RegularHyperslab(ref hyper) => set_regular_hyperslab(space_id, hyper)?,
            RawSelection::ComplexHyperslab => fail!("Complex hyperslabs are not supported"),
        };
        Ok(())
    }

    pub unsafe fn extract_from_dataspace(space_id: hid_t) -> Result<Self> {
        Ok(match H5Sget_select_type(space_id) {
            H5S_sel_type::H5S_SEL_NONE => RawSelection::None,
            H5S_sel_type::H5S_SEL_ALL => RawSelection::All,
            H5S_sel_type::H5S_SEL_POINTS => RawSelection::Points(get_points_selection(space_id)?),
            H5S_sel_type::H5S_SEL_HYPERSLABS => get_regular_hyperslab(space_id)?
                .map_or(RawSelection::ComplexHyperslab, RawSelection::RegularHyperslab),
            sel_type => fail!("Invalid selection type: {:?}", sel_type as c_int),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceOrIndex {
    Index(isize),
    Slice { start: isize, step: isize, end: Option<isize>, block: bool },
    Unlimited { start: isize, step: isize, block: bool },
}

impl SliceOrIndex {
    pub fn to_unlimited(self) -> Result<Self> {
        Ok(match self {
            SliceOrIndex::Index(_) => fail!("Cannot make index selection unlimited"),
            SliceOrIndex::Slice { end: Some(_), .. } => {
                fail!("Cannot make bounded slice unlimited")
            }
            SliceOrIndex::Slice { start, step, end: None, block } => {
                SliceOrIndex::Unlimited { start, step, block }
            }
            SliceOrIndex::Unlimited { .. } => self,
        })
    }

    pub fn to_block(self) -> Result<Self> {
        Ok(match self {
            SliceOrIndex::Index(_) => fail!("Cannot make index selection block-like"),
            SliceOrIndex::Slice { start, step, end, .. } => {
                SliceOrIndex::Slice { start, step, end, block: true }
            }
            SliceOrIndex::Unlimited { start, step, .. } => {
                SliceOrIndex::Unlimited { start, step, block: true }
            }
        })
    }

    pub fn is_index(self) -> bool {
        if let SliceOrIndex::Index(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_slice(self) -> bool {
        if let SliceOrIndex::Slice { .. } = self {
            true
        } else {
            false
        }
    }

    pub fn is_unlimited(self) -> bool {
        if let SliceOrIndex::Unlimited { .. } = self {
            true
        } else {
            false
        }
    }
}

impl<T: Into<ndarray::SliceOrIndex>> From<T> for SliceOrIndex {
    fn from(slice: T) -> Self {
        match slice.into() {
            ndarray::SliceOrIndex::Index(index) => SliceOrIndex::Index(index),
            ndarray::SliceOrIndex::Slice { start, end, step } => {
                SliceOrIndex::Slice { start, step, end, block: false }
            }
        }
    }
}

impl Display for SliceOrIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SliceOrIndex::Index(index) => write!(f, "{}", index)?,
            SliceOrIndex::Slice { start, end, step, block } => {
                if start != 0 {
                    write!(f, "{}", start)?;
                }
                write!(f, "..")?;
                if let Some(end) = end {
                    write!(f, "{}", end)?;
                }
                if step != 1 {
                    write!(f, ";{}", step)?;
                }
                if block {
                    write!(f, "(B)")?;
                }
            }
            SliceOrIndex::Unlimited { start, step, block } => {
                if start != 0 {
                    write!(f, "{}", start)?;
                }
                write!(f, "..∞")?;
                if step != 1 {
                    write!(f, ";{}", step)?;
                }
                if block {
                    write!(f, "(B)")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hyperslab {
    dims: Vec<SliceOrIndex>,
}

impl Hyperslab {
    pub fn new<T: Into<Hyperslab>>(hyper: T) -> Self {
        hyper.into()
    }

    pub fn is_unlimited(&self) -> bool {
        self.iter().any(|&s| s.is_unlimited())
    }

    pub fn unlimited_axis(&self) -> Option<usize> {
        self.iter().enumerate().skip_while(|(_, s)| !s.is_unlimited()).next().map(|(i, _)| i)
    }

    pub fn set_unlimited(&self, axis: usize) -> Result<Self> {
        let unlim = self.unlimited_axis();
        if unlim.is_some() && unlim != Some(axis) {
            fail!("The hyperslab already has one unlimited axis");
        } else if axis < self.len() {
            let mut hyper = self.clone();
            hyper.dims[axis] = hyper.dims[axis].to_unlimited()?;
            Ok(hyper)
        } else {
            fail!("Invalid axis for making hyperslab unlimited: {}", axis);
        }
    }

    pub fn set_block(&self, axis: usize) -> Result<Self> {
        if axis < self.len() {
            let mut hyper = self.clone();
            hyper.dims[axis] = hyper.dims[axis].to_block()?;
            Ok(hyper)
        } else {
            fail!("Invalid axis for changing the slice to block-like: {}", axis);
        }
    }

    #[doc(hidden)]
    pub fn into_raw<S: AsRef<[Ix]>>(self, shape: S) -> Result<RawHyperslab> {
        let shape = shape.as_ref();
        let ndim = shape.len();
        ensure!(self.len() == ndim, "Slice ndim ({}) != shape ndim ({})", self.len(), ndim);
        let n_unlimited: usize = self.iter().map(|s| s.is_unlimited() as usize).sum();
        ensure!(n_unlimited <= 1, "Expected at most 1 unlimited dimension, got {}", n_unlimited);
        let hyper = RawHyperslab::from(
            self.iter()
                .zip(shape)
                .enumerate()
                .map(|(i, (slice, &dim))| slice_info_to_raw(i, slice, dim))
                .collect::<Result<Vec<_>>>()?,
        );
        Ok(hyper)
    }

    #[doc(hidden)]
    pub fn from_raw(hyper: RawHyperslab) -> Result<Self> {
        let mut dims = vec![];
        for (i, slice) in hyper.iter().enumerate() {
            let block = if slice.block == 1 {
                false
            } else if slice.block == slice.step {
                true
            } else {
                fail!("Unsupported block/step for axis {}: {}/{}", i, slice.block, slice.step);
            };
            dims.push(match slice.count {
                Some(count) => SliceOrIndex::Slice {
                    start: slice.start as _,
                    step: slice.step as _,
                    end: Some(
                        (slice.start
                            + if count == 0 { 0 } else { (count - 1) * slice.step + slice.block })
                            as _,
                    ),
                    block,
                },
                None => SliceOrIndex::Unlimited {
                    start: slice.start as _,
                    step: slice.step as _,
                    block,
                },
            });
        }
        Ok(Self { dims })
    }
}

impl Deref for Hyperslab {
    type Target = [SliceOrIndex];

    fn deref(&self) -> &Self::Target {
        &self.dims
    }
}

impl From<Vec<SliceOrIndex>> for Hyperslab {
    fn from(dims: Vec<SliceOrIndex>) -> Self {
        Self { dims }
    }
}

impl From<()> for Hyperslab {
    fn from(_: ()) -> Self {
        vec![].into()
    }
}

impl From<RangeFull> for Hyperslab {
    fn from(_: RangeFull) -> Self {
        (0..).into()
    }
}

impl From<SliceOrIndex> for Hyperslab {
    fn from(slice: SliceOrIndex) -> Self {
        vec![slice].into()
    }
}

impl<T, D> From<&ndarray::SliceInfo<T, D>> for Hyperslab
where
    T: AsRef<[ndarray::SliceOrIndex]>,
    D: ndarray::Dimension,
{
    fn from(slice: &ndarray::SliceInfo<T, D>) -> Self {
        slice.deref().as_ref().iter().cloned().map(Into::into).collect::<Vec<_>>().into()
    }
}

fn slice_info_to_raw(axis: usize, slice: &SliceOrIndex, dim: Ix) -> Result<RawSlice> {
    let err_msg = || format!("out of bounds for axis {} with size {}", axis, dim);
    let (start, step, count, block) = match slice {
        &SliceOrIndex::Index(index) => {
            let idx = abs_index(dim, index);
            ensure!(idx >= 0 && idx < dim as _, "Index {} {}", index, err_msg());
            (idx as _, 1, Some(1), 1)
        }
        &SliceOrIndex::Slice { start, step, end, block } => {
            ensure!(step >= 1, "Slice stride {} < 1 for axis {}", step, axis);
            let s = abs_index(dim, start);
            ensure!(s >= 0 && s <= dim as _, "Slice start {} {}", start, err_msg());
            let end = end.unwrap_or(dim as _);
            let e = abs_index(dim, end);
            ensure!(e >= 0 && e <= dim as _, "Slice end {} {}", end, err_msg());
            let block = if block { step } else { 1 };
            let count = if e < s + block { 0 } else { 1 + (e - s - block) / step };
            (s as _, step as _, Some(count as _), block as _)
        }
        &SliceOrIndex::Unlimited { start, step, block } => {
            ensure!(step >= 1, "Slice stride {} < 1 for axis {}", step, axis);
            let s = abs_index(dim, start);
            ensure!(s >= 0 && s <= dim as _, "Slice start {} {}", start, err_msg());
            let block = if block { step } else { 1 };
            (s as _, step as _, None, block as _)
        }
    };
    Ok(RawSlice { start, step, count, block })
}

impl Display for Hyperslab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice: &[_] = self.as_ref();
        write!(f, "(")?;
        for i in 0..slice.len() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", slice[i])?;
        }
        if slice.len() == 1 {
            write!(f, ",")?;
        }
        write!(f, ")")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    All,
    Points(Array2<Ix>),
    Hyperslab(Hyperslab),
}

impl Default for Selection {
    fn default() -> Self {
        Selection::All
    }
}

impl Selection {
    pub fn new<T: Into<Self>>(selection: T) -> Self {
        selection.into()
    }

    #[doc(hidden)]
    pub fn into_raw<S: AsRef<[Ix]>>(self, shape: S) -> Result<RawSelection> {
        let shape = shape.as_ref();
        Ok(match self {
            Selection::All => RawSelection::All,
            Selection::Points(coords) => {
                check_coords(&coords, shape)?;
                if coords.shape()[0] == 0 {
                    RawSelection::None
                } else {
                    RawSelection::Points(coords)
                }
            }
            Selection::Hyperslab(hyper) => {
                let hyper = hyper.into_raw(shape)?;
                if hyper.is_none() {
                    RawSelection::None
                } else if hyper.is_all(shape) {
                    RawSelection::All
                } else {
                    RawSelection::RegularHyperslab(hyper)
                }
            }
        })
    }

    #[doc(hidden)]
    pub fn from_raw(selection: RawSelection) -> Result<Self> {
        Ok(match selection {
            RawSelection::None => Selection::Points(Array2::default((0, 0))),
            RawSelection::All => Selection::All,
            RawSelection::Points(coords) => Selection::Points(coords),
            RawSelection::RegularHyperslab(hyper) => Hyperslab::from_raw(hyper)?.into(),
            RawSelection::ComplexHyperslab => fail!("Cannot convert complex hyperslabs"),
        })
    }

    pub fn in_ndim(&self) -> Option<usize> {
        match self {
            Selection::All => None,
            Selection::Points(ref points) => {
                if points.shape() == &[0, 0] {
                    None
                } else {
                    Some(points.shape()[1])
                }
            }
            Selection::Hyperslab(ref hyper) => Some(hyper.len()),
        }
    }

    pub fn out_ndim(&self) -> Option<usize> {
        match self {
            Selection::All => None,
            Selection::Points(ref points) => Some((points.shape() != &[0, 0]) as usize),
            Selection::Hyperslab(ref hyper) => {
                Some(hyper.iter().map(|&s| s.is_slice() as usize).sum())
            }
        }
    }

    pub fn out_shape<S: AsRef<[Ix]>>(&self, in_shape: S) -> Result<Vec<Ix>> {
        let in_shape = in_shape.as_ref();
        match self {
            Selection::All => Ok(in_shape.to_owned()),
            Selection::Points(ref points) => check_coords(points, in_shape)
                .and(Ok(if points.shape() == &[0, 0] { vec![] } else { vec![points.shape()[0]] })),
            Selection::Hyperslab(ref hyper) => hyper
                .clone()
                .into_raw(in_shape)?
                .iter()
                .zip(hyper.iter())
                .filter_map(|(&r, &s)| match (r.count, s.is_index()) {
                    (Some(_), true) => None,
                    (Some(count), false) => Some(Ok(count * r.block)),
                    (None, _) => {
                        Some(Err("Unable to get the shape for unlimited hyperslab".into()))
                    }
                })
                .collect(),
        }
    }

    pub fn is_all(&self) -> bool {
        self == &Selection::All
    }

    pub fn is_points(&self) -> bool {
        if let &Selection::Points(ref points) = self {
            points.shape() != &[0, 0]
        } else {
            false
        }
    }

    pub fn is_none(&self) -> bool {
        if let &Selection::Points(ref points) = self {
            points.shape() == &[0, 0]
        } else {
            false
        }
    }

    pub fn is_hyperslab(&self) -> bool {
        if let &Selection::Hyperslab(_) = self {
            true
        } else {
            false
        }
    }
}

impl Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Selection::All => write!(f, ".."),
            Selection::Points(ref points) => {
                if points.shape() == &[0, 0] {
                    write!(f, "[]")
                } else {
                    write!(f, "{}", points)
                }
            }
            Selection::Hyperslab(hyper) => write!(f, "{}", hyper),
        }
    }
}

impl From<&Selection> for Selection {
    fn from(sel: &Selection) -> Self {
        sel.clone()
    }
}

impl From<RangeFull> for Selection {
    fn from(_: RangeFull) -> Self {
        Selection::All
    }
}

impl From<()> for Selection {
    fn from(_: ()) -> Self {
        Hyperslab::from(()).into()
    }
}

impl From<SliceOrIndex> for Selection {
    fn from(slice: SliceOrIndex) -> Self {
        Selection::Hyperslab(slice.into())
    }
}

impl From<Hyperslab> for Selection {
    fn from(hyper: Hyperslab) -> Self {
        Selection::Hyperslab(hyper)
    }
}

impl<T, D> From<&ndarray::SliceInfo<T, D>> for Selection
where
    T: AsRef<[ndarray::SliceOrIndex]>,
    D: ndarray::Dimension,
{
    fn from(slice: &ndarray::SliceInfo<T, D>) -> Self {
        Hyperslab::from(slice).into()
    }
}

impl From<Array2<Ix>> for Selection {
    fn from(points: Array2<Ix>) -> Self {
        Selection::Points(points)
    }
}

impl From<Array1<Ix>> for Selection {
    fn from(points: Array1<Ix>) -> Self {
        let n = points.len();
        Selection::Points(if n == 0 {
            Array2::zeros((0, 0))
        } else {
            points.into_shape((n, 1)).unwrap().into_dimensionality().unwrap()
        })
    }
}

impl From<ArrayView2<'_, Ix>> for Selection {
    fn from(points: ArrayView2<'_, Ix>) -> Self {
        points.to_owned().into()
    }
}

impl From<ArrayView1<'_, Ix>> for Selection {
    fn from(points: ArrayView1<'_, Ix>) -> Self {
        points.to_owned().into()
    }
}

impl From<&Array2<Ix>> for Selection {
    fn from(points: &Array2<Ix>) -> Self {
        points.clone().into()
    }
}

impl From<&Array1<Ix>> for Selection {
    fn from(points: &Array1<Ix>) -> Self {
        points.clone().into()
    }
}

impl From<Vec<Ix>> for Selection {
    fn from(points: Vec<Ix>) -> Self {
        Array1::from(points).into()
    }
}

impl From<&[Ix]> for Selection {
    fn from(points: &[Ix]) -> Self {
        ArrayView1::from(points).into()
    }
}

macro_rules! impl_fixed {
    () => ();

    ($head:expr, $($tail:expr,)*) => (
        impl From<[Ix; $head]> for Selection {
            fn from(points: [Ix; $head]) -> Self {
                points.as_ref().into()
            }
        }

        impl From<&[Ix; $head]> for Selection {
            fn from(points: &[Ix; $head]) -> Self {
                points.as_ref().into()
            }
        }

        impl_fixed! { $($tail,)* }
    )
}

impl_fixed! { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, }

macro_rules! impl_tuple {
    () => ();

    ($head:ident, $($tail:ident,)*) => (
        #[allow(non_snake_case)]
        impl<$head, $($tail,)*> From<($head, $($tail,)*)> for Hyperslab
            where $head: Into<SliceOrIndex>, $($tail: Into<SliceOrIndex>,)*
        {
            fn from(slice: ($head, $($tail,)*)) -> Self {
                let ($head, $($tail,)*) = slice;
                vec![($head).into(), $(($tail).into(),)*].into()
            }
        }

        #[allow(non_snake_case)]
        impl<$head, $($tail,)*> From<($head, $($tail,)*)> for Selection
            where $head: Into<SliceOrIndex>, $($tail: Into<SliceOrIndex>,)*
        {
            fn from(slice: ($head, $($tail,)*)) -> Self {
                Hyperslab::from(slice).into()
            }
        }

        impl_tuple! { $($tail,)* }
    )
}

impl_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

macro_rules! impl_slice_scalar {
    ($tp:ty) => {
        impl From<$tp> for Hyperslab {
            fn from(slice: $tp) -> Self {
                (slice,).into()
            }
        }

        impl From<$tp> for Selection {
            fn from(slice: $tp) -> Self {
                Hyperslab::from(slice).into()
            }
        }
    };
}

impl_slice_scalar!(isize);
impl_slice_scalar!(usize);
impl_slice_scalar!(i32);
impl_slice_scalar!(ndarray::Slice);
impl_slice_scalar!(ndarray::SliceOrIndex);

macro_rules! impl_range_scalar {
    ($index:ty) => {
        impl_slice_scalar!(Range<$index>);
        impl_slice_scalar!(RangeFrom<$index>);
        impl_slice_scalar!(RangeInclusive<$index>);
        impl_slice_scalar!(RangeTo<$index>);
        impl_slice_scalar!(RangeToInclusive<$index>);
    };
}

impl_range_scalar!(isize);
impl_range_scalar!(usize);
impl_range_scalar!(i32);

#[cfg(test)]
mod test {
    use ndarray::{arr1, arr2, s, Array2};
    use pretty_assertions::assert_eq;

    use super::{
        Hyperslab, RawHyperslab, RawSelection, RawSlice, Selection, SliceOrIndex, SliceOrIndex::*,
    };
    use crate::internal_prelude::*;

    #[test]
    fn test_slice_or_index_impl() -> Result<()> {
        let s = SliceOrIndex::from(2);
        assert_eq!(s, Index(2));
        assert!(s.is_index());
        assert!(!s.is_slice());
        assert!(!s.is_unlimited());
        assert_err!(s.to_unlimited(), "Cannot make index selection unlimited");
        assert_err!(s.to_block(), "Cannot make index selection block-like");

        let s = SliceOrIndex::from(2..=5);
        assert_eq!(s, Slice { start: 2, step: 1, end: Some(6), block: false });
        assert!(!s.is_index());
        assert!(s.is_slice());
        assert!(!s.is_unlimited());
        assert_err!(s.to_unlimited(), "Cannot make bounded slice unlimited");
        assert_eq!(s.to_block()?, Slice { start: 2, step: 1, end: Some(6), block: true });
        assert_eq!(
            SliceOrIndex::from(*s![1..2;3].get(0).unwrap()),
            Slice { start: 1, step: 3, end: Some(2), block: false }
        );

        let s = SliceOrIndex::from(3..).to_unlimited()?;
        assert_eq!(s, Unlimited { start: 3, step: 1, block: false });
        assert!(!s.is_index());
        assert!(!s.is_slice());
        assert!(s.is_unlimited());
        assert_eq!(s.to_unlimited()?, s);
        assert_eq!(s.to_block()?, Unlimited { start: 3, step: 1, block: true });

        for (s, f) in &[
            (Index(-1), "-1"),
            (Slice { start: 0, step: 1, end: None, block: false }, ".."),
            (Slice { start: 0, step: 1, end: None, block: true }, "..(B)"),
            (Slice { start: -1, step: 3, end: None, block: false }, "-1..;3"),
            (Slice { start: -1, step: 1, end: None, block: true }, "-1..(B)"),
            (Slice { start: 0, step: 1, end: Some(5), block: false }, "..5"),
            (Slice { start: 0, step: 3, end: Some(5), block: true }, "..5;3(B)"),
            (Slice { start: -1, step: 1, end: Some(5), block: false }, "-1..5"),
            (Slice { start: -1, step: 1, end: Some(5), block: true }, "-1..5(B)"),
            (Unlimited { start: 0, step: 1, block: false }, "..∞"),
            (Unlimited { start: 0, step: 3, block: true }, "..∞;3(B)"),
            (Unlimited { start: -1, step: 1, block: false }, "-1..∞"),
            (Unlimited { start: -1, step: 3, block: true }, "-1..∞;3(B)"),
        ] {
            assert_eq!(format!("{}", s), f.to_owned());
        }

        Ok(())
    }

    #[test]
    fn test_selection_hyperslab_new() {
        macro_rules! check {
            ($hs1:expr, $hs2:expr) => {
                assert_eq!(Hyperslab::new($hs1).as_ref().to_owned(), $hs2);
                let s = Selection::new($hs1);
                assert_eq!(s, Selection::new(Hyperslab::new($hs2)));
                assert_eq!(s, Selection::Hyperslab(Hyperslab::new($hs2)));
            };
        }

        check!((), vec![]);
        check!(Index(2), vec![Index(2)]);
        check!(
            s![-1, 2..;3, ..4],
            vec![
                Index(-1),
                Slice { start: 2, step: 3, end: None, block: false },
                Slice { start: 0, step: 1, end: Some(4), block: false },
            ]
        );

        check!(
            ndarray::Slice::new(-1, None, 2),
            vec![Slice { start: -1, step: 2, end: None, block: false }]
        );
        check!(ndarray::SliceOrIndex::Index(10), vec![Index(10)]);

        check!(-1, vec![Index(-1)]);
        check!(-1..2, vec![Slice { start: -1, step: 1, end: Some(2), block: false }]);
        check!(-1..=2, vec![Slice { start: -1, step: 1, end: Some(3), block: false }]);
        check!(3.., vec![Slice { start: 3, step: 1, end: None, block: false }]);
        check!(..-1, vec![Slice { start: 0, step: 1, end: Some(-1), block: false }]);
        check!(..=-1, vec![Slice { start: 0, step: 1, end: None, block: false }]);

        check!(
            (-1..2, Index(-1)),
            vec![Slice { start: -1, step: 1, end: Some(2), block: false }, Index(-1)]
        );

        assert_eq!(
            Hyperslab::new(..).as_ref().to_owned(),
            vec![Slice { start: 0, step: 1, end: None, block: false }]
        );
        assert_eq!(Selection::new(..), Selection::All);
    }

    #[test]
    fn test_selection_points_new() {
        macro_rules! check {
            ($e:expr, $p:expr) => {
                let s = Selection::from($e);
                assert_eq!(s, Selection::Points($p.clone()));
            };
        }

        let a2 = arr2(&[[1, 2], [3, 4]]);
        check!(a2.clone(), &a2);
        check!(&a2, &a2);
        check!(a2.view(), &a2);
        let a1 = arr1(&[1, 2, 3]);
        let a1_2 = arr2(&[[1], [2], [3]]);
        check!(a1.clone(), &a1_2);
        check!(&a1, &a1_2);
        check!(a1.view(), &a1_2);
        check!(a1.as_slice().unwrap(), &a1_2);
        check!(a1.to_vec(), &a1_2);
        check!([1, 2, 3], &a1_2);
        check!(&[1, 2, 3], &a1_2);

        let s = Selection::new(&[]);
        assert_eq!(s, Selection::Points(Array2::zeros((0, 0))));
    }

    #[test]
    fn test_hyperslab_impl() -> Result<()> {
        let h = Hyperslab::new(s![0, 1..10, 2..;3]);
        assert_eq!(
            h.as_ref().to_owned(),
            vec![
                Index(0),
                Slice { start: 1, step: 1, end: Some(10), block: false },
                Slice { start: 2, step: 3, end: None, block: false },
            ]
        );
        assert!(!h.is_unlimited());
        assert!(h.unlimited_axis().is_none());

        assert_err!(h.set_unlimited(0), "Cannot make index selection unlimited");
        assert_err!(h.set_unlimited(1), "Cannot make bounded slice unlimited");
        assert_err!(h.set_unlimited(3), "Invalid axis for making hyperslab unlimited: 3");
        let u = h.set_unlimited(2)?;
        assert!(u.is_unlimited());
        assert_eq!(u.unlimited_axis(), Some(2));
        assert_eq!(
            u.as_ref().to_owned(),
            vec![
                Index(0),
                Slice { start: 1, step: 1, end: Some(10), block: false },
                Unlimited { start: 2, step: 3, block: false },
            ]
        );
        assert_err!(u.set_unlimited(1), "The hyperslab already has one unlimited axis");
        assert_eq!(u.set_unlimited(2)?, u);

        assert_err!(u.set_block(0), "Cannot make index selection block-like");
        assert_err!(u.set_block(3), "Invalid axis for changing the slice to block-like: 3");
        let b = u.set_block(1)?;
        assert_eq!(
            b.as_ref().to_owned(),
            vec![
                Index(0),
                Slice { start: 1, step: 1, end: Some(10), block: true },
                Unlimited { start: 2, step: 3, block: false },
            ]
        );
        let b = b.set_block(2)?;
        assert_eq!(
            b.as_ref().to_owned(),
            vec![
                Index(0),
                Slice { start: 1, step: 1, end: Some(10), block: true },
                Unlimited { start: 2, step: 3, block: true },
            ]
        );
        assert_eq!(b.set_block(1)?.set_block(2)?, b);

        Ok(())
    }

    #[test]
    fn test_selection_default() {
        assert!(Selection::default().is_all());
    }

    #[test]
    fn test_selection_all_impl() {
        let s = Selection::new(..);
        assert_eq!(s, s);
        assert!(s.is_all() && !s.is_hyperslab() && !s.is_points() && !s.is_none());
        assert_ne!(s, Selection::new(()));
        assert_ne!(s, Selection::new(&[]));
        assert_eq!(s.in_ndim(), None);
        assert_eq!(s.out_ndim(), None);
        assert_eq!(s.out_shape(&[1, 2, 3]).unwrap(), &[1, 2, 3]);
        assert_eq!(format!("{}", s), "..");
    }

    #[test]
    fn test_selection_points_impl() {
        let s = Selection::new(arr2(&[[1, 2, 3], [4, 5, 6]]));
        assert_eq!(s, s);
        assert!(!s.is_all() && !s.is_hyperslab() && s.is_points() && !s.is_none());
        assert_ne!(s, Selection::new(()));
        assert_ne!(s, Selection::new(..));
        assert_eq!(s.in_ndim(), Some(3));
        assert_eq!(s.out_ndim(), Some(1));
        assert_eq!(s.out_shape(&[5, 10, 15]).unwrap(), &[2]);
        assert_eq!(format!("{}", s), "[[1, 2, 3],\n [4, 5, 6]]");
    }

    #[test]
    fn test_selection_none_impl() {
        let s = Selection::new(&[]);
        assert_eq!(s, s);
        assert!(!s.is_all() && !s.is_hyperslab() && !s.is_points() && s.is_none());
        assert_eq!(s.in_ndim(), None);
        assert_eq!(s.out_shape(&[1, 2, 3]).unwrap(), &[]);
        assert_eq!(format!("{}", s), "[]");
    }

    #[test]
    fn test_selection_hyperslab_impl() {
        let s = Selection::new(s![1, 2..;2]);
        assert_eq!(s, s);
        assert!(!s.is_all() && s.is_hyperslab() && !s.is_points() && !s.is_none());
        assert_ne!(s, Selection::new(..));
        assert_ne!(s, Selection::new(&[]));
        assert_eq!(s.in_ndim(), Some(2));
        assert_eq!(s.out_ndim(), Some(1));
        assert_eq!(s.out_shape(&[10, 20]).unwrap(), &[9]);
        assert_eq!(format!("{}", Selection::new(s![1])), "(1,)");
        assert_eq!(format!("{}", Selection::new(())), "()");

        let h = Hyperslab::new(s![1, 2..;3, ..4, 5..]).set_unlimited(1).unwrap();
        assert_eq!(format!("{}", h), "(1, 2..∞;3, ..4, 5..)");
        let s = Selection::new(h);
        assert_eq!(format!("{}", s), "(1, 2..∞;3, ..4, 5..)");
        assert_err!(s.out_shape(&[2, 3, 4, 5]), "Unable to get the shape for unlimited hyperslab");
    }

    #[test]
    fn test_hyperslab_into_from_raw_err() {
        fn check<H: Into<Hyperslab>, S: AsRef<[Ix]>>(hyper: H, shape: S, err: &str) {
            assert_err!(hyper.into().into_raw(shape.as_ref()), err);
        }

        check(
            Hyperslab::new(vec![
                Unlimited { start: 0, step: 1, block: false },
                Unlimited { start: 0, step: 1, block: false },
            ]),
            &[1, 2],
            "Expected at most 1 unlimited dimension, got 2",
        );

        check(s![1, 2], &[1, 2, 3], "Slice ndim (2) != shape ndim (3)");

        check(s![0, ..;-1], &[1, 2], "Slice stride -1 < 1 for axis 1");

        check(s![0, 0], &[0, 1], "Index 0 out of bounds for axis 0 with size 0");
        check(s![.., 1], &[0, 1], "Index 1 out of bounds for axis 1 with size 1");
        check(s![-3], &[2], "Index -3 out of bounds for axis 0 with size 2");
        check(s![2], &[2], "Index 2 out of bounds for axis 0 with size 2");

        check(s![0, 3..], &[1, 2], "Slice start 3 out of bounds for axis 1 with size 2");
        check(s![-2..;2, 0], &[1, 2], "Slice start -2 out of bounds for axis 0 with size 1");
        check(s![0, ..=3], &[1, 2], "Slice end 4 out of bounds for axis 1 with size 2");
        check(s![..-3;2, 0], &[1, 2], "Slice end -3 out of bounds for axis 0 with size 1");

        check(
            (0, Unlimited { start: 0, step: -1, block: true }),
            &[1, 2],
            "Slice stride -1 < 1 for axis 1",
        );
        check(
            (0, Unlimited { start: 3, step: 1, block: false }),
            &[1, 2],
            "Slice start 3 out of bounds for axis 1 with size 2",
        );
        check(
            (Unlimited { start: -2, step: 2, block: true }, 0),
            &[1, 2],
            "Slice start -2 out of bounds for axis 0 with size 1",
        );

        assert_err!(
            Hyperslab::from_raw(vec![RawSlice::new(0, 2, Some(1), 3)].into()),
            "Unsupported block/step for axis 0: 3/2"
        );
    }

    #[test]
    fn test_points_into_raw_err() {
        assert_err!(
            Selection::new(arr2(&[[1, 2], [3, 5]])).out_shape(&[4, 3]),
            "Index 5 out of bounds for axis 1 with size 3"
        );
    }

    #[test]
    fn test_hyperslab_into_from_raw() -> Result<()> {
        fn check<S, H, RH, RS, H2, S2>(
            shape: S, hyper: H, exp_raw_hyper: RH, exp_raw_sel: Option<RS>, exp_hyper2: H2,
            exp_sel2: Option<S2>,
        ) -> Result<()>
        where
            S: AsRef<[Ix]>,
            H: Into<Hyperslab>,
            RH: Into<RawHyperslab>,
            RS: Into<RawSelection>,
            H2: Into<Hyperslab>,
            S2: Into<Selection>,
        {
            let shape = shape.as_ref();
            let hyper = hyper.into();
            let exp_raw_hyper = exp_raw_hyper.into();
            let exp_raw_sel = exp_raw_sel.map(Into::into).unwrap_or(exp_raw_hyper.clone().into());
            let exp_hyper2 = exp_hyper2.into();
            let exp_sel2 = exp_sel2.map(Into::into).unwrap_or(exp_hyper2.clone().into());

            let raw_hyper = hyper.clone().into_raw(shape)?;
            assert_eq!(raw_hyper, exp_raw_hyper);

            let sel = Selection::new(hyper.clone());
            let raw_sel = sel.clone().into_raw(shape)?;
            assert_eq!(raw_sel, exp_raw_sel);

            let hyper2 = Hyperslab::from_raw(raw_hyper.clone())?;
            assert_eq!(hyper2, exp_hyper2);

            let sel2 = Selection::from_raw(raw_sel.clone())?;
            assert_eq!(sel2, exp_sel2);

            let raw_hyper2 = hyper2.clone().into_raw(shape)?;
            assert_eq!(raw_hyper2, raw_hyper);

            let raw_sel2 = sel2.clone().into_raw(shape)?;
            assert_eq!(raw_sel2, raw_sel);

            Ok(())
        }

        check(&[], (), vec![], Some(RawSelection::All), (), Some(Selection::All))?;

        check(
            &[5, 5, 5],
            s![.., 0..5, ..=4],
            vec![RawSlice::new(0, 1, Some(5), 1); 3],
            Some(RawSelection::All),
            s![..5, ..5, ..5],
            Some(Selection::All),
        )?;

        check(
            &[0; 6],
            s![.., 0.., ..0, 0..0, ..;1, ..;2],
            vec![
                RawSlice::new(0, 1, Some(0), 1),
                RawSlice::new(0, 1, Some(0), 1),
                RawSlice::new(0, 1, Some(0), 1),
                RawSlice::new(0, 1, Some(0), 1),
                RawSlice::new(0, 1, Some(0), 1),
                RawSlice::new(0, 2, Some(0), 1),
            ],
            Some(RawSelection::None),
            s![..0, ..0, ..0, ..0, ..0, ..0;2],
            Some(Selection::new(&[])),
        )?;

        check(
            &[3; 10],
            s![.., ..;2, 1.., 1..;2, -3..=1, -3..=-1;2, ..=-1, ..=-1;3, 0..-1, 2..=-1],
            vec![
                RawSlice::new(0, 1, Some(3), 1),
                RawSlice::new(0, 2, Some(2), 1),
                RawSlice::new(1, 1, Some(2), 1),
                RawSlice::new(1, 2, Some(1), 1),
                RawSlice::new(0, 1, Some(2), 1),
                RawSlice::new(0, 2, Some(2), 1),
                RawSlice::new(0, 1, Some(3), 1),
                RawSlice::new(0, 3, Some(1), 1),
                RawSlice::new(0, 1, Some(2), 1),
                RawSlice::new(2, 1, Some(1), 1),
            ],
            None as Option<RawSelection>,
            s![..3, ..3;2, 1..3, 1..2;2, ..=1, ..3;2, ..3, ..1;3, 0..2, 2..3],
            None as Option<Selection>,
        )?;

        check(
            &[10; 4],
            s![-5.., -10, 1..-1;2, 1],
            vec![
                RawSlice::new(5, 1, Some(5), 1),
                RawSlice::new(0, 1, Some(1), 1),
                RawSlice::new(1, 2, Some(4), 1),
                RawSlice::new(1, 1, Some(1), 1),
            ],
            None as Option<RawSelection>,
            s![5..10, 0..1, 1..8;2, 1..2],
            None as Option<Selection>,
        )?;

        check(
            &[10; 3],
            Hyperslab::new(s![-5..;2, -10, 1..-3;3])
                .set_unlimited(0)?
                .set_block(0)?
                .set_block(2)?,
            vec![
                RawSlice::new(5, 2, None, 2),
                RawSlice::new(0, 1, Some(1), 1),
                RawSlice::new(1, 3, Some(2), 3),
            ],
            None as Option<RawSelection>,
            Hyperslab::new(s![5..;2, 0..1, 1..7;3]).set_unlimited(0)?.set_block(0)?.set_block(2)?,
            None as Option<Selection>,
        )?;

        check(
            &[7; 7],
            Hyperslab::new(s![1..2;3, 1..3;3, 1..4;3, 1..5;3, 1..6;3, 1..7;3, ..7;3]),
            vec![
                RawSlice::new(1, 3, Some(1), 1),
                RawSlice::new(1, 3, Some(1), 1),
                RawSlice::new(1, 3, Some(1), 1),
                RawSlice::new(1, 3, Some(2), 1),
                RawSlice::new(1, 3, Some(2), 1),
                RawSlice::new(1, 3, Some(2), 1),
                RawSlice::new(0, 3, Some(3), 1),
            ],
            None as Option<RawSelection>,
            Hyperslab::new(s![1..2;3, 1..2;3, 1..2;3, 1..5;3, 1..5;3, 1..5;3, ..7;3]),
            None as Option<Selection>,
        )?;

        check(
            &[7; 4],
            Hyperslab::new(s![1..4;3, 1..5;3, 1..6;3, 1..7;3])
                .set_block(0)?
                .set_block(1)?
                .set_block(2)?
                .set_block(3)?,
            vec![
                RawSlice::new(1, 3, Some(1), 3),
                RawSlice::new(1, 3, Some(1), 3),
                RawSlice::new(1, 3, Some(1), 3),
                RawSlice::new(1, 3, Some(2), 3),
            ],
            None as Option<RawSelection>,
            Hyperslab::new(s![1..4;3, 1..4;3, 1..4;3, 1..7;3])
                .set_block(0)?
                .set_block(1)?
                .set_block(2)?
                .set_block(3)?,
            None as Option<Selection>,
        )?;

        Ok(())
    }

    #[test]
    fn test_in_out_shape_ndim() -> Result<()> {
        fn check<S: Into<Selection>, E: AsRef<[Ix]>>(
            sel: S, exp_in_ndim: Option<usize>, exp_out_shape: E, exp_out_ndim: Option<usize>,
        ) -> Result<()> {
            let in_shape = [7, 8];
            let sel = sel.into();
            assert_eq!(sel.in_ndim(), exp_in_ndim);
            let out_shape = sel.out_shape(&in_shape)?;
            let out_ndim = sel.out_ndim();
            assert_eq!(out_shape.as_slice(), exp_out_shape.as_ref());
            assert_eq!(out_ndim, exp_out_ndim);
            if let Some(out_ndim) = out_ndim {
                assert_eq!(out_shape.len(), out_ndim);
            } else {
                assert_eq!(out_shape.len(), in_shape.len());
            }
            Ok(())
        }

        check(.., None, [7, 8], None)?;
        check(Array2::zeros((0, 0)), None, [], Some(0))?;
        check(arr2(&[[0, 1]]), Some(2), [1], Some(1))?;
        check(arr2(&[[0, 1], [2, 3], [4, 5]]), Some(2), [3], Some(1))?;
        check(s![1, 2], Some(2), [], Some(0))?;
        check(s![1, 2..;2], Some(2), [3], Some(1))?;
        check(s![1..;3, 2], Some(2), [2], Some(1))?;
        check(s![1..;3, 2..;2], Some(2), [2, 3], Some(2))?;
        check(Hyperslab::new(s![1, 2..;2]).set_block(1)?, Some(2), [6], Some(1))?;
        check(Hyperslab::new(s![1..;3, 2]).set_block(0)?, Some(2), [6], Some(1))?;
        check(
            Hyperslab::new(s![1..;3, 2..;2]).set_block(0)?.set_block(1)?,
            Some(2),
            [6, 6],
            Some(2),
        )?;

        assert_err!(
            check(arr2(&[[1, 2, 3]]), Some(3), [], None),
            "Slice ndim (3) != shape ndim (2)"
        );
        assert_err!(
            check(arr2(&[[7, 1]]), Some(2), [], None),
            "Index 7 out of bounds for axis 0 with size 7"
        );

        Ok(())
    }

    #[test]
    fn test_selection_into_from_raw() -> Result<()> {
        fn check<Sh, S, RS, S2>(
            shape: Sh, sel: S, exp_raw_sel: RS, exp_sel2: Option<S2>,
        ) -> Result<()>
        where
            Sh: AsRef<[Ix]>,
            S: Into<Selection>,
            RS: Into<RawSelection>,
            S2: Into<Selection>,
        {
            let shape = shape.as_ref();
            let sel = sel.into();
            let exp_raw_sel = exp_raw_sel.into();
            let exp_sel2 = exp_sel2.map_or(sel.clone(), Into::into);

            let raw_sel = sel.clone().into_raw(shape)?;
            assert_eq!(raw_sel, exp_raw_sel);

            let sel2 = Selection::from_raw(raw_sel.clone())?;
            assert_eq!(sel2, exp_sel2);

            let raw_sel2 = sel2.clone().into_raw(shape)?;
            assert_eq!(raw_sel2, raw_sel);

            Ok(())
        }

        check(&[5, 6], .., RawSelection::All, None as Option<Selection>)?;
        check(&[5, 6], Array2::zeros((0, 0)), RawSelection::None, None as Option<Selection>)?;
        check(&[5, 6], Array2::zeros((0, 2)), RawSelection::None, Some(Array2::zeros((0, 0))))?;
        check(
            &[5, 6],
            arr2(&[[1, 2]]),
            RawSelection::Points(arr2(&[[1, 2]])),
            None as Option<Selection>,
        )?;
        check(&[5, 6], s![1..1;2, 3], RawSelection::None, Some(&[]))?;
        check(&[5, 6], s![-5.., 0..], RawSelection::All, Some(..))?;
        check(
            &[5, 6],
            s![1..;2, 3],
            vec![RawSlice::new(1, 2, Some(2), 1), RawSlice::new(3, 1, Some(1), 1)],
            Some(s![1..4;2, 3..4]),
        )?;

        assert_err!(
            Selection::from_raw(RawSelection::ComplexHyperslab),
            "Cannot convert complex hyperslabs"
        );

        Ok(())
    }

    #[test]
    fn test_apply_extract_selection() -> Result<()> {
        use crate::sync::sync;
        use hdf5_sys::h5s::{H5Sclose, H5Screate_simple};
        use std::ptr;

        fn check<Sh>(
            shape: Sh, raw_sel: RawSelection, exp_raw_sel2: Option<RawSelection>,
        ) -> Result<()>
        where
            Sh: AsRef<[Ix]>,
        {
            let shape = shape.as_ref();
            let c_shape = shape.iter().map(|&x| x as _).collect::<Vec<_>>();
            let exp_raw_sel2 = exp_raw_sel2.unwrap_or(raw_sel.clone());
            sync(|| unsafe {
                let space_id =
                    h5check(H5Screate_simple(shape.len() as _, c_shape.as_ptr(), ptr::null_mut()))?;
                raw_sel.apply_to_dataspace(space_id)?;
                let raw_sel2 = RawSelection::extract_from_dataspace(space_id)?;
                assert_eq!(raw_sel2, exp_raw_sel2);
                H5Sclose(space_id);
                Ok(())
            })
        }

        let _e = silence_errors();

        check(&[1, 2], RawSelection::None, None)?;
        check(&[1, 2], RawSelection::All, None)?;
        check(&[1, 2], RawSelection::Points(arr2(&[[0, 1], [0, 0]])), None)?;

        let exp = if cfg!(hdf5_1_10_0) { None } else { Some(RawSelection::ComplexHyperslab) };
        check(
            &[8, 9, 10, 11],
            vec![
                RawSlice::new(1, 2, None, 2),
                RawSlice::new(1, 2, Some(2), 2),
                RawSlice::new(1, 1, Some(3), 1),
                RawSlice::new(1, 2, Some(4), 1),
            ]
            .into(),
            exp,
        )?;

        assert_err!(
            check(&[1, 2], RawSelection::ComplexHyperslab, None),
            "Complex hyperslabs are not supported"
        );
        assert_err!(
            check(&[1, 2], RawSelection::Points(Array2::zeros((0, 2))), None),
            "H5Sselect_elements(): elements not specified"
        );

        Ok(())
    }
}

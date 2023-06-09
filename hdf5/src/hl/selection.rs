use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
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
#[cfg(feature = "1.10.0")]
use hdf5_sys::h5s::{H5Sget_regular_hyperslab, H5Sis_regular_hyperslab};

use crate::hl::extents::Ix;
use crate::internal_prelude::*;

unsafe fn get_points_selection(space_id: hid_t) -> Result<Array2<Ix>> {
    let npoints = h5check(H5Sget_select_elem_npoints(space_id))? as usize;
    let ndim = h5check(H5Sget_simple_extent_ndims(space_id))? as usize;
    let mut coords = vec![0; npoints * ndim];
    h5check(H5Sget_select_elem_pointlist(space_id, 0, npoints as _, coords.as_mut_ptr()))?;
    let coords = if mem::size_of::<hsize_t>() == mem::size_of::<Ix>() {
        #[allow(clippy::transmute_undefined_repr)]
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
            Cow::Borrowed(slice::from_raw_parts(coords.as_ptr().cast(), coords.len()))
        }
        _ => Cow::Owned(coords.iter().map(|&x| x as _).collect()),
    };
    h5check(H5Sselect_elements(space_id, H5S_SELECT_SET, nelem, coords.as_ptr()))?;
    Ok(())
}

#[cfg_attr(not(feature = "1.10.0"), allow(unused))]
unsafe fn get_regular_hyperslab(space_id: hid_t) -> Result<Option<RawHyperslab>> {
    #[cfg(feature = "1.10.0")]
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
    if coords.shape() == [0, 0] {
        return Ok(());
    }
    let ndim = coords.shape()[1];
    ensure!(ndim == shape.len(), "Slice ndim ({}) != shape ndim ({})", ndim, shape.len());
    for (i, &dim) in shape.iter().enumerate() {
        for &d in coords.slice(s![.., i]).iter() {
            ensure!(d < dim, "Index {} out of bounds for axis {} with size {}", d, i, dim);
        }
    }
    Ok(())
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
        Self::All
    }
}

impl From<RawHyperslab> for RawSelection {
    fn from(hyper: RawHyperslab) -> Self {
        Self::RegularHyperslab(hyper)
    }
}

impl From<Vec<RawSlice>> for RawSelection {
    fn from(dims: Vec<RawSlice>) -> Self {
        Self::RegularHyperslab(dims.into())
    }
}

impl RawSelection {
    pub unsafe fn apply_to_dataspace(&self, space_id: hid_t) -> Result<()> {
        match self {
            Self::None => {
                h5check(H5Sselect_none(space_id))?;
            }
            Self::All => {
                h5check(H5Sselect_all(space_id))?;
            }
            Self::Points(ref coords) => set_points_selection(space_id, coords.view())?,
            Self::RegularHyperslab(ref hyper) => set_regular_hyperslab(space_id, hyper)?,
            Self::ComplexHyperslab => fail!("Complex hyperslabs are not supported"),
        };
        Ok(())
    }

    pub unsafe fn extract_from_dataspace(space_id: hid_t) -> Result<Self> {
        Ok(match H5Sget_select_type(space_id) {
            H5S_sel_type::H5S_SEL_NONE => Self::None,
            H5S_sel_type::H5S_SEL_ALL => Self::All,
            H5S_sel_type::H5S_SEL_POINTS => Self::Points(get_points_selection(space_id)?),
            H5S_sel_type::H5S_SEL_HYPERSLABS => get_regular_hyperslab(space_id)?
                .map_or(Self::ComplexHyperslab, Self::RegularHyperslab),
            sel_type => fail!("Invalid selection type: {:?}", sel_type as c_int),
        })
    }
}

/// A selector of a one-dimensional array
///
/// The following examples will use an array of 11 elements
/// to illustrate the various selections. The active elements
/// are marked with an `s`.
/// ```text
/// // An array of 11 elements
/// x x x x x x x x x x x
/// ```
///
/// ```text
/// Index(4)
/// _ _ _ _ s _ _ _ _ _ _
/// ```
/// ```text
/// Slice { start: 0, step: 3, end: 4, block: 1 }
/// s _ _ s _ _ _ _ _ _ _
/// ```
/// ```text
/// SliceTo { start: 2, step: 3, end: 8, block: 1 }
/// _ _ s _ _ s _ _ _ _ _
/// ```
/// ```text
/// SliceCount { start: 1, step: 3, count: 2, block: 1 }
/// _ s _ _ s _ _ s _ _ _
/// ```
/// ```text
/// Unlimited { start: 0, step: 3, block: 1 }
/// s _ _ s _ _ s _ _ s _
/// ```
/// ```text
/// Unlimited { start: 2, step: 3, block: 1 }
/// _ _ s _ _ s _ _ s _ _
/// ```
/// ```text
/// Unlimited { start: 0, step: 4, block: 2 }
/// s s _ _ s s _ _ s s _
/// ```
///
/// See also [`this hdf5 tutorial`](https://support.hdfgroup.org/HDF5/Tutor/select.html)
/// for more information on hyperslab selections.
#[derive(Clone, Copy, Debug, Eq)]
pub enum SliceOrIndex {
    /// A single index
    Index(Ix),
    /// Up to the given index
    SliceTo { start: Ix, step: Ix, end: Ix, block: Ix },
    /// The given count of elements
    SliceCount { start: Ix, step: Ix, count: Ix, block: Ix },
    /// An unlimited hyperslab
    Unlimited { start: Ix, step: Ix, block: Ix },
}

impl PartialEq for SliceOrIndex {
    fn eq(&self, other: &Self) -> bool {
        use SliceOrIndex::{Index, SliceCount, SliceTo, Unlimited};
        match (self, other) {
            (Index(s), Index(o)) => s == o,
            (
                SliceTo { start: sstart, step: sstep, end: send, block: sblock },
                SliceTo { start: ostart, step: ostep, end: oend, block: oblock },
            ) => (sstart == ostart) & (sstep == ostep) & (send == oend) & (sblock == oblock),
            (
                SliceCount { start: sstart, step: sstep, count: scount, block: sblock },
                SliceCount { start: ostart, step: ostep, count: ocount, block: oblock },
            ) => (sstart == ostart) & (sstep == ostep) & (scount == ocount) & (sblock == oblock),
            (
                Unlimited { start: sstart, step: sstep, block: sblock },
                Unlimited { start: ostart, step: ostep, block: oblock },
            ) => (sstart == ostart) & (sstep == ostep) & (sblock == oblock),
            (
                SliceTo { start: sstart, step: sstep, end: _, block: sblock },
                SliceCount { start: ostart, step: ostep, count: ocount, block: oblock },
            ) => {
                if (sstart != ostart) | (sstep != ostep) | (sblock != oblock) {
                    return false;
                }
                self.count().unwrap() == *ocount
            }
            (SliceCount { .. }, SliceTo { .. }) => other == self,
            _ => false,
        }
    }
}

impl SliceOrIndex {
    pub fn to_unlimited(self) -> Result<Self> {
        Ok(match self {
            Self::Index(_) => fail!("Cannot make index selection unlimited"),
            Self::SliceTo { start, step, block, .. }
            | Self::SliceCount { start, step, block, .. }
            | Self::Unlimited { start, step, block } => Self::Unlimited { start, step, block },
        })
    }

    pub fn is_index(self) -> bool {
        matches!(self, Self::Index(_))
    }

    pub fn is_slice(self) -> bool {
        matches!(self, Self::SliceTo { .. } | Self::SliceCount { .. } | Self::Unlimited { .. })
    }

    pub fn is_unlimited(self) -> bool {
        matches!(self, Self::Unlimited { .. })
    }

    fn set_blocksize(self, blocksize: Ix) -> Result<Self> {
        Ok(match self {
            Self::Index(_) => fail!("Cannot set blocksize for index selection"),
            Self::SliceTo { start, step, end, .. } => {
                Self::SliceTo { start, step, end, block: blocksize }
            }
            Self::SliceCount { start, step, count, .. } => {
                Self::SliceCount { start, step, count, block: blocksize }
            }
            Self::Unlimited { start, step, .. } => {
                Self::Unlimited { start, step, block: blocksize }
            }
        })
    }

    /// Number of elements contained in the `SliceOrIndex`
    fn count(self) -> Option<usize> {
        use SliceOrIndex::{Index, SliceCount, SliceTo, Unlimited};
        match self {
            Index(_) => Some(1),
            SliceTo { start, step, end, block } => {
                Some((start + block.saturating_sub(1)..end).step_by(step).count())
            }
            SliceCount { count, .. } => Some(count),
            Unlimited { .. } => None,
        }
    }
}

impl TryFrom<ndarray::SliceInfoElem> for SliceOrIndex {
    type Error = Error;
    fn try_from(slice: ndarray::SliceInfoElem) -> Result<Self, Self::Error> {
        Ok(match slice {
            ndarray::SliceInfoElem::Index(index) => match Ix::try_from(index) {
                Err(_) => fail!("Index must be non-negative"),
                Ok(index) => Self::Index(index),
            },
            ndarray::SliceInfoElem::Slice { start, end, step } => {
                let start =
                    Ix::try_from(start).map_err(|_| Error::from("Index must be non-negative"))?;
                let step =
                    Ix::try_from(step).map_err(|_| Error::from("Step must be non-negative"))?;
                let end = end.map(|end| {
                    Ix::try_from(end).map_err(|_| Error::from("End must be non-negative"))
                });
                match end {
                    Some(Ok(end)) => Self::SliceTo { start, step, end, block: 1 },
                    None => Self::Unlimited { start, step, block: 1 },
                    Some(Err(e)) => return Err(e),
                }
            }
            ndarray::SliceInfoElem::NewAxis => fail!("ndarray NewAxis can not be mapped to hdf5"),
        })
    }
}

impl TryFrom<ndarray::SliceInfoElem> for Hyperslab {
    type Error = Error;
    fn try_from(slice: ndarray::SliceInfoElem) -> Result<Self, Self::Error> {
        Ok(vec![slice.try_into()?].into())
    }
}

impl TryFrom<ndarray::SliceInfoElem> for Selection {
    type Error = Error;
    fn try_from(slice: ndarray::SliceInfoElem) -> Result<Self, Self::Error> {
        Hyperslab::try_from(slice).map(Into::into)
    }
}

impl From<RangeFull> for SliceOrIndex {
    fn from(_r: RangeFull) -> Self {
        Self::Unlimited { start: 0, step: 1, block: 1 }
    }
}

impl TryFrom<ndarray::Slice> for SliceOrIndex {
    type Error = std::num::TryFromIntError;
    fn try_from(slice: ndarray::Slice) -> Result<Self, Self::Error> {
        let ndarray::Slice { start, end, step } = slice;
        let start = start.try_into()?;
        let step = step.try_into()?;
        let end = end.map(TryInto::try_into);
        match end {
            Some(Ok(end)) => Ok(Self::SliceTo { start, end, step, block: 1 }),
            None => Ok(Self::Unlimited { start, step, block: 1 }),
            Some(Err(e)) => Err(e),
        }
    }
}

impl From<usize> for SliceOrIndex {
    fn from(val: usize) -> Self {
        Self::Index(val as _)
    }
}

impl From<usize> for Hyperslab {
    fn from(slice: usize) -> Self {
        (slice,).into()
    }
}

impl From<usize> for Selection {
    fn from(slice: usize) -> Self {
        Hyperslab::from(slice).into()
    }
}

impl From<Range<usize>> for SliceOrIndex {
    fn from(val: Range<usize>) -> Self {
        Self::SliceTo { start: val.start as _, step: 1, end: val.end, block: 1 }
    }
}

impl From<Range<usize>> for Hyperslab {
    fn from(val: Range<usize>) -> Self {
        vec![val.into()].into()
    }
}

impl From<Range<usize>> for Selection {
    fn from(val: Range<usize>) -> Self {
        Hyperslab::from(val).into()
    }
}

impl From<RangeToInclusive<usize>> for SliceOrIndex {
    fn from(val: RangeToInclusive<usize>) -> Self {
        let end = val.end + 1;
        Self::SliceTo { start: 0, step: 1, end, block: 1 }
    }
}

impl From<RangeToInclusive<usize>> for Hyperslab {
    fn from(val: RangeToInclusive<usize>) -> Self {
        vec![val.into()].into()
    }
}

impl From<RangeToInclusive<usize>> for Selection {
    fn from(val: RangeToInclusive<usize>) -> Self {
        Hyperslab::from(val).into()
    }
}

impl From<RangeFrom<usize>> for SliceOrIndex {
    fn from(val: RangeFrom<usize>) -> Self {
        Self::Unlimited { start: val.start, step: 1, block: 1 }
    }
}

impl From<RangeFrom<usize>> for Hyperslab {
    fn from(val: RangeFrom<usize>) -> Self {
        vec![val.into()].into()
    }
}

impl From<RangeFrom<usize>> for Selection {
    fn from(val: RangeFrom<usize>) -> Self {
        Hyperslab::from(val).into()
    }
}

impl From<RangeInclusive<usize>> for SliceOrIndex {
    fn from(val: RangeInclusive<usize>) -> Self {
        Self::SliceTo { start: *val.start(), step: 1, end: *val.end() + 1, block: 1 }
    }
}

impl From<RangeInclusive<usize>> for Hyperslab {
    fn from(val: RangeInclusive<usize>) -> Self {
        vec![val.into()].into()
    }
}

impl From<RangeInclusive<usize>> for Selection {
    fn from(val: RangeInclusive<usize>) -> Self {
        Hyperslab::from(val).into()
    }
}

impl From<RangeTo<usize>> for SliceOrIndex {
    fn from(val: RangeTo<usize>) -> Self {
        Self::SliceTo { start: 0, step: 1, end: val.end, block: 1 }
    }
}

impl From<RangeTo<usize>> for Hyperslab {
    fn from(val: RangeTo<usize>) -> Self {
        vec![val.into()].into()
    }
}

impl From<RangeTo<usize>> for Selection {
    fn from(val: RangeTo<usize>) -> Self {
        Hyperslab::from(val).into()
    }
}

impl Display for SliceOrIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Index(index) => write!(f, "{index}")?,
            Self::SliceTo { start, end, step, block } => {
                if start != 0 {
                    write!(f, "{start}")?;
                }
                write!(f, "..")?;
                write!(f, "{end}")?;
                if step != 1 {
                    write!(f, ";{step}")?;
                }
                if block != 1 {
                    write!(f, "(Bx{block})")?;
                }
            }
            Self::SliceCount { start, step, count, block } => {
                if start != 0 {
                    write!(f, "{start}")?;
                }
                write!(f, "+{count}")?;
                if step != 1 {
                    write!(f, ";{step}")?;
                }
                if block != 1 {
                    write!(f, "(Bx{block})")?;
                }
            }
            Self::Unlimited { start, step, block } => {
                if start != 0 {
                    write!(f, "{start}")?;
                }
                // \u{221e} = ∞
                write!(f, "..\u{221e}")?;
                if step != 1 {
                    write!(f, ";{step}")?;
                }
                if block != 1 {
                    write!(f, "(Bx{block})")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A descriptor of a selection of an N-dimensional array.
///
/// The Hyperslab consists of [`slices`](SliceOrIndex) in N dimensions,
/// spanning an N-dimensional hypercube. This type is used as a [`selector`](Selection)
/// for retrieving and putting data to a [`Container`](Container).
pub struct Hyperslab {
    dims: Vec<SliceOrIndex>,
}

impl Hyperslab {
    pub fn new<T: Into<Self>>(hyper: T) -> Self {
        hyper.into()
    }

    pub fn try_new<T: TryInto<Self>>(hyper: T) -> Result<Self, T::Error> {
        hyper.try_into()
    }

    pub fn is_unlimited(&self) -> bool {
        self.iter().any(|&s| s.is_unlimited())
    }

    pub fn unlimited_axis(&self) -> Option<usize> {
        self.iter().enumerate().find_map(|(i, s)| if s.is_unlimited() { Some(i) } else { None })
    }

    pub fn set_unlimited(&self, axis: usize) -> Result<Self> {
        if axis < self.len() {
            let mut hyper = self.clone();
            hyper.dims[axis] = hyper.dims[axis].to_unlimited()?;
            Ok(hyper)
        } else {
            fail!("Invalid axis for making hyperslab unlimited: {}", axis);
        }
    }

    pub fn set_block(&self, axis: usize, blocksize: Ix) -> Result<Self> {
        ensure!(axis < self.len(), "Invalid axis for changing the slice to block-like: {}", axis);
        let mut hyper = self.clone();
        hyper.dims[axis] = hyper.dims[axis].set_blocksize(blocksize)?;
        Ok(hyper)
    }

    #[doc(hidden)]
    pub fn into_raw<S: AsRef<[Ix]>>(self, shape: S) -> Result<RawHyperslab> {
        let shape = shape.as_ref();
        let ndim = shape.len();
        ensure!(self.len() == ndim, "Slice ndim ({}) != shape ndim ({})", self.len(), ndim);
        //let n_unlimited: usize = self.iter().map(|s| s.is_unlimited() as usize).sum();
        //ensure!(n_unlimited <= 1, "Expected at most 1 unlimited dimension, got {}", n_unlimited);
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
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_raw(hyper: RawHyperslab) -> Result<Self> {
        let mut dims = vec![];
        for (axis, slice) in hyper.iter().enumerate() {
            ensure!(slice.step >= slice.block, "Blocks can not overlap (axis: {})", axis);
            dims.push(match slice.count {
                Some(count) => SliceOrIndex::SliceCount {
                    start: slice.start,
                    step: slice.step,
                    count,
                    block: slice.block,
                },
                None => SliceOrIndex::Unlimited {
                    start: slice.start,
                    step: slice.step,
                    block: slice.block,
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

impl TryFrom<ndarray::Slice> for Hyperslab {
    type Error = Error;
    fn try_from(slice: ndarray::Slice) -> Result<Self, Self::Error> {
        Ok(vec![SliceOrIndex::try_from(slice).map_err(|_| Error::from("Invalid slice"))?].into())
    }
}

impl<T, Din, Dout> TryFrom<ndarray::SliceInfo<T, Din, Dout>> for Hyperslab
where
    T: AsRef<[ndarray::SliceInfoElem]>,
    Din: ndarray::Dimension,
    Dout: ndarray::Dimension,
{
    type Error = Error;
    fn try_from(slice: ndarray::SliceInfo<T, Din, Dout>) -> Result<Self, Self::Error> {
        slice
            .deref()
            .as_ref()
            .iter()
            .copied()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>>>()
            .map(Into::into)
    }
}

/// Turns `SliceOrIndex` into real dimensions given `dim` as the maximum dimension
fn slice_info_to_raw(axis: usize, slice: &SliceOrIndex, dim: Ix) -> Result<RawSlice> {
    let err_msg = || format!("out of bounds for axis {axis} with size {dim}");
    let (start, step, count, block) = match *slice {
        SliceOrIndex::Index(index) => {
            ensure!(index < dim, "Index {} {}", index, err_msg());
            (index, 1, 1, 1)
        }
        SliceOrIndex::SliceTo { start, step, end, block } => {
            ensure!(step >= 1, "Slice stride {} < 1 for axis {}", step, axis);
            ensure!(start <= dim, "Slice start {} {}", start, err_msg());
            ensure!(end <= dim, "Slice end {} {}", end, err_msg());
            ensure!(step > 0, "Stride {} {}", step, err_msg());
            let count = slice.count().unwrap();
            (start, step, count, block)
        }
        SliceOrIndex::SliceCount { start, step, count, block } => {
            ensure!(step >= 1, "Slice stride {} < 1 for axis {}", step, axis);
            ensure!(start <= dim as _, "Slice start {} {}", start, err_msg());
            let end = start + block.saturating_sub(1) + step * count.saturating_sub(1);
            ensure!(end <= dim, "Slice end {} {}", end, err_msg());
            (start, step, count, block)
        }
        SliceOrIndex::Unlimited { start, step, block } => {
            // Replace infinite slice with one limited by the current dimension
            return slice_info_to_raw(
                axis,
                &SliceOrIndex::SliceTo { start, step, end: dim, block },
                dim,
            );
        }
    };
    Ok(RawSlice { start, step, count: Some(count), block })
}

impl Display for Hyperslab {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slice: &[_] = self.as_ref();
        write!(f, "(")?;
        for (i, s) in slice.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{s}")?;
        }
        if slice.len() == 1 {
            write!(f, ",")?;
        }
        write!(f, ")")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A selection used for reading and writing to a [`Container`](Container).
pub enum Selection {
    All,
    Points(Array2<Ix>),
    Hyperslab(Hyperslab),
}

impl Default for Selection {
    fn default() -> Self {
        Self::All
    }
}

impl Selection {
    pub fn new<T: Into<Self>>(selection: T) -> Self {
        selection.into()
    }

    pub fn try_new<T: TryInto<Self>>(selection: T) -> Result<Self, T::Error> {
        selection.try_into()
    }

    #[doc(hidden)]
    pub fn into_raw<S: AsRef<[Ix]>>(self, shape: S) -> Result<RawSelection> {
        let shape = shape.as_ref();
        Ok(match self {
            Self::All => RawSelection::All,
            Self::Points(coords) => {
                check_coords(&coords, shape)?;
                if coords.shape()[0] == 0 {
                    RawSelection::None
                } else {
                    RawSelection::Points(coords)
                }
            }
            Self::Hyperslab(hyper) => {
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
            RawSelection::None => Self::Points(Array2::default((0, 0))),
            RawSelection::All => Self::All,
            RawSelection::Points(coords) => Self::Points(coords),
            RawSelection::RegularHyperslab(hyper) => Hyperslab::from_raw(hyper)?.into(),
            RawSelection::ComplexHyperslab => fail!("Cannot convert complex hyperslabs"),
        })
    }

    pub fn in_ndim(&self) -> Option<usize> {
        match self {
            Self::All => None,
            Self::Points(ref points) => {
                if points.shape() == [0, 0] {
                    None
                } else {
                    Some(points.shape()[1])
                }
            }
            Self::Hyperslab(ref hyper) => Some(hyper.len()),
        }
    }

    pub fn out_ndim(&self) -> Option<usize> {
        match self {
            Self::All => None,
            Self::Points(ref points) => Some(usize::from(points.shape() != [0, 0])),
            Self::Hyperslab(ref hyper) => {
                Some(hyper.iter().map(|&s| usize::from(s.is_slice())).sum())
            }
        }
    }

    pub fn out_shape<S: AsRef<[Ix]>>(&self, in_shape: S) -> Result<Vec<Ix>> {
        let in_shape = in_shape.as_ref();
        match self {
            Self::All => Ok(in_shape.to_owned()),
            Self::Points(ref points) => check_coords(points, in_shape)
                .and(Ok(if points.shape() == [0, 0] { vec![] } else { vec![points.shape()[0]] })),
            Self::Hyperslab(ref hyper) => hyper
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
        self == &Self::All
    }

    pub fn is_points(&self) -> bool {
        if let Self::Points(ref points) = self {
            points.shape() != [0, 0]
        } else {
            false
        }
    }

    pub fn is_none(&self) -> bool {
        if let Self::Points(points) = self {
            points.shape() == [0, 0]
        } else {
            false
        }
    }

    pub fn is_hyperslab(&self) -> bool {
        matches!(self, Self::Hyperslab(_))
    }
}

impl Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::All => write!(f, ".."),
            Self::Points(ref points) => {
                if points.shape() == [0, 0] {
                    write!(f, "[]")
                } else {
                    write!(f, "{points}")
                }
            }
            Self::Hyperslab(hyper) => write!(f, "{hyper}"),
        }
    }
}

impl From<&Self> for Selection {
    fn from(sel: &Self) -> Self {
        sel.clone()
    }
}

impl From<RangeFull> for Selection {
    fn from(_: RangeFull) -> Self {
        Self::All
    }
}

impl From<()> for Selection {
    fn from(_: ()) -> Self {
        Hyperslab::from(()).into()
    }
}

impl From<SliceOrIndex> for Selection {
    fn from(slice: SliceOrIndex) -> Self {
        Self::Hyperslab(slice.into())
    }
}

impl From<Hyperslab> for Selection {
    fn from(hyper: Hyperslab) -> Self {
        Self::Hyperslab(hyper)
    }
}

impl TryFrom<ndarray::Slice> for Selection {
    type Error = Error;
    fn try_from(slice: ndarray::Slice) -> Result<Self, Self::Error> {
        Hyperslab::try_from(slice).map(Into::into)
    }
}

impl<T, Din, Dout> TryFrom<ndarray::SliceInfo<T, Din, Dout>> for Selection
where
    T: AsRef<[ndarray::SliceInfoElem]>,
    Din: ndarray::Dimension,
    Dout: ndarray::Dimension,
{
    type Error = Error;
    fn try_from(slice: ndarray::SliceInfo<T, Din, Dout>) -> Result<Self, Self::Error> {
        Hyperslab::try_from(slice).map(Into::into)
    }
}

impl From<Array2<Ix>> for Selection {
    fn from(points: Array2<Ix>) -> Self {
        Self::Points(points)
    }
}

impl From<Array1<Ix>> for Selection {
    fn from(points: Array1<Ix>) -> Self {
        let n = points.len();
        Self::Points(if n == 0 {
            Array2::zeros((0, 0))
        } else {
            points.insert_axis(ndarray::Axis(1))
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

impl<const N: usize> From<[Ix; N]> for Selection {
    fn from(points: [Ix; N]) -> Self {
        points.as_ref().into()
    }
}

impl<const N: usize> From<&[Ix; N]> for Selection {
    fn from(points: &[Ix; N]) -> Self {
        points.as_ref().into()
    }
}

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

#[cfg(test)]
mod test {
    use ndarray::{arr1, arr2, s, Array2};
    use pretty_assertions::assert_eq;

    use super::{
        Hyperslab, RawHyperslab, RawSelection, RawSlice, Selection, SliceOrIndex, SliceOrIndex::*,
    };
    use crate::internal_prelude::*;

    #[test]
    fn count() {
        use SliceOrIndex::*;
        assert_eq!(Unlimited { start: 0, step: 1, block: 1 }.count(), None);
        assert_eq!(Index(23412).count(), Some(1));
        assert_eq!(SliceCount { start: 0, step: 1431, count: 41, block: 4 }.count(), Some(41));
        assert_eq!(SliceTo { start: 0, step: 1, end: 1, block: 1 }.count(), Some(1));
        assert_eq!(SliceTo { start: 0, step: 10, end: 1, block: 1 }.count(), Some(1));
        assert_eq!(SliceTo { start: 0, step: 1, end: 10, block: 1 }.count(), Some(10));
        assert_eq!(SliceTo { start: 0, step: 10, end: 10, block: 1 }.count(), Some(1));
        assert_eq!(SliceTo { start: 0, step: 9, end: 10, block: 1 }.count(), Some(2));
        assert_eq!(SliceTo { start: 0, step: 9, end: 10, block: 2 }.count(), Some(1));
        assert_eq!(SliceTo { start: 1, step: 3, end: 6, block: 1 }.count(), Some(2));
    }

    #[test]
    fn test_slice_or_index_impl() -> Result<()> {
        use std::convert::TryFrom;
        let s = SliceOrIndex::from(2);
        assert_eq!(s, Index(2));
        assert!(s.is_index());
        assert!(!s.is_slice());
        assert!(!s.is_unlimited());
        assert_err!(s.to_unlimited(), "Cannot make index selection unlimited");
        assert_err!(s.set_blocksize(2), "Cannot set blocksize for index selection");

        let s = SliceOrIndex::from(2..=5);
        assert_eq!(s, SliceTo { start: 2, step: 1, end: 6, block: 1 });
        assert!(!s.is_index());
        assert!(s.is_slice());
        assert!(!s.is_unlimited());
        assert_eq!(s.to_unlimited().unwrap(), Unlimited { start: 2, step: 1, block: 1 });
        assert_eq!(s.set_blocksize(4)?, SliceTo { start: 2, step: 1, end: 6, block: 4 });
        assert_eq!(
            SliceOrIndex::try_from(*s![1..2;3].get(0).unwrap()).unwrap(),
            SliceTo { start: 1, step: 3, end: 2, block: 1 }
        );

        let s = SliceOrIndex::from(3..).to_unlimited()?;
        assert_eq!(s, Unlimited { start: 3, step: 1, block: 1 });
        assert!(!s.is_index());
        assert!(s.is_slice());
        assert!(s.is_unlimited());
        assert_eq!(s.to_unlimited()?, s);

        for (s, f) in &[
            (Unlimited { start: 0, step: 1, block: 1 }, "..∞"),
            (Unlimited { start: 0, step: 1, block: 2 }, "..∞(Bx2)"),
            (SliceTo { start: 0, step: 1, end: 5, block: 1 }, "..5"),
            (SliceTo { start: 0, step: 3, end: 5, block: 2 }, "..5;3(Bx2)"),
            (Unlimited { start: 0, step: 1, block: 1 }, "..∞"),
            (Unlimited { start: 0, step: 3, block: 2 }, "..∞;3(Bx2)"),
            (SliceCount { start: 1, step: 3, count: 5, block: 1 }, "1+5;3"),
            (SliceCount { start: 0, step: 3, count: 5, block: 2 }, "+5;3(Bx2)"),
            (SliceCount { start: 1, step: 3, count: 5, block: 2 }, "1+5;3(Bx2)"),
        ] {
            assert_eq!(&format!("{}", s), f);
        }

        Ok(())
    }

    #[test]
    fn test_selection_hyperslab_new() {
        macro_rules! check {
            ($hs1:expr, $hs2:expr) => {
                assert_eq!(Hyperslab::try_new($hs1).unwrap().as_ref().to_owned(), $hs2);
                let s = Selection::try_new($hs1).unwrap();
                assert_eq!(s, Selection::new(Hyperslab::new($hs2)));
                assert_eq!(s, Selection::Hyperslab(Hyperslab::new($hs2)));
            };
        }

        check!((), vec![]);
        check!(Index(2), vec![Index(2)]);
        check!(ndarray::SliceInfoElem::Index(10), vec![Index(10)]);
        check!(3.., vec![Unlimited { start: 3, step: 1, block: 1 }]);

        assert_eq!(
            Hyperslab::new(..).as_ref().to_owned(),
            vec![Unlimited { start: 0, step: 1, block: 1 }]
        );
        assert_eq!(Selection::new(..), Selection::All);

        use std::convert::TryFrom;
        assert!(Selection::try_from(s![-1, 2..;3, ..4]).is_err());
        assert!(Selection::try_from(ndarray::Slice::new(-1, None, 2)).is_err());
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
        let h = Hyperslab::try_new(s![0, 1..10, 2..;3])?;
        assert_eq!(
            h.as_ref().to_owned(),
            vec![
                Index(0),
                SliceTo { start: 1, step: 1, end: 10, block: 1 },
                Unlimited { start: 2, step: 3, block: 1 },
            ]
        );
        assert!(h.is_unlimited());
        assert_eq!(h.unlimited_axis(), Some(2));

        assert_err!(h.set_unlimited(0), "Cannot make index selection unlimited");
        h.set_unlimited(1)?;
        assert_err!(h.set_unlimited(3), "Invalid axis for making hyperslab unlimited: 3");
        let u = h.set_unlimited(2)?;
        assert!(u.is_unlimited());
        assert_eq!(u.unlimited_axis(), Some(2));
        assert_eq!(
            u.as_ref().to_owned(),
            vec![
                Index(0),
                SliceTo { start: 1, step: 1, end: 10, block: 1 },
                Unlimited { start: 2, step: 3, block: 1 },
            ]
        );
        u.set_unlimited(1).unwrap();
        assert_eq!(u.set_unlimited(2)?, u);

        assert_err!(u.set_block(0, 1), "Cannot set blocksize for index selection");
        assert_err!(u.set_block(3, 1), "Invalid axis for changing the slice to block-like: 3");
        let b = u.set_block(1, 2)?;
        assert_eq!(
            b.as_ref().to_owned(),
            vec![
                Index(0),
                SliceTo { start: 1, step: 1, end: 10, block: 2 },
                Unlimited { start: 2, step: 3, block: 1 },
            ]
        );
        let b = b.set_block(2, 2)?;
        assert_eq!(
            b.as_ref().to_owned(),
            vec![
                Index(0),
                SliceTo { start: 1, step: 1, end: 10, block: 2 },
                Unlimited { start: 2, step: 3, block: 2 },
            ]
        );
        assert_eq!(b.set_block(1, 2)?.set_block(2, 2)?, b);

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
        let s = Selection::try_new(s![1, 2..;2]).unwrap();
        assert_eq!(s, s);
        assert!(!s.is_all() && s.is_hyperslab() && !s.is_points() && !s.is_none());
        assert_ne!(s, Selection::new(..));
        assert_ne!(s, Selection::new(&[]));
        assert_eq!(s.in_ndim(), Some(2));
        assert_eq!(s.out_ndim(), Some(1));
        assert_eq!(s.out_shape(&[10, 20]).unwrap(), &[9]);
        assert_eq!(format!("{}", Selection::try_new(s![1]).unwrap()), "(1,)");
        assert_eq!(format!("{}", Selection::new(())), "()");

        let h = Hyperslab::try_new(s![1, 2..;3, ..4, 5..]).unwrap().set_unlimited(1).unwrap();
        assert_eq!(format!("{}", h), "(1, 2..∞;3, ..4, 5..∞)");
        let s = Selection::new(h);
        assert_eq!(format!("{}", s), "(1, 2..∞;3, ..4, 5..∞)");
        assert_eq!(s.out_shape(&[2, 3, 4, 5]).unwrap(), &[1, 4, 0]);
    }

    #[test]
    fn test_hyperslab_into_from_raw_err() {
        use std::convert::TryInto;
        #[track_caller]
        fn check<H: TryInto<Hyperslab>, S: AsRef<[Ix]>>(hyper: H, shape: S, err: &str)
        where
            H::Error: std::fmt::Debug,
        {
            let hyper = hyper.try_into().unwrap();
            assert_err!(hyper.into_raw(shape.as_ref()), err);
        }

        check(s![1, 2], &[1, 2, 3], "Slice ndim (2) != shape ndim (3)");
        assert!(Hyperslab::try_new(s![0, ..;-1]).is_err());
        check(s![0, 0], &[0, 1], "Index 0 out of bounds for axis 0 with size 0");
        check(s![.., 1], &[0, 1], "Index 1 out of bounds for axis 1 with size 1");
        assert!(Hyperslab::try_new(s![-3]).is_err());
        check(s![2], &[2], "Index 2 out of bounds for axis 0 with size 2");

        check(s![0, 3..], &[1, 2], "Slice start 3 out of bounds for axis 1 with size 2");
        assert!(Hyperslab::try_new(s![-2..;2, 0]).is_err());
        check(s![0, ..=3], &[1, 2], "Slice end 4 out of bounds for axis 1 with size 2");
        assert!(Hyperslab::try_new(s![-2..;2, 0]).is_err());

        check(
            (0, Unlimited { start: 3, step: 1, block: 1 }),
            &[1, 2],
            "Slice start 3 out of bounds for axis 1 with size 2",
        );

        assert_err!(
            Hyperslab::from_raw(vec![RawSlice::new(0, 2, Some(1), 3)].into()),
            "Blocks can not overlap (axis: 0)"
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
        use std::convert::TryInto;
        fn check<S, H, RH, RS, H2, S2>(
            shape: S, hyper: H, exp_raw_hyper: RH, exp_raw_sel: Option<RS>, exp_hyper2: H2,
            exp_sel2: Option<S2>,
        ) where
            S: AsRef<[Ix]>,
            H: TryInto<Hyperslab>,
            H::Error: std::fmt::Debug,
            RH: Into<RawHyperslab>,
            RS: Into<RawSelection>,
            H2: TryInto<Hyperslab>,
            H2::Error: std::fmt::Debug,
            S2: TryInto<Selection>,
            S2::Error: std::fmt::Debug,
        {
            let shape = shape.as_ref();
            let hyper = hyper.try_into().unwrap();
            let exp_raw_hyper = exp_raw_hyper.into();
            let exp_raw_sel = exp_raw_sel.map(Into::into).unwrap_or(exp_raw_hyper.clone().into());
            let exp_hyper2 = exp_hyper2.try_into().unwrap();
            let exp_sel2 = exp_sel2
                .map(|x| TryInto::try_into(x).unwrap())
                .unwrap_or(exp_hyper2.clone().try_into().unwrap());

            let raw_hyper = hyper.clone().into_raw(shape).unwrap();
            assert_eq!(raw_hyper, exp_raw_hyper);

            let sel = Selection::new(hyper.clone());
            let raw_sel = sel.clone().into_raw(shape).unwrap();
            assert_eq!(raw_sel, exp_raw_sel);

            let hyper2 = Hyperslab::from_raw(raw_hyper.clone()).unwrap();
            assert_eq!(hyper2, exp_hyper2);

            let sel2 = Selection::from_raw(raw_sel.clone()).unwrap();
            assert_eq!(sel2, exp_sel2);

            let raw_hyper2 = hyper2.clone().into_raw(shape).unwrap();
            assert_eq!(raw_hyper2, raw_hyper);

            let raw_sel2 = sel2.clone().into_raw(shape).unwrap();
            assert_eq!(raw_sel2, raw_sel);
        }

        check(&[], (), vec![], Some(RawSelection::All), (), Some(Selection::All));

        check(
            &[5, 5, 5],
            s![.., 0..5, ..=4],
            vec![RawSlice::new(0, 1, Some(5), 1); 3],
            Some(RawSelection::All),
            s![..5, ..5, ..5],
            Some(Selection::All),
        );

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
        );

        assert!(Hyperslab::try_new(
            s![.., ..;2, 1.., 1..;2, -3..=1, -3..=-1;2, ..=-1, ..=-1;3, 0..-1, 2..=-1]
        )
        .is_err());
        assert!(Hyperslab::try_new(
            s![.., ..;2, 1.., 1..;2, -3..=1, -3..=-1;2, ..=-1, ..=-1;3, 0..-1, 2..=-1]
        )
        .is_err());
        assert!(Hyperslab::try_new(s![-5.., -10, 1..-1;2, 1],).is_err());
        assert!(Hyperslab::try_new(s![5..10, 0..1, 1..8;2, 1..2],).is_ok());

        check(
            &[7; 7],
            Hyperslab::try_new(s![1..2;3, 1..3;3, 1..4;3, 1..5;3, 1..6;3, 1..7;3, ..7;3])?,
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
            Hyperslab::try_new(s![1..2;3, 1..2;3, 1..2;3, 1..5;3, 1..5;3, 1..5;3, ..7;3])?,
            None as Option<Selection>,
        );

        Ok(())
    }

    #[test]
    fn test_in_out_shape_ndim() -> Result<()> {
        use std::convert::TryInto;
        fn check<S: TryInto<Selection>, E: AsRef<[Ix]>>(
            sel: S, exp_in_ndim: Option<usize>, exp_out_shape: E, exp_out_ndim: Option<usize>,
        ) -> Result<()>
        where
            S::Error: std::fmt::Debug,
        {
            let in_shape = [7, 8];
            let sel = sel.try_into().unwrap();
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
        check(Hyperslab::try_new(s![1, 2..;2])?.set_block(1, 6)?, Some(2), [6], Some(1))?;
        check(Hyperslab::try_new(s![1..;3, 2])?.set_block(0, 6)?, Some(2), [6], Some(1))?;
        check(
            Hyperslab::try_new(s![1..;3, 2..;2])?.set_block(0, 6)?.set_block(1, 6)?,
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
        use std::convert::TryInto;
        fn check<Sh, S, RS, S2>(
            shape: Sh, sel: S, exp_raw_sel: RS, exp_sel2: Option<S2>,
        ) -> Result<()>
        where
            Sh: AsRef<[Ix]>,
            S: TryInto<Selection>,
            S::Error: std::fmt::Debug,
            RS: Into<RawSelection>,
            S2: TryInto<Selection>,
            S2::Error: std::fmt::Debug,
        {
            let shape = shape.as_ref();
            let sel = sel.try_into().unwrap();
            let exp_raw_sel = exp_raw_sel.into();
            let exp_sel2 = exp_sel2.map_or(sel.clone(), |x| x.try_into().unwrap());

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

        check(&[1, 2], RawSelection::None, None)?;
        check(&[1, 2], RawSelection::All, None)?;
        check(&[1, 2], RawSelection::Points(arr2(&[[0, 1], [0, 0]])), None)?;

        let exp =
            if cfg!(feature = "1.10.0") { None } else { Some(RawSelection::ComplexHyperslab) };
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

    #[test]
    fn use_selection_on_dataset() {
        with_tmp_file(|file| {
            let ds = file.new_dataset::<u8>().shape((5, 5)).create("ds_fixed").unwrap();
            assert_eq!(&ds.shape(), &[5, 5]);
            let ds = file.new_dataset::<u8>().shape((0.., 0..)).create("ds_twounlim").unwrap();
            assert_eq!(&ds.shape(), &[0, 0]);
            ds.resize((5, 5)).unwrap();
            assert_eq!(&ds.shape(), &[5, 5]);
            let ds = file.new_dataset::<u8>().shape((5, 0..)).create("ds_oneunlim0").unwrap();
            assert_eq!(&ds.shape(), &[5, 0]);
            ds.resize((5, 5)).unwrap();
            assert_eq!(&ds.shape(), &[5, 5]);
            let ds = file.new_dataset::<u8>().shape((0.., 5)).create("ds_oneunlim1").unwrap();
            assert_eq!(&ds.shape(), &[0, 5]);
            ds.resize((5, 5)).unwrap();
            assert_eq!(&ds.shape(), &[5, 5]);
        })
    }
}

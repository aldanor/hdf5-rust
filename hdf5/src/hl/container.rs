use std::convert::TryInto;
use std::fmt::{self, Debug};
use std::io;
use std::mem;
use std::ops::Deref;

use ndarray::{Array, Array1, Array2, ArrayD, ArrayView, ArrayView1};

use hdf5_sys::h5a::{H5Aget_space, H5Aget_storage_size, H5Aget_type, H5Aread, H5Awrite};
use hdf5_sys::h5d::{H5Dget_space, H5Dget_storage_size, H5Dget_type, H5Dread, H5Dwrite};
use hdf5_sys::h5p::H5Pcreate;

use crate::internal_prelude::*;

#[derive(Debug)]
pub struct Reader<'a> {
    obj: &'a Container,
    conv: Conversion,
}

impl<'a> Reader<'a> {
    /// Creates a reader for a dataset/attribute.
    ///
    /// Any conversions (including hard/soft) are allowed by default.
    pub fn new(obj: &'a Container) -> Self {
        Self { obj, conv: Conversion::Soft }
    }

    /// Set maximum allowed conversion level.
    pub fn conversion(mut self, conv: Conversion) -> Self {
        self.conv = conv;
        self
    }

    /// Disallow all conversions.
    pub fn no_convert(mut self) -> Self {
        self.conv = Conversion::NoOp;
        self
    }

    fn read_into_buf<T: H5Type>(
        &self, buf: *mut T, fspace: Option<&Dataspace>, mspace: Option<&Dataspace>,
    ) -> Result<()> {
        let file_dtype = self.obj.dtype()?;
        let mem_dtype = Datatype::from_type::<T>()?;
        file_dtype.ensure_convertible(&mem_dtype, self.conv)?;
        let (obj_id, tp_id) = (self.obj.id(), mem_dtype.id());

        if self.obj.is_attr() {
            h5try!(H5Aread(obj_id, tp_id, buf.cast()));
        } else {
            let fspace_id = fspace.map_or(H5S_ALL, |f| f.id());
            let mspace_id = mspace.map_or(H5S_ALL, |m| m.id());
            let xfer =
                PropertyList::from_id(h5call!(H5Pcreate(*crate::globals::H5P_DATASET_XFER))?)?;
            if !hdf5_types::USING_H5_ALLOCATOR {
                crate::hl::plist::set_vlen_manager_libc(xfer.id())?;
            }
            h5try!(H5Dread(obj_id, tp_id, mspace_id, fspace_id, xfer.id(), buf.cast()));
        }
        Ok(())
    }

    /// Reads a slice of an n-dimensional array.
    /// If the dimensionality `D` has a fixed number of dimensions, it must match the dimensionality of
    /// the slice, after singleton dimensions are dropped.
    /// Use the multi-dimensional slice macro `s![]` from `ndarray` to conveniently create
    /// a multidimensional slice.
    pub fn read_slice<T, S, D>(&self, selection: S) -> Result<Array<T, D>>
    where
        T: H5Type,
        S: TryInto<Selection>,
        Error: From<S::Error>,
        D: ndarray::Dimension,
    {
        ensure!(!self.obj.is_attr(), "Slicing cannot be used on attribute datasets");

        let selection = selection.try_into()?;
        let obj_space = self.obj.space()?;

        let out_shape = selection.out_shape(obj_space.shape())?;
        let out_size: Ix = out_shape.iter().product();
        let fspace = obj_space.select(selection)?;

        if let Some(ndim) = D::NDIM {
            let out_ndim = out_shape.len();
            ensure!(ndim == out_ndim, "Selection ndim ({}) != array ndim ({})", out_ndim, ndim);
        } else {
            let fsize = fspace.selection_size();
            ensure!(
                out_size == fsize,
                "Selected size mismatch: {} != {} (shouldn't happen)",
                out_size,
                fsize
            );
        }

        if out_size == 0 {
            Ok(unsafe { Array::from_shape_vec_unchecked(out_shape, vec![]).into_dimensionality()? })
        } else if obj_space.ndim() == 0 {
            self.read()
        } else {
            let mspace = Dataspace::try_new(&out_shape)?;
            let mut buf = Vec::with_capacity(out_size);
            self.read_into_buf(buf.as_mut_ptr(), Some(&fspace), Some(&mspace))?;
            unsafe {
                buf.set_len(out_size);
            };
            let arr = ArrayD::from_shape_vec(out_shape, buf)?;
            Ok(arr.into_dimensionality()?)
        }
    }

    /// Reads a dataset/attribute into an n-dimensional array.
    ///
    /// If the array has a fixed number of dimensions, it must match the dimensionality
    /// of the dataset/attribute.
    pub fn read<T: H5Type, D: ndarray::Dimension>(&self) -> Result<Array<T, D>> {
        let shape = self.obj.get_shape()?;
        if let Some(ndim) = D::NDIM {
            let obj_ndim = shape.ndim();
            ensure!(obj_ndim == ndim, "ndim mismatch: expected {}, got {}", ndim, obj_ndim);
        }
        let vec = self.read_raw()?;
        let arr = ArrayD::from_shape_vec(shape, vec)?;
        Ok(arr.into_dimensionality()?)
    }

    /// Reads a dataset/attribute into a vector in memory order.
    pub fn read_raw<T: H5Type>(&self) -> Result<Vec<T>> {
        let size = self.obj.space()?.size();
        let mut vec = Vec::with_capacity(size);
        self.read_into_buf(vec.as_mut_ptr(), None, None).map(|_| {
            unsafe {
                vec.set_len(size);
            };
            vec
        })
    }

    /// Reads a dataset/attribute into a 1-dimensional array.
    ///
    /// The dataset/attribute must be 1-dimensional.
    pub fn read_1d<T: H5Type>(&self) -> Result<Array1<T>> {
        self.read()
    }

    /// Reads the given `slice` of the dataset into a 1-dimensional array.
    /// The slice must yield a 1-dimensional result.
    pub fn read_slice_1d<T, S>(&self, selection: S) -> Result<Array1<T>>
    where
        T: H5Type,
        S: TryInto<Selection>,
        Error: From<S::Error>,
    {
        self.read_slice(selection)
    }

    /// Reads a dataset/attribute into a 2-dimensional array.
    ///
    /// The dataset/attribute must be 2-dimensional.
    pub fn read_2d<T: H5Type>(&self) -> Result<Array2<T>> {
        self.read()
    }

    /// Reads the given `slice` of the dataset into a 2-dimensional array.
    /// The slice must yield a 2-dimensional result.
    pub fn read_slice_2d<T, S>(&self, selection: S) -> Result<Array2<T>>
    where
        T: H5Type,
        S: TryInto<Selection>,
        Error: From<S::Error>,
    {
        self.read_slice(selection)
    }

    /// Reads a dataset/attribute into an array with dynamic number of dimensions.
    pub fn read_dyn<T: H5Type>(&self) -> Result<ArrayD<T>> {
        self.read()
    }

    /// Reads a scalar dataset/attribute.
    pub fn read_scalar<T: H5Type>(&self) -> Result<T> {
        let obj_ndim = self.obj.get_shape()?.ndim();
        ensure!(obj_ndim == 0, "ndim mismatch: expected scalar, got {}", obj_ndim);
        let mut val = mem::MaybeUninit::<T>::uninit();
        self.read_into_buf(val.as_mut_ptr(), None, None).map(|_| unsafe { val.assume_init() })
    }
}

#[derive(Debug)]
pub struct Writer<'a> {
    obj: &'a Container,
    conv: Conversion,
}

impl<'a> Writer<'a> {
    /// Creates a writer for a dataset/attribute.
    ///
    /// Any conversions (including hard/soft) are allowed by default.
    pub fn new(obj: &'a Container) -> Self {
        Self { obj, conv: Conversion::Soft }
    }

    /// Set maximum allowed conversion level.
    pub fn conversion(mut self, conv: Conversion) -> Self {
        self.conv = conv;
        self
    }

    /// Disallow all conversions.
    pub fn no_convert(mut self) -> Self {
        self.conv = Conversion::NoOp;
        self
    }

    fn write_from_buf<T: H5Type>(
        &self, buf: *const T, fspace: Option<&Dataspace>, mspace: Option<&Dataspace>,
    ) -> Result<()> {
        let file_dtype = self.obj.dtype()?;
        let mem_dtype = Datatype::from_type::<T>()?;
        mem_dtype.ensure_convertible(&file_dtype, self.conv)?;
        let (obj_id, tp_id) = (self.obj.id(), mem_dtype.id());

        if self.obj.is_attr() {
            h5try!(H5Awrite(obj_id, tp_id, buf.cast()));
        } else {
            let fspace_id = fspace.map_or(H5S_ALL, |f| f.id());
            let mspace_id = mspace.map_or(H5S_ALL, |m| m.id());
            h5try!(H5Dwrite(obj_id, tp_id, mspace_id, fspace_id, H5P_DEFAULT, buf.cast()));
        }
        Ok(())
    }

    /// Writes all data from the array `arr` into the given `slice` of the target dataset.
    /// The shape of `arr` must match the shape the set of elements included in the slice.
    /// If the array has a fixed number of dimensions, it must match the dimensionality of
    /// dataset. Use the multi-dimensional slice macro `s![]` from `ndarray` to conveniently create
    /// a multidimensional slice.
    pub fn write_slice<'b, A, T, S, D>(&self, arr: A, selection: S) -> Result<()>
    where
        A: Into<ArrayView<'b, T, D>>,
        T: H5Type,
        S: TryInto<Selection>,
        Error: From<S::Error>,
        D: ndarray::Dimension,
    {
        ensure!(!self.obj.is_attr(), "Slicing cannot be used on attribute datasets");

        let selection = selection.try_into()?;
        let obj_space = self.obj.space()?;

        let out_shape = selection.out_shape(obj_space.shape())?;
        let out_size: Ix = out_shape.iter().product();
        let fspace = obj_space.select(selection)?;
        let view = arr.into();

        if let Some(ndim) = D::NDIM {
            let out_ndim = out_shape.len();
            ensure!(ndim == out_ndim, "Selection ndim ({}) != array ndim ({})", out_ndim, ndim);
        } else {
            let fsize = fspace.selection_size();
            ensure!(
                out_size == fsize,
                "Selected size mismatch: {} != {} (shouldn't happen)",
                out_size,
                fsize
            );
            ensure!(
                view.shape() == out_shape.as_slice(),
                "Shape mismatch: memory ({:?}) != destination ({:?})",
                view.shape(),
                out_shape
            );
        }

        if out_size == 0 {
            Ok(())
        } else if obj_space.ndim() == 0 {
            self.write(view)
        } else {
            let mspace = Dataspace::try_new(view.shape())?;
            // TODO: support strided arrays (C-ordering we have to require regardless)
            ensure!(
                view.is_standard_layout(),
                "Input array is not in standard layout or non-contiguous"
            );

            self.write_from_buf(view.as_ptr(), Some(&fspace), Some(&mspace))
        }
    }

    /// Writes an n-dimensional array view into a dataset/attribute.
    ///
    /// The shape of the view must match the shape of the dataset/attribute exactly.
    /// The input argument must be convertible to an array view (this includes slices).
    pub fn write<'b, A, T, D>(&self, arr: A) -> Result<()>
    where
        A: Into<ArrayView<'b, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        let view = arr.into();
        ensure!(
            view.is_standard_layout(),
            "input array is not in standard layout or is not contiguous"
        );

        let src = view.shape();
        let dst = &*self.obj.get_shape()?;
        if src != dst {
            fail!("shape mismatch when writing: memory = {:?}, destination = {:?}", src, dst);
        }

        self.write_from_buf(view.as_ptr(), None, None)
    }

    /// Writes a 1-dimensional array view into a dataset/attribute in memory order.
    ///
    /// The number of elements in the view must match the number of elements in the
    /// destination dataset/attribute. The input argument must be convertible to a
    /// 1-dimensional array view (this includes slices).
    pub fn write_raw<'b, A, T>(&self, arr: A) -> Result<()>
    where
        A: Into<ArrayView1<'b, T>>,
        T: H5Type,
    {
        let view = arr.into();
        ensure!(
            view.is_standard_layout(),
            "input array is not in standard layout or is not contiguous"
        );

        let src = view.len();
        let dst = self.obj.get_shape()?.size();
        if src != dst {
            fail!("length mismatch when writing: memory = {:?}, destination = {:?}", src, dst);
        }
        self.write_from_buf(view.as_ptr(), None, None)
    }

    /// Writes a scalar dataset/attribute.
    pub fn write_scalar<T: H5Type>(&self, val: &T) -> Result<()> {
        let ndim = self.obj.get_shape()?.ndim();
        ensure!(ndim == 0, "ndim mismatch: expected scalar, got {}", ndim);
        self.write_from_buf(val as *const _, None, None)
    }
}

#[derive(Debug, Clone)]
pub struct ByteReader {
    obj: Container,
    pos: u64,
    dt: Datatype,
    obj_space: Dataspace,
    xfer: PropertyList,
}

impl ByteReader {
    pub fn new(obj: &Container) -> Result<Self> {
        ensure!(!obj.is_attr(), "ByteReader cannot be used on attribute datasets");

        let obj = obj.clone();
        let file_dtype = obj.dtype()?;
        let mem_dtype = Datatype::from_type::<u8>()?;
        file_dtype.ensure_convertible(&mem_dtype, Conversion::NoOp)?;

        let obj_space = obj.space()?;
        ensure!(obj_space.shape().len() == 1, "Only rank 1 datasets can be read via ByteReader");
        let xfer = PropertyList::from_id(h5call!(H5Pcreate(*crate::globals::H5P_DATASET_XFER))?)?;
        if !hdf5_types::USING_H5_ALLOCATOR {
            crate::hl::plist::set_vlen_manager_libc(xfer.id())?;
        }
        Ok(Self { obj, pos: 0, obj_space, dt: mem_dtype, xfer })
    }

    fn dataset_len(&self) -> usize {
        self.obj_space.shape()[0]
    }

    fn remaining_len(&self) -> usize {
        self.dataset_len().saturating_sub(self.pos as usize)
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.dataset_len() as u64
    }
}

impl io::Read for ByteReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let pos = self.pos as usize;
        let amt = std::cmp::min(buf.len(), self.remaining_len());
        let selection = Selection::new(pos..pos + amt);
        let out_shape = selection.out_shape(self.obj_space.shape())?;
        let fspace = self.obj_space.select(selection)?;
        let mspace = Dataspace::try_new(&out_shape)?;
        h5call!(H5Dread(
            self.obj.id(),
            self.dt.id(),
            mspace.id(),
            fspace.id(),
            self.xfer.id(),
            buf.as_mut_ptr().cast()
        ))?;
        self.pos += amt as u64;
        Ok(out_shape[0])
    }
}

impl io::Seek for ByteReader {
    fn seek(&mut self, style: io::SeekFrom) -> io::Result<u64> {
        let (base_pos, offset) = match style {
            io::SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            io::SeekFrom::End(n) => (self.dataset_len() as u64, n),
            io::SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset.is_negative() {
            base_pos.checked_sub(offset.wrapping_abs() as u64)
        } else {
            base_pos.checked_add(offset as u64)
        };
        match new_pos {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid seek to a negative or overflowing position",
            )),
        }
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.pos)
    }
}

#[repr(transparent)]
#[derive(Clone)]
/// An object which can be read or written to.
pub struct Container(Handle);

impl ObjectClass for Container {
    const NAME: &'static str = "container";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_DATASET, H5I_ATTR];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }
}

impl Debug for Container {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Container {
    type Target = Location;

    fn deref(&self) -> &Location {
        unsafe { self.transmute() }
    }
}

impl Container {
    pub(crate) fn is_attr(&self) -> bool {
        self.handle().id_type() == H5I_ATTR
    }

    /// Creates a reader wrapper for this dataset/attribute, allowing to
    /// set custom type conversion options when reading.
    pub fn as_reader(&self) -> Reader {
        Reader::new(self)
    }

    /// Creates a writer wrapper for this dataset/attribute, allowing to
    /// set custom type conversion options when writing.
    pub fn as_writer(&self) -> Writer {
        Writer::new(self)
    }

    /// Creates `ByteReader` which implements [`Read`](std::io::Read)
    /// and [`Seek`](std::io::Seek).
    ///
    /// ``ByteReader`` only supports 1-D `u8` datasets.
    pub fn as_byte_reader(&self) -> Result<ByteReader> {
        ByteReader::new(self)
    }

    /// Returns the datatype of the dataset/attribute.
    pub fn dtype(&self) -> Result<Datatype> {
        if self.is_attr() {
            Datatype::from_id(h5try!(H5Aget_type(self.id())))
        } else {
            Datatype::from_id(h5try!(H5Dget_type(self.id())))
        }
    }

    /// Returns the dataspace of the dataset/attribute.
    pub fn space(&self) -> Result<Dataspace> {
        if self.is_attr() {
            Dataspace::from_id(h5try!(H5Aget_space(self.id())))
        } else {
            Dataspace::from_id(h5try!(H5Dget_space(self.id())))
        }
    }

    #[doc(hidden)]
    pub fn get_shape(&self) -> Result<Vec<Ix>> {
        self.space().map(|s| s.shape())
    }

    /// Returns the shape of the dataset/attribute.
    pub fn shape(&self) -> Vec<Ix> {
        self.space().ok().map_or_else(Vec::new, |s| s.shape())
    }

    /// Returns the number of dimensions in the dataset/attribute.
    pub fn ndim(&self) -> usize {
        self.space().ok().map_or(0, |s| s.ndim())
    }

    /// Returns the total number of elements in the dataset/attribute.
    pub fn size(&self) -> usize {
        self.shape().iter().product()
    }

    /// Returns whether this dataset/attribute is a scalar.
    pub fn is_scalar(&self) -> bool {
        self.space().ok().map_or(false, |s| s.is_scalar())
    }

    /// Returns the amount of file space required for the dataset/attribute. Note that this
    /// only accounts for the space which has actually been allocated (it can be equal to zero).
    pub fn storage_size(&self) -> u64 {
        if self.is_attr() {
            h5lock!(H5Aget_storage_size(self.id())) as _
        } else {
            h5lock!(H5Dget_storage_size(self.id())) as _
        }
    }

    /// Reads a dataset/attribute into an n-dimensional array.
    ///
    /// If the array has a fixed number of dimensions, it must match the dimensionality
    /// of the dataset/attribute.
    pub fn read<T: H5Type, D: ndarray::Dimension>(&self) -> Result<Array<T, D>> {
        self.as_reader().read()
    }

    /// Reads a dataset/attribute into a vector in memory order.
    pub fn read_raw<T: H5Type>(&self) -> Result<Vec<T>> {
        self.as_reader().read_raw()
    }

    /// Reads a dataset/attribute into a 1-dimensional array.
    ///
    /// The dataset/attribute must be 1-dimensional.
    pub fn read_1d<T: H5Type>(&self) -> Result<Array1<T>> {
        self.as_reader().read_1d()
    }

    /// Reads the given `slice` of the dataset into a 1-dimensional array.
    /// The slice must yield a 1-dimensional result.
    pub fn read_slice_1d<T, S>(&self, selection: S) -> Result<Array1<T>>
    where
        T: H5Type,
        S: TryInto<Selection>,
        Error: From<S::Error>,
    {
        self.as_reader().read_slice_1d(selection)
    }

    /// Reads a dataset/attribute into a 2-dimensional array.
    ///
    /// The dataset/attribute must be 2-dimensional.
    pub fn read_2d<T: H5Type>(&self) -> Result<Array2<T>> {
        self.as_reader().read_2d()
    }

    /// Reads the given `slice` of the dataset into a 2-dimensional array.
    /// The slice must yield a 2-dimensional result.
    pub fn read_slice_2d<T, S>(&self, selection: S) -> Result<Array2<T>>
    where
        T: H5Type,
        S: TryInto<Selection>,
        Error: From<S::Error>,
    {
        self.as_reader().read_slice_2d(selection)
    }

    /// Reads a dataset/attribute into an array with dynamic number of dimensions.
    pub fn read_dyn<T: H5Type>(&self) -> Result<ArrayD<T>> {
        self.as_reader().read_dyn()
    }

    /// Reads a slice of an n-dimensional array.
    /// If the dimensionality `D` has a fixed number of dimensions, it must match the dimensionality of
    /// the slice, after singleton dimensions are dropped.
    /// Use the multi-dimensional slice macro `s![]` from `ndarray` to conveniently create
    /// a multidimensional slice.
    pub fn read_slice<T, S, D>(&self, selection: S) -> Result<Array<T, D>>
    where
        T: H5Type,
        S: TryInto<Selection>,
        Error: From<S::Error>,
        D: ndarray::Dimension,
    {
        self.as_reader().read_slice(selection)
    }

    /// Reads a scalar dataset/attribute.
    pub fn read_scalar<T: H5Type>(&self) -> Result<T> {
        self.as_reader().read_scalar()
    }

    /// Writes an n-dimensional array view into a dataset/attribute.
    ///
    /// The shape of the view must match the shape of the dataset/attribute exactly.
    /// The input argument must be convertible to an array view (this includes slices).
    pub fn write<'b, A, T, D>(&self, arr: A) -> Result<()>
    where
        A: Into<ArrayView<'b, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        self.as_writer().write(arr)
    }

    /// Writes a 1-dimensional array view into a dataset/attribute in memory order.
    ///
    /// The number of elements in the view must match the number of elements in the
    /// destination dataset/attribute. The input argument must be convertible to a
    /// 1-dimensional array view (this includes slices).
    pub fn write_raw<'b, A, T>(&self, arr: A) -> Result<()>
    where
        A: Into<ArrayView1<'b, T>>,
        T: H5Type,
    {
        self.as_writer().write_raw(arr)
    }

    /// Writes all data from the array `arr` into the given `slice` of the target dataset.
    /// The shape of `arr` must match the shape the set of elements included in the slice.
    /// If the array has a fixed number of dimensions, it must match the dimensionality of
    /// dataset. Use the multi-dimensional slice macro `s![]` from `ndarray` to conveniently create
    /// a multidimensional slice.
    pub fn write_slice<'b, A, T, S, D>(&self, arr: A, selection: S) -> Result<()>
    where
        A: Into<ArrayView<'b, T, D>>,
        T: H5Type,
        S: TryInto<Selection>,
        Error: From<S::Error>,
        D: ndarray::Dimension,
    {
        self.as_writer().write_slice(arr, selection)
    }

    /// Writes a scalar dataset/attribute.
    pub fn write_scalar<T: H5Type>(&self, val: &T) -> Result<()> {
        self.as_writer().write_scalar(val)
    }
}

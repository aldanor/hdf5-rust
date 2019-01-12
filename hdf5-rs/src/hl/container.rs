use std::fmt::{self, Debug};
use std::mem;
use std::ops::Deref;

use ndarray::{Array, Array1, Array2, ArrayD, ArrayView, ArrayView1, Ix1, Ix2};
use ndarray::{SliceInfo, SliceOrIndex};

use libhdf5_sys::h5a::{H5Aget_space, H5Aget_storage_size, H5Aget_type, H5Aread, H5Awrite};
use libhdf5_sys::h5d::{H5Dget_space, H5Dget_storage_size, H5Dget_type, H5Dread, H5Dwrite};

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

        let fspace_id = fspace.map_or(H5S_ALL, |f| f.id());
        let mspace_id = mspace.map_or(H5S_ALL, |m| m.id());

        if self.obj.is_attr() {
            h5try!(H5Aread(obj_id, tp_id, buf as *mut _));
        } else {
            h5try!(H5Dread(obj_id, tp_id, mspace_id, fspace_id, H5P_DEFAULT, buf as *mut _));
        }
        Ok(())
    }

    /// Reads a slice of an n-dimensional array.
    /// If the array has a fixed number of dimensions, it must match the dimensionality of
    /// dataset. Use the multi-dimensional slice macro `s![]` from `ndarray` to conveniently create
    /// a multidimensional slice.
    pub fn read_slice<T, S, D>(&self, slice: &SliceInfo<S, D>) -> Result<Array<T, D>>
    where
        T: H5Type,
        S: AsRef<[SliceOrIndex]>,
        D: ndarray::Dimension,
    {
        ensure!(!self.obj.is_attr(), "slicing cannot be used on attribute datasets");

        let shape = self.obj.get_shape()?;
        if let Some(ndim) = D::NDIM {
            let obj_ndim = shape.ndim();
            ensure!(obj_ndim == ndim, "ndim mismatch: expected {}, got {}", ndim, obj_ndim);
        }

        let slice_s: &[SliceOrIndex] = slice.as_ref();
        let slice_dim = slice_s.len();
        if shape.ndim() != slice_dim {
            let obj_ndim = shape.ndim();
            ensure!(
                obj_ndim == slice_dim,
                "slice dimension mismatch: dataset has {} dims, slice has {} dims",
                obj_ndim,
                slice_dim
            );
        }

        if shape.ndim() == 0 {
            // Fall back to a simple read for the scalar case, slicing has no effect
            self.read()
        } else {
            let fspace = self.obj.space()?;
            let out_shape = fspace.select_slice(slice)?;

            // Remove dimensions from out_shape that were SliceOrIndex::Index in the slice
            let reduced_shape: Vec<_> = slice_s
                .iter()
                .zip(out_shape.iter().cloned())
                .filter_map(|(slc, sz)| match slc {
                    SliceOrIndex::Index(_) => None,
                    _ => Some(sz),
                })
                .collect();

            let mspace = Dataspace::try_new(&out_shape, false)?;
            let size = out_shape.iter().product();
            let mut vec = Vec::with_capacity(size);
            unsafe {
                vec.set_len(size);
            }

            self.read_into_buf(vec.as_mut_ptr(), Some(&fspace), Some(&mspace))?;

            let arr = ArrayD::from_shape_vec(reduced_shape, vec).str_err()?;
            arr.into_dimensionality().str_err()
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
        let arr = ArrayD::from_shape_vec(shape, vec).str_err()?;
        arr.into_dimensionality().str_err()
    }

    /// Reads a dataset/attribute into a vector in memory order.
    pub fn read_raw<T: H5Type>(&self) -> Result<Vec<T>> {
        let size = self.obj.space()?.size();
        let mut vec = Vec::with_capacity(size);
        unsafe {
            vec.set_len(size);
        }
        self.read_into_buf(vec.as_mut_ptr(), None, None).map(|_| vec)
    }

    /// Reads a dataset/attribute into a 1-dimensional array.
    ///
    /// The dataset/attribute must be 1-dimensional.
    pub fn read_1d<T: H5Type>(&self) -> Result<Array1<T>> {
        self.read()
    }

    /// Reads the given `slice` of the dataset into a 1-dimensional array.
    ///
    /// The dataset must be 1-dimensional.
    pub fn read_slice_1d<T, S>(&self, slice: &SliceInfo<S, Ix1>) -> Result<Array1<T>>
    where
        T: H5Type,
        S: AsRef<[SliceOrIndex]>,
    {
        self.read_slice(slice)
    }

    /// Reads a dataset/attribute into a 2-dimensional array.
    ///
    /// The dataset/attribute must be 2-dimensional.
    pub fn read_2d<T: H5Type>(&self) -> Result<Array2<T>> {
        self.read()
    }

    /// Reads the given `slice` of the dataset into a 2-dimensional array.
    ///
    /// The dataset must be 2-dimensional.
    pub fn read_slice_2d<T, S>(&self, slice: &SliceInfo<S, Ix2>) -> Result<Array2<T>>
    where
        T: H5Type,
        S: AsRef<[SliceOrIndex]>,
    {
        self.read_slice(slice)
    }

    /// Reads a dataset/attribute into an array with dynamic number of dimensions.
    pub fn read_dyn<T: H5Type>(&self) -> Result<ArrayD<T>> {
        self.read()
    }

    /// Reads a scalar dataset/attribute.
    pub fn read_scalar<T: H5Type>(&self) -> Result<T> {
        let obj_ndim = self.obj.get_shape()?.ndim();
        ensure!(obj_ndim == 0, "ndim mismatch: expected scalar, got {}", obj_ndim);
        let mut val: T = unsafe { mem::uninitialized() };
        self.read_into_buf(&mut val as *mut _, None, None).map(|_| val)
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

        let fspace_id = fspace.map_or(H5S_ALL, |f| f.id());
        let mspace_id = mspace.map_or(H5S_ALL, |m| m.id());

        if self.obj.is_attr() {
            h5try!(H5Awrite(obj_id, tp_id, buf as *const _));
        } else {
            h5try!(H5Dwrite(obj_id, tp_id, mspace_id, fspace_id, H5P_DEFAULT, buf as *const _));
        }
        Ok(())
    }

    /// Writes all data from the array `arr` into the given `slice` of the target dataset.
    /// The shape of `arr` must match the shape the set of elements included in the slice.
    /// If the array has a fixed number of dimensions, it must match the dimensionality of
    /// dataset. Use the multi-dimensional slice macro `s![]` from `ndarray` to conveniently create
    /// a multidimensional slice.
    pub fn write_slice<'b, A, T, S, D>(&self, arr: A, slice: &SliceInfo<S, D>) -> Result<()>
    where
        A: Into<ArrayView<'b, T, D>>,
        T: H5Type,
        S: AsRef<[SliceOrIndex]>,
        D: ndarray::Dimension,
    {
        ensure!(!self.obj.is_attr(), "slicing cannot be used on attribute datasets");

        let shape = self.obj.get_shape()?;
        if let Some(ndim) = D::NDIM {
            let obj_ndim = shape.ndim();
            ensure!(obj_ndim == ndim, "ndim mismatch: expected {}, got {}", ndim, obj_ndim);
        }

        let slice_s: &[SliceOrIndex] = slice.as_ref();
        let slice_dim = slice_s.len();
        if shape.ndim() != slice_dim {
            let obj_ndim = shape.ndim();
            ensure!(
                obj_ndim == slice_dim,
                "slice dimension mismatch: dataset has {} dims, slice has {} dims",
                obj_ndim,
                slice_dim
            );
        }

        if shape.ndim() == 0 {
            // Fall back to a simple read for the scalar case
            // Slicing has no effect
            self.write(arr)
        } else {
            let fspace = self.obj.space()?;
            let slice_shape = fspace.select_slice(slice)?;

            let view = arr.into();
            let data_shape = view.shape();

            // Restore dimensions that are SliceOrIndex::Index in the slice.
            let mut data_shape_hydrated = Vec::new();
            let mut pos = 0;
            for s in slice_s {
                if let SliceOrIndex::Index(_) = s {
                    data_shape_hydrated.push(1);
                } else {
                    data_shape_hydrated.push(data_shape[pos]);
                    pos += 1;
                }
            }

            let mspace = Dataspace::try_new(&slice_shape, false)?;

            // FIXME - we can handle non-standard input arrays by creating a memory space
            // that reflects the same slicing/ordering that this ArrayView represents.
            // we could also convert the array into a standard layout, but this is probably expensive.
            ensure!(
                view.is_standard_layout(),
                "input array is not in standard layout or is not contiguous"
            );

            if slice_shape != data_shape_hydrated {
                fail!(
                    "shape mismatch when writing slice: memory = {:?}, destination = {:?}",
                    data_shape_hydrated,
                    slice_shape
                );
            }

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

#[repr(transparent)]
pub struct Container(Handle);

impl ObjectClass for Container {
    const NAME: &'static str = "container";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_DATASET, H5I_ATTR];

    fn from_handle(handle: Handle) -> Self {
        Container(handle)
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
        get_id_type(self.id()) == H5I_ATTR
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
        self.space().map(|s| s.dims())
    }

    /// Returns the shape of the dataset/attribute.
    pub fn shape(&self) -> Vec<Ix> {
        self.space().ok().map_or_else(Vec::new, |s| s.dims())
    }

    /// Returns the number of dimensions in the dataset/attribute.
    pub fn ndim(&self) -> usize {
        self.space().ok().map_or(0, |s| s.ndim())
    }

    /// Returns the total number of elements in the dataset/attribute.
    pub fn size(&self) -> usize {
        self.shape().size()
    }

    /// Returns whether this dataset/attribute is a scalar.
    pub fn is_scalar(&self) -> bool {
        self.ndim() == 0
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

    /// Reads a dataset/attribute into a 2-dimensional array.
    ///
    /// The dataset/attribute must be 2-dimensional.
    pub fn read_2d<T: H5Type>(&self) -> Result<Array2<T>> {
        self.as_reader().read_2d()
    }

    /// Reads a dataset/attribute into an array with dynamic number of dimensions.
    pub fn read_dyn<T: H5Type>(&self) -> Result<ArrayD<T>> {
        self.as_reader().read_dyn()
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

    /// Writes a scalar dataset/attribute.
    pub fn write_scalar<T: H5Type>(&self, val: &T) -> Result<()> {
        self.as_writer().write_scalar(val)
    }
}

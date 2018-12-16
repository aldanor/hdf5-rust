use std::cell::Cell;
use std::fmt::{self, Debug};
use std::mem;
use std::ops::Deref;

use ndarray::{Array, Array1, Array2, ArrayD, ArrayView};

use libhdf5_sys::h5a::{H5Aget_space, H5Aget_storage_size, H5Aget_type, H5Aread, H5Awrite};
use libhdf5_sys::h5d::{H5Dget_space, H5Dget_storage_size, H5Dget_type, H5Dread, H5Dwrite};

use crate::internal_prelude::*;

#[derive(Debug)]
pub struct Reader<'a> {
    obj: &'a Container,
    conv: Cell<Conversion>,
}

impl<'a> Reader<'a> {
    pub fn new(obj: &'a Container) -> Self {
        Self { obj, conv: Cell::new(Conversion::NoOp) }
    }

    pub fn noop(&self) -> &Self {
        self.conv.replace(Conversion::NoOp);
        self
    }

    pub fn hard(&self) -> &Self {
        self.conv.replace(Conversion::Hard);
        self
    }

    pub fn soft(&self) -> &Self {
        self.conv.replace(Conversion::Soft);
        self
    }

    fn read_into_buf<T: H5Type>(&self, buf: *mut T) -> Result<()> {
        let file_dtype = self.obj.dtype()?;
        let mem_dtype = Datatype::from_type::<T>()?;
        file_dtype.ensure_convertible(&mem_dtype, self.conv.get())?;
        let (obj_id, tp_id) = (self.obj.id(), mem_dtype.id());
        if self.obj.is_attr() {
            h5try!(H5Aread(obj_id, tp_id, buf as *mut _));
        } else {
            h5try!(H5Dread(obj_id, tp_id, H5S_ALL, H5S_ALL, H5P_DEFAULT, buf as *mut _));
        }
        Ok(())
    }

    pub fn read_vec<T: H5Type>(&self) -> Result<Vec<T>> {
        let size = self.obj.space()?.size();
        let mut vec = Vec::with_capacity(size);
        unsafe {
            vec.set_len(size);
        }
        self.read_into_buf(vec.as_mut_ptr()).map(|_| vec)
    }

    pub fn read_arr<T: H5Type, D: ndarray::Dimension>(&self) -> Result<Array<T, D>> {
        let shape = self.obj.get_shape()?;
        if let Some(ndim) = D::NDIM {
            ensure!(shape.ndim() == ndim, "ndim mismatch");
        }
        let vec = self.read_vec::<T>()?;
        let arr = ArrayD::from_shape_vec(shape, vec).str_err()?;
        arr.into_dimensionality().str_err()
    }

    pub fn read_dyn<T: H5Type>(&self) -> Result<ArrayD<T>> {
        self.read_arr()
    }

    pub fn read_1d<T: H5Type>(&self) -> Result<Array1<T>> {
        self.read_arr()
    }

    pub fn read_2d<T: H5Type>(&self) -> Result<Array2<T>> {
        self.read_arr()
    }

    pub fn read_scalar<T: H5Type>(&self) -> Result<T> {
        ensure!(self.obj.get_shape()?.ndim() == 0, "scalar expected");
        let mut val: T = unsafe { mem::uninitialized() };
        self.read_into_buf(&mut val as *mut _).map(|_| val)
    }
}

#[derive(Debug)]
pub struct Writer<'a> {
    obj: &'a Container,
    conv: Cell<Conversion>,
}

impl<'a> Writer<'a> {
    pub fn new(obj: &'a Container) -> Self {
        Self { obj, conv: Cell::new(Conversion::NoOp) }
    }

    pub fn noop(&self) -> &Self {
        self.conv.replace(Conversion::NoOp);
        self
    }

    pub fn hard(&self) -> &Self {
        self.conv.replace(Conversion::Hard);
        self
    }

    pub fn soft(&self) -> &Self {
        self.conv.replace(Conversion::Soft);
        self
    }

    fn write_buf<T: H5Type>(&self, buf: *const T) -> Result<()> {
        let file_dtype = self.obj.dtype()?;
        let mem_dtype = Datatype::from_type::<T>()?;
        mem_dtype.ensure_convertible(&file_dtype, self.conv.get())?;
        let (obj_id, tp_id) = (self.obj.id(), mem_dtype.id());
        if self.obj.is_attr() {
            h5try!(H5Awrite(obj_id, tp_id, buf as *const _));
        } else {
            h5try!(H5Dwrite(obj_id, tp_id, H5S_ALL, H5S_ALL, H5P_DEFAULT, buf as *const _));
        }
        Ok(())
    }

    pub fn write_arr<'b, A, T, D>(&self, arr: A) -> Result<()>
    where
        A: Into<ArrayView<'b, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        let view = arr.into();
        let src = view.shape();
        let dst = &*self.obj.get_shape()?;
        if src != dst {
            fail!("shape mismatch when writing: memory = {:?}, destination = {:?}", src, dst);
        }
        self.write_buf(view.as_ptr())
    }

    pub fn write_scalar<T: H5Type>(&self, val: &T) -> Result<()> {
        ensure!(self.obj.get_shape()?.is_empty(), "shape mismatch");
        self.write_buf(val as *const _)
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

    pub fn as_reader(&self) -> Reader {
        Reader::new(self)
    }

    pub fn as_writer(&self) -> Writer {
        Writer::new(self)
    }

    pub fn dtype(&self) -> Result<Datatype> {
        if self.is_attr() {
            Datatype::from_id(h5try!(H5Aget_type(self.id())))
        } else {
            Datatype::from_id(h5try!(H5Dget_type(self.id())))
        }
    }

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

    pub fn read_vec<T: H5Type>(&self) -> Result<Vec<T>> {
        self.as_reader().read_vec()
    }

    pub fn read_arr<T: H5Type, D: ndarray::Dimension>(&self) -> Result<Array<T, D>> {
        self.as_reader().read_arr()
    }

    pub fn read_dyn<T: H5Type>(&self) -> Result<ArrayD<T>> {
        self.as_reader().read_dyn()
    }

    pub fn read_1d<T: H5Type>(&self) -> Result<Array1<T>> {
        self.as_reader().read_1d()
    }

    pub fn read_2d<T: H5Type>(&self) -> Result<Array2<T>> {
        self.as_reader().read_2d()
    }

    pub fn read_scalar<T: H5Type>(&self) -> Result<T> {
        self.as_reader().read_scalar()
    }

    pub fn write_arr<'b, A, T, D>(&self, arr: A) -> Result<()>
    where
        A: Into<ArrayView<'b, T, D>>,
        T: H5Type,
        D: ndarray::Dimension,
    {
        self.as_writer().write_arr(arr)
    }

    pub fn write_scalar<T: H5Type>(&self, val: &T) -> Result<()> {
        self.as_writer().write_scalar(val)
    }
}

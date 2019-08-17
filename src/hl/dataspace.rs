use std::fmt::{self, Debug};
use std::ops::Deref;
use std::ptr;

use hdf5_sys::h5s::{
    H5S_class_t, H5Scopy, H5Screate, H5Screate_simple, H5Sdecode, H5Sencode,
    H5Sget_simple_extent_dims, H5Sget_simple_extent_ndims, H5Sget_simple_extent_npoints,
    H5Sget_simple_extent_type, H5Sselect_valid, H5S_UNLIMITED,
};

use crate::hl::extents::{Extent, Extents, Ix};
use crate::internal_prelude::*;

/// Represents the HDF5 dataspace object.
#[repr(transparent)]
#[derive(Clone)]
pub struct Dataspace(Handle);

impl ObjectClass for Dataspace {
    const NAME: &'static str = "dataspace";
    const VALID_TYPES: &'static [H5I_type_t] = &[H5I_DATASPACE];

    fn from_handle(handle: Handle) -> Self {
        Self(handle)
    }

    fn handle(&self) -> &Handle {
        &self.0
    }

    fn short_repr(&self) -> Option<String> {
        if let Ok(e) = self.extents() {
            Some(format!("{}", e))
        } else {
            Some("(invalid)".into())
        }
    }
}

impl Debug for Dataspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_fmt(f)
    }
}

impl Deref for Dataspace {
    type Target = Object;

    fn deref(&self) -> &Object {
        unsafe { self.transmute() }
    }
}

pub(crate) unsafe fn get_shape(space_id: hid_t) -> Result<Vec<Ix>> {
    let ndim = h5check(H5Sget_simple_extent_ndims(space_id))? as usize;
    let mut dims = vec![0; ndim];
    h5check(H5Sget_simple_extent_dims(space_id, dims.as_mut_ptr(), ptr::null_mut()))?;
    Ok(dims.into_iter().map(|x| x as _).collect())
}

pub(crate) unsafe fn get_simple_extents(space_id: hid_t) -> Result<SimpleExtents> {
    let ndim = h5check(H5Sget_simple_extent_ndims(space_id))? as usize;
    let (mut dims, mut maxdims) = (vec![0; ndim], vec![0; ndim]);
    h5check(H5Sget_simple_extent_dims(space_id, dims.as_mut_ptr(), maxdims.as_mut_ptr()))?;
    let mut extents = Vec::with_capacity(ndim);
    for i in 0..ndim {
        let (dim, max) = (dims[i] as _, maxdims[i]);
        let max = if max == H5S_UNLIMITED { None } else { Some(max as _) };
        extents.push(Extent::new(dim, max))
    }
    Ok(SimpleExtents::from_vec(extents))
}

impl Dataspace {
    pub fn try_new<T: Into<Extents>>(extents: T) -> Result<Self> {
        Self::from_extents(extents.into())
    }

    pub fn copy(&self) -> Self {
        Self::from_id(h5lock!(H5Scopy(self.id()))).unwrap_or_else(|_| Self::invalid())
    }

    pub fn ndim(&self) -> usize {
        h5call!(H5Sget_simple_extent_ndims(self.id())).unwrap_or(0) as _
    }

    pub fn shape(&self) -> Vec<Ix> {
        h5lock!(get_shape(self.id())).unwrap_or_default()
    }

    pub fn maxdims(&self) -> Vec<Option<Ix>> {
        self.extents().unwrap_or(Extents::Null).maxdims()
    }

    pub fn is_resizable(&self) -> bool {
        self.maxdims().iter().any(Option::is_none)
    }

    pub fn is_null(&self) -> bool {
        h5lock!(H5Sget_simple_extent_type(self.id())) == H5S_class_t::H5S_NULL
    }

    pub fn is_scalar(&self) -> bool {
        h5lock!(H5Sget_simple_extent_type(self.id())) == H5S_class_t::H5S_SCALAR
    }

    pub fn is_simple(&self) -> bool {
        h5lock!(H5Sget_simple_extent_type(self.id())) == H5S_class_t::H5S_SIMPLE
    }

    pub fn is_valid(&self) -> bool {
        h5lock!(H5Sselect_valid(self.id())) > 0
    }

    pub fn size(&self) -> usize {
        match h5lock!(H5Sget_simple_extent_type(self.id())) {
            H5S_class_t::H5S_SIMPLE => {
                h5call!(H5Sget_simple_extent_npoints(self.id())).unwrap_or(0) as _
            }
            H5S_class_t::H5S_SCALAR => 1,
            _ => 0,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut len: size_t = 0;
        h5lock!({
            h5try!(H5Sencode(self.id(), ptr::null_mut() as *mut _, &mut len as *mut _));
            let mut buf = vec![0u8; len];
            h5try!(H5Sencode(self.id(), buf.as_mut_ptr() as *mut _, &mut len as *mut _));
            Ok(buf)
        })
    }

    pub fn decode<T>(buf: T) -> Result<Self>
    where
        T: AsRef<[u8]>,
    {
        h5lock!(Self::from_id(h5try!(H5Sdecode(buf.as_ref().as_ptr() as *const _))))
    }

    fn from_extents(extents: Extents) -> Result<Self> {
        h5lock!(Self::from_id(match extents {
            Extents::Null => H5Screate(H5S_class_t::H5S_NULL),
            Extents::Scalar => H5Screate(H5S_class_t::H5S_SCALAR),
            Extents::Simple(ref e) => {
                let (mut dims, mut maxdims) = (vec![], vec![]);
                for extent in e.iter() {
                    dims.push(extent.dim as _);
                    maxdims.push(extent.max.map(|x| x as _).unwrap_or(H5S_UNLIMITED));
                }
                H5Screate_simple(e.ndim() as _, dims.as_ptr(), maxdims.as_ptr())
            }
        }))
    }

    pub fn extents(&self) -> Result<Extents> {
        h5lock!(match H5Sget_simple_extent_type(self.id()) {
            H5S_class_t::H5S_NULL => Ok(Extents::Null),
            H5S_class_t::H5S_SCALAR => Ok(Extents::Scalar),
            H5S_class_t::H5S_SIMPLE => get_simple_extents(self.id()).map(Extents::Simple),
            extent_type => fail!("Invalid extents type: {}", extent_type as c_int),
        })
    }
}

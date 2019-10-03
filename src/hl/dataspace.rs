use std::fmt::{self, Debug};
use std::ops::Deref;
use std::ptr;

use hdf5_sys::h5s::{
    H5S_class_t, H5Scopy, H5Screate, H5Screate_simple, H5Sdecode, H5Sencode, H5Sget_select_npoints,
    H5Sget_simple_extent_dims, H5Sget_simple_extent_ndims, H5Sget_simple_extent_npoints,
    H5Sget_simple_extent_type, H5Sselect_valid, H5S_UNLIMITED,
};

use crate::hl::extents::{Extent, Extents, Ix};
use crate::hl::selection::RawSelection;
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

    pub fn selection_size(&self) -> usize {
        h5call!(H5Sget_select_npoints(self.id())).ok().map_or(0, |x| x as _)
    }

    #[doc(hidden)]
    pub fn select_raw<S: Into<RawSelection>>(&self, raw_sel: S) -> Result<Self> {
        let raw_sel = raw_sel.into();
        sync(|| unsafe {
            let space = self.copy();
            raw_sel.apply_to_dataspace(space.id())?;
            ensure!(space.is_valid(), "Invalid selection, out of extents");
            Ok(space)
        })
    }

    pub fn select<S: Into<Selection>>(&self, selection: S) -> Result<Self> {
        let raw_sel = selection.into().into_raw(&self.shape())?;
        self.select_raw(raw_sel)
    }

    #[doc(hidden)]
    pub fn get_raw_selection(&self) -> Result<RawSelection> {
        sync(|| unsafe { RawSelection::extract_from_dataspace(self.id()) })
    }

    pub fn get_selection(&self) -> Result<Selection> {
        let raw_sel = self.get_raw_selection()?;
        Selection::from_raw(raw_sel)
    }
}

#[cfg(test)]
mod tests {
    use hdf5_sys::h5i::H5I_INVALID_HID;

    use super::Dataspace;
    use crate::internal_prelude::*;

    #[test]
    fn test_dataspace_err() {
        let _e = silence_errors();
        assert_err!(Dataspace::from_id(H5I_INVALID_HID), "Invalid dataspace id");
    }

    #[test]
    fn test_dataspace_null() -> Result<()> {
        let space = Dataspace::try_new(Extents::Null)?;
        assert_eq!(space.ndim(), 0);
        assert_eq!(space.shape(), vec![]);
        assert_eq!(space.maxdims(), vec![]);
        assert_eq!(space.size(), 0);
        assert!(space.is_null());
        assert_eq!(space.extents()?, Extents::Null);
        Ok(())
    }

    #[test]
    fn test_dataspace_scalar() -> Result<()> {
        let space = Dataspace::try_new(())?;
        assert_eq!(space.ndim(), 0);
        assert_eq!(space.shape(), vec![]);
        assert_eq!(space.maxdims(), vec![]);
        assert_eq!(space.size(), 1);
        assert!(space.is_scalar());
        assert_eq!(space.extents()?, Extents::Scalar);
        Ok(())
    }

    #[test]
    fn test_dataspace_simple() -> Result<()> {
        let space = Dataspace::try_new(123)?;
        assert_eq!(space.ndim(), 1);
        assert_eq!(space.shape(), vec![123]);
        assert_eq!(space.maxdims(), vec![Some(123)]);
        assert_eq!(space.size(), 123);
        assert!(space.is_simple());
        assert_eq!(space.extents()?, Extents::simple(123));
        assert!(!space.is_resizable());

        let space = Dataspace::try_new((5, 6..=10, 7..))?;
        assert_eq!(space.ndim(), 3);
        assert_eq!(space.shape(), vec![5, 6, 7]);
        assert_eq!(space.maxdims(), vec![Some(5), Some(10), None]);
        assert_eq!(space.size(), 210);
        assert!(space.is_simple());
        assert_eq!(space.extents()?, Extents::simple((5, 6..=10, 7..)));
        assert!(space.is_resizable());

        Ok(())
    }

    #[test]
    fn test_dataspace_copy() -> Result<()> {
        let space = Dataspace::try_new((5, 6..=10, 7..))?;
        let space_copy = space.copy();
        assert!(space_copy.is_valid());
        assert_eq!(space_copy.ndim(), space.ndim());
        assert_eq!(space_copy.shape(), space.shape());
        assert_eq!(space_copy.maxdims(), space.maxdims());
        Ok(())
    }

    #[test]
    fn test_dataspace_encode() -> Result<()> {
        let space = Dataspace::try_new((5, 6..=10, 7..))?;
        let encoded = space.encode()?;
        let decoded = Dataspace::decode(&encoded)?;
        assert_eq!(decoded.extents().unwrap(), space.extents().unwrap());
        Ok(())
    }

    #[test]
    fn test_dataspace_repr() -> Result<()> {
        assert_eq!(&format!("{:?}", Dataspace::try_new(Extents::Null)?), "<HDF5 dataspace: null>");
        assert_eq!(&format!("{:?}", Dataspace::try_new(())?), "<HDF5 dataspace: scalar>");
        assert_eq!(&format!("{:?}", Dataspace::try_new(123)?), "<HDF5 dataspace: (123,)>");
        assert_eq!(
            &format!("{:?}", Dataspace::try_new((5, 6..=10, 7..))?),
            "<HDF5 dataspace: (5, 6..=10, 7..)>"
        );
        Ok(())
    }
}

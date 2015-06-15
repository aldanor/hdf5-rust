use ffi::h5::hsize_t;
use ffi::h5i::{H5I_DATASPACE, H5I_INVALID_HID, hid_t};
use ffi::h5s::{H5S_UNLIMITED, H5Sget_simple_extent_dims, H5Sget_simple_extent_ndims, H5Scopy,
               H5Screate_simple};

use error::Result;
use handle::{Handle, ID, FromID, get_id_type};
use object::Object;

use std::{fmt, ptr, slice};
use libc::c_int;

pub type Ix = usize;

pub trait Dimension: Clone {
    fn ndim(&self) -> usize;
    fn dims(&self) -> Vec<Ix>;

    fn size(&self) -> Ix {
        let dims = self.dims();
        if dims.is_empty() { 0 } else { dims.iter().fold(1, |acc, &el| acc * el) }
    }
}

impl<'a, T: Dimension> Dimension for &'a T {
    fn ndim(&self) -> usize { Dimension::ndim(*self) }
    fn dims(&self) -> Vec<Ix> { Dimension::dims(*self) }
}

impl Dimension for Vec<Ix> {
    fn ndim(&self) -> usize { self.len() }
    fn dims(&self) -> Vec<Ix> { self.clone() }
}

macro_rules! count_ty {
    () => { 0 };
    ($_i:ty, $($rest:ty,)*) => { 1 + count_ty!($($rest,)*) }
}

macro_rules! impl_tuple {
    () => (
        impl Dimension for () {
            fn ndim(&self) -> usize { 0 }
            fn dims(&self) -> Vec<Ix> { vec![] }
        }
    );

    ($head:ty, $($tail:ty,)*) => (
        impl Dimension for ($head, $($tail,)*) {
            #[inline]
            fn ndim(&self) -> usize {
                count_ty!($head, $($tail,)*)
            }

            #[inline]
            fn dims(&self) -> Vec<Ix> {
                unsafe {
                    slice::from_raw_parts(self as *const _ as *const Ix, self.ndim())
                }.iter().cloned().collect()
            }
        }

        impl_tuple! { $($tail,)* }
    )
}

impl_tuple! { Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, }

impl Dimension for Ix {
    fn ndim(&self) -> usize { 1 }
    fn dims(&self) -> Vec<Ix> { vec![*self] }
}

pub struct Dataspace {
    handle: Handle,
}

impl Dataspace {
    pub fn new<D: Dimension>(d: D) -> Result<Dataspace> {
        let rank = d.ndim();
        let mut dims: Vec<hsize_t> = vec![];
        let mut max_dims: Vec<hsize_t> = vec![];
        for dim in d.dims().iter() {
            dims.push(*dim as hsize_t);
            max_dims.push(H5S_UNLIMITED);
        }
        Dataspace::from_id(h5try!(H5Screate_simple(rank as c_int, dims.as_ptr(),
                                                   max_dims.as_ptr())))
    }
}

impl Dimension for Dataspace {
    fn ndim(&self) -> usize {
        h5call!(H5Sget_simple_extent_ndims(self.id())).unwrap_or(0) as usize
    }

    fn dims(&self) -> Vec<Ix> {
        let ndim = self.ndim();
        if ndim > 0 {
            let mut dims: Vec<hsize_t> = Vec::with_capacity(ndim);
            unsafe { dims.set_len(ndim); }
            if h5call!(H5Sget_simple_extent_dims(self.id(), dims.as_mut_ptr(),
                                                 ptr::null_mut())).is_ok() {
                return dims.iter().cloned().map(|x| x as usize).collect();
            }
        }
        vec![]
    }
}

impl ID for Dataspace {
    fn id(&self) -> hid_t {
        self.handle.id()
    }
}

impl FromID for Dataspace {
    fn from_id(id: hid_t) -> Result<Dataspace> {
        match get_id_type(id) {
            H5I_DATASPACE => Ok(Dataspace { handle: try!(Handle::new(id)) }),
            _             => Err(From::from(format!("Invalid dataspace id: {}", id))),
        }
    }
}

impl Object for Dataspace {}

impl Clone for Dataspace {
    fn clone(&self) -> Dataspace {
        let id = h5call!(H5Scopy(self.id())).unwrap_or(H5I_INVALID_HID);
        Dataspace::from_id(id).unwrap_or(Dataspace { handle: Handle::invalid() })
    }
}

impl fmt::Debug for Dataspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Dataspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.is_valid() {
            return "<HDF5 dataspace: invalid id>".fmt(f);
        }
        let mut dims = String::new();
        for (i, dim) in self.dims().iter().enumerate() {
            if i > 0 {
                dims.push_str(", ");
            }
            dims.push_str(&format!("{}", dim));
        }
        if self.ndim() == 1 {
            dims.push_str(",");
        }
        format!("<HDF5 dataspace: ({})>", dims).fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Dimension, Ix, Dataspace};
    use error::silence_errors;
    use handle::{ID, FromID};
    use object::Object;
    use ffi::h5i::H5I_INVALID_HID;
    use ffi::h5s::H5S_UNLIMITED;

    #[test]
    pub fn test_dimension() {
        fn f<D: Dimension>(d: D) -> (usize, Vec<Ix>, Ix) { (d.ndim(), d.dims(), d.size()) }

        assert_eq!(f(()), (0, vec![], 0));
        assert_eq!(f(&()), (0, vec![], 0));
        assert_eq!(f(2), (1, vec![2], 2));
        assert_eq!(f(&3), (1, vec![3], 3));
        assert_eq!(f((4,)), (1, vec![4], 4));
        assert_eq!(f(&(5,)), (1, vec![5], 5));
        assert_eq!(f((1, 2)), (2, vec![1, 2], 2));
        assert_eq!(f(&(3, 4)), (2, vec![3, 4], 12));
        assert_eq!(f(vec![2, 3]), (2, vec![2, 3], 6));
        assert_eq!(f(&vec![4, 5]), (2, vec![4, 5], 20));
    }

    #[test]
    pub fn test_debug_display() {
        assert_eq!(format!("{}", Dataspace::new(()).unwrap()), "<HDF5 dataspace: ()>");
        assert_eq!(format!("{:?}", Dataspace::new(()).unwrap()), "<HDF5 dataspace: ()>");
        assert_eq!(format!("{}", Dataspace::new(3).unwrap()), "<HDF5 dataspace: (3,)>");
        assert_eq!(format!("{:?}", Dataspace::new(3).unwrap()), "<HDF5 dataspace: (3,)>");
        assert_eq!(format!("{}", Dataspace::new((1, 2)).unwrap()), "<HDF5 dataspace: (1, 2)>");
        assert_eq!(format!("{:?}", Dataspace::new((1, 2)).unwrap()), "<HDF5 dataspace: (1, 2)>");
    }

    #[test]
    pub fn test_dataspace() {
        silence_errors();
        assert_err!(Dataspace::new(H5S_UNLIMITED as usize),
                    "current dimension must have a specific size");
        let d = Dataspace::new((5, 6)).unwrap();
        assert_eq!((d.ndim(), d.dims(), d.size()), (2, vec![5, 6], 30));
        assert_eq!(Dataspace::new(()).unwrap().dims(), vec![]);
        assert_err!(Dataspace::from_id(H5I_INVALID_HID), "Invalid dataspace id");
        let dc = d.clone();
        assert!(dc.is_valid() && dc.id() != d.id());
        assert_eq!((d.ndim(), d.dims(), d.size()), (dc.ndim(), dc.dims(), dc.size()));
    }
}

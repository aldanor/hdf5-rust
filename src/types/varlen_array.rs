use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr;
use std::slice;

use libc::size_t;

use ffi::h5t::hvl_t;
use types::{ValueType, ToValueType};

#[repr(C)]
#[unsafe_no_drop_flag]
pub struct VarLenArray<T: Clone> {
    vl: hvl_t,
    tag: PhantomData<T>,
}

impl<T: Clone> VarLenArray<T> {
    pub fn new(arr: &[T]) -> VarLenArray<T> {
        unsafe {
            let p = ::libc::malloc(arr.len() * mem::size_of::<T>());
            ptr::copy_nonoverlapping(arr.as_ptr(), p as *mut T, arr.len());
            VarLenArray {
                vl: hvl_t { len: arr.len() as size_t, p: p },
                tag: PhantomData,
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.vl.len as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vl.len == 0
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }
}

impl<T: Clone> Drop for VarLenArray<T> {
    fn drop(&mut self) {
        if self.vl.len != 0 {
            self.vl.len = 0;
        }
        if !self.vl.p.is_null() {
            unsafe { ::libc::free(self.vl.p) };
            self.vl.p = ptr::null_mut();
        }
    }
}

unsafe impl<T: Clone + ToValueType> ToValueType for VarLenArray<T> {
    fn value_type() -> ValueType {
        ValueType::VarLenArray(Box::new(<T as ToValueType>::value_type()))
    }
}

impl<T: Clone> Deref for VarLenArray<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        if self.vl.len == 0 || self.vl.p.is_null() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.vl.p as *const T,
                                           self.vl.len as usize) }
        }
    }
}

impl<'a, T: Clone> From<&'a [T]> for VarLenArray<T> {
    fn from(arr: &[T]) -> VarLenArray<T> {
        VarLenArray::new(arr)
    }
}

impl<T: Clone> Default for VarLenArray<T> {
    fn default() -> VarLenArray<T> {
        VarLenArray::new(&[])
    }
}

#[cfg(test)]
pub mod tests {
    use super::VarLenArray;
    use types::{ValueType, ToValueType};

    type S = VarLenArray<u16>;

    #[test]
    pub fn test_value_type() {
        use std::mem;
        use ffi::h5t::hvl_t;

        assert_eq!(S::value_type(),
                   ValueType::VarLenArray(Box::new(u16::value_type())));
        assert_eq!(mem::size_of::<VarLenArray<u8>>(),
                   mem::size_of::<hvl_t>());
    }

    #[test]
    pub fn test_default_empty() {
        assert_eq!(&*S::default(), &[]);
        assert_eq!(&*S::default(), &*S::new(&[]));
        assert!(S::default().is_empty());
        assert_eq!(S::default().len(), 0);
    }
}

use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr;
use std::slice;

use libc::{self, size_t};

/* This trait is borrowed from arrayvec::Array (C) @bluss */
pub unsafe trait Array: 'static {
    type Item;

    fn as_ptr(&self) -> *const Self::Item;
    fn as_mut_ptr(&mut self) -> *mut Self::Item;
    fn capacity() -> usize;
}

macro_rules! impl_array {
    () => ();

    ($n:expr, $($ns:expr,)*) => (
        unsafe impl<T: 'static> Array for [T; $n] {
            type Item = T;

            #[inline(always)]
            fn as_ptr(&self) -> *const T {
                self as *const _ as *const _
            }

            #[inline(always)]
            fn as_mut_ptr(&mut self) -> *mut T {
                self as *mut _ as *mut _
            }

            #[inline(always)]
            fn capacity() -> usize {
                $n
            }
        }

        impl_array!($($ns,)*);
    );
}

impl_array!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
);
impl_array!(
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 58, 59, 60, 61, 62, 63,
);
impl_array!(
    64, 70, 72, 80, 90, 96, 100, 110, 120, 128, 130, 140, 150, 160, 170, 180, 190, 192, 200, 210,
    220, 224, 230, 240, 250,
);
impl_array!(
    256, 300, 384, 400, 500, 512, 600, 700, 768, 800, 900, 1000, 1024, 2048, 4096, 8192, 16384,
    32768,
);

#[repr(C)]
pub struct VarLenArray<T: Copy> {
    len: size_t,
    ptr: *const T,
    tag: PhantomData<T>,
}

impl<T: Copy> VarLenArray<T> {
    pub unsafe fn from_parts(p: *const T, len: usize) -> VarLenArray<T> {
        let (len, ptr) = if !p.is_null() && len != 0 {
            let dst = libc::malloc(len * mem::size_of::<T>());
            ptr::copy_nonoverlapping(p, dst as *mut _, len);
            (len, dst)
        } else {
            (0, ptr::null_mut())
        };
        VarLenArray { len, ptr: ptr as *const _, tag: PhantomData }
    }

    #[inline]
    pub fn from_slice(arr: &[T]) -> VarLenArray<T> {
        unsafe { VarLenArray::from_parts(arr.as_ptr(), arr.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as _
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }
}

impl<T: Copy> Drop for VarLenArray<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                libc::free(self.ptr as *mut _);
            }
            self.ptr = ptr::null();
            if self.len != 0 {
                self.len = 0;
            }
        }
    }
}

impl<T: Copy> Clone for VarLenArray<T> {
    #[inline]
    fn clone(&self) -> VarLenArray<T> {
        VarLenArray::from_slice(&*self)
    }
}

impl<T: Copy> Deref for VarLenArray<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        if self.len == 0 || self.ptr.is_null() {
            &[]
        } else {
            unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
        }
    }
}

impl<'a, T: Copy> From<&'a [T]> for VarLenArray<T> {
    #[inline]
    fn from(arr: &[T]) -> VarLenArray<T> {
        VarLenArray::from_slice(arr)
    }
}

impl<T: Copy> Into<Vec<T>> for VarLenArray<T> {
    #[inline]
    fn into(self) -> Vec<T> {
        self.iter().cloned().collect()
    }
}

impl<T: Copy, A: Array<Item = T>> From<A> for VarLenArray<T> {
    #[inline]
    fn from(arr: A) -> VarLenArray<T> {
        unsafe { VarLenArray::from_parts(arr.as_ptr(), A::capacity()) }
    }
}

impl<T: Copy> Default for VarLenArray<T> {
    #[inline]
    fn default() -> VarLenArray<T> {
        unsafe { VarLenArray::from_parts(ptr::null(), 0) }
    }
}

impl<T: Copy + PartialEq> PartialEq for VarLenArray<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Copy + Eq> Eq for VarLenArray<T> {}

impl<T: Copy + PartialEq> PartialEq<[T]> for VarLenArray<T> {
    #[inline]
    fn eq(&self, other: &[T]) -> bool {
        self.as_slice() == other
    }
}

impl<T: Copy + PartialEq, A: Array<Item = T>> PartialEq<A> for VarLenArray<T> {
    #[inline]
    fn eq(&self, other: &A) -> bool {
        self.as_slice() == unsafe { slice::from_raw_parts(other.as_ptr(), A::capacity()) }
    }
}

impl<T: Copy + fmt::Debug> fmt::Debug for VarLenArray<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Array, VarLenArray};

    type S = VarLenArray<u16>;

    #[test]
    pub fn test_array_trait() {
        type T = [u32; 256];
        assert_eq!(<T as Array>::capacity(), 256);
        let mut arr = [1, 2, 3];
        assert_eq!(arr.as_ptr(), &arr[0] as *const _);
        assert_eq!(arr.as_mut_ptr(), &mut arr[0] as *mut _);
    }

    #[test]
    pub fn test_vla_empty_default() {
        assert_eq!(&*S::default(), &[]);
        assert!(S::default().is_empty());
        assert_eq!(S::default().len(), 0);
    }

    #[test]
    pub fn test_vla_array_traits() {
        use std::slice;

        let s = &[1u16, 2, 3];
        let a = VarLenArray::from_slice(s);
        assert_eq!(a.as_slice(), s);
        assert_eq!(a.len(), 3);
        assert!(!a.is_empty());
        assert_eq!(unsafe { slice::from_raw_parts(a.as_ptr(), a.len()) }, s);
        assert_eq!(&*a, s);
        let c = a.clone();
        assert_eq!(&*a, &*c);
        let v: Vec<u16> = c.into();
        assert_eq!(v, vec![1, 2, 3]);
        assert_eq!(&*a, &*VarLenArray::from(*s));
        let f: [u16; 3] = [1, 2, 3];
        assert_eq!(&*a, &*VarLenArray::from(f));
        assert_eq!(format!("{:?}", a), "[1, 2, 3]");
        assert_eq!(a, [1, 2, 3]);
        assert_eq!(&a, s);
        assert_eq!(&a, a.as_slice());
        assert_eq!(a, a);
        let v: Vec<_> = a.iter().cloned().collect();
        assert_eq!(v, vec![1, 2, 3]);
    }
}

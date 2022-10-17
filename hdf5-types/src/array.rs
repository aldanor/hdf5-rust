use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::ptr;
use std::slice;

#[repr(C)]
pub struct VarLenArray<T: Copy> {
    len: usize,
    ptr: *const T,
    tag: PhantomData<T>,
}

impl<T: Copy> VarLenArray<T> {
    pub unsafe fn from_parts(p: *const T, len: usize) -> Self {
        let (len, ptr) = if !p.is_null() && len != 0 {
            let dst = crate::malloc(len * mem::size_of::<T>());
            ptr::copy_nonoverlapping(p, dst.cast(), len);
            (len, dst)
        } else {
            (0, ptr::null_mut())
        };
        Self { len, ptr: ptr as *const _, tag: PhantomData }
    }

    #[inline]
    pub fn from_slice(arr: &[T]) -> Self {
        unsafe { Self::from_parts(arr.as_ptr(), arr.len()) }
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
                crate::free(self.ptr as *mut _);
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
    fn clone(&self) -> Self {
        Self::from_slice(self)
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
    fn from(arr: &[T]) -> Self {
        Self::from_slice(arr)
    }
}

impl<T: Copy> From<VarLenArray<T>> for Vec<T> {
    #[inline]
    fn from(v: VarLenArray<T>) -> Self {
        v.iter().copied().collect()
    }
}

impl<T: Copy, const N: usize> From<[T; N]> for VarLenArray<T> {
    #[inline]
    fn from(arr: [T; N]) -> Self {
        unsafe { Self::from_parts(arr.as_ptr(), arr.len()) }
    }
}

impl<T: Copy> Default for VarLenArray<T> {
    #[inline]
    fn default() -> Self {
        unsafe { Self::from_parts(ptr::null(), 0) }
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

impl<T: Copy + PartialEq, const N: usize> PartialEq<[T; N]> for VarLenArray<T> {
    #[inline]
    fn eq(&self, other: &[T; N]) -> bool {
        self.as_slice() == other
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
    use super::VarLenArray;

    type S = VarLenArray<u16>;

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

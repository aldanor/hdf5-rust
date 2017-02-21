/* This trait is borrowed from arrayvec::Array (C) bluss */

pub unsafe trait Array {
    type Item;

    fn as_ptr(&self) -> *const Self::Item;
    fn as_mut_ptr(&mut self) -> *mut Self::Item;
    fn capacity() -> usize;
}

macro_rules! impl_array {
    () => ();

    ($n:expr, $($ns:expr,)*) => (
        unsafe impl<T> Array for [T; $n] {
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

impl_array!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
            16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,);
impl_array!(32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
            48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,);
impl_array!(64, 70, 72, 80, 90, 96, 100, 110, 120, 128, 130, 140, 150,
            160, 170, 180, 190, 192, 200, 210, 220, 224, 230, 240, 250,);
impl_array!(256, 300, 384, 400, 500, 512, 600, 700, 768, 800, 900, 1000, 1024,
            2048, 4096, 8192, 16384, 32768,);

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_array_trait() {
        type T = [u32; 256];
        assert_eq!(<T as Array>::capacity(), 256);
        let mut arr = [1, 2, 3];
        assert_eq!(arr.as_ptr(), &arr[0] as *const _);
        assert_eq!(arr.as_mut_ptr(), &mut arr[0] as *mut _);
    }
}

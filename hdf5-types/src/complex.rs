use super::{CompoundField, CompoundType, H5Type, TypeDescriptor};

use std::mem::size_of;

use num_complex::Complex;

unsafe impl<T: H5Type> H5Type for Complex<T> {
    fn type_descriptor() -> TypeDescriptor {
        // Complex<T> should be FFI-equivalent to [T; 2]
        // https://docs.rs/num-complex/0.4.3/num_complex/struct.Complex.html#representation-and-foreign-function-interface-compatibility
        TypeDescriptor::Compound(CompoundType {
            fields: vec![
                // Compatible with h5py definition of complex
                CompoundField::typed::<T>("r", 0, 0),
                CompoundField::typed::<T>("i", size_of::<T>(), 1),
            ],
            size: size_of::<T>() * 2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_complex::{Complex32, Complex64};
    use std::mem::size_of_val;

    #[test]
    fn complex() {
        assert_eq!(Complex32::type_descriptor().size(), 8);
        assert_eq!(Complex32::type_descriptor().size(), size_of::<Complex32>());
        assert_eq!(Complex64::type_descriptor().size(), 16);
        assert_eq!(Complex64::type_descriptor().size(), size_of::<Complex64>());
    }

    #[test]
    fn alignment() {
        use std::ptr::addr_of;

        let a = Complex { re: 0.0, im: 1.0 };

        assert_eq!(size_of_val(&a), size_of_val(&a.re) + size_of_val(&a.im));
        assert_eq!(size_of_val(&a.re), size_of_val(&a.im));

        let base: *const u8 = addr_of!(a).cast::<u8>();
        let ptr_r = addr_of!(a.re).cast::<u8>();
        let ptr_i = addr_of!(a.im).cast::<u8>();

        assert_eq!(unsafe { ptr_r.offset_from(base) }, 0);
        assert_eq!(unsafe { ptr_i.offset_from(base) }, size_of_val(&a.re) as isize);
    }
}

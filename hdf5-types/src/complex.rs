use super::{CompoundField, CompoundType, H5Type, TypeDescriptor};

use std::mem::size_of;

use num_complex::Complex;

unsafe impl<T: H5Type> H5Type for Complex<T> {
    fn type_descriptor() -> TypeDescriptor {
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

    #[test]
    fn complex() {
        assert_eq!(Complex32::type_descriptor().size(), 8);
        assert_eq!(Complex64::type_descriptor().size(), 16);
    }
}

use std::slice;

/// A scalar integer type used by `Dimension` trait for indexing.
pub type Ix = usize;

/// A trait for the shape and index types.
pub trait Dimension: Clone {
    fn ndim(&self) -> usize;
    fn dims(&self) -> Vec<Ix>;

    fn size(&self) -> Ix {
        let dims = self.dims();
        if dims.is_empty() {
            1
        } else {
            dims.iter().product()
        }
    }
}

impl<'a, T: Dimension> Dimension for &'a T {
    fn ndim(&self) -> usize {
        Dimension::ndim(*self)
    }
    fn dims(&self) -> Vec<Ix> {
        Dimension::dims(*self)
    }
}

impl Dimension for Vec<Ix> {
    fn ndim(&self) -> usize {
        self.len()
    }
    fn dims(&self) -> Vec<Ix> {
        self.clone()
    }
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
                    slice::from_raw_parts(self as *const _ as *const _, self.ndim())
                }.iter().cloned().collect()
            }
        }

        impl_tuple! { $($tail,)* }
    )
}

impl_tuple! { Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, Ix, }

impl Dimension for Ix {
    fn ndim(&self) -> usize {
        1
    }
    fn dims(&self) -> Vec<Ix> {
        vec![*self]
    }
}

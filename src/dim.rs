use std::slice;

/// A scalar integer type used by `Dimension` trait for indexing.
pub type Ix = usize;

/// A trait for the shape and index types.
pub trait Dimension {
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

impl Dimension for [Ix] {
    fn ndim(&self) -> usize {
        self.len()
    }

    fn dims(&self) -> Vec<Ix> {
        self.to_vec()
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

    (@impl <$tp:ty>, $head:ty, $($tail:ty,)*) => (
        impl Dimension for $tp {
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
    );

    ($head:ty, $($tail:ty,)*) => (
        impl_tuple! { @impl <($head, $($tail,)*)>, $head, $($tail,)* }
        impl_tuple! { @impl <[Ix; count_ty!($head, $($tail,)*)]>, $head, $($tail,)* }
        impl_tuple! { $($tail,)* }
    );
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

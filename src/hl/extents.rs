use std::borrow::Borrow;
use std::convert::identity;
use std::fmt::{self, Debug, Display};
use std::ops::{Deref, RangeFrom, RangeInclusive};

use hdf5_sys::h5s::H5S_MAX_RANK;

pub type Ix = usize;

/// Current and maximum dimension size for a particular dimension.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Extent {
    /// Current dimension size.
    pub dim: Ix,
    /// Maximum dimension size (or `None` if unlimited).
    pub max: Option<Ix>,
}

impl Debug for Extent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Extent({})", self)
    }
}

impl Display for Extent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(max) = self.max {
            if self.dim != max {
                write!(f, "{}..={}", self.dim, max)
            } else {
                write!(f, "{}", self.dim)
            }
        } else {
            write!(f, "{}..", self.dim)
        }
    }
}

impl From<Ix> for Extent {
    fn from(dim: Ix) -> Self {
        Self { dim, max: Some(dim) }
    }
}

impl From<(Ix, Option<Ix>)> for Extent {
    fn from((dim, max): (Ix, Option<Ix>)) -> Self {
        Self { dim, max }
    }
}

impl From<RangeFrom<Ix>> for Extent {
    fn from(range: RangeFrom<Ix>) -> Self {
        Self { dim: range.start, max: None }
    }
}

impl From<RangeInclusive<Ix>> for Extent {
    fn from(range: RangeInclusive<Ix>) -> Self {
        Self { dim: *range.start(), max: Some(*range.end()) }
    }
}

impl<T: Into<Extent> + Clone> From<&T> for Extent {
    fn from(extent: &T) -> Self {
        extent.clone().into()
    }
}

impl Extent {
    pub fn new(dim: Ix, max: Option<Ix>) -> Self {
        Self { dim, max }
    }

    /// Creates a new extent with maximum size equal to the current size.
    pub fn fixed(dim: Ix) -> Self {
        Self { dim, max: Some(dim) }
    }

    /// Creates a new extent with unlimited maximum size.
    pub fn resizable(dim: Ix) -> Self {
        Self { dim, max: None }
    }

    pub fn is_fixed(&self) -> bool {
        self.max.map_or(false, |max| self.dim >= max)
    }

    pub fn is_resizable(&self) -> bool {
        self.max.is_none()
    }

    pub fn is_unlimited(&self) -> bool {
        self.is_resizable()
    }

    pub fn is_valid(&self) -> bool {
        self.max.unwrap_or(self.dim) >= self.dim
    }
}

/// Extents for a simple dataspace, a multidimensional array of elements.
///
/// The dimensionality of the dataspace (or the rank of the array) is fixed and is defined
/// at creation time. The size of each dimension can grow during the life time of the
/// dataspace from the current size up to the maximum size. Both the current size and the
/// maximum size are specified at creation time. The sizes of dimensions at any particular
/// time in the life of a dataspace are called the current dimensions, or the dataspace
/// extent. They can be queried along with the maximum sizes.
#[derive(Clone, PartialEq, Eq)]
pub struct SimpleExtents {
    inner: Vec<Extent>,
}

impl SimpleExtents {
    pub fn from_vec(extents: Vec<Extent>) -> Self {
        Self { inner: extents }
    }

    pub fn new<T>(extents: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<Extent>,
    {
        Self::from_vec(extents.into_iter().map(Into::into).collect())
    }

    pub fn fixed<T>(extents: T) -> Self
    where
        T: IntoIterator,
        T::Item: Borrow<Ix>,
    {
        Self::from_vec(extents.into_iter().map(|x| Extent::fixed(x.borrow().clone())).collect())
    }

    pub fn resizable<T>(extents: T) -> Self
    where
        T: IntoIterator,
        T::Item: Borrow<Ix>,
    {
        Self::from_vec(extents.into_iter().map(|x| Extent::resizable(x.borrow().clone())).collect())
    }

    pub fn ndim(&self) -> usize {
        self.inner.len()
    }

    pub fn dims(&self) -> Vec<Ix> {
        self.inner.iter().map(|e| e.dim).collect()
    }

    pub fn maxdims(&self) -> Vec<Option<Ix>> {
        self.inner.iter().map(|e| e.max).collect()
    }

    pub fn size(&self) -> usize {
        self.inner.iter().fold(1, |acc, x| acc * x.dim)
    }

    pub fn is_fixed(&self) -> bool {
        !self.inner.is_empty() && self.inner.iter().map(Extent::is_fixed).all(identity)
    }

    pub fn is_resizable(&self) -> bool {
        !self.inner.is_empty() && self.inner.iter().map(Extent::is_unlimited).all(identity)
    }

    pub fn is_unlimited(&self) -> bool {
        self.inner.iter().map(Extent::is_unlimited).any(identity)
    }

    pub fn is_valid(&self) -> bool {
        self.inner.iter().map(Extent::is_valid).all(identity) && self.ndim() <= H5S_MAX_RANK as _
    }
}

impl Deref for SimpleExtents {
    type Target = [Extent];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Debug for SimpleExtents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SimpleExtents({})", self)
    }
}

impl Display for SimpleExtents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.ndim() == 0 {
            write!(f, "()")
        } else if self.ndim() == 1 {
            write!(f, "({},)", self[0])
        } else {
            let extents = self.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
            write!(f, "({})", extents)
        }
    }
}

macro_rules! impl_tuple {
    () => ();

    ($head:ident, $($tail:ident,)*) => (
        #[allow(non_snake_case)]
        impl<$head, $($tail,)*> From<($head, $($tail,)*)> for SimpleExtents
            where $head: Into<Extent>, $($tail: Into<Extent>,)*
        {
            fn from(extents: ($head, $($tail,)*)) -> Self {
                let ($head, $($tail,)*) = extents;
                Self::from_vec(vec![($head).into(), $(($tail).into(),)*])
            }
        }

        impl_tuple! { $($tail,)* }
    )
}

impl_tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

macro_rules! impl_fixed {
    ($tp:ty,) => ();

    ($tp:ty, $head:expr, $($tail:expr,)*) => (
        impl From<[$tp; $head]> for SimpleExtents {
            fn from(extents: [$tp; $head]) -> Self {
                Self::from_vec(extents.iter().map(Extent::from).collect())
            }
        }

        impl From<&[$tp; $head]> for SimpleExtents {
            fn from(extents: &[$tp; $head]) -> Self {
                Self::from_vec(extents.iter().map(Extent::from).collect())
            }
        }

        impl_fixed! { $tp, $($tail,)* }
    )
}

macro_rules! impl_from {
    ($tp:ty) => {
        impl From<$tp> for SimpleExtents {
            fn from(extent: $tp) -> Self {
                (extent,).into()
            }
        }

        impl From<&$tp> for SimpleExtents {
            fn from(extent: &$tp) -> Self {
                (extent.clone(),).into()
            }
        }

        impl From<Vec<$tp>> for SimpleExtents {
            fn from(extents: Vec<$tp>) -> Self {
                Self::from_vec(extents.iter().map(Extent::from).collect())
            }
        }

        impl From<&Vec<$tp>> for SimpleExtents {
            fn from(extents: &Vec<$tp>) -> Self {
                Self::from_vec(extents.iter().map(Extent::from).collect())
            }
        }

        impl From<&[$tp]> for SimpleExtents {
            fn from(extents: &[$tp]) -> Self {
                Self::from_vec(extents.iter().map(Extent::from).collect())
            }
        }

        impl_fixed! { $tp, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, }
    };
}

impl_from!(Ix);
impl_from!((Ix, Option<Ix>));
impl_from!(RangeFrom<Ix>);
impl_from!(RangeInclusive<Ix>);
impl_from!(Extent);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Extents {
    /// A null dataspace contains no data elements.
    ///
    /// Note that no selections can be appliedmto a null dataset as there is nothing to select.
    Null,

    /// A scalar dataspace, representing just one element.
    ///
    /// The datatype of this one element may be very complex, e.g., a compound structure
    /// with members being of any allowed HDF5 datatype, including multidimensional arrays,
    /// strings, and nested compound structures. By convention, the rank of a scalar dataspace
    /// is always 0 (zero); it may be thought of as a single, dimensionless point, though
    /// that point may be complex.
    Scalar,

    /// A simple dataspace, a multidimensional array of elements.
    ///
    /// The dimensionality of the dataspace (or the rank of the array) is fixed and is defined
    /// at creation time. The size of each dimension can grow during the life time of the
    /// dataspace from the current size up to the maximum size. Both the current size and the
    /// maximum size are specified at creation time. The sizes of dimensions at any particular
    /// time in the life of a dataspace are called the current dimensions, or the dataspace
    /// extent. They can be queried along with the maximum sizes.
    Simple(SimpleExtents),
}

impl Extents {
    pub fn new<T: Into<Self>>(extents: T) -> Self {
        extents.into()
    }

    /// Creates extents for a *null* dataspace.
    pub fn null() -> Self {
        Extents::Null
    }

    /// Creates extents for a *scalar* dataspace.
    pub fn scalar() -> Self {
        Extents::Scalar
    }

    /// Creates extents for a *simple* dataspace.
    pub fn simple<T: Into<SimpleExtents>>(extents: T) -> Self {
        Extents::Simple(extents.into())
    }

    fn as_simple(&self) -> Option<&SimpleExtents> {
        match self {
            Extents::Simple(ref e) => Some(e),
            _ => None,
        }
    }

    /// Returns true if the extents type is *null*.
    pub fn is_null(&self) -> bool {
        self == &Extents::Null
    }

    /// Returns true if the extents type is *scalar*.
    pub fn is_scalar(&self) -> bool {
        self == &Extents::Scalar
    }

    /// Returns true if the extents type is *simple*.
    pub fn is_simple(&self) -> bool {
        self.as_simple().is_some()
    }

    /// Returns the dataspace rank (or zero for null/scalar extents).
    pub fn ndim(&self) -> usize {
        self.as_simple().map_or(0, SimpleExtents::ndim)
    }

    /// Returns the current extents (or empty list for null/scalar extents).
    pub fn dims(&self) -> Vec<Ix> {
        self.as_simple().map_or_else(Vec::new, SimpleExtents::dims)
    }

    /// Returns the maximum extents (or empty list for null/scalar extents).
    pub fn maxdims(&self) -> Vec<Option<Ix>> {
        self.as_simple().map_or_else(Vec::new, SimpleExtents::maxdims)
    }

    /// Returns the total number of elements.
    pub fn size(&self) -> usize {
        match self {
            Extents::Null => 0,
            Extents::Scalar => 1,
            Extents::Simple(extents) => extents.size(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.as_simple().map_or(true, SimpleExtents::is_valid)
    }

    pub fn is_unlimited(&self) -> bool {
        self.as_simple().map_or(true, SimpleExtents::is_unlimited)
    }

    pub fn is_resizable(&self) -> bool {
        self.as_simple().map_or(true, SimpleExtents::is_resizable)
    }

    pub fn resizable(self) -> Self {
        match self {
            Extents::Simple(extents) => SimpleExtents::resizable(extents.dims()).into(),
            _ => self.clone(),
        }
    }
}

impl Display for Extents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Extents::Null => write!(f, "null"),
            Extents::Scalar => write!(f, "scalar"),
            Extents::Simple(ref e) => write!(f, "{}", e),
        }
    }
}

impl<T: Into<SimpleExtents>> From<T> for Extents {
    fn from(extents: T) -> Self {
        let extents = extents.into();
        if extents.is_empty() {
            Extents::Scalar
        } else {
            Extents::Simple(extents)
        }
    }
}

impl From<()> for Extents {
    fn from(_: ()) -> Self {
        Extents::Scalar
    }
}

impl From<&Extents> for Extents {
    fn from(extents: &Extents) -> Self {
        extents.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Extent, Extents, SimpleExtents};

    #[test]
    pub fn test_extent() {
        let e1 = Extent { dim: 1, max: None };
        let e2 = Extent { dim: 1, max: Some(2) };
        let e3 = Extent { dim: 2, max: Some(2) };

        assert_eq!(Extent::new(1, Some(2)), e2);

        assert_eq!(Extent::from(2), e3);
        assert_eq!(Extent::from((1, Some(2))), e2);
        assert_eq!(Extent::from(1..), e1);
        assert_eq!(Extent::from(1..=2), e2);

        assert_eq!(Extent::from(&2), e3);
        assert_eq!(Extent::from(&(1, Some(2))), e2);
        assert_eq!(Extent::from(&(1..)), e1);
        assert_eq!(Extent::from(&(1..=2)), e2);

        assert_eq!(format!("{}", e1), "1..");
        assert_eq!(format!("{:?}", e1), "Extent(1..)");
        assert_eq!(format!("{}", e2), "1..=2");
        assert_eq!(format!("{:?}", e2), "Extent(1..=2)");
        assert_eq!(format!("{}", e3), "2");
        assert_eq!(format!("{:?}", e3), "Extent(2)");

        assert_eq!(Extent::resizable(1), e1);
        assert_eq!(Extent::new(1, Some(2)), e2);
        assert_eq!(Extent::fixed(2), e3);

        assert!(!e1.is_fixed() && !e2.is_fixed() && e3.is_fixed());
        assert!(e1.is_resizable() && !e2.is_resizable() && !e3.is_resizable());
        assert!(e1.is_unlimited() && !e2.is_unlimited() && !e3.is_unlimited());

        assert!(e1.is_valid() && e2.is_valid() && e3.is_valid());
        assert!(!Extent::new(3, Some(2)).is_valid());
    }

    #[test]
    pub fn test_simple_extents() {
        type SE = SimpleExtents;

        let e1 = Extent::from(1..);
        let e2 = Extent::from(2..=3);
        let e3 = Extent::from(4);

        let v = vec![e1, e2, e3];
        let se = SE::from_vec(v.clone());
        assert_eq!(se.to_vec(), v);
        assert_eq!(se.len(), 3);
        assert_eq!(se.ndim(), 3);
        assert_eq!(se.dims(), vec![1, 2, 4]);
        assert_eq!(se.maxdims(), vec![None, Some(3), Some(4)]);

        let se1 = SE::new(&[(1, None), (2, Some(3)), (4, Some(4))]);
        let se2 = SE::fixed(&[1, 2]);
        let se3 = SE::resizable(&[1, 2]);

        assert_eq!(se1, se);
        assert_eq!(se2, SE::new(&[1..=1, 2..=2]));
        assert_eq!(se3, SE::new(&[1.., 2..]));

        assert!(!se1.is_fixed() && se2.is_fixed() && !se3.is_fixed());
        assert!(se1.is_unlimited() && !se2.is_unlimited() && se3.is_unlimited());
        assert!(!se1.is_resizable() && !se2.is_resizable() && se3.is_resizable());

        assert!(se1.is_valid() && se2.is_valid() && se3.is_valid());
        assert!(!SE::new(&[1..=2, 4..=3]).is_valid());
        assert!(!SE::new(vec![1; 100]).is_valid());

        assert_eq!(format!("{}", se1), "(1.., 2..=3, 4)");
        assert_eq!(format!("{:?}", se1), "SimpleExtents((1.., 2..=3, 4))");
        assert_eq!(format!("{}", se2), "(1, 2)");
        assert_eq!(format!("{:?}", se2), "SimpleExtents((1, 2))");
        assert_eq!(format!("{}", se3), "(1.., 2..)");
        assert_eq!(format!("{:?}", se3), "SimpleExtents((1.., 2..))");
        assert_eq!(format!("{}", SE::new(&[1..])), "(1..,)");
        assert_eq!(format!("{:?}", SE::new(&[1..])), "SimpleExtents((1..,))");

        assert_eq!(
            SE::from((1, 2.., 3..=4, (5, Some(6)), Extent::from(7..=8))),
            SE::new(&[(1, Some(1)), (2, None), (3, Some(4)), (5, Some(6)), (7, Some(8))])
        );
        assert_eq!(SE::from(1), SE::new(&[1]));
        assert_eq!(SE::from(&1), SE::new(&[1]));
        assert_eq!(SE::from(1..), SE::new(&[1..]));
        assert_eq!(SE::from(&(1..)), SE::new(&[1..]));
        assert_eq!(SE::from(1..=2), SE::new(&[1..=2]));
        assert_eq!(SE::from(&(1..=2)), SE::new(&[1..=2]));
        assert_eq!(SE::from((1, Some(2))), SE::new(&[1..=2]));
        assert_eq!(SE::from(&(1, Some(2))), SE::new(&[1..=2]));
        assert_eq!(SE::from(Extent::from(1..=2)), SE::new(&[1..=2]));
        assert_eq!(SE::from(&Extent::from(1..=2)), SE::new(&[1..=2]));
        assert_eq!(SE::from(vec![1, 2]), SE::new(&[1, 2]));
        assert_eq!(SE::from(vec![1, 2].as_slice()), SE::new(&[1, 2]));
        assert_eq!(SE::from([1, 2]), SE::new(&[1, 2]));
        assert_eq!(SE::from(&[1, 2]), SE::new(&[1, 2]));
        assert_eq!(SE::from(&vec![1, 2]), SE::new(&[1, 2]));
    }

    #[test]
    pub fn test_extents() {
        let e = Extents::new(&[3, 4]);
        assert_eq!(e.ndim(), 2);
        assert_eq!(e.dims(), vec![3, 4]);
        assert_eq!(e.size(), 12);
        assert!(!e.is_scalar());
        assert!(!e.is_null());
        assert!(e.is_simple());
        assert!(e.is_valid());
        assert!(!e.is_resizable());
        assert!(!e.is_unlimited());
        assert_eq!(e.maxdims(), vec![Some(3), Some(4)]);
        assert_eq!(e.as_simple(), Some(&SimpleExtents::new(&[3, 4])));

        let e = Extents::new([1, 2]).resizable();
        assert_eq!(e.dims(), vec![1, 2]);
        assert_eq!(e.maxdims(), vec![None, None]);

        let e = Extents::new((3..=2, 4));
        assert!(!e.is_valid());

        let e = Extents::new((3.., 4));
        assert_eq!(e.ndim(), 2);
        assert_eq!(e.dims(), vec![3, 4]);
        assert_eq!(e.size(), 12);
        assert!(!e.is_resizable());
        assert!(e.is_unlimited());
        assert_eq!(e.maxdims(), vec![None, Some(4)]);

        let e = Extents::new((3.., 4..));
        assert!(e.is_resizable());
        assert!(e.is_unlimited());
        assert_eq!(e.maxdims(), vec![None, None]);

        let e = Extents::new(());
        assert!(e.is_scalar());

        let e = Extents::new([0usize; 0]);
        assert!(e.is_scalar());

        let e = Extents::null();
        assert_eq!(e.ndim(), 0);
        assert_eq!(e.dims(), vec![]);
        assert_eq!(e.size(), 0);
        assert!(!e.is_scalar());
        assert!(e.is_null());
        assert!(!e.is_simple());
        assert!(e.is_valid());

        let e = Extents::scalar();
        assert_eq!(e.ndim(), 0);
        assert_eq!(e.dims(), vec![]);
        assert_eq!(e.size(), 1);
        assert!(e.is_scalar());
        assert!(!e.is_null());
        assert!(!e.is_simple());
        assert!(e.is_valid());
    }
}

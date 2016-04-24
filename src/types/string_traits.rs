macro_rules! impl_string_eq {
    ($lhs:ty, $rhs:ty $(,$t:ident: $b:ident<$a:ident=$v:ty>)*) => {
        impl<'a $(,$t: $b<$a=$v>)*> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { PartialEq::eq(&self[..], &other[..]) }
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { PartialEq::ne(&self[..], &other[..]) }
        }

        impl<'a $(,$t: $b<$a=$v>)*> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool { PartialEq::eq(&self[..], &other[..]) }
            #[inline]
            fn ne(&self, other: &$lhs) -> bool { PartialEq::ne(&self[..], &other[..]) }
        }
    }
}

macro_rules! impl_string_traits {
    ($nm:ident, $ty:ty $(,$t:ident: $b:ident<$a:ident=$v:ty>)*) => (
        impl<'a $(,$t: $b<$a=$v>)*> From<&'a str> for $ty {
            fn from(s: &'a str) -> $ty {
                $nm::from_str(s)
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> From<String> for $ty {
            fn from(s: String) -> $ty {
                $nm::from_str(&s)
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> Into<Vec<u8>> for $ty {
            fn into(self) -> Vec<u8> {
                self.as_bytes().to_vec()
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> ::std::ops::Deref for $ty {
            type Target = str;

            #[inline]
            fn deref(&self) -> &str {
                unsafe { ::std::str::from_utf8_unchecked(self.as_bytes()) }
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> ::std::borrow::Borrow<str> for $ty {
            #[inline]
            fn borrow(&self) -> &str {
                self
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> AsRef<str> for $ty {
            #[inline]
            fn as_ref(&self) -> &str {
                self
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> ::std::ops::Index<::std::ops::RangeFull> for $ty {
            type Output = str;

            #[inline]
            fn index(&self, _: ::std::ops::RangeFull) -> &str {
                self
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> PartialEq for $ty {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                PartialEq::eq(&self[..], &other[..])
            }

            #[inline]
            fn ne(&self, other: &Self) -> bool {
                PartialEq::ne(&self[..], &other[..])
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> Eq for $ty { }

        impl_string_eq!($ty, str $(,$t: $b<$a=$v>)*);
        impl_string_eq!($ty, &'a str $(,$t: $b<$a=$v>)*);
        impl_string_eq!($ty, String $(,$t: $b<$a=$v>)*);
        impl_string_eq!($ty, ::std::borrow::Cow<'a, str> $(,$t: $b<$a=$v>)*);

        impl<'a $(,$t: $b<$a=$v>)*> ::std::fmt::Debug for $ty {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                (**self).fmt(f)
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> ::std::fmt::Display for $ty {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                (**self).fmt(f)
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> ::std::hash::Hash for $ty {
            #[inline]
            fn hash<H: ::std::hash::Hasher>(&self, hasher: &mut H) {
                use std::hash::Hash;
                (**self).hash(hasher)
            }
        }

        impl<'a $(,$t: $b<$a=$v>)*> Default for $ty {
            #[inline]
            fn default() -> $ty {
                $nm::new()
            }
        }
    )
}

#[macro_export]
macro_rules! h5def {
    ($(#[repr($t:ident)] $(#[$a:meta])* enum $s:ident { $($i:ident = $v:expr),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone, Copy)] #[repr($t)] $(#[$a])*
            enum $s { $($i = $v),+ }
            h5def!(@impl_enum $s($t) { $($i = $v),+ });
        )*
    );

    ($(#[repr($t:ident)] $(#[$a:meta])* pub enum $s:ident { $($i:ident = $v:expr),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone, Copy)] #[repr($t)] $(#[$a])*
            pub enum $s { $($i = $v),+ }
            h5def!(@impl_enum $s($t) { $($i = $v),+ });
        )*
    );

    ($($(#[$a:meta])* struct $s:ident { $($i:ident: $t:ty),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone)] #[repr(C)] $(#[$a])*
            struct $s { $($i: $t),+ }
            h5def!(@impl_struct $s { $($i: $t),+ });
        )*
    );

    ($($(#[$a:meta])* pub struct $s:ident { $($i:ident: $t:ty),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone)] #[repr(C)] $(#[$a])*
            pub struct $s { $($i: $t),+ }
            h5def!(@impl_struct $s { $($i: $t),+ });
        )*
    );

    ($($(#[$a:meta])* pub struct $s:ident { $(pub $i:ident: $t:ty),+$(,)* })*) => (
        $(
            #[allow(dead_code)] #[derive(Clone)] #[repr(C)] $(#[$a])*
            pub struct $s { $(pub $i: $t),+ }
            h5def!(@impl_struct $s { $($i: $t),+ });
        )*
    );

    (@impl_enum $s:ident($t:ident) { $($i:ident = $v:expr),+ }) => (
        unsafe impl $crate::types::ToValueType for $s {
            fn value_type() -> $crate::types::ValueType {
                use $crate::types::value_type::{ValueType, EnumType, EnumMember, IntSize};

                ValueType::Enum(
                    EnumType {
                        size: match ::std::mem::size_of::<$t>() {
                            1 => IntSize::U1, 2 => IntSize::U2, 4 => IntSize::U4, 8 => IntSize::U8,
                            _ => panic!("invalid int size"),
                        },
                        signed: ::std::$t::MIN != 0,
                        members: vec![$(
                            EnumMember {
                                name: stringify!($i).into(),
                                value: $v as $t as u64,
                            }),+],
                    }
                )
            }
        }
    );

    (@impl_struct $s:ident { $($i:ident: $t:ty),+ }) => (
        unsafe impl $crate::types::ToValueType for $s {
            fn value_type() -> $crate::types::ValueType {
                use $crate::types::value_type::{ValueType, CompoundType, CompoundField, ToValueType};

                let base = 0usize as *const $s;
                ValueType::Compound(
                    CompoundType {
                        fields: vec![$(
                            CompoundField {
                                name: stringify!($i).into(),
                                ty: <$t as ToValueType>::value_type(),
                                offset: unsafe { &((*base).$i) as *const $t as usize }
                            }),+],
                        size: ::std::mem::size_of::<$s>(),
                    }
                )
            }
        }
    );
}

#[cfg(test)]
pub mod tests {
    use types::value_type::*;
    use types::ValueType as VT;

    #[test]
    pub fn test_enum_type() {
        h5def!(#[repr(i64)] enum Foo { A = 1, B = -2 });
        assert_eq!(Foo::value_type(), VT::Enum(EnumType {
            size: IntSize::U8,
            signed: true,
            members: vec![
                EnumMember { name: "A".into(), value: 1 },
                EnumMember { name: "B".into(), value: -2i64 as u64 },
            ]
        }));
        assert_eq!(Foo::value_type().size(), 8);

        h5def!(#[repr(u8)] #[derive(Debug)] pub enum Bar { A = 1, B = 2, });
        assert_eq!(Bar::value_type(), VT::Enum(EnumType {
            size: IntSize::U1,
            signed: false,
            members: vec![
                EnumMember { name: "A".into(), value: 1 },
                EnumMember { name: "B".into(), value: 2 },
            ]
        }));
        assert_eq!(format!("{:?}", Bar::A), "A");
        assert_eq!(Bar::value_type().size(), 1);

        h5def!(#[repr(u8)] enum E1 { A = 1, B = 2 }
               #[repr(u8)] enum E2 { A = 1, B = 2});
        assert_eq!(E1::value_type(), Bar::value_type());
        assert_eq!(E2::value_type(), Bar::value_type());

        h5def!(#[repr(u8)] pub enum E3 { A = 1, B = 2 }
               #[repr(u8)] pub enum E4 { A = 1, B = 2});
        assert_eq!(E3::value_type(), Bar::value_type());
        assert_eq!(E4::value_type(), Bar::value_type());
    }

    #[test]
    pub fn test_compound_type() {
        h5def!(struct Foo { a: i64, b: u64 });
        assert_eq!(Foo::value_type(), VT::Compound(CompoundType {
            fields: vec![
                CompoundField { name: "a".into(), ty: i64::value_type(), offset: 0 },
                CompoundField { name: "b".into(), ty: u64::value_type(), offset: 8 },
            ],
            size: 16,
        }));
        assert_eq!(Foo::value_type().size(), 16);

        h5def!(pub struct Bar { a: i64, b: u64 });
        assert_eq!(Bar::value_type(), Foo::value_type());

        h5def!(#[derive(Debug)] pub struct Baz { pub a: i64, pub b: u64 });
        assert_eq!(Baz::value_type(), Foo::value_type());
        assert!(format!("{:?}", Baz { a: 1, b: 2 }).len() > 0);

        h5def!(struct S1 { a: i64, b: u64 }
               struct S2 { a: i64, b: u64 } );
        assert_eq!(S1::value_type(), Foo::value_type());
        assert_eq!(S2::value_type(), Foo::value_type());

        h5def!(pub struct S3 { a: i64, b: u64 }
               pub struct S4 { a: i64, b: u64 } );
        assert_eq!(S3::value_type(), Foo::value_type());
        assert_eq!(S4::value_type(), Foo::value_type());

        h5def!(pub struct S5 { pub a: i64, pub b: u64 }
               pub struct S6 { pub a: i64, pub b: u64 });
        assert_eq!(S5::value_type(), Foo::value_type());
        assert_eq!(S6::value_type(), Foo::value_type());
    }
}

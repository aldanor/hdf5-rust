#[macro_use]
extern crate hdf5_derive;

use std::marker::PhantomData;
use std::mem;

use hdf5::types::TypeDescriptor as TD;
use hdf5::types::*;

#[derive(H5Type)]
#[repr(C)]
struct A {
    a: i64,
    b: u64,
}

#[derive(H5Type)]
#[repr(C)]
struct B {
    a: [A; 4],
    b: FixedAscii<8>,
    c: VarLenArray<f64>,
    d: bool,
    e: FixedUnicode<7>,
    f: VarLenAscii,
    g: VarLenUnicode,
}

#[derive(H5Type)]
#[repr(C)]
struct T(i64, pub u64);

#[derive(H5Type, Copy, Clone)]
#[repr(packed)]
struct P1 {
    x: u8,
    y: u64,
}

#[derive(H5Type, Copy, Clone)]
#[repr(packed)]
struct P2(i8, u32);

#[derive(H5Type)]
#[repr(transparent)]
struct T1 {
    _x: u64,
}

#[derive(H5Type)]
#[repr(transparent)]
struct T2(i32);

#[test]
fn test_compound_packed() {
    assert_eq!(
        P1::type_descriptor(),
        TD::Compound(CompoundType {
            fields: vec![
                CompoundField::typed::<u8>("x", 0, 0),
                CompoundField::typed::<u64>("y", 1, 1),
            ],
            size: 9,
        })
    );
    assert_eq!(
        P2::type_descriptor(),
        TD::Compound(CompoundType {
            fields: vec![
                CompoundField::typed::<i8>("0", 0, 0),
                CompoundField::typed::<u32>("1", 1, 1),
            ],
            size: 5,
        })
    );
}

#[test]
fn test_compound_transparent() {
    assert_eq!(T1::type_descriptor(), u64::type_descriptor(),);
    assert_eq!(T2::type_descriptor(), i32::type_descriptor(),);
}

#[test]
fn test_compound_simple() {
    assert_eq!(
        A::type_descriptor(),
        TD::Compound(CompoundType {
            fields: vec![
                CompoundField::typed::<i64>("a", 0, 0),
                CompoundField::typed::<u64>("b", 8, 1),
            ],
            size: 16,
        })
    );
    assert_eq!(A::type_descriptor().size(), 16);
}

#[test]
fn test_compound_complex() {
    assert_eq!(
        B::type_descriptor(),
        TD::Compound(CompoundType {
            fields: vec![
                CompoundField::new("a", TD::FixedArray(Box::new(A::type_descriptor()), 4), 0, 0),
                CompoundField::new("b", TD::FixedAscii(8), 64, 1),
                CompoundField::new("c", TD::VarLenArray(Box::new(TD::Float(FloatSize::U8))), 72, 2),
                CompoundField::new("d", TD::Boolean, 88, 3),
                CompoundField::new("e", TD::FixedUnicode(7), 89, 4),
                CompoundField::new("f", TD::VarLenAscii, 96, 5),
                CompoundField::new("g", TD::VarLenUnicode, 104, 6),
            ],
            size: 112,
        })
    );
    assert_eq!(B::type_descriptor().size(), 112);
}

#[test]
fn test_compound_tuple() {
    assert_eq!(
        T::type_descriptor(),
        TD::Compound(CompoundType {
            fields: vec![
                CompoundField::typed::<i64>("0", 0, 0),
                CompoundField::typed::<u64>("1", 8, 1),
            ],
            size: 16,
        })
    );
    assert_eq!(T::type_descriptor().size(), 16);
}

#[derive(H5Type, Clone, Copy)]
#[repr(i16)]
#[allow(dead_code)]
enum E1 {
    X = -2,
    Y = 3,
}

#[test]
fn test_enum_simple() {
    assert_eq!(
        E1::type_descriptor(),
        TD::Enum(EnumType {
            size: IntSize::U2,
            signed: true,
            members: vec![
                EnumMember { name: "X".into(), value: -2i16 as _ },
                EnumMember { name: "Y".into(), value: 3u64 },
            ]
        })
    );
    assert_eq!(E1::type_descriptor().size(), 2);
}

#[test]
fn test_enum_base_type() {
    macro_rules! check_base_type {
        ($ty:ident, $signed:expr, $size:expr) => {{
            #[repr($ty)]
            #[allow(dead_code)]
            #[derive(H5Type)]
            enum E {
                X = 42,
            }
            let td = E::type_descriptor();
            assert_eq!(td.size(), mem::size_of::<$ty>());
            assert_eq!(td.size(), mem::size_of::<E>());
            match td {
                TD::Enum(e) => {
                    assert_eq!(e.signed, ::std::$ty::MIN != 0);
                    assert_eq!(e.size, IntSize::from_int($size).unwrap());
                    assert_eq!(e.members.len(), 1);
                    assert_eq!(e.members[0].name, "X");
                    assert_eq!(e.members[0].value as $ty, 42);
                }
                _ => panic!(),
            }
        }};
    }

    check_base_type!(u8, false, 1);
    check_base_type!(u16, false, 2);
    check_base_type!(u32, false, 4);
    check_base_type!(u64, false, 8);
    check_base_type!(i8, true, 1);
    check_base_type!(i16, true, 2);
    check_base_type!(i32, true, 4);
    check_base_type!(i64, true, 8);
    check_base_type!(usize, false, mem::size_of::<usize>());
    check_base_type!(isize, true, mem::size_of::<isize>());
}

#[derive(H5Type)]
#[repr(C)]
struct G1<T: H5Type> {
    x: u32,
    y: T,
    z: f32,
}

#[derive(H5Type)]
#[repr(C)]
struct C1 {
    x: u32,
    y: i64,
    z: f32,
}

#[derive(H5Type)]
#[repr(C)]
struct G2<T: H5Type>(u32, T, f32);

#[derive(H5Type)]
#[repr(C)]
struct C2(u32, i64, f32);

#[test]
fn test_generics() {
    assert_eq!(G1::<i64>::type_descriptor(), C1::type_descriptor());
    assert_eq!(G2::<i64>::type_descriptor(), C2::type_descriptor());
}

#[derive(H5Type)]
#[repr(C)]
struct G3<T: 'static> {
    x: i16,
    y: PhantomData<T>,
    z: u32,
}

#[derive(H5Type)]
#[repr(C)]
struct C3 {
    x: i16,
    z: u32,
}

#[derive(H5Type)]
#[repr(C)]
struct G4<T: 'static>(i16, PhantomData<T>, u32);

#[derive(H5Type)]
#[repr(C)]
struct C4(i16, u32);

#[test]
fn test_phantom_data() {
    assert_eq!(G3::<String>::type_descriptor(), C3::type_descriptor());
    assert_eq!(G4::<String>::type_descriptor(), C4::type_descriptor());
}

extern crate hdf5_types;
#[macro_use]
extern crate hdf5_types_derive;

use hdf5_types::TypeDescriptor as TD;
use hdf5_types::*;

#[derive(H5Type)]
struct A {
    a: i64,
    b: u64,
}

#[derive(H5Type)]
#[repr(C)]
struct B {
    a: [A; 4],
    b: FixedAscii<[u8; 8]>,
    c: VarLenArray<f64>,
    d: bool,
    e: FixedUnicode<[u8; 7]>,
    f: VarLenAscii,
    g: VarLenUnicode,
}

#[derive(H5Type)]
struct U;

#[derive(H5Type)]
struct T(i64, pub u64);

#[test]
fn test_compound_unit() {
    assert_eq!(U::type_descriptor(),
               TD::Compound(CompoundType {
                   fields: vec![],
                   size: 0,
               }));
}

#[test]
fn test_compound_simple() {
    assert_eq!(A::type_descriptor(),
               TD::Compound(CompoundType {
                   fields: vec![
                       CompoundField {
                           name: "a".into(),
                           ty: TD::Integer(IntSize::U8),
                           offset: 0,
                       },
                       CompoundField {
                           name: "b".into(),
                           ty: TD::Unsigned(IntSize::U8),
                           offset: 8,
                       }],
                   size: 16,
               }));
    assert_eq!(A::type_descriptor().size(), 16);
}

#[test]
fn test_compound_complex() {
    assert_eq!(B::type_descriptor(),
               TD::Compound(CompoundType {
                   fields: vec![
                       CompoundField {
                           name: "a".into(),
                           ty: TD::FixedArray(Box::new(A::type_descriptor()), 4),
                           offset: 0,
                       },
                       CompoundField {
                           name: "b".into(),
                           ty: TD::FixedAscii(8),
                           offset: 64,
                       },
                       CompoundField {
                           name: "c".into(),
                           ty: TD::VarLenArray(Box::new(TD::Float(FloatSize::U8))),
                           offset: 72,
                       },
                       CompoundField {
                           name: "d".into(),
                           ty: TD::Boolean,
                           offset: 88,
                       },
                       CompoundField {
                           name: "e".into(),
                           ty: TD::FixedUnicode(7),
                           offset: 89,
                       },
                       CompoundField {
                           name: "f".into(),
                           ty: TD::VarLenAscii,
                           offset: 96,
                       },
                       CompoundField {
                           name: "g".into(),
                           ty: TD::VarLenUnicode,
                           offset: 104,
                       }],
                   size: 112,
               }));
    assert_eq!(B::type_descriptor().size(), 112);
}

#[test]
fn test_compound_tuple() {
    assert_eq!(T::type_descriptor(),
               TD::Compound(CompoundType {
                   fields: vec![
                       CompoundField {
                           name: "0".into(),
                           ty: TD::Integer(IntSize::U8),
                           offset: 0,
                       },
                       CompoundField {
                           name: "1".into(),
                           ty: TD::Unsigned(IntSize::U8),
                           offset: 8,
                       }],
                   size: 16,
               }));
    assert_eq!(T::type_descriptor().size(), 16);
}

#[derive(H5Type)]
#[derive(Clone, Copy)]
#[repr(C)]
#[repr(i16)]
#[repr(isize)]
#[allow(dead_code)]
enum E1 {
    X = -2,
    Y = 3,
}

#[test]
fn test_enum_simple() {
    assert_eq!(E1::type_descriptor(),
               TD::Enum(EnumType {
                   size: IntSize::U2,
                   signed: true,
                   members: vec![
                       EnumMember { name: "X".into(), value: -2i16 as u64 },
                       EnumMember { name: "Y".into(), value: 3u64 },
                   ]
               }));
    assert_eq!(E1::type_descriptor().size(), 2);
}

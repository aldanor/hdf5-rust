use std::fmt::{self, Debug, Display};
use std::mem;
use std::ptr;
use std::slice;

use libc;

use crate::h5type::{hvl_t, CompoundType, EnumType, FloatSize, H5Type, IntSize, TypeDescriptor};
use crate::string::{VarLenAscii, VarLenUnicode};

fn read_raw<T: Copy>(buf: &[u8]) -> T {
    debug_assert_eq!(mem::size_of::<T>(), buf.len());
    unsafe { *(buf.as_ptr() as *const T) }
}

fn write_raw<T: Copy>(out: &mut [u8], value: T) {
    debug_assert_eq!(mem::size_of::<T>(), out.len());
    unsafe {
        *(out.as_mut_ptr() as *mut T) = value;
    }
}

unsafe trait DynDrop {
    fn dyn_drop(&mut self) {}
}

unsafe trait DynClone {
    fn dyn_clone(&mut self, out: &mut [u8]);
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DynInteger {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
}

impl DynInteger {
    pub(self) fn read(buf: &[u8], signed: bool, size: IntSize) -> Self {
        use DynInteger::*;
        match (signed, size) {
            (true, IntSize::U1) => Int8(read_raw(buf)),
            (true, IntSize::U2) => Int16(read_raw(buf)),
            (true, IntSize::U4) => Int32(read_raw(buf)),
            (true, IntSize::U8) => Int64(read_raw(buf)),
            (false, IntSize::U1) => UInt8(read_raw(buf)),
            (false, IntSize::U2) => UInt16(read_raw(buf)),
            (false, IntSize::U4) => UInt32(read_raw(buf)),
            (false, IntSize::U8) => UInt64(read_raw(buf)),
        }
    }

    pub(self) fn as_u64(self) -> u64 {
        use DynInteger::*;
        match self {
            Int8(x) => x as _,
            Int16(x) => x as _,
            Int32(x) => x as _,
            Int64(x) => x as _,
            UInt8(x) => x as _,
            UInt16(x) => x as _,
            UInt32(x) => x as _,
            UInt64(x) => x as _,
        }
    }
}

unsafe impl DynClone for DynInteger {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        use DynInteger::*;
        match self {
            Int8(x) => write_raw(out, *x),
            Int16(x) => write_raw(out, *x),
            Int32(x) => write_raw(out, *x),
            Int64(x) => write_raw(out, *x),
            UInt8(x) => write_raw(out, *x),
            UInt16(x) => write_raw(out, *x),
            UInt32(x) => write_raw(out, *x),
            UInt64(x) => write_raw(out, *x),
        }
    }
}

impl Debug for DynInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DynInteger::*;
        match *self {
            Int8(x) => Debug::fmt(&x, f),
            Int16(x) => Debug::fmt(&x, f),
            Int32(x) => Debug::fmt(&x, f),
            Int64(x) => Debug::fmt(&x, f),
            UInt8(x) => Debug::fmt(&x, f),
            UInt16(x) => Debug::fmt(&x, f),
            UInt32(x) => Debug::fmt(&x, f),
            UInt64(x) => Debug::fmt(&x, f),
        }
    }
}

impl Display for DynInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl From<DynInteger> for DynScalar {
    fn from(value: DynInteger) -> Self {
        DynScalar::Integer(value)
    }
}

impl From<DynInteger> for DynValue<'_> {
    fn from(value: DynInteger) -> Self {
        DynScalar::Integer(value).into()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum DynScalar {
    Integer(DynInteger),
    Float32(f32),
    Float64(f64),
    Boolean(bool),
}

unsafe impl DynClone for DynScalar {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        use DynScalar::*;
        match self {
            Integer(x) => x.dyn_clone(out),
            Float32(x) => write_raw(out, *x),
            Float64(x) => write_raw(out, *x),
            Boolean(x) => write_raw(out, *x),
        }
    }
}

impl Debug for DynScalar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DynScalar::*;
        match self {
            Integer(x) => Debug::fmt(&x, f),
            Float32(x) => Debug::fmt(&x, f),
            Float64(x) => Debug::fmt(&x, f),
            Boolean(x) => Debug::fmt(&x, f),
        }
    }
}

impl Display for DynScalar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl From<DynScalar> for DynValue<'static> {
    fn from(value: DynScalar) -> Self {
        DynValue::Scalar(value)
    }
}

#[derive(Copy, Clone)]
pub struct DynEnum<'a> {
    tp: &'a EnumType,
    value: DynInteger,
}

impl<'a> DynEnum<'a> {
    pub fn new(tp: &'a EnumType, value: DynInteger) -> Self {
        Self { tp, value }
    }

    pub fn name(&self) -> Option<&str> {
        let value = self.value.as_u64();
        for member in &self.tp.members {
            if member.value == value {
                return Some(&member.name);
            }
        }
        None
    }
}

unsafe impl DynClone for DynEnum<'_> {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        self.value.dyn_clone(out)
    }
}

impl PartialEq for DynEnum<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for DynEnum<'_> {}

impl Debug for DynEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.name() {
            Some(name) => f.write_str(name),
            None => Debug::fmt(&self.value, f),
        }
    }
}

impl Display for DynEnum<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<'a> From<DynEnum<'a>> for DynValue<'a> {
    fn from(value: DynEnum<'a>) -> Self {
        DynValue::Enum(value)
    }
}

pub struct DynCompound<'a> {
    tp: &'a CompoundType,
    buf: &'a [u8],
}

impl<'a> DynCompound<'a> {
    pub fn new(tp: &'a CompoundType, buf: &'a [u8]) -> Self {
        Self { tp, buf }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, DynValue)> {
        self.tp.fields.iter().map(move |field| {
            (
                field.name.as_ref(),
                DynValue::new(&field.ty, &self.buf[field.offset..(field.offset + field.ty.size())]),
            )
        })
    }
}

unsafe impl DynDrop for DynCompound<'_> {
    fn dyn_drop(&mut self) {
        for (_, mut value) in self.iter() {
            value.dyn_drop();
        }
    }
}

unsafe impl DynClone for DynCompound<'_> {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), self.tp.size);
        for (i, (_, mut value)) in self.iter().enumerate() {
            let field = &self.tp.fields[i];
            value.dyn_clone(&mut out[field.offset..(field.offset + field.ty.size())]);
        }
    }
}

impl PartialEq for DynCompound<'_> {
    fn eq(&self, other: &Self) -> bool {
        let (mut it1, mut it2) = (self.iter(), other.iter());
        loop {
            match (it1.next(), it2.next()) {
                (Some(v1), Some(v2)) => {
                    if v1 != v2 {
                        return false;
                    }
                }
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}

struct RawStr<'a>(&'a str);

impl Debug for RawStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl Debug for DynCompound<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut b = f.debug_map();
        for (name, value) in self.iter() {
            b.entry(&RawStr(name), &value);
        }
        b.finish()
    }
}

impl Display for DynCompound<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<'a> From<DynCompound<'a>> for DynValue<'a> {
    fn from(value: DynCompound<'a>) -> Self {
        DynValue::Compound(value)
    }
}

pub struct DynArray<'a> {
    tp: &'a TypeDescriptor,
    buf: &'a [u8],
    len: Option<usize>,
}

impl<'a> DynArray<'a> {
    pub fn new(tp: &'a TypeDescriptor, buf: &'a [u8], len: Option<usize>) -> Self {
        Self { tp, buf, len }
    }

    fn get_ptr(&self) -> *const u8 {
        match self.len {
            Some(_) => self.buf.as_ptr(),
            None => read_raw::<hvl_t>(self.buf).ptr as *const u8,
        }
    }

    fn get_len(&self) -> usize {
        match self.len {
            Some(len) => len,
            None => read_raw::<hvl_t>(self.buf).len,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = DynValue> {
        let ptr = self.get_ptr();
        let len = self.get_len();
        let size = self.tp.size();
        let buf = if !ptr.is_null() && len != 0 {
            unsafe { slice::from_raw_parts(ptr, len * size) }
        } else {
            [].as_ref()
        };
        (0..len).map(move |i| DynValue::new(&self.tp, &buf[(i * size)..((i + 1) * size)]))
    }
}

unsafe impl DynDrop for DynArray<'_> {
    fn dyn_drop(&mut self) {
        for mut value in self.iter() {
            value.dyn_drop();
        }
        if self.len.is_none() && !self.get_ptr().is_null() {
            unsafe {
                libc::free(self.get_ptr() as *mut _);
            }
        }
    }
}

unsafe impl DynClone for DynArray<'_> {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        let (len, ptr, size) = (self.get_len(), self.get_ptr(), self.tp.size());
        let out = if self.len.is_none() {
            debug_assert_eq!(out.len(), mem::size_of::<hvl_t>());
            if !self.get_ptr().is_null() {
                unsafe {
                    let dst = libc::malloc(len * size) as *mut u8;
                    ptr::copy_nonoverlapping(ptr, dst, len * size);
                    (*(out.as_mut_ptr() as *mut hvl_t)).ptr = dst as _;
                    slice::from_raw_parts_mut(dst, len * size)
                }
            } else {
                return;
            }
        } else {
            out
        };
        debug_assert_eq!(out.len(), len * size);
        for (i, mut value) in self.iter().enumerate() {
            value.dyn_clone(&mut out[(i * size)..((i + 1) * size)]);
        }
    }
}

impl PartialEq for DynArray<'_> {
    fn eq(&self, other: &Self) -> bool {
        let (mut it1, mut it2) = (self.iter(), other.iter());
        loop {
            match (it1.next(), it2.next()) {
                (Some(v1), Some(v2)) => {
                    if v1 != v2 {
                        return false;
                    }
                }
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}

impl Debug for DynArray<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut b = f.debug_list();
        for value in self.iter() {
            b.entry(&value);
        }
        b.finish()
    }
}

impl Display for DynArray<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<'a> From<DynArray<'a>> for DynValue<'a> {
    fn from(value: DynArray<'a>) -> Self {
        DynValue::Array(value)
    }
}

pub struct DynFixedString<'a> {
    buf: &'a [u8],
    unicode: bool,
}

impl<'a> DynFixedString<'a> {
    pub fn new(buf: &'a [u8], unicode: bool) -> Self {
        Self { buf, unicode }
    }

    pub fn raw_len(&self) -> usize {
        self.buf.iter().rev().skip_while(|&c| *c == 0).count()
    }

    pub fn get_buf(&self) -> &[u8] {
        &self.buf[..self.raw_len()]
    }
}

unsafe impl DynClone for DynFixedString<'_> {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        debug_assert_eq!(self.buf.len(), out.len());
        out.clone_from_slice(self.buf);
    }
}

impl PartialEq for DynFixedString<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.unicode == other.unicode && self.get_buf() == other.get_buf()
    }
}

impl Eq for DynFixedString<'_> {}

impl Debug for DynFixedString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = unsafe { mem::transmute::<_, &str>(self.get_buf()) };
        Debug::fmt(&s, f)
    }
}

impl Display for DynFixedString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<'a> From<DynFixedString<'a>> for DynString<'a> {
    fn from(value: DynFixedString<'a>) -> Self {
        DynString::Fixed(value)
    }
}

impl<'a> From<DynFixedString<'a>> for DynValue<'a> {
    fn from(value: DynFixedString<'a>) -> Self {
        DynString::Fixed(value).into()
    }
}

pub struct DynVarLenString<'a> {
    buf: &'a [u8],
    unicode: bool,
}

impl<'a> DynVarLenString<'a> {
    pub fn new(buf: &'a [u8], unicode: bool) -> Self {
        Self { buf, unicode }
    }

    fn get_ptr(&self) -> *const u8 {
        if self.unicode {
            self.as_unicode().as_ptr()
        } else {
            self.as_ascii().as_ptr()
        }
    }

    fn raw_len(&self) -> usize {
        if self.unicode {
            self.as_unicode().as_bytes().len()
        } else {
            self.as_ascii().as_bytes().len()
        }
    }

    fn as_ascii(&self) -> &VarLenAscii {
        unsafe { &*(self.buf.as_ptr() as *const VarLenAscii) }
    }

    fn as_unicode(&self) -> &VarLenUnicode {
        unsafe { &*(self.buf.as_ptr() as *const VarLenUnicode) }
    }
}

unsafe impl DynDrop for DynVarLenString<'_> {
    fn dyn_drop(&mut self) {
        if !self.get_ptr().is_null() {
            unsafe {
                libc::free(self.get_ptr() as *mut _);
            }
        }
    }
}

unsafe impl DynClone for DynVarLenString<'_> {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), mem::size_of::<usize>());
        if !self.get_ptr().is_null() {
            unsafe {
                let raw_len = self.raw_len();
                let dst = libc::malloc(raw_len + 1) as *mut _;
                ptr::copy_nonoverlapping(self.get_ptr(), dst, raw_len);
                *dst.add(raw_len) = 0;
                *(out.as_mut_ptr() as *mut *const u8) = dst as _;
            }
        }
    }
}

impl PartialEq for DynVarLenString<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self.unicode, other.unicode) {
            (true, true) => self.as_unicode() == other.as_unicode(),
            (false, false) => self.as_ascii() == other.as_ascii(),
            _ => false,
        }
    }
}

impl Eq for DynVarLenString<'_> {}

impl Debug for DynVarLenString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.unicode {
            Debug::fmt(&self.as_unicode(), f)
        } else {
            Debug::fmt(&self.as_ascii(), f)
        }
    }
}

impl Display for DynVarLenString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<'a> From<DynVarLenString<'a>> for DynString<'a> {
    fn from(value: DynVarLenString<'a>) -> Self {
        DynString::VarLen(value)
    }
}

impl<'a> From<DynVarLenString<'a>> for DynValue<'a> {
    fn from(value: DynVarLenString<'a>) -> Self {
        DynString::VarLen(value).into()
    }
}

#[derive(PartialEq, Eq)]
pub enum DynString<'a> {
    Fixed(DynFixedString<'a>),
    VarLen(DynVarLenString<'a>),
}

unsafe impl DynDrop for DynString<'_> {
    fn dyn_drop(&mut self) {
        if let DynString::VarLen(string) = self {
            string.dyn_drop();
        }
    }
}

unsafe impl DynClone for DynString<'_> {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        use DynString::*;
        match self {
            Fixed(x) => x.dyn_clone(out),
            VarLen(x) => x.dyn_clone(out),
        }
    }
}

impl Debug for DynString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DynString::*;
        match self {
            Fixed(x) => Debug::fmt(&x, f),
            VarLen(x) => Debug::fmt(&x, f),
        }
    }
}

impl Display for DynString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<'a> From<DynString<'a>> for DynValue<'a> {
    fn from(value: DynString<'a>) -> Self {
        DynValue::String(value)
    }
}

#[derive(PartialEq)]
pub enum DynValue<'a> {
    Scalar(DynScalar),
    Enum(DynEnum<'a>),
    Compound(DynCompound<'a>),
    Array(DynArray<'a>),
    String(DynString<'a>),
}

impl<'a> DynValue<'a> {
    pub fn new(tp: &'a TypeDescriptor, buf: &'a [u8]) -> Self {
        use TypeDescriptor::*;
        debug_assert_eq!(tp.size(), buf.len());

        match tp {
            Integer(size) => DynInteger::read(buf, true, *size).into(),
            Unsigned(size) => DynInteger::read(buf, true, *size).into(),
            Float(FloatSize::U4) => DynScalar::Float32(read_raw(buf)).into(),
            Float(FloatSize::U8) => DynScalar::Float64(read_raw(buf)).into(),
            Boolean => DynScalar::Boolean(read_raw(buf)).into(),
            Enum(ref tp) => DynEnum::new(tp, DynInteger::read(buf, tp.signed, tp.size)).into(),
            Compound(ref tp) => DynCompound::new(tp, buf).into(),
            FixedArray(ref tp, n) => DynArray::new(tp, buf, Some(*n)).into(),
            VarLenArray(ref tp) => DynArray::new(tp, buf, None).into(),
            FixedAscii(_) => DynFixedString::new(buf, false).into(),
            FixedUnicode(_) => DynFixedString::new(buf, true).into(),
            VarLenAscii => DynVarLenString::new(buf, false).into(),
            VarLenUnicode => DynVarLenString::new(buf, true).into(),
        }
    }
}

unsafe impl DynDrop for DynValue<'_> {
    fn dyn_drop(&mut self) {
        use DynValue::*;
        match self {
            Compound(x) => x.dyn_drop(),
            Array(x) => x.dyn_drop(),
            String(x) => x.dyn_drop(),
            _ => (),
        }
    }
}

unsafe impl DynClone for DynValue<'_> {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        use DynValue::*;
        match self {
            Scalar(x) => x.dyn_clone(out),
            Enum(x) => x.dyn_clone(out),
            Compound(x) => x.dyn_clone(out),
            Array(x) => x.dyn_clone(out),
            String(x) => x.dyn_clone(out),
        }
    }
}

impl Debug for DynValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use DynValue::*;
        match self {
            Scalar(x) => Debug::fmt(&x, f),
            Enum(x) => Debug::fmt(&x, f),
            Compound(x) => Debug::fmt(&x, f),
            Array(x) => Debug::fmt(&x, f),
            String(x) => Debug::fmt(&x, f),
        }
    }
}

impl Display for DynValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

pub struct OwnedDynValue {
    tp: TypeDescriptor,
    buf: Vec<u8>,
}

impl OwnedDynValue {
    pub fn new<T: H5Type>(value: T) -> Self {
        let ptr = &value as *const _ as *const u8;
        let len = mem::size_of_val(&value);
        let buf = unsafe { std::slice::from_raw_parts(ptr, len) };
        mem::forget(value);
        Self { tp: T::type_descriptor(), buf: buf.to_owned() }
    }

    pub fn get(&self) -> DynValue {
        DynValue::new(&self.tp, &self.buf)
    }
}

impl Drop for OwnedDynValue {
    fn drop(&mut self) {
        self.get().dyn_drop()
    }
}

impl Clone for OwnedDynValue {
    fn clone(&self) -> Self {
        let mut buf = self.buf.clone();
        self.get().dyn_clone(&mut buf);
        Self { tp: self.tp.clone(), buf }
    }
}

impl PartialEq for OwnedDynValue {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl Debug for OwnedDynValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.get(), f)
    }
}

impl Display for OwnedDynValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
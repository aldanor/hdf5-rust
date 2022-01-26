use std::fmt::{self, Debug, Display};
use std::mem;
use std::ptr;
use std::slice;

use crate::h5type::{hvl_t, CompoundType, EnumType, FloatSize, H5Type, IntSize, TypeDescriptor};
use crate::string::{VarLenAscii, VarLenUnicode};

fn read_raw<T: Copy>(buf: &[u8]) -> T {
    debug_assert_eq!(mem::size_of::<T>(), buf.len());
    unsafe { *(buf.as_ptr().cast::<T>()) }
}

fn write_raw<T: Copy>(out: &mut [u8], value: T) {
    debug_assert_eq!(mem::size_of::<T>(), out.len());
    unsafe {
        *(out.as_mut_ptr().cast()) = value;
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
        match (signed, size) {
            (true, IntSize::U1) => Self::Int8(read_raw(buf)),
            (true, IntSize::U2) => Self::Int16(read_raw(buf)),
            (true, IntSize::U4) => Self::Int32(read_raw(buf)),
            (true, IntSize::U8) => Self::Int64(read_raw(buf)),
            (false, IntSize::U1) => Self::UInt8(read_raw(buf)),
            (false, IntSize::U2) => Self::UInt16(read_raw(buf)),
            (false, IntSize::U4) => Self::UInt32(read_raw(buf)),
            (false, IntSize::U8) => Self::UInt64(read_raw(buf)),
        }
    }

    pub(self) fn as_u64(self) -> u64 {
        match self {
            Self::Int8(x) => x as _,
            Self::Int16(x) => x as _,
            Self::Int32(x) => x as _,
            Self::Int64(x) => x as _,
            Self::UInt8(x) => x as _,
            Self::UInt16(x) => x as _,
            Self::UInt32(x) => x as _,
            Self::UInt64(x) => x as _,
        }
    }
}

unsafe impl DynClone for DynInteger {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        match self {
            Self::Int8(x) => write_raw(out, *x),
            Self::Int16(x) => write_raw(out, *x),
            Self::Int32(x) => write_raw(out, *x),
            Self::Int64(x) => write_raw(out, *x),
            Self::UInt8(x) => write_raw(out, *x),
            Self::UInt16(x) => write_raw(out, *x),
            Self::UInt32(x) => write_raw(out, *x),
            Self::UInt64(x) => write_raw(out, *x),
        }
    }
}

impl Debug for DynInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Int8(x) => Debug::fmt(&x, f),
            Self::Int16(x) => Debug::fmt(&x, f),
            Self::Int32(x) => Debug::fmt(&x, f),
            Self::Int64(x) => Debug::fmt(&x, f),
            Self::UInt8(x) => Debug::fmt(&x, f),
            Self::UInt16(x) => Debug::fmt(&x, f),
            Self::UInt32(x) => Debug::fmt(&x, f),
            Self::UInt64(x) => Debug::fmt(&x, f),
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
        Self::Integer(value)
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
        match self {
            Self::Integer(x) => x.dyn_clone(out),
            Self::Float32(x) => write_raw(out, *x),
            Self::Float64(x) => write_raw(out, *x),
            Self::Boolean(x) => write_raw(out, *x),
        }
    }
}

impl Debug for DynScalar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Integer(x) => Debug::fmt(&x, f),
            Self::Float32(x) => Debug::fmt(&x, f),
            Self::Float64(x) => Debug::fmt(&x, f),
            Self::Boolean(x) => Debug::fmt(&x, f),
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
        self.value.dyn_clone(out);
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
        (0..len).map(move |i| DynValue::new(self.tp, &buf[(i * size)..((i + 1) * size)]))
    }
}

unsafe impl DynDrop for DynArray<'_> {
    fn dyn_drop(&mut self) {
        for mut value in self.iter() {
            value.dyn_drop();
        }
        if self.len.is_none() && !self.get_ptr().is_null() {
            unsafe {
                crate::free(self.get_ptr() as *mut _);
            }
        }
    }
}

unsafe impl DynClone for DynArray<'_> {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        let (len, ptr, size) = (self.get_len(), self.get_ptr(), self.tp.size());
        let out = if self.len.is_none() {
            debug_assert_eq!(out.len(), mem::size_of::<hvl_t>());
            if self.get_ptr().is_null() {
                return;
            }
            unsafe {
                let dst = crate::malloc(len * size).cast();
                ptr::copy_nonoverlapping(ptr, dst, len * size);
                // Alignment is always at least usize for pointers from `hdf5-c`
                let outptr = out.as_mut_ptr().cast::<hvl_t>();
                ptr::write(ptr::addr_of_mut!((*outptr).ptr), dst.cast());
                slice::from_raw_parts_mut(dst, len * size)
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
        let s = unsafe { std::str::from_utf8_unchecked(self.get_buf()) };
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
        // Alignment is always at least usize for pointers from `hdf5-c`
        unsafe { &*(self.buf.as_ptr().cast::<VarLenAscii>()) }
    }

    fn as_unicode(&self) -> &VarLenUnicode {
        // Alignment is always at least usize for pointers from `hdf5-c`
        unsafe { &*(self.buf.as_ptr().cast::<VarLenUnicode>()) }
    }
}

unsafe impl DynDrop for DynVarLenString<'_> {
    fn dyn_drop(&mut self) {
        if !self.get_ptr().is_null() {
            unsafe {
                crate::free(self.get_ptr() as *mut _);
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
                let dst = crate::malloc(raw_len + 1).cast();
                ptr::copy_nonoverlapping(self.get_ptr(), dst, raw_len);
                dst.add(raw_len).write(0);
                // Alignment is always at least usize for pointers from `hdf5-c`
                let outptr = out.as_mut_ptr().cast::<*const u8>();
                ptr::write(outptr, dst.cast());
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
        match self {
            Self::Fixed(x) => x.dyn_clone(out),
            Self::VarLen(x) => x.dyn_clone(out),
        }
    }
}

impl Debug for DynString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Fixed(x) => Debug::fmt(&x, f),
            Self::VarLen(x) => Debug::fmt(&x, f),
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
            Integer(size) | Unsigned(size) => DynInteger::read(buf, true, *size).into(),
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
        match self {
            Self::Compound(x) => x.dyn_drop(),
            Self::Array(x) => x.dyn_drop(),
            Self::String(x) => x.dyn_drop(),
            _ => (),
        }
    }
}

unsafe impl DynClone for DynValue<'_> {
    fn dyn_clone(&mut self, out: &mut [u8]) {
        match self {
            Self::Scalar(x) => x.dyn_clone(out),
            Self::Enum(x) => x.dyn_clone(out),
            Self::Compound(x) => x.dyn_clone(out),
            Self::Array(x) => x.dyn_clone(out),
            Self::String(x) => x.dyn_clone(out),
        }
    }
}

impl Debug for DynValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Scalar(x) => Debug::fmt(&x, f),
            Self::Enum(x) => Debug::fmt(&x, f),
            Self::Compound(x) => Debug::fmt(&x, f),
            Self::Array(x) => Debug::fmt(&x, f),
            Self::String(x) => Debug::fmt(&x, f),
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
    buf: Box<[u8]>,
}

impl OwnedDynValue {
    pub fn new<T: H5Type>(value: T) -> Self {
        let ptr = (&value as *const T).cast::<u8>();
        let len = mem::size_of_val(&value);
        let buf = unsafe { std::slice::from_raw_parts(ptr, len) };
        mem::forget(value);
        Self { tp: T::type_descriptor(), buf: buf.to_owned().into_boxed_slice() }
    }

    pub fn get(&self) -> DynValue {
        DynValue::new(&self.tp, &self.buf)
    }

    pub fn type_descriptor(&self) -> &TypeDescriptor {
        &self.tp
    }

    #[doc(hidden)]
    pub unsafe fn get_buf(&self) -> &[u8] {
        &self.buf
    }

    #[doc(hidden)]
    pub unsafe fn from_raw(tp: TypeDescriptor, buf: Box<[u8]>) -> Self {
        Self { tp, buf }
    }

    /// Cast to the concrete type
    ///
    /// Will fail if the type-descriptors are not equal
    pub fn cast<T: H5Type>(mut self) -> Result<T, Self> {
        use mem::MaybeUninit;
        if self.tp != T::type_descriptor() {
            return Err(self);
        }
        debug_assert_eq!(self.tp.size(), self.buf.len());
        let mut out = MaybeUninit::<T>::uninit();
        unsafe {
            ptr::copy_nonoverlapping(
                self.buf.as_ptr(),
                out.as_mut_ptr().cast::<u8>(),
                self.buf.len(),
            );
        }
        // For safety we must ensure any nested structures are not live at the same time,
        // as this could cause a double free in `dyn_drop`.
        // We must deallocate only the top level of Self

        // The zero-sized array has a special case to not drop ptr if len is zero,
        // so `dyn_drop` of `DynArray` is a nop
        self.tp = <[u8; 0]>::type_descriptor();
        // We must also swap out the buffer to ensure we can create the `DynValue`
        let mut b: Box<[u8]> = Box::new([]);
        mem::swap(&mut self.buf, &mut b);

        Ok(unsafe { out.assume_init() })
    }
}

impl<T: H5Type> From<T> for OwnedDynValue {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl Drop for OwnedDynValue {
    fn drop(&mut self) {
        self.get().dyn_drop();
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use unindent::unindent;

    use crate::array::VarLenArray;
    use crate::h5type::{TypeDescriptor as TD, *};
    use crate::string::{FixedAscii, FixedUnicode, VarLenAscii, VarLenUnicode};

    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    #[repr(i16)]
    enum Color {
        Red = -10_000,
        Green = 0,
        Blue = 10_000,
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    #[repr(C)]
    pub struct Point {
        coords: [f32; 2],
        color: Color,
        nice: bool,
    }

    #[derive(Clone, Debug, PartialEq)]
    #[repr(C)]
    struct Data {
        points: VarLenArray<Point>,
        fa: FixedAscii<5>,
        fu: FixedUnicode<5>,
        va: VarLenAscii,
        vu: VarLenUnicode,
    }

    #[derive(Clone, Debug, PartialEq)]
    #[repr(C)]
    struct BigStruct {
        ints: (i8, i16, i32, i64),
        uints: (u8, u16, u32, u64),
        floats: (f32, f64),
        data: Data,
    }

    fn td_color() -> TD {
        TD::Enum(EnumType {
            size: IntSize::U2,
            signed: true,
            members: vec![
                EnumMember { name: "Red".into(), value: -10_000i16 as _ },
                EnumMember { name: "Green".into(), value: 0 },
                EnumMember { name: "Blue".into(), value: 10_000 },
            ],
        })
    }

    fn td_point() -> TD {
        let coords = TD::FixedArray(Box::new(TD::Float(FloatSize::U4)), 2);
        TD::Compound(CompoundType {
            fields: Vec::from(
                [
                    CompoundField::new("coords", coords, 0, 0),
                    CompoundField::new("color", td_color(), 8, 1),
                    CompoundField::new("nice", TD::Boolean, 10, 2),
                ]
                .as_ref(),
            ),
            size: 12,
        })
    }

    fn td_data() -> TD {
        let points = TD::VarLenArray(Box::new(td_point()));
        TD::Compound(CompoundType {
            fields: Vec::from(
                [
                    CompoundField::new("points", points, 0, 0),
                    CompoundField::new("fa", TD::FixedAscii(5), 16, 1),
                    CompoundField::new("fu", TD::FixedUnicode(5), 21, 2),
                    CompoundField::new("va", TD::VarLenAscii, 32, 3),
                    CompoundField::new("vu", TD::VarLenUnicode, 40, 4),
                ]
                .as_ref(),
            ),
            size: 48,
        })
    }

    fn td_big_struct() -> TD {
        let ints = TD::Compound(CompoundType {
            fields: Vec::from(
                [
                    CompoundField::typed::<i32>("2", 0, 2),
                    CompoundField::typed::<i16>("1", 4, 1),
                    CompoundField::typed::<i8>("0", 6, 0),
                    CompoundField::typed::<i64>("3", 8, 3),
                ]
                .as_ref(),
            ),
            size: 16,
        });
        let uints = TD::Compound(CompoundType {
            fields: Vec::from(
                [
                    CompoundField::typed::<u32>("2", 0, 2),
                    CompoundField::typed::<u16>("1", 4, 1),
                    CompoundField::typed::<u8>("0", 6, 0),
                    CompoundField::typed::<u64>("3", 8, 3),
                ]
                .as_ref(),
            ),
            size: 16,
        });
        let floats = TD::Compound(CompoundType {
            fields: Vec::from(
                [CompoundField::typed::<f32>("0", 0, 0), CompoundField::typed::<f64>("1", 8, 1)]
                    .as_ref(),
            ),
            size: 16,
        });
        TD::Compound(CompoundType {
            fields: Vec::from(
                [
                    CompoundField::new("ints", ints, 0, 0),
                    CompoundField::new("uints", uints, 16, 1),
                    CompoundField::new("floats", floats, 32, 2),
                    CompoundField::new("data", td_data(), 48, 3),
                ]
                .as_ref(),
            ),
            size: 96,
        })
    }

    fn big_struct_1() -> BigStruct {
        BigStruct {
            ints: (-10, 20, -30, 40),
            uints: (30, 40, 50, 60),
            floats: (-3.14, 2.71),
            data: Data {
                points: VarLenArray::from_slice(
                    [
                        Point { coords: [-1.0, 2.0], color: Color::Red, nice: true },
                        Point { coords: [0.1, 0.], color: Color::Green, nice: false },
                        Point { coords: [10., 0.], color: Color::Blue, nice: true },
                    ]
                    .as_ref(),
                ),
                fa: FixedAscii::from_ascii(b"12345").unwrap(),
                fu: FixedUnicode::from_str("∀").unwrap(),
                va: VarLenAscii::from_ascii(b"wat").unwrap(),
                vu: VarLenUnicode::from_str("⨁∀").unwrap(),
            },
        }
    }

    fn big_struct_2() -> BigStruct {
        BigStruct {
            ints: (1, 2, 3, 4),
            uints: (3, 4, 5, 6),
            floats: (-1., 2.),
            data: Data {
                points: VarLenArray::from_slice([].as_ref()),
                fa: FixedAscii::from_ascii(b"").unwrap(),
                fu: FixedUnicode::from_str("").unwrap(),
                va: VarLenAscii::from_ascii(b"").unwrap(),
                vu: VarLenUnicode::from_str("").unwrap(),
            },
        }
    }

    unsafe impl crate::h5type::H5Type for BigStruct {
        fn type_descriptor() -> TypeDescriptor {
            td_big_struct()
        }
    }

    #[test]
    fn test_dyn_value_from() {
        assert_eq!(OwnedDynValue::from(-42i16), OwnedDynValue::new(-42i16));
        let s = big_struct_2();
        assert_eq!(OwnedDynValue::from(s.clone()), OwnedDynValue::new(s.clone()));
    }

    #[test]
    fn test_dyn_value_clone_drop() {
        let val1 = OwnedDynValue::new(big_struct_1());
        let val2 = OwnedDynValue::new(big_struct_2());

        assert_eq!(val1, val1);
        assert_eq!(val1.clone(), val1);
        assert_eq!(val1.clone(), val1.clone().clone());

        assert_eq!(val2, val2);
        assert_eq!(val2.clone(), val2);
        assert_eq!(val2.clone(), val2.clone().clone());

        assert_ne!(val1, val2);
        assert_ne!(val2, val1);
    }

    #[test]
    fn test_dyn_value_display() {
        let val1 = OwnedDynValue::new(big_struct_1());
        let val2 = OwnedDynValue::new(big_struct_2());

        let val1_flat = unindent(
            "\
             {ints: {2: -30, 1: 20, 0: -10, 3: 40}, \
             uints: {2: 50, 1: 40, 0: 30, 3: 60}, \
             floats: {0: -3.14, 1: 2.71}, \
             data: {points: [{coords: [-1.0, 2.0], color: Red, nice: true}, \
             {coords: [0.1, 0.0], color: Green, nice: false}, \
             {coords: [10.0, 0.0], color: Blue, nice: true}], \
             fa: \"12345\", fu: \"∀\", va: \"wat\", vu: \"⨁∀\"}}",
        );

        let val1_nice = unindent(
            r#"
        {
            ints: {
                2: -30,
                1: 20,
                0: -10,
                3: 40,
            },
            uints: {
                2: 50,
                1: 40,
                0: 30,
                3: 60,
            },
            floats: {
                0: -3.14,
                1: 2.71,
            },
            data: {
                points: [
                    {
                        coords: [
                            -1.0,
                            2.0,
                        ],
                        color: Red,
                        nice: true,
                    },
                    {
                        coords: [
                            0.1,
                            0.0,
                        ],
                        color: Green,
                        nice: false,
                    },
                    {
                        coords: [
                            10.0,
                            0.0,
                        ],
                        color: Blue,
                        nice: true,
                    },
                ],
                fa: "12345",
                fu: "∀",
                va: "wat",
                vu: "⨁∀",
            },
        }"#,
        );

        let val2_flat = unindent(
            "\
             {ints: {2: 3, 1: 2, 0: 1, 3: 4}, \
             uints: {2: 5, 1: 4, 0: 3, 3: 6}, \
             floats: {0: -1.0, 1: 2.0}, \
             data: {points: [], fa: \"\", fu: \"\", va: \"\", vu: \"\"}}",
        );

        let val2_nice = unindent(
            r#"
            {
                ints: {
                    2: 3,
                    1: 2,
                    0: 1,
                    3: 4,
                },
                uints: {
                    2: 5,
                    1: 4,
                    0: 3,
                    3: 6,
                },
                floats: {
                    0: -1.0,
                    1: 2.0,
                },
                data: {
                    points: [],
                    fa: "",
                    fu: "",
                    va: "",
                    vu: "",
                },
            }"#,
        );

        assert_eq!(format!("{}", val1), val1_flat);
        assert_eq!(format!("{:?}", val1), val1_flat);
        assert_eq!(format!("{:#?}", val1.clone()), val1_nice);

        assert_eq!(format!("{}", val2), val2_flat);
        assert_eq!(format!("{:?}", val2), val2_flat);
        assert_eq!(format!("{:#?}", val2.clone()), val2_nice);
    }
}

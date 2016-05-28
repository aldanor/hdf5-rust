use error::Result;
use handle::{Handle, ID, FromID, get_id_type};
use object::Object;
use types::{ValueType, ToValueType, IntSize, FloatSize};
use util::to_cstring;

use libc::{c_char, c_void};

use ffi::h5::hsize_t;
use ffi::h5i::{hid_t, H5I_DATATYPE};
use ffi::h5t::{H5Tcreate, H5Tset_size, H5Tinsert, H5Tenum_create, H5Tenum_insert,
               H5Tcopy, H5Tarray_create2, H5T_str_t, H5Tset_strpad, H5T_cset_t,
               H5Tset_cset, H5Tvlen_create, H5T_COMPOUND, H5T_VARIABLE};

#[cfg(target_endian = "big")]
use globals::{
    H5T_STD_I8BE, H5T_STD_I16BE,
    H5T_STD_I32BE, H5T_STD_I64BE,
    H5T_STD_U8BE, H5T_STD_U16BE,
    H5T_STD_U32BE, H5T_STD_U64BE,
    H5T_IEEE_F32BE, H5T_IEEE_F64BE,
};

#[cfg(target_endian = "little")]
use globals::{
    H5T_STD_I8LE, H5T_STD_I16LE,
    H5T_STD_I32LE, H5T_STD_I64LE,
    H5T_STD_U8LE, H5T_STD_U16LE,
    H5T_STD_U32LE, H5T_STD_U64LE,
    H5T_IEEE_F32LE, H5T_IEEE_F64LE,
};

use globals::{H5T_NATIVE_INT8, H5T_C_S1};

pub struct Datatype {
    handle: Handle,
}

#[doc(hidden)]
impl ID for Datatype {
    fn id(&self) -> hid_t {
        self.handle.id()
    }
}

#[doc(hidden)]
impl FromID for Datatype {
    fn from_id(id: hid_t) -> Result<Datatype> {
        match get_id_type(id) {
            H5I_DATATYPE => Ok(Datatype { handle: try!(Handle::new(id)) }),
            _ => Err(From::from(format!("Invalid datatype id: {}", id))),
        }
    }
}

impl Object for Datatype { }

pub trait ToDatatype {
    fn to_datatype() -> Result<Datatype>;
}

#[cfg(target_endian = "big")]
macro_rules! be_le {
    ($be:expr, $le:expr) => (h5try_s!(H5Tcopy(*$be)))
}

#[cfg(target_endian = "little")]
macro_rules! be_le {
    ($be:expr, $le:expr) => (h5try_s!(H5Tcopy(*$le)))
}

fn value_type_to_datatype(value_type: &ValueType) -> Result<Datatype> {
    use types::ValueType::*;

    let datatype_id: Result<_> = h5lock!({
        match *value_type {
            Integer(size) => Ok(match size {
                IntSize::U1 => be_le!(H5T_STD_I8BE, H5T_STD_I8LE),
                IntSize::U2 => be_le!(H5T_STD_I16BE, H5T_STD_I16LE),
                IntSize::U4 => be_le!(H5T_STD_I32BE, H5T_STD_I32LE),
                IntSize::U8 => be_le!(H5T_STD_I64BE, H5T_STD_I64LE),
            }),
            Unsigned(size) => Ok(match size {
                IntSize::U1 => be_le!(H5T_STD_U8BE, H5T_STD_U8LE),
                IntSize::U2 => be_le!(H5T_STD_U16BE, H5T_STD_U16LE),
                IntSize::U4 => be_le!(H5T_STD_U32BE, H5T_STD_U32LE),
                IntSize::U8 => be_le!(H5T_STD_U64BE, H5T_STD_U64LE),
            }),
            Float(size) => Ok(match size {
                FloatSize::U4 => be_le!(H5T_IEEE_F32BE, H5T_IEEE_F32LE),
                FloatSize::U8 => be_le!(H5T_IEEE_I16BE, H5T_IEEE_F64LE),
            }),
            Boolean => {
                let bool_id = h5try_s!(H5Tenum_create(*H5T_NATIVE_INT8));
                h5try_s!(H5Tenum_insert(bool_id, b"FALSE\0".as_ptr() as *const c_char,
                                        &0i8 as *const i8 as *const c_void));
                h5try_s!(H5Tenum_insert(bool_id, b"TRUE\0".as_ptr() as *const c_char,
                                        &1i8 as *const i8 as *const c_void));
                Ok(bool_id)
            },
            Enum(ref enum_type) => {
                let base = try!(value_type_to_datatype(&enum_type.base_type()));
                let enum_id = h5try_s!(H5Tenum_create(base.id()));
                for member in &enum_type.members {
                    let name = try!(to_cstring(member.name.as_ref()));
                    h5try_s!(H5Tenum_insert(enum_id, name.as_ptr(),
                                            &member.value as *const u64 as *const c_void));
                }
                Ok(enum_id)
            },
            Compound(ref compound_type) => {
                let compound_id = h5try_s!(H5Tcreate(H5T_class_t::H5T_COMPOUND, 1));
                for field in &compound_type.fields {
                    let name = try!(to_cstring(field.name.as_ref()));
                    let field_dt = try!(value_type_to_datatype(&field.ty));
                    h5try_s!(H5Tset_size(compound_id, field.offset + field.ty.size()));
                    h5try_s!(H5Tinsert(compound_id, name.as_ptr(), field.offset, field_dt.id()));
                }
                h5try_s!(H5Tset_size(compound_id, compound_type.size));
                Ok(compound_id)
            },
            FixedArray(ref ty, len) => {
                let elem_dt = try!(value_type_to_datatype(&ty));
                let dims = len as hsize_t;
                Ok(h5try_s!(H5Tarray_create2(elem_dt.id(), 1, &dims as *const hsize_t)))
            },
            FixedString(size) => {
                let string_id = h5try_s!(H5Tcopy(*H5T_C_S1));
                h5try_s!(H5Tset_cset(string_id, H5T_cset_t::H5T_CSET_UTF8));
                h5try_s!(H5Tset_size(string_id, size));
                h5try_s!(H5Tset_strpad(string_id, H5T_str_t::H5T_STR_NULLPAD));
                Ok(string_id)
            },
            VarLenArray(ref ty) => {
                let elem_dt = try!(value_type_to_datatype(&ty));
                Ok(h5try_s!(H5Tvlen_create(elem_dt.id())))
            },
            VarLenString => {
                let string_id = h5try_s!(H5Tcopy(*H5T_C_S1));
                h5try_s!(H5Tset_cset(string_id, H5T_cset_t::H5T_CSET_UTF8));
                h5try_s!(H5Tset_size(string_id, H5T_VARIABLE));
                Ok(string_id)
            },
        }
    });

    Datatype::from_id(try!(datatype_id))
}

impl<T: ToValueType> ToDatatype for T {
    fn to_datatype() -> Result<Datatype> {
        value_type_to_datatype(&T::value_type())
    }
}

#[cfg(test)]
pub mod tests {
    use super::ToDatatype;
    use types::FixedString;

    #[test]
    pub fn test_smoke() {
        h5def!(struct A { a: i64, b: u64 });
        A::to_datatype().unwrap();

        h5def!(#[repr(i64)] enum X { A = 1, B = -2 });
        X::to_datatype().unwrap();

        i8::to_datatype().unwrap();
        i16::to_datatype().unwrap();
        i32::to_datatype().unwrap();
        i64::to_datatype().unwrap();

        u8::to_datatype().unwrap();
        u16::to_datatype().unwrap();
        u32::to_datatype().unwrap();
        u64::to_datatype().unwrap();

        bool::to_datatype().unwrap();

        f32::to_datatype().unwrap();
        f64::to_datatype().unwrap();

        type T = [u32; 10];
        T::to_datatype().unwrap();

        FixedString::<[_; 5]>::to_datatype().unwrap();
    }

    #[test]
    #[cfg(feature = "varlen")]
    pub fn test_varlen() {
        use types::{VarLenArray, VarLenString};

        VarLenArray::<u16>::to_datatype().unwrap();
        VarLenString::to_datatype().unwrap();
    }
}

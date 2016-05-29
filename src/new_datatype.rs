use error::Result;
use handle::{Handle, ID, FromID, get_id_type};
use object::Object;
use types::{
    ValueType, ToValueType, IntSize, FloatSize, EnumMember,
    EnumType, CompoundField, CompoundType
};
use util::{to_cstring, string_from_cstr};

use libc::{c_char, c_void};

use ffi::h5::hsize_t;
use ffi::h5i::{hid_t, H5I_DATATYPE};
use ffi::h5t::{
    H5Tcreate, H5Tset_size, H5Tinsert, H5Tenum_create, H5Tenum_insert, H5Tcopy,
    H5Tarray_create2, H5T_str_t, H5Tset_strpad, H5T_cset_t, H5Tset_cset, H5Tvlen_create,
    H5Tget_class, H5T_VARIABLE, H5T_class_t, H5Tget_size, H5Tget_sign, H5Tget_nmembers,
    H5Tget_super, H5Tget_member_name, H5Tget_member_type, H5Tget_member_offset,
    H5Tget_member_value, H5Tget_array_ndims, H5Tget_array_dims2, H5Tis_variable_str
};

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
        h5lock_s!(match get_id_type(id) {
            H5I_DATATYPE => Ok(Datatype { handle: try!(Handle::new(id)) }),
            _ => Err(From::from(format!("Invalid datatype id: {}", id))),
        })
    }
}

impl Object for Datatype { }

fn datatype_to_value_type(datatype: &Datatype) -> Result<ValueType> {
    use ffi::h5t::H5T_class_t::*;
    use ffi::h5t::H5T_sign_t::*;
    use types::ValueType::*;

    h5lock!({
        let id = datatype.id();
        let size = h5try_s!(H5Tget_size(id)) as usize;
        match H5Tget_class(id) {
            H5T_INTEGER => {
                let signed = match H5Tget_sign(id) {
                    H5T_SGN_NONE => false,
                    H5T_SGN_2 => true,
                    _ => return Err("Invalid sign of integer datatype".into())
                };
                let size = try!(IntSize::from_int(size)
                                .ok_or("Invalid size of integer datatype"));
                Ok(if signed {
                    ValueType::Integer(size)
                } else {
                    ValueType::Unsigned(size)
                })
            },
            H5T_FLOAT => {
                let size = try!(FloatSize::from_int(size)
                                .ok_or("Invalid size of float datatype"));
                Ok(ValueType::Float(size))
            },
            H5T_ENUM => {
                let mut members: Vec<EnumMember> = Vec::new();
                for idx in 0 .. h5try_s!(H5Tget_nmembers(id)) as u32 {
                    let mut value: u64 = 0;
                    h5try_s!(H5Tget_member_value(
                        id, idx, &mut value as *mut _ as *mut c_void
                    ));
                    let name = H5Tget_member_name(id, idx);
                    members.push(EnumMember { name: string_from_cstr(name), value: value });
                    ::libc::free(name as *mut c_void);
                }
                let base_dt = try!(Datatype::from_id(H5Tget_super(id)));
                let (size, signed) = try!(match try!(base_dt.to_value_type()) {
                    Integer(size) => Ok((size, true)),
                    Unsigned(size) => Ok((size, false)),
                    _ => Err("Invalid base type for enum datatype"),
                });
                let bool_members = [
                    EnumMember { name: "FALSE".to_owned(), value: 0 },
                    EnumMember { name: "TRUE".to_owned(), value: 1 },
                ];
                if size == IntSize::U1 && members == bool_members {
                    Ok(ValueType::Boolean)
                } else {
                    Ok(ValueType::Enum(
                        EnumType { size: size, signed: signed, members : members }
                    ))
                }
            },
            H5T_COMPOUND => {
                let mut fields: Vec<CompoundField> = Vec::new();
                for idx in 0 .. h5try_s!(H5Tget_nmembers(id)) as u32 {
                    let name = H5Tget_member_name(id, idx);
                    let offset = h5try_s!(H5Tget_member_offset(id, idx)) as usize;
                    let ty = try!(Datatype::from_id(h5try_s!(H5Tget_member_type(id, idx))));
                    fields.push(CompoundField {
                        name: string_from_cstr(name),
                        ty: try!(ty.to_value_type()),
                        offset: offset
                    });
                    ::libc::free(name as *mut c_void);
                }
                Ok(ValueType::Compound(CompoundType { fields: fields, size: size }))
            },
            H5T_ARRAY => {
                let base_dt = try!(Datatype::from_id(H5Tget_super(id)));
                let ndims = h5try_s!(H5Tget_array_ndims(id));
                if ndims == 1 {
                    let mut len: hsize_t = 0;
                    h5try_s!(H5Tget_array_dims2(id, &mut len as *mut hsize_t));
                    Ok(ValueType::FixedArray(
                        Box::new(try!(base_dt.to_value_type())), len as usize
                    ))
                } else {
                    Err("Multi-dimensional array datatypes are not supported".into())
                }
            },
            H5T_STRING => {
                if h5try_s!(H5Tis_variable_str(id)) == 1 {
                    Ok(ValueType::VarLenString)
                } else {
                    Ok(ValueType::FixedString(size))
                }
            },
            H5T_VLEN => {
                let base_dt = try!(Datatype::from_id(H5Tget_super(id)));
                Ok(ValueType::VarLenArray(Box::new(try!(base_dt.to_value_type()))))
            },
            _ => Err("Unsupported datatype class".into())
        }
    })
}

impl Datatype {
    pub fn to_value_type(&self) -> Result<ValueType> {
        datatype_to_value_type(self)
    }
}

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

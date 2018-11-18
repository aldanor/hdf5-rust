use std::fmt;

use hdf5_types::{
    CompoundField, CompoundType, EnumMember, EnumType, FloatSize, H5Type, IntSize, TypeDescriptor,
};
use libhdf5_sys::h5t::{
    H5T_class_t, H5T_cset_t, H5T_str_t, H5Tarray_create2, H5Tcopy, H5Tcreate, H5Tenum_create,
    H5Tenum_insert, H5Tequal, H5Tget_array_dims2, H5Tget_array_ndims, H5Tget_class, H5Tget_cset,
    H5Tget_member_name, H5Tget_member_offset, H5Tget_member_type, H5Tget_member_value,
    H5Tget_nmembers, H5Tget_sign, H5Tget_size, H5Tget_super, H5Tinsert, H5Tis_variable_str,
    H5Tset_cset, H5Tset_size, H5Tset_strpad, H5Tvlen_create, H5T_VARIABLE,
};

use crate::globals::{H5T_C_S1, H5T_NATIVE_INT8};
use crate::internal_prelude::*;

#[cfg(target_endian = "big")]
use crate::globals::{
    H5T_IEEE_F32BE, H5T_IEEE_F64BE, H5T_STD_I16BE, H5T_STD_I32BE, H5T_STD_I64BE, H5T_STD_I8BE,
    H5T_STD_U16BE, H5T_STD_U32BE, H5T_STD_U64BE, H5T_STD_U8BE,
};

#[cfg(target_endian = "little")]
use crate::globals::{
    H5T_IEEE_F32LE, H5T_IEEE_F64LE, H5T_STD_I16LE, H5T_STD_I32LE, H5T_STD_I64LE, H5T_STD_I8LE,
    H5T_STD_U16LE, H5T_STD_U32LE, H5T_STD_U64LE, H5T_STD_U8LE,
};

#[cfg(target_endian = "big")]
macro_rules! be_le {
    ($be:expr, $le:expr) => {
        h5try!(H5Tcopy(*$be))
    };
}

#[cfg(target_endian = "little")]
macro_rules! be_le {
    ($be:expr, $le:expr) => {
        h5try!(H5Tcopy(*$le))
    };
}

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
        h5lock!(match get_id_type(id) {
            H5I_DATATYPE => Ok(Datatype { handle: Handle::new(id)? }),
            _ => Err(From::from(format!("Invalid datatype id: {}", id))),
        })
    }
}

impl Object for Datatype {}

impl fmt::Debug for Datatype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Datatype {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.is_valid() {
            f.write_str("<HDF5 datatype: invalid id>")
        } else {
            f.write_str("<HDF5 datatype>") // TODO: type descriptor details
        }
    }
}

impl PartialEq for Datatype {
    fn eq(&self, other: &Datatype) -> bool {
        h5call!(H5Tequal(self.id(), other.id())).unwrap_or(0) == 1
    }
}

impl Datatype {
    /// Get the total size of the datatype in bytes.
    pub fn size(&self) -> usize {
        h5call!(H5Tget_size(self.id())).unwrap_or(0) as usize
    }

    pub fn to_descriptor(&self) -> Result<TypeDescriptor> {
        use hdf5_types::TypeDescriptor as TD;
        use libhdf5_sys::h5t::{H5T_class_t::*, H5T_sign_t::*};

        h5lock!({
            let id = self.id();
            let size = h5try!(H5Tget_size(id)) as usize;
            match H5Tget_class(id) {
                H5T_INTEGER => {
                    let signed = match H5Tget_sign(id) {
                        H5T_SGN_NONE => false,
                        H5T_SGN_2 => true,
                        _ => return Err("Invalid sign of integer datatype".into()),
                    };
                    let size = IntSize::from_int(size).ok_or("Invalid size of integer datatype")?;
                    Ok(if signed { TD::Integer(size) } else { TD::Unsigned(size) })
                }
                H5T_FLOAT => {
                    let size = FloatSize::from_int(size).ok_or("Invalid size of float datatype")?;
                    Ok(TD::Float(size))
                }
                H5T_ENUM => {
                    let mut members: Vec<EnumMember> = Vec::new();
                    for idx in 0..h5try!(H5Tget_nmembers(id)) as _ {
                        let mut value: u64 = 0;
                        h5try!(H5Tget_member_value(id, idx, &mut value as *mut _ as *mut _));
                        let name = H5Tget_member_name(id, idx);
                        members.push(EnumMember { name: string_from_cstr(name), value });
                        libc::free(name as *mut _);
                    }
                    let base_dt = Datatype::from_id(H5Tget_super(id))?;
                    let (size, signed) = match base_dt.to_descriptor()? {
                        TD::Integer(size) => Ok((size, true)),
                        TD::Unsigned(size) => Ok((size, false)),
                        _ => Err("Invalid base type for enum datatype"),
                    }?;
                    let bool_members = [
                        EnumMember { name: "FALSE".to_owned(), value: 0 },
                        EnumMember { name: "TRUE".to_owned(), value: 1 },
                    ];
                    if size == IntSize::U1 && members == bool_members {
                        Ok(TD::Boolean)
                    } else {
                        Ok(TD::Enum(EnumType { size, signed, members }))
                    }
                }
                H5T_COMPOUND => {
                    let mut fields: Vec<CompoundField> = Vec::new();
                    for idx in 0..h5try!(H5Tget_nmembers(id)) as _ {
                        let name = H5Tget_member_name(id, idx);
                        let offset = h5try!(H5Tget_member_offset(id, idx));
                        let ty = Datatype::from_id(h5try!(H5Tget_member_type(id, idx)))?;
                        fields.push(CompoundField {
                            name: string_from_cstr(name),
                            ty: ty.to_descriptor()?,
                            offset: offset as _,
                        });
                        libc::free(name as *mut _);
                    }
                    Ok(TD::Compound(CompoundType { fields, size }))
                }
                H5T_ARRAY => {
                    let base_dt = Datatype::from_id(H5Tget_super(id))?;
                    let ndims = h5try!(H5Tget_array_ndims(id));
                    if ndims == 1 {
                        let mut len: hsize_t = 0;
                        h5try!(H5Tget_array_dims2(id, &mut len as *mut _));
                        Ok(TD::FixedArray(Box::new(base_dt.to_descriptor()?), len as _))
                    } else {
                        Err("Multi-dimensional array datatypes are not supported".into())
                    }
                }
                H5T_STRING => {
                    let is_variable = h5try!(H5Tis_variable_str(id)) == 1;
                    let encoding = h5lock!(H5Tget_cset(id));
                    match (is_variable, encoding) {
                        (false, H5T_cset_t::H5T_CSET_ASCII) => Ok(TD::FixedAscii(size)),
                        (false, H5T_cset_t::H5T_CSET_UTF8) => Ok(TD::FixedUnicode(size)),
                        (true, H5T_cset_t::H5T_CSET_ASCII) => Ok(TD::VarLenAscii),
                        (true, H5T_cset_t::H5T_CSET_UTF8) => Ok(TD::VarLenUnicode),
                        _ => Err("Invalid encoding for string datatype".into()),
                    }
                }
                H5T_VLEN => {
                    let base_dt = Datatype::from_id(H5Tget_super(id))?;
                    Ok(TD::VarLenArray(Box::new(base_dt.to_descriptor()?)))
                }
                _ => Err("Unsupported datatype class".into()),
            }
        })
    }

    pub fn from_type<T: H5Type>() -> Result<Datatype> {
        Datatype::from_descriptor(&<T as H5Type>::type_descriptor())
    }

    pub fn from_descriptor(desc: &TypeDescriptor) -> Result<Datatype> {
        use crate::types::TypeDescriptor as TD;

        unsafe fn string_type(size: Option<usize>, encoding: H5T_cset_t) -> Result<hid_t> {
            let string_id = h5try!(H5Tcopy(*H5T_C_S1));
            let padding = if size.is_none() {
                H5T_str_t::H5T_STR_NULLPAD
            } else {
                H5T_str_t::H5T_STR_NULLTERM
            };
            let size = size.unwrap_or(H5T_VARIABLE);
            h5try!(H5Tset_cset(string_id, encoding));
            h5try!(H5Tset_strpad(string_id, padding));
            h5try!(H5Tset_size(string_id, size));
            Ok(string_id)
        }

        let datatype_id: Result<_> = h5lock!({
            match *desc {
                TD::Integer(size) => Ok(match size {
                    IntSize::U1 => be_le!(H5T_STD_I8BE, H5T_STD_I8LE),
                    IntSize::U2 => be_le!(H5T_STD_I16BE, H5T_STD_I16LE),
                    IntSize::U4 => be_le!(H5T_STD_I32BE, H5T_STD_I32LE),
                    IntSize::U8 => be_le!(H5T_STD_I64BE, H5T_STD_I64LE),
                }),
                TD::Unsigned(size) => Ok(match size {
                    IntSize::U1 => be_le!(H5T_STD_U8BE, H5T_STD_U8LE),
                    IntSize::U2 => be_le!(H5T_STD_U16BE, H5T_STD_U16LE),
                    IntSize::U4 => be_le!(H5T_STD_U32BE, H5T_STD_U32LE),
                    IntSize::U8 => be_le!(H5T_STD_U64BE, H5T_STD_U64LE),
                }),
                TD::Float(size) => Ok(match size {
                    FloatSize::U4 => be_le!(H5T_IEEE_F32BE, H5T_IEEE_F32LE),
                    FloatSize::U8 => be_le!(H5T_IEEE_I16BE, H5T_IEEE_F64LE),
                }),
                TD::Boolean => {
                    let bool_id = h5try!(H5Tenum_create(*H5T_NATIVE_INT8));
                    h5try!(H5Tenum_insert(
                        bool_id,
                        b"FALSE\0".as_ptr() as *const _,
                        &0i8 as *const _ as *const _
                    ));
                    h5try!(H5Tenum_insert(
                        bool_id,
                        b"TRUE\0".as_ptr() as *const _,
                        &1i8 as *const _ as *const _
                    ));
                    Ok(bool_id)
                }
                TD::Enum(ref enum_type) => {
                    let base = Datatype::from_descriptor(&enum_type.base_type())?;
                    let enum_id = h5try!(H5Tenum_create(base.id()));
                    for member in &enum_type.members {
                        let name = to_cstring(member.name.as_ref())?;
                        h5try!(H5Tenum_insert(
                            enum_id,
                            name.as_ptr(),
                            &member.value as *const _ as *const _
                        ));
                    }
                    Ok(enum_id)
                }
                TD::Compound(ref compound_type) => {
                    let compound_id = h5try!(H5Tcreate(H5T_class_t::H5T_COMPOUND, 1));
                    for field in &compound_type.fields {
                        let name = to_cstring(field.name.as_ref())?;
                        let field_dt = Datatype::from_descriptor(&field.ty)?;
                        h5try!(H5Tset_size(compound_id, field.offset + field.ty.size()));
                        h5try!(H5Tinsert(compound_id, name.as_ptr(), field.offset, field_dt.id()));
                    }
                    h5try!(H5Tset_size(compound_id, compound_type.size));
                    Ok(compound_id)
                }
                TD::FixedArray(ref ty, len) => {
                    let elem_dt = Datatype::from_descriptor(&ty)?;
                    let dims = len as hsize_t;
                    Ok(h5try!(H5Tarray_create2(elem_dt.id(), 1, &dims as *const _)))
                }
                TD::FixedAscii(size) => string_type(Some(size), H5T_cset_t::H5T_CSET_ASCII),
                TD::FixedUnicode(size) => string_type(Some(size), H5T_cset_t::H5T_CSET_UTF8),
                TD::VarLenArray(ref ty) => {
                    let elem_dt = Datatype::from_descriptor(&ty)?;
                    Ok(h5try!(H5Tvlen_create(elem_dt.id())))
                }
                TD::VarLenAscii => string_type(None, H5T_cset_t::H5T_CSET_ASCII),
                TD::VarLenUnicode => string_type(None, H5T_cset_t::H5T_CSET_UTF8),
            }
        });

        Datatype::from_id(datatype_id?)
    }
}

#[cfg(test)]
pub mod tests {
    use hdf5_derive::H5Type;
    use hdf5_types::TypeDescriptor as TD;
    use hdf5_types::*;

    use crate::internal_prelude::*;

    macro_rules! check_roundtrip {
        ($ty:ty, $desc:expr) => {{
            let desc = <$ty as H5Type>::type_descriptor();
            assert_eq!(desc, $desc);
            let dt = Datatype::from_type::<$ty>().unwrap();
            assert_eq!(desc, dt.to_descriptor().unwrap());
            assert_eq!(dt.size(), desc.size());
        }};
    }

    #[test]
    pub fn test_datatype_roundtrip() {
        check_roundtrip!(i8, TD::Integer(IntSize::U1));
        check_roundtrip!(i16, TD::Integer(IntSize::U2));
        check_roundtrip!(i32, TD::Integer(IntSize::U4));
        check_roundtrip!(i64, TD::Integer(IntSize::U8));
        check_roundtrip!(u8, TD::Unsigned(IntSize::U1));
        check_roundtrip!(u16, TD::Unsigned(IntSize::U2));
        check_roundtrip!(u32, TD::Unsigned(IntSize::U4));
        check_roundtrip!(u64, TD::Unsigned(IntSize::U8));
        check_roundtrip!(f32, TD::Float(FloatSize::U4));
        check_roundtrip!(f64, TD::Float(FloatSize::U8));
        check_roundtrip!(bool, TD::Boolean);
        check_roundtrip!([bool; 5], TD::FixedArray(Box::new(TD::Boolean), 5));
        check_roundtrip!(VarLenArray<bool>, TD::VarLenArray(Box::new(TD::Boolean)));
        check_roundtrip!(FixedAscii<[_; 5]>, TD::FixedAscii(5));
        check_roundtrip!(FixedUnicode<[_; 5]>, TD::FixedUnicode(5));
        check_roundtrip!(VarLenAscii, TD::VarLenAscii);
        check_roundtrip!(VarLenUnicode, TD::VarLenUnicode);

        #[allow(dead_code)]
        #[derive(H5Type)]
        #[repr(i64)]
        enum X {
            A = 1,
            B = -2,
        };
        let x_desc = TD::Enum(EnumType {
            size: IntSize::U8,
            signed: true,
            members: vec![
                EnumMember { name: "A".into(), value: 1 },
                EnumMember { name: "B".into(), value: -2i64 as _ },
            ],
        });
        check_roundtrip!(X, x_desc);

        #[derive(H5Type)]
        #[repr(C)]
        struct A {
            a: i64,
            b: u64,
        };
        let a_desc = TD::Compound(CompoundType {
            fields: vec![
                CompoundField { name: "a".into(), ty: i64::type_descriptor(), offset: 0 },
                CompoundField { name: "b".into(), ty: u64::type_descriptor(), offset: 8 },
            ],
            size: 16,
        });
        check_roundtrip!(A, a_desc);

        #[derive(H5Type)]
        #[repr(C)]
        struct C {
            a: [X; 2],
            b: [[A; 4]; 32],
        };
        let c_desc = TD::Compound(CompoundType {
            fields: vec![
                CompoundField {
                    name: "a".into(),
                    ty: TD::FixedArray(Box::new(x_desc), 2),
                    offset: 0,
                },
                CompoundField {
                    name: "b".into(),
                    ty: TD::FixedArray(Box::new(TD::FixedArray(Box::new(a_desc), 4)), 32),
                    offset: 2 * 8,
                },
            ],
            size: 2 * 8 + 4 * 32 * 16,
        });
        check_roundtrip!(C, c_desc);
    }

    #[test]
    pub fn test_invalid_datatype() {
        assert_err!(Datatype::from_id(H5I_INVALID_HID), "Invalid datatype id");
    }

    #[test]
    pub fn test_eq() {
        assert_eq!(Datatype::from_type::<u32>().unwrap(), Datatype::from_type::<u32>().unwrap());
        assert_ne!(Datatype::from_type::<u16>().unwrap(), Datatype::from_type::<u32>().unwrap());
    }

    #[test]
    pub fn test_debug_display() {
        assert_eq!(format!("{}", Datatype::from_type::<u32>().unwrap()), "<HDF5 datatype>");
        assert_eq!(format!("{:?}", Datatype::from_type::<u32>().unwrap()), "<HDF5 datatype>");
    }
}

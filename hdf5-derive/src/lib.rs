#![crate_type = "proc-macro"]

#![recursion_limit = "192"]

#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

extern crate hdf5_types;

use std::iter;
use std::mem;
use std::str::FromStr;

use proc_macro::TokenStream;
use syn::{Body, VariantData, Ident, Ty, ConstExpr, Attribute, TyGenerics};

#[proc_macro_derive(H5Type)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: String = input.to_string();
    let ast = syn::parse_macro_input(&input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let body = impl_trait(name, &ast.body, &ast.attrs, &ty_generics);
    let dummy = Ident::new(format!("_IMPL_H5TYPE_FOR_{}", name));
    let gen = quote! {
        #[allow(dead_code, unused_variables, unused_attributes)]
        const #dummy: () = {
            extern crate hdf5_types as _h5t;
            #[automatically_derived]
            unsafe impl #impl_generics _h5t::H5Type for #name #ty_generics #where_clause {
                #[inline]
                fn type_descriptor() -> _h5t::TypeDescriptor {
                    #body
                }
            }
        };
    };
    gen.parse().unwrap()
}

fn impl_compound(ty: &Ident, ty_generics: &TyGenerics,
                 fields: Vec<Ident>, names: Vec<Ident>, types: Vec<Ty>) -> quote::Tokens
{
    quote! {
        let origin: *const #ty #ty_generics = ::std::ptr::null();
        _h5t::TypeDescriptor::Compound(
            _h5t::CompoundType {
                fields: vec![#(
                    _h5t::CompoundField {
                        name: stringify!(#names).to_owned(),
                        ty: <#types as _h5t::H5Type>::type_descriptor(),
                        offset: unsafe { &((*origin).#fields) as *const _ as usize },
                    }
                ),*],
                size: ::std::mem::size_of::<#ty #ty_generics>()
            }
        )
    }
}

fn impl_enum(names: Vec<Ident>, values: Vec<ConstExpr>, repr: &Ident)-> quote::Tokens {
    let size = Ident::new(format!(
        "U{}", usize::from_str(&repr.as_ref()[1..]).unwrap_or(mem::size_of::<usize>() * 8) / 8));
    let signed = repr.as_ref().starts_with('i');
    let repr = iter::repeat(repr);
    quote! {
        _h5t::TypeDescriptor::Enum(
            _h5t::EnumType {
                size: _h5t::IntSize::#size,
                signed: #signed,
                members: vec![#(
                    _h5t::EnumMember {
                        name: stringify!(#names).to_owned(),
                        value: (#values) as #repr as u64,
                    }
                ),*],
            }
        )
    }
}

fn is_phantom_data(ty: &Ty) -> bool {
    match *ty {
        Ty::Path(None, ref path) => {
            path.segments.as_slice().last()
                .map(|x| x.ident.as_ref() == "PhantomData").unwrap_or(false)
        },
        _ => false,
    }
}

fn find_repr(attrs: &[Attribute], expected: &[&str]) -> Option<Ident> {
    use syn::{AttrStyle, MetaItem, NestedMetaItem};

    for attr in attrs.iter() {
        if attr.style == AttrStyle::Outer && !attr.is_sugared_doc {
            if let MetaItem::List(ref name, ref meta_items) = attr.value {
                if name.as_ref() == "repr" {
                    for meta_item in meta_items.iter() {
                        if let NestedMetaItem::MetaItem(MetaItem::Word(ref ident)) = *meta_item {
                            if expected.iter().any(|&s| ident.as_ref() == s) {
                                return Some(Ident::new(ident.as_ref()));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

macro_rules! pluck {
    ($seq:expr, $key:tt) => (
        ($seq).iter().map(|f| f.$key.clone()).collect::<Vec<_>>()
    );
    ($seq:expr, ?$key:tt) => (
        ($seq).iter().map(|f| f.$key.clone().unwrap()).collect::<Vec<_>>()
    );
}

fn impl_trait(ty: &Ident, body: &Body, attrs: &[Attribute],
              ty_generics: &TyGenerics) -> quote::Tokens
{
    match *body {
        Body::Struct(VariantData::Unit) => {
            panic!("Cannot derive H5Type for unit structs");
        },
        Body::Struct(VariantData::Struct(ref fields)) => {
            let fields: Vec<_> = fields.iter()
                .filter(|f| !is_phantom_data(&f.ty))
                .collect();
            if fields.is_empty() {
                panic!("Cannot derive H5Type for empty structs");
            }
            find_repr(attrs, &["C"])
                .expect("H5Type requires #[repr(C)] for structs");
            let names = pluck!(fields, ?ident);
            impl_compound(ty, ty_generics, names.clone(), names, pluck!(fields, ty))
        },
        Body::Struct(VariantData::Tuple(ref fields)) => {
            let (index, fields): (Vec<_>, Vec<_>) = fields.iter()
                .enumerate()
                .filter(|&(_, f)| !is_phantom_data(&f.ty))
                .map(|(i, f)| (Ident::new(format!("{}", i)), f))
                .unzip();
            if fields.is_empty() {
                panic!("Cannot derive H5Type for empty tuple structs");
            }
            find_repr(attrs, &["C"])
                .expect("H5Type requires #[repr(C)] for structs");
            let names = (0..fields.len()).map(|i| Ident::new(format!("{}", i))).collect();
            impl_compound(ty, ty_generics, index, names, pluck!(fields, ty))
        },
        Body::Enum(ref variants) => {
            if variants.iter().any(|f| f.data != VariantData::Unit || f.discriminant.is_none()) {
                panic!("H5Type can only be derived for enums with scalar discriminants");
            } else if variants.is_empty() {
                panic!("Cannot derive H5Type for empty enums")
            }
            let enum_reprs = &["i8", "i16", "i32", "i64",
                               "u8", "u16", "u32", "u64",
                               "isize", "usize"];
            let repr = find_repr(attrs, enum_reprs)
                .expect("H5Type can only be derived for enums with explicit representation");
            impl_enum(pluck!(variants, ident), pluck!(variants, ?discriminant), &repr)
        },
    }
}

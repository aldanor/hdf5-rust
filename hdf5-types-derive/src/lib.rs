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
use syn::{Body, VariantData, Ident, Ty, ConstExpr, Attribute};

#[proc_macro_derive(H5Type)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: String = input.to_string();
    let ast = syn::parse_macro_input(&input).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let body = impl_trait(name, &ast.body, &ast.attrs);
    let gen = quote! {
        #[allow(dead_code, unused_variables)]
        unsafe impl #impl_generics ::hdf5_types::H5Type for #name #ty_generics #where_clause {
            #[inline]
            fn type_descriptor() -> ::hdf5_types::TypeDescriptor {
                #body
            }
        }
    };
    gen.parse().unwrap()
}

fn impl_compound(ty: &Ident, names: Vec<Ident>, types: Vec<Ty>) -> quote::Tokens {
    let (names, names2) = (&names, &names);
    quote! {
        let origin = 0usize as *const #ty;
        ::hdf5_types::TypeDescriptor::Compound(
            ::hdf5_types::CompoundType {
                fields: vec![#(
                    ::hdf5_types::CompoundField {
                        name: stringify!(#names).to_owned(),
                        ty: <#types as ::hdf5_types::H5Type>::type_descriptor(),
                        offset: unsafe { &((*origin).#names2) as *const _ as usize },
                    }
                ),*],
                size: ::std::mem::size_of::<#ty>()
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
        ::hdf5_types::TypeDescriptor::Enum(
            ::hdf5_types::EnumType {
                size: ::hdf5_types::IntSize::#size,
                signed: #signed,
                members: vec![#(
                    ::hdf5_types::EnumMember {
                        name: stringify!(#names).to_owned(),
                        value: (#values) as #repr as u64,
                    }
                ),*],
            }
        )
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

fn impl_trait(ty: &Ident, body: &Body, attrs: &[Attribute]) -> quote::Tokens {
    match *body {
        Body::Struct(VariantData::Unit) => {
            panic!("Cannot derive H5Type for unit structs");
        },
        Body::Struct(VariantData::Struct(ref fields)) => {
            if fields.is_empty() {
                panic!("Cannot derive H5Type for empty structs");
            }
            find_repr(attrs, &["C"])
                .expect("H5Type requires #[repr(C)] for structs");
            impl_compound(ty, pluck!(fields, ?ident), pluck!(fields, ty))
        },
        Body::Struct(VariantData::Tuple(ref fields)) => {
            if fields.is_empty() {
                panic!("Cannot derive H5Type for empty tuple structs");
            }
            find_repr(attrs, &["C"])
                .expect("H5Type requires #[repr(C)] for structs");
            let index = (0..fields.len()).map(|i| format!("{}", i)).map(Ident::new);
            impl_compound(ty, index.collect(), pluck!(fields, ty))
        },
        Body::Enum(ref variants) => {
            if variants.iter().any(|f| f.data != VariantData::Unit || f.discriminant.is_none()) {
                panic!("H5Type can only be derived for enums with scalar discriminants");
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

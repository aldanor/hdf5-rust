#![crate_type = "proc-macro"]

#![recursion_limit = "192"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

extern crate hdf5_types;

use std::mem;
use std::str::FromStr;

use proc_macro::TokenStream;
use syn::{Body, VariantData, Ident, Ty, ConstExpr, Attribute};

#[proc_macro_derive(H5Type)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: String = input.to_string();
    let ast = syn::parse_macro_input(&input).expect("#[derive(H5Type)]: unable to parse input");
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let body = h5type_impl(&ast.body, &ast.attrs);
    let gen = quote! {
        #[allow(dead_code, unused_variables)]
        unsafe impl #impl_generics ::hdf5_types::H5Type for #name #ty_generics #where_clause {
            #[inline]
            fn type_descriptor() -> ::hdf5_types::TypeDescriptor {
                let ty_size = ::std::mem::size_of::<#name>();
                let origin = 0usize as *const #name;
                #body
            }
        }
    };
    gen.parse().expect("#[derive(H5Type)]: unable to parse output")
}

fn h5type_impl_compound(names: Vec<&Option<Ident>>, types: Vec<&Ty>) -> quote::Tokens {
    let names_c = names.clone();
    let (fname1, fname2, fty) = (names.iter(), names_c.iter(), types.iter());

    quote! {
        ::hdf5_types::TypeDescriptor::Compound(
            ::hdf5_types::CompoundType {
                fields: vec![#(
                    ::hdf5_types::CompoundField {
                        name: stringify!(#fname1).to_owned(),
                        ty: <#fty as ::hdf5_types::H5Type>::type_descriptor(),
                        offset: unsafe { &((*origin).#fname2) as *const _ as usize },
                    }
                ),*],
                size: ty_size
            }
        )
    }
}

fn h5type_impl_enum(names: Vec<&Ident>, values: Vec<&ConstExpr>,
                    size: usize, signed: bool) -> quote::Tokens {
    let (names, values) = (names.iter(), values.iter());
    let size = Ident::new(format!("U{}", size));
    quote! {
        ::hdf5_types::TypeDescriptor::Enum(
            ::hdf5_types::EnumType {
                size: ::hdf5_types::IntSize::#size,
                signed: #signed,
                members: vec![#(
                    ::hdf5_types::EnumMember {
                        name: stringify!(#names).to_owned(),
                        value: (#values) as i64 as u64,
                    }
                ),*],
            }
        )
    }
}

fn h5type_find_repr(attrs: &Vec<Attribute>, expected: &[&str]) -> Option<Ident> {
    use syn::{AttrStyle, MetaItem, NestedMetaItem};

    for attr in attrs.iter() {
        if attr.style == AttrStyle::Outer && !attr.is_sugared_doc {
            if let MetaItem::List(ref name, ref meta_items) = attr.value {
                if name.as_ref() == "repr" {
                    for meta_item in meta_items.iter() {
                        if let &NestedMetaItem::MetaItem(MetaItem::Word(ref ident)) = meta_item {
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

fn h5type_impl(body: &Body, attrs: &Vec<Attribute>) -> quote::Tokens {
    match *body {
        Body::Struct(VariantData::Unit) => {
            quote! {
                ::hdf5_types::TypeDescriptor::Compound(
                    ::hdf5_types::CompoundType { fields: vec![], size: ty_size }
                )
            }
        }
        Body::Struct(VariantData::Struct(ref fields)) => {
            // TODO: check for repr(C)
            h5type_impl_compound(fields.iter().map(|f| &f.ident).collect(),
                                 fields.iter().map(|f| &f.ty).collect())
        },
        Body::Struct(VariantData::Tuple(ref fields)) => {
            // TODO: check for repr(C)
            let index: Vec<_> = (0..fields.len())
                .map(|i| Some(Ident::new(format!("{}", i)))).collect();
            h5type_impl_compound(index.iter().collect(),
                                 fields.iter().map(|f| &f.ty).collect())
        }
        Body::Enum(ref variants) => {
            if !variants.iter().all(|f| f.data == VariantData::Unit) {
                panic!("H5Type can only be derived for enums with scalar variants");
            } else if !variants.iter().all(|f| f.discriminant.is_some()) {
                panic!("H5Type can only be derived for enums with explicit discriminants");
            }
            let discriminants: Vec<_> = variants.iter()
                .map(|f| f.discriminant.clone().unwrap()).collect();
            let valid_reprs = &["i8", "i16", "i32", "i64",
                                "u8", "u16", "u32", "u64",
                                "isize", "usize"];
            let repr = h5type_find_repr(attrs, valid_reprs)
                .expect("H5Type can only be derived for enums with explicit representation");
            let repr = repr.as_ref();
            let size = usize::from_str(&repr[1..]).unwrap_or(mem::size_of::<usize>() * 8) / 8;
            let signed = repr.starts_with("i");
            h5type_impl_enum(variants.iter().map(|f| &f.ident).collect(),
                             discriminants.iter().collect(), size, signed)
        },
    }
}

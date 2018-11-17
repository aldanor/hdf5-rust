#![recursion_limit = "192"]

extern crate proc_macro;

use std::iter;
use std::mem;
use std::str::FromStr;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, AttrStyle, Attribute, Data, DeriveInput, Expr, Fields, Meta, NestedMeta,
    Type, TypeGenerics, TypePath,
};

#[proc_macro_derive(H5Type)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let body = impl_trait(&name, &input.data, &input.attrs, &ty_generics);
    let dummy = Ident::new(&format!("_IMPL_H5TYPE_FOR_{}", name), Span::call_site());
    let expanded = quote! {
        #[allow(dead_code, unused_variables, unused_attributes)]
        const #dummy: () = {
            #[automatically_derived]
            unsafe impl #impl_generics ::hdf5_types::H5Type for #name #ty_generics #where_clause {
                #[inline]
                fn type_descriptor() -> ::hdf5_types::TypeDescriptor {
                    #body
                }
            }
        };
    };
    proc_macro::TokenStream::from(expanded)
}

fn impl_compound<F>(
    ty: &Ident, ty_generics: &TypeGenerics, fields: Vec<F>, names: Vec<String>, types: Vec<Type>,
) -> TokenStream
where
    F: ToTokens,
{
    quote! {
        let origin: *const #ty #ty_generics = ::std::ptr::null();
        ::hdf5_types::TypeDescriptor::Compound(
            ::hdf5_types::CompoundType {
                fields: vec![#(
                    ::hdf5_types::CompoundField {
                        name: #names.to_owned(),
                        ty: <#types as ::hdf5_types::H5Type>::type_descriptor(),
                        offset: unsafe { &((*origin).#fields) as *const _ as _ },
                    }
                ),*],
                size: ::std::mem::size_of::<#ty #ty_generics>()
            }
        )
    }
}

fn impl_enum(names: Vec<Ident>, values: Vec<Expr>, repr: &Ident) -> TokenStream {
    let size = Ident::new(
        &format!(
            "U{}",
            usize::from_str(&repr.to_string()[1..]).unwrap_or(mem::size_of::<usize>() * 8) / 8
        ),
        Span::call_site(),
    );
    let signed = repr.to_string().starts_with('i');
    let repr = iter::repeat(repr);
    quote! {
        ::hdf5_types::TypeDescriptor::Enum(
            ::hdf5_types::EnumType {
                size: ::hdf5_types::IntSize::#size,
                signed: #signed,
                members: vec![#(
                    ::hdf5_types::EnumMember {
                        name: stringify!(#names).to_owned(),
                        value: (#values) as #repr as _,
                    }
                ),*],
            }
        )
    }
}

fn is_phantom_data(ty: &Type) -> bool {
    match *ty {
        Type::Path(TypePath { qself: None, ref path }) => {
            path.segments.iter().last().map(|x| x.ident == "PhantomData").unwrap_or(false)
        }
        _ => false,
    }
}

fn find_repr(attrs: &[Attribute], expected: &[&str]) -> Option<Ident> {
    for attr in attrs.iter() {
        if attr.style == AttrStyle::Outer {
            if let Ok(Meta::List(ref list)) = attr.parse_meta() {
                if list.ident == "repr" {
                    for item in list.nested.iter() {
                        if let NestedMeta::Meta(Meta::Word(ref ident)) = *item {
                            if expected.iter().any(|&s| *ident == s) {
                                return Some(Ident::new(&ident.to_string(), Span::call_site()));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

fn pluck<'a, I, F, T, S>(iter: I, func: F) -> Vec<S>
where
    I: Iterator<Item = &'a T>,
    F: Fn(&'a T) -> S,
    T: 'a,
{
    iter.map(func).collect()
}

fn impl_trait(
    ty: &Ident, data: &Data, attrs: &[Attribute], ty_generics: &TypeGenerics,
) -> TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Unit => {
                panic!("Cannot derive H5Type for unit structs");
            }
            Fields::Named(ref fields) => {
                let fields: Vec<_> =
                    fields.named.iter().filter(|f| !is_phantom_data(&f.ty)).collect();
                if fields.is_empty() {
                    panic!("Cannot derive H5Type for empty structs");
                }
                find_repr(attrs, &["C"]).expect("H5Type requires #[repr(C)] for structs");
                let types = pluck(fields.iter(), |f| f.ty.clone());
                let fields = pluck(fields.iter(), |f| f.ident.clone().unwrap());
                let names = fields.iter().map(|f| f.to_string()).collect();
                impl_compound(ty, ty_generics, fields, names, types)
            }
            Fields::Unnamed(ref fields) => {
                let (index, fields): (Vec<usize>, Vec<_>) = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter(|&(_, f)| !is_phantom_data(&f.ty))
                    .unzip();
                if fields.is_empty() {
                    panic!("Cannot derive H5Type for empty tuple structs");
                }
                find_repr(attrs, &["C"]).expect("H5Type requires #[repr(C)] for structs");
                let names = (0..fields.len()).map(|f| f.to_string()).collect();
                let types = pluck(fields.iter(), |f| f.ty.clone());
                impl_compound(ty, ty_generics, index, names, types)
            }
        },
        Data::Enum(ref data) => {
            let variants = &data.variants;
            if variants.iter().any(|v| v.fields != Fields::Unit || v.discriminant.is_none()) {
                panic!("H5Type can only be derived for enums with scalar discriminants");
            } else if variants.is_empty() {
                panic!("Cannot derive H5Type for empty enums")
            }
            let enum_reprs =
                &["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "isize", "usize"];
            let repr = find_repr(attrs, enum_reprs)
                .expect("H5Type can only be derived for enums with explicit representation");
            let names = pluck(variants.iter(), |v| v.ident.clone());
            let values = pluck(variants.iter(), |v| v.discriminant.clone().unwrap().1);
            impl_enum(names, values, &repr)
        }
        Data::Union(_) => {
            panic!("Cannot derive H5Type for tagged unions");
        }
    }
}

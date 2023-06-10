#![recursion_limit = "192"]

use std::iter;
use std::mem;
use std::str::FromStr;

use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, AttrStyle, Attribute, Data, DeriveInput, Expr, Fields, Index, LitStr, Type,
    TypeGenerics, TypePath,
};

#[proc_macro_derive(H5Type, attributes(hdf5))]
#[proc_macro_error]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let body = impl_trait(&name, &input.data, &input.attrs, &ty_generics);
    let dummy = Ident::new(&format!("_IMPL_H5TYPE_FOR_{}", name), Span::call_site());
    let expanded = quote! {
        #[allow(dead_code, unused_variables, unused_attributes)]
        const #dummy: () = {
            extern crate hdf5 as _h5;

            #[automatically_derived]
            unsafe impl #impl_generics _h5::types::H5Type for #name #ty_generics #where_clause {
                #[inline]
                fn type_descriptor() -> _h5::types::TypeDescriptor {
                    #body
                }
            }
        };
    };
    proc_macro::TokenStream::from(expanded)
}

fn impl_compound<F>(
    ty: &Ident, ty_generics: &TypeGenerics, fields: &[F], names: &[String], types: &[Type],
) -> TokenStream
where
    F: ToTokens,
{
    quote! {
        let origin = ::std::mem::MaybeUninit::<#ty #ty_generics>::uninit();
        let origin_ptr = origin.as_ptr();
        let mut fields = vec![#(
            _h5::types::CompoundField {
                name: #names.to_owned(),
                ty: <#types as _h5::types::H5Type>::type_descriptor(),
                offset: unsafe {
                    ::std::ptr::addr_of!((*origin_ptr).#fields).cast::<u8>()
                        .offset_from(origin_ptr.cast()) as usize
                },
                index: 0,
            }
        ),*];
        for i in 0..fields.len() {
            fields[i].index = i;
        }
        let size = ::std::mem::size_of::<#ty #ty_generics>();
        _h5::types::TypeDescriptor::Compound(_h5::types::CompoundType { fields, size })
    }
}

fn impl_transparent(ty: &Type) -> TokenStream {
    quote! {
        <#ty as _h5::types::H5Type>::type_descriptor()
    }
}

fn impl_enum(names: &[String], values: &[Expr], repr: &Ident) -> TokenStream {
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
        _h5::types::TypeDescriptor::Enum(
            _h5::types::EnumType {
                size: _h5::types::IntSize::#size,
                signed: #signed,
                members: vec![#(
                    _h5::types::EnumMember {
                        name: #names.to_owned(),
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
            path.segments.iter().last().map_or(false, |x| x.ident == "PhantomData")
        }
        _ => false,
    }
}

fn find_repr(attrs: &[Attribute], expected: &[&str]) -> Option<Ident> {
    let mut repr = None;
    for attr in attrs.iter() {
        if attr.style != AttrStyle::Outer {
            continue;
        }
        if !attr.path().is_ident("repr") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if expected.iter().any(|s| meta.path.is_ident(s)) {
                if repr.is_some() {
                    abort!(meta.path, "ambiguous repr attribute");
                } else {
                    repr = meta.path.get_ident().cloned();
                }
            }
            Ok(())
        })
        .ok()?;
    }
    repr
}

fn find_hdf5_rename(attrs: &[Attribute]) -> Option<String> {
    let mut rename = None;
    let attr = attrs.iter().find(|a| a.path().is_ident("hdf5"))?;
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("rename") && rename.is_none() {
            rename = Some(meta.value()?.parse::<LitStr>()?.value());
        }
        Ok(())
    })
    .ok()?;
    rename
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
                abort!(ty, "cannot derive `H5Type` for unit structs");
            }
            Fields::Named(ref fields) => {
                let fields: Vec<_> =
                    fields.named.iter().filter(|f| !is_phantom_data(&f.ty)).collect();
                if fields.is_empty() {
                    abort!(ty, "cannot derive `H5Type` for empty structs");
                }

                let repr =
                    find_repr(attrs, &["C", "packed", "transparent"]).unwrap_or_else(|| {
                        abort!(ty,
                    "`H5Type` requires repr(C), repr(packed) or repr(transparent) for structs")
                    });
                if repr == "transparent" {
                    assert_eq!(fields.len(), 1);
                    impl_transparent(&fields[0].ty)
                } else {
                    let types = pluck(fields.iter(), |f| f.ty.clone());
                    let names = pluck(fields.iter(), |f| {
                        find_hdf5_rename(&f.attrs)
                            .unwrap_or_else(|| f.ident.as_ref().unwrap().to_string())
                    });
                    let fields = pluck(fields.iter(), |f| f.ident.clone().unwrap());
                    impl_compound(ty, ty_generics, &fields, &names, &types)
                }
            }
            Fields::Unnamed(ref fields) => {
                let (index, fields): (Vec<Index>, Vec<_>) = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter(|&(_, f)| !is_phantom_data(&f.ty))
                    .map(|(i, f)| (Index::from(i), f))
                    .unzip();
                if fields.is_empty() {
                    abort!(ty, "cannot derive `H5Type` for empty tuple structs")
                }

                let repr =  find_repr(attrs, &["C", "packed", "transparent"]).unwrap_or_else(|| {
                        abort!(ty,
                    "`H5Type` requires repr(C), repr(packed) or repr(transparent) for tuple structs")
                    });
                if repr == "transparent" {
                    assert_eq!(fields.len(), 1);
                    impl_transparent(&fields[0].ty)
                } else {
                    let names = fields
                        .iter()
                        .enumerate()
                        .map(|(n, f)| find_hdf5_rename(&f.attrs).unwrap_or_else(|| n.to_string()))
                        .collect::<Vec<_>>();
                    let types = pluck(fields.iter(), |f| f.ty.clone());
                    impl_compound(ty, ty_generics, &index, &names, &types)
                }
            }
        },
        Data::Enum(ref data) => {
            let variants = &data.variants;

            if variants.iter().any(|v| v.fields != Fields::Unit || v.discriminant.is_none()) {
                abort!(ty, "`H5Type` can only be derived for enums with scalar discriminants")
            }

            if variants.is_empty() {
                abort!(ty, "cannot derive `H5Type` for empty enums")
            }

            let enum_reprs =
                &["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "isize", "usize"];
            let repr = find_repr(attrs, enum_reprs).unwrap_or_else(|| {
                abort!(ty, "`H5Type` can only be derived for enums with explicit representation")
            });
            let names = variants
                .iter()
                .map(|v| find_hdf5_rename(&v.attrs).unwrap_or_else(|| v.ident.to_string()))
                .collect::<Vec<_>>();
            let values = pluck(variants.iter(), |v| v.discriminant.clone().unwrap().1);
            impl_enum(&names, &values, &repr)
        }
        Data::Union(_) => {
            abort!(ty, "cannot derive `H5Type` for tagged unions");
        }
    }
}

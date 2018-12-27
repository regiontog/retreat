#![deny(clippy::all)]
#![recursion_limit = "256"]
extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{parse_macro_input, parse_quote, DeriveInput};

mod build;
mod imprinter;
mod size;

type DeriveResult = Result<TokenStream, syn::Error>;

fn unwrap(expanded: syn::Result<proc_macro2::TokenStream>) -> proc_macro::TokenStream {
    proc_macro::TokenStream::from(expanded.unwrap_or_else(|e| syn::Error::to_compile_error(&e)))
}

#[proc_macro_derive(Build, attributes(noserc))]
pub fn derive_build(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    unwrap(build::derive(input))
}

#[proc_macro_derive(WriteTypeInfo)]
pub fn derive_imprinter(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    //TODO: Option for specifying crate local WriteTypeInfo struct.
    unwrap(imprinter::derive(input))
}

#[proc_macro_derive(SizableStatic)]
pub fn derive_size_static(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    unwrap(size::derive_static(input))
}

#[proc_macro_derive(SizableDynamic)]
pub fn derive_size_dynamic(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    unwrap(size::derive_dynamic(input))
}

//TODO: Special derive for static enums! (They need special versions
// of Build/WriteTypeInfo that reserves space for the longest variant) Also
// figure out how to swap variant variant in place. Otherwise this version of
// enum's are strictly worse than the dynamic kind

struct Options {
    arena: Option<syn::LifetimeDef>,
}

impl Options {
    fn from(attrs: &[syn::Attribute]) -> Result<Self, syn::Error> {
        use freyr::prelude::ResultPrelude;

        let attrs: syn::Result<Vec<_>> = attrs
            .iter()
            .filter(|attr| validate_path(&attr.path))
            .filter_map(|attr| {
                attr.parse_meta()
                    .map(|meta| match meta {
                        syn::Meta::List(ml) => Some(Ok(ml.nested)),
                        _ => None,
                    })
                    .unwrap_or_else(|e| Some(Err(e)))
            })
            .flat_map(|result_of_iter| {
                result_of_iter
                    .map(|punctuated| punctuated.into_iter())
                    .flip_inner_iter()
            })
            .collect();

        let attrs = attrs?;

        let arena: syn::Ident = parse_quote!(arena);

        Ok(Options {
            arena: match attrs
                .iter()
                .filter_map(|nested| match nested {
                    syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) if nv.ident == arena => {
                        match &nv.lit {
                            syn::Lit::Str(s) => Some(s.parse()),
                            _ => None,
                        }
                    }
                    _ => None,
                })
                .next()
            {
                Some(result) => Some(result?),
                None => None,
            },
        })
    }

    fn arena_generics(&self, generics: &syn::Generics) -> syn::LifetimeDef {
        self.arena
            .clone()
            .or_else(|| {
                generics
                    .params
                    .iter()
                    .filter_map(|param| match param {
                        syn::GenericParam::Lifetime(ft) => Some(ft),
                        _ => None,
                    })
                    .next()
                    .cloned()
            })
            .unwrap_or_else(|| parse_quote!('arena))
    }
}

fn validate_path(path: &syn::Path) -> bool {
    let noserc: syn::Ident = parse_quote!(noserc);

    path.segments.len() == 1
        && path.segments[0].arguments.is_empty()
        && path.segments[0].ident == noserc
}

fn split_for_impl_add(
    generics: &mut syn::Generics,
    lifetime: &syn::LifetimeDef,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let ty_generics = ty_generics.into_token_stream();
    let where_clause = where_clause.into_token_stream();

    let impl_generics = if generics
        .lifetimes()
        .find(|lt| lt.lifetime == lifetime.lifetime)
        .is_none()
    {
        generics
            .params
            .push(syn::GenericParam::Lifetime(lifetime.clone()));
        let (impl_generics, _, _) = generics.split_for_impl();
        impl_generics
    } else {
        impl_generics
    };

    (impl_generics.into_token_stream(), ty_generics, where_clause)
}

#[allow(clippy::needless_lifetimes)]
pub(crate) fn build_from_fields<'a>(
    ty: &proc_macro2::TokenStream,
    fields: &'a syn::Fields,
    body: impl FnOnce(
        Box<dyn Iterator<Item = syn::Ident> + 'a>,
        Box<dyn Iterator<Item = &syn::Type> + 'a>,
    ) -> proc_macro2::TokenStream,
    ret: impl FnOnce(proc_macro2::TokenStream) -> proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let types = fields.iter().map(|f| &f.ty);

    match &fields {
        syn::Fields::Unit => {
            let body = body(Box::new(std::iter::empty()), Box::new(std::iter::empty()));
            let ret = ret(quote! {
                #ty
            });
            quote! {
                #body
                #ret
            }
        }
        syn::Fields::Named(fields) => {
            let fields1 = fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().clone());
            let fields2 = fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap().clone());

            let body = body(Box::new(fields1), Box::new(types));
            let ret = ret(quote!(#ty {
                #(#fields2,)*
            }));

            quote! {
                #body
                #ret
            }
        }
        syn::Fields::Unnamed(fields) => {
            let fields1 = (0..fields.unnamed.len())
                .map(|i| syn::Ident::new(&format!("field{}", i), proc_macro2::Span::call_site()));
            let fields2 = fields1.clone();

            let body = body(Box::new(fields1), Box::new(types));
            let ret = ret(quote! {
                #ty(#(#fields2,)*)
            });

            quote! {
                #body
                #ret
            }
        }
    }
}

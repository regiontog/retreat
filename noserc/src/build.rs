use quote::quote;
use syn::{spanned::Spanned, DataEnum, DataStruct, DeriveInput};

pub(crate) fn derive(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    match &input.data {
        syn::Data::Struct(data) => {
            let data = data.clone();
            struct_derive(input, &data)
        }
        syn::Data::Enum(data) => {
            let data = data.clone();
            enum_derive(input, &data)
        }
        _ => Err(syn::Error::new(
            input.span(),
            "'Build' derive does not support this data type!",
        )),
    }
}

pub(crate) fn struct_derive(mut input: DeriveInput, data: &DataStruct) -> crate::DeriveResult {
    let name = input.ident;
    let options = crate::Options::from(&input.attrs)?;

    let build_impl = crate::build_from_fields(
        &quote!(#name),
        &data.fields,
        |fields, types| {
            quote! {
                #(let (arena, #fields) = <#types>::build(arena)?;)*
            }
        },
        |constructor| {
            quote! {
                Ok((arena, #constructor))
            }
        },
    );

    let unsafe_build_impl = crate::build_from_fields(
        &quote!(#name),
        &data.fields,
        |fields, types| {
            quote! {
                #(let (arena, #fields) = <#types>::unchecked_build(arena);)*
            }
        },
        |constructor| {
            quote! {
                (arena, #constructor)
            }
        },
    );

    let arena_generics = options.arena_generics(&input.generics);

    let (impl_generics, ty_generics, where_clause) =
        crate::split_for_impl_add(&mut input.generics, &arena_generics);

    Ok(quote! {
        impl #impl_generics ::noser::traits::Build <#arena_generics> for #name #ty_generics #where_clause {
            #[inline]
            fn build<'_a>(arena: &'_a mut [u8]) -> ::noser::Result<(&'_a mut [u8], Self)>
                where '_a: #arena_generics
            {
                #build_impl
            }

            #[inline]
            unsafe fn unchecked_build<'_a>(arena: &'_a mut [u8]) -> (&'_a mut [u8], Self)
                where '_a: #arena_generics
            {
                #unsafe_build_impl
            }
        }
    })
}

pub(crate) fn enum_derive(mut input: DeriveInput, data: &DataEnum) -> crate::DeriveResult {
    let name = input.ident;
    let options = crate::Options::from(&input.attrs)?;

    let arena_generics = options.arena_generics(&input.generics);

    let (impl_generics, ty_generics, where_clause) =
        crate::split_for_impl_add(&mut input.generics, &arena_generics);

    let num_variants = data.variants.len();
    let variant_bytes = ((num_variants as f64).log2() / 8.0).ceil().max(0.) as usize;

    let variants = 0..num_variants as u64;
    let variants2 = 0..num_variants as u64;

    let builders = data.variants.iter().map(|v| {
        let ident = &v.ident;
        crate::build_from_fields(
            &quote!(#name::#ident),
            &v.fields,
            |fields, types| {
                quote! {
                    #(let (arena, #fields) = <#types>::build(arena)?;)*
                }
            },
            |constructor| {
                quote! {
                    Ok((arena, #constructor))
                }
            },
        )
    });

    let unsafe_builders = data.variants.iter().map(|v| {
        let ident = &v.ident;
        crate::build_from_fields(
            &quote!(#name::#ident),
            &v.fields,
            |fields, types| {
                quote! {
                    let arena = &mut arena[#variant_bytes..];
                    #(let (arena, #fields) = <#types>::unchecked_build(arena);)*
                }
            },
            |constructor| {
                quote! {
                    (arena, #constructor)
                }
            },
        )
    });

    Ok(quote! {
        impl #impl_generics ::noser::traits::Build <#arena_generics> for #name #ty_generics #where_clause {
            #[inline]
            fn build<'_a>(arena: &'_a mut [u8]) -> ::noser::Result<(&'_a mut [u8], Self)>
                where '_a: #arena_generics
            {
                use noser::prelude::SliceExt;
                let (left, arena) = arena.noser_split(#variant_bytes as ::noser::Ptr)?;

                match ::noser::read_var_len_int(left, #variant_bytes) {
                    #(#variants => {
                        #builders
                    }),*
                    _ => Err(::noser::NoserError::Malformed)
                }
            }

            #[inline]
            unsafe fn unchecked_build<'_a>(arena: &'_a mut [u8]) -> (&'_a mut [u8], Self)
                where '_a: #arena_generics
            {
                match ::noser::read_var_len_int(arena, #variant_bytes) {
                    #(#variants2 => {
                        #unsafe_builders
                    }),*
                    _ => panic!("Malformed arena")
                }
            }
        }
    })
}

use quote::quote;
use syn::{spanned::Spanned, DataEnum, DataStruct, DeriveInput};

pub(crate) fn derive_static(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    match &input.data {
        syn::Data::Struct(data) => {
            let data = data.clone();
            struct_derive_static(input, &data)
        }
        _ => Err(syn::Error::new(
            input.span(),
            "'StaticSizeable' derive does not support this data type!",
        )),
    }
}

pub(crate) fn derive_dynamic(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    match &input.data {
        syn::Data::Struct(data) => {
            let data = data.clone();
            struct_derive_dynamic(input, &data)
        }
        syn::Data::Enum(data) => {
            let data = data.clone();
            enum_derive_dynamic(input, &data)
        }
        _ => Err(syn::Error::new(
            input.span(),
            "'DynamicSizeable' derive does not support this data type!",
        )),
    }
}

pub(crate) fn struct_derive_static(input: DeriveInput, data: &DataStruct) -> crate::DeriveResult {
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let types = data.fields.iter().map(|f| &f.ty);

    Ok(quote! {
        impl #impl_generics ::noser::traits::size::Sizeable for #name #ty_generics #where_clause {
            type Strategy = ::noser::traits::size::Static;

            fn read_size(arena: &[u8]) -> ::noser::traits::size::ReadReturn<Self> {
                let mut size = 0;
                #(size += <#types as ::noser::traits::size::Sizeable>::static_size();)*
                Ok(size)
            }
        }
    })
}

pub(crate) fn struct_derive_dynamic(input: DeriveInput, data: &DataStruct) -> crate::DeriveResult {
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let types = data.fields.iter().map(|f| &f.ty);

    Ok(quote! {
        impl #impl_generics ::noser::traits::size::Sizeable for #name #ty_generics #where_clause {
            type Strategy = ::noser::traits::size::Dynamic;

            fn read_size(arena: &[u8]) -> ::noser::traits::size::ReadReturn<Self> {
                use ::noser::prelude::SliceExt;
                use ::noser::traits::size::NoError;

                let mut size = 0;
                let mut cur_size = 0;
                #(
                    cur_size = <#types as ::noser::traits::size::Sizeable>::read_size(arena).map_err(Into::into)?;
                    let (_, arena) = arena.noser_split_imut(cur_size)?;
                    size += cur_size;
                )*
                Ok(size)
            }
        }
    })
}

pub(crate) fn enum_derive_dynamic(input: DeriveInput, data: &DataEnum) -> crate::DeriveResult {
    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let num_variants = data.variants.len();
    let variant_bytes = ((num_variants as f64).log2() / 8.0).ceil().max(0.) as usize;

    let variants = 0..num_variants as u64;

    let read_sizes = data.variants.iter().map(|v| {
        let types = v.fields.iter().map(|f| &f.ty);

        quote! {
            #(
                cur_size = <#types as ::noser::traits::size::Sizeable>::read_size(arena).map_err(Into::into)?;
                let (_, arena) = arena.noser_split_imut(cur_size)?;
                size += cur_size;
            )*
            Ok(size)
        }
    });

    Ok(quote! {
        impl #impl_generics ::noser::traits::size::Sizeable for #name #ty_generics #where_clause {
            type Strategy = ::noser::traits::size::Dynamic;

            fn read_size(arena: &[u8]) -> ::noser::traits::size::ReadReturn<Self> {
                use ::noser::prelude::SliceExt;
                use ::noser::traits::size::NoError;

                let mut size = #variant_bytes as ::noser::Ptr;
                let mut cur_size = 0;

                let (left, arena) = arena.noser_split_imut(#variant_bytes as ::noser::Ptr)?;

                match ::noser::read_var_len_int(left, #variant_bytes) {
                    #(#variants => {
                        #read_sizes
                    }),*
                    _ => Err(::noser::NoserError::Malformed)
                }
            }
        }
    })
}

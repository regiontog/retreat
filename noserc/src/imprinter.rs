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

pub(crate) fn struct_derive(input: DeriveInput, data: &DataStruct) -> crate::DeriveResult {
    use heck::ShoutySnakeCase;

    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let types = data.fields.iter().map(|f| &f.ty);
    let types2 = data.fields.iter().map(|f| &f.ty);

    let imprinter_struct_str = format!("Imprint{}", name.to_string());
    let imprinter_struct = syn::Ident::new(&imprinter_struct_str, proc_macro2::Span::call_site());
    let imprinter_static = syn::Ident::new(
        &imprinter_struct_str.to_shouty_snake_case(),
        proc_macro2::Span::call_site(),
    );

    Ok(quote! {
        pub(crate) struct #imprinter_struct;

        impl #impl_generics ::noser::traits::WriteTypeInfo<#name #ty_generics> for #imprinter_struct #where_clause {
            #[inline]
            fn imprint(&self, arena: &mut [u8]) -> ::noser::Result<()> {
                use noser::prelude::SliceExt;
                #(
                let imprinter = <crate::WriteTypeInfo<#types>>::default();
                let (left, arena) = arena.noser_split(imprinter.result_size())?;
                imprinter.imprint(left)?;
                )*
                Ok(())
            }

            #[inline]
            fn result_size(&self) -> ::noser::Ptr {
                let mut size = 0;
                #(size += <crate::WriteTypeInfo<#types2>>::default().result_size();)*
                size
            }
        }

        pub(crate) static #imprinter_static: #imprinter_struct = #imprinter_struct {};

        impl #impl_generics Default for crate::WriteTypeInfo<'_, #name #ty_generics> #where_clause {
            #[inline]
            fn default() -> crate::WriteTypeInfo<'static, #name #ty_generics> {
                crate::WriteTypeInfo(&#imprinter_static)
            }
        }
    })
}

pub(crate) fn enum_derive(input: DeriveInput, data: &DataEnum) -> crate::DeriveResult {
    let name = input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let num_variants = data.variants.len();
    let variant_bytes = ((num_variants as f64).log2() / 8.0).ceil().max(0.) as usize;
    let variant_bytes_repeat = std::iter::repeat(variant_bytes);
    let variant_vals = 0..num_variants as u64;

    let variants = data.variants.iter().map(|v| &v.ident);
    let variants2 = data.variants.iter().map(|v| &v.ident);
    let variants3 = data.variants.iter().map(|v| &v.ident);

    let imprinters = data.variants.iter().map(|v| {
        let types = v.fields.iter().map(|f| &f.ty);

        quote! {
            #(
            let imprinter = <crate::WriteTypeInfo<#types>>::default();
            let (left, arena) = arena.noser_split(imprinter.result_size())?;
            imprinter.imprint(left)?;
            )*
            Ok(())
        }
    });

    let sizes = data.variants.iter().map(|v| {
        let types = v.fields.iter().map(|f| &f.ty);

        quote! {
            #(size += <crate::WriteTypeInfo<#types>>::default().result_size();)*
        }
    });

    let imprinter_enum_str = format!("Imprint{}", name.to_string());
    let imprinter_enum = syn::Ident::new(&imprinter_enum_str, proc_macro2::Span::call_site());
    let imprinter_enum_repeat = std::iter::repeat(&imprinter_enum);
    let imprinter_enum_repeat2 = std::iter::repeat(&imprinter_enum);

    Ok(quote! {
        pub(crate) enum #imprinter_enum {
            #(#variants,)*
        }

        impl #impl_generics ::noser::traits::WriteTypeInfo<#name #ty_generics> for #imprinter_enum #where_clause {
            #[inline]
            fn imprint(&self, arena: &mut [u8]) -> ::noser::Result<()> {
                use noser::prelude::SliceExt;

                let (left, arena) = arena.noser_split(#variant_bytes as ::noser::Ptr)?;

                match self {
                    #(#imprinter_enum_repeat::#variants2 => {
                        ::noser::write_var_len_int(left, #variant_bytes_repeat, #variant_vals);
                        #imprinters
                    },)*
                }
            }

            #[inline]
            fn result_size(&self) -> ::noser::Ptr {
                let mut size = #variant_bytes as ::noser::Ptr;
                match self {
                    #(#imprinter_enum_repeat2::#variants3 => {
                        #sizes
                    },)*
                };
                size
            }
        }
    })
}

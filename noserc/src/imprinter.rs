use quote::quote;
use syn::{parse_quote, spanned::Spanned, DataEnum, DataStruct, DeriveInput};

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
            "'WriteTypeInfo' derive does not support this data type!",
        )),
    }
}

pub(crate) fn struct_derive(mut input: DeriveInput, data: &DataStruct) -> crate::DeriveResult {
    use heck::ShoutySnakeCase;

    let name = input.ident;

    for type_param in input.generics.type_params_mut() {
        type_param
            .bounds
            .push(parse_quote!(::noser::traits::DefaultWriter));
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let types = data.fields.iter().map(|f| &f.ty);
    let types2 = data.fields.iter().map(|f| &f.ty);
    let types3 = data.fields.iter().map(|f| &f.ty);
    let types4 = data.fields.iter().map(|f| &f.ty);
    let types5 = data.fields.iter().map(|f| &f.ty);

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
                let imprinter = <#types as ::noser::traits::DefaultWriter>::writer();
                let (left, arena) = arena.noser_split(::noser::traits::WriteTypeInfo::<#types2>::result_size(imprinter))?;
                ::noser::traits::WriteTypeInfo::<#types3>::imprint(imprinter, left)?;
                )*
                Ok(())
            }

            #[inline]
            fn result_size(&self) -> ::noser::Ptr {
                let mut size = 0;
                #(size += ::noser::traits::WriteTypeInfo::<#types4>::result_size(<#types5 as ::noser::traits::DefaultWriter>::writer());)*
                size
            }
        }

        pub(crate) static #imprinter_static: #imprinter_struct = #imprinter_struct {};

        impl #impl_generics ::noser::traits::DefaultWriter for #name #ty_generics #where_clause {
            type Writer = #imprinter_struct;

            #[inline]
            fn writer() -> &'static Self::Writer {
                &#imprinter_static
            }
        }
    })
}

pub(crate) fn enum_derive(mut input: DeriveInput, data: &DataEnum) -> crate::DeriveResult {
    let name = input.ident;

    for type_param in input.generics.type_params_mut() {
        type_param
            .bounds
            .push(parse_quote!(::noser::traits::DefaultWriter));
    }

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
        let types2 = v.fields.iter().map(|f| &f.ty);
        let types3 = v.fields.iter().map(|f| &f.ty);

        quote! {
            #(
            let imprinter = <#types as ::noser::traits::DefaultWriter>::writer();
            let (left, arena) = arena.noser_split(::noser::traits::WriteTypeInfo::<#types2>::result_size(imprinter))?;
            ::noser::traits::WriteTypeInfo::<#types3>::imprint(imprinter, left)?;
            )*
            Ok(())
        }
    });

    let sizes = data.variants.iter().map(|v| {
        let types = v.fields.iter().map(|f| &f.ty);
        let types2 = v.fields.iter().map(|f| &f.ty);

        quote! {
            #(size += ::noser::traits::WriteTypeInfo::<#types>::result_size(<#types2 as ::noser::traits::DefaultWriter>::writer());)*
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

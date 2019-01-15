use quote::quote;
use syn::{parse_quote, spanned::Spanned, DataEnum, DeriveInput};

pub(crate) fn derive(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    match &input.data {
        syn::Data::Enum(data) => {
            let data = data.clone();
            enum_derive(input, &data)
        }
        _ => Err(syn::Error::new(
            input.span(),
            "'StaticEnum' derive does not support this data type!",
        )),
    }
}

pub(crate) fn enum_derive(mut input: DeriveInput, data: &DataEnum) -> crate::DeriveResult {
    let options = crate::Options::from(&input.attrs)?;
    let arena_generics = options.arena_generics(&input.generics);

    for type_param in input.generics.type_params_mut() {
        type_param.bounds.push(parse_quote!(
            ::noser::traits::Build<#arena_generics>
        ));
        type_param
            .bounds
            .push(parse_quote!(::noser::traits::size::StaticSizeable));
    }

    let (impl_generics, ty_generics, where_clause) =
        crate::split_for_impl_add(&mut input.generics, &arena_generics);

    let name = &input.ident;

    let variant_enum_str = format!("{}Variant", name.to_string());
    let variant_enum = syn::Ident::new(&variant_enum_str, proc_macro2::Span::call_site());

    let variant_enum_repeat = std::iter::repeat(&variant_enum);
    let variant_enum_repeat2 = variant_enum_repeat.clone();
    let variant_enum_repeat3 = variant_enum_repeat.clone();
    let variant_enum_repeat4 = variant_enum_repeat.clone();

    let num_variants = data.variants.len();
    let variant_bytes = ((num_variants as f64).log2() / 8.0).ceil().max(0.) as usize;
    let variant_vals = 0..num_variants as u64;
    let variant_vals2 = 0..num_variants as u64;

    let variants = data.variants.iter().map(|v| &v.ident);
    let variants2 = data.variants.iter().map(|v| &v.ident);
    let variants3 = data.variants.iter().map(|v| &v.ident);
    let variants4 = data.variants.iter().map(|v| &v.ident);
    let variants5 = data.variants.iter().map(|v| &v.ident);

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
                    Ok(#constructor)
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
                    #constructor
                }
            },
        )
    });

    let sizes = data.variants.iter().map(|variant| {
        let types = variant.fields.iter().map(|f| &f.ty);

        quote! {
            {
                let mut size = 0;
                #(size += <#types as ::noser::traits::size::Sizeable>::static_size();)*
                size
            }
        }
    });

    Ok(quote! {
        enum #variant_enum {
            #(#variants,)*
        }

        impl ::noser::traits::Tagged for #variant_enum {
            #[inline]
            fn variant_tag(&self) -> u64 {
                match self {
                    #(#variant_enum_repeat::#variants2 => #variant_vals,)*
                }
            }

            #[inline]
            fn from_tag(tag: u64) -> ::noser::Result<Self> {
                match tag {
                    #(#variant_vals2 => Ok(#variant_enum_repeat2::#variants3),)*
                    _ => Err(::noser::NoserError::Malformed)
                }
            }

        }

        impl #impl_generics ::noser::traits::StaticEnum<#arena_generics> for #name #ty_generics #where_clause {
            type VariantEnum = #variant_enum;

            const VARIANT_BYTES: usize = #variant_bytes;
            const CONTENTS_SIZE: std::option::Option<usize> = None;

            fn calculate_contents_size() -> usize {
                let mut max_size = 0;
                #(
                    let maybe_max = #sizes;
                    if maybe_max > max_size {
                        max_size = maybe_max;
                    }
                )*
                max_size as usize
            }

            #[inline]
            fn construct_variant(variant: &Self::VariantEnum, arena: &#arena_generics mut [u8]) -> ::noser::Result<Self> {
                use ::noser::traits::Build;

                match variant {
                    #(#variant_enum_repeat3::#variants4 => { #builders },)*
                }
            }

            #[inline]
            fn unchecked_construct_variant(variant: &Self::VariantEnum, arena: &#arena_generics mut [u8]) -> Self {
                use ::noser::traits::Build;

                match variant {
                    #(#variant_enum_repeat4::#variants5 => { #unsafe_builders },)*
                }
            }
        }
    })
}

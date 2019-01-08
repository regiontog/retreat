use quote::quote;
use syn::{spanned::Spanned, DataEnum, DataStruct, DeriveInput};

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
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let name = &input.ident;

    let variant_enum_str = format!("{}Variant", name.to_string());
    let variant_enum = syn::Ident::new(&variant_enum_str, proc_macro2::Span::call_site());

    let variant_enum_repeat = std::iter::repeat(&variant_enum);
    let variant_enum_repeat2 = std::iter::repeat(&variant_enum);

    let num_variants = data.variants.len();
    let variant_bytes = ((num_variants as f64).log2() / 8.0).ceil().max(0.) as usize;
    let variant_vals = 0..num_variants as u64;
    let variant_vals2 = 0..num_variants as u64;

    let variants = data.variants.iter().map(|v| &v.ident);
    let variants2 = data.variants.iter().map(|v| &v.ident);
    let variants3 = data.variants.iter().map(|v| &v.ident);

    let types = data
        .variants
        .iter()
        .flat_map(|v| v.fields.iter().map(|f| &f.ty));

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

        impl #impl_generics ::noser::traits::StaticEnum for #name #ty_generics #where_clause {
            type VariantEnum = #variant_enum;

            #[inline]
            fn variant_bytes() -> usize {
                #variant_bytes
            }

            #[inline]
            fn static_size() -> usize {
                use noser::traits::WriteTypeInfoErased;

                let mut size = 0;
                #(size += <#types as ::noser::traits::DefaultWriter>::writer().result_size();)*
                size as usize
            }

            #[inline]
            fn construct_variant(variant: &Self::VariantEnum, arena: &mut [u8]) -> ::noser::Result<Self> {
                unimplemented!()
            }

            #[inline]
            fn unchecked_construct_variant(variant: &Self::VariantEnum, arena: &mut [u8]) -> Self {
                unimplemented!()
            }
        }
    })
}

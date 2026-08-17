use quote::{ToTokens, quote};

pub(crate) fn impl_for_struct(
    input: &syn::DeriveInput,
    data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let Some((kind, fields)) = super::helpers::extract_struct_info(data) else {
        unimplemented!(
            "`WriteToBuffer` can only be derived for structs with named fields or with a single unnamed `u8` field"
        )
    };

    if kind == super::helpers::StructKind::TupleStruct {
        let ty = &fields[0].ty;

        return quote! {
            impl #impl_generics grib_template_helpers::WriteToBuffer for #name #type_generics #where_clause {
                fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
                    <#ty as grib_template_helpers::WriteToBuffer>::write_to_buffer(&self.0, buf)
                }

                fn num_bytes_required(&self) -> usize {
                    <#ty as grib_template_helpers::WriteToBuffer>::num_bytes_required(&self.0)
                }
            }
        };
    }

    let mut writes = Vec::new();
    let mut sizes = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let (ty, self_ident) =
            if let Some(num_octets) = super::helpers::NumOctets::try_from(field).ok() {
                (
                    quote! { grib_template_helpers::NonStdLenUint<#ty> },
                    quote! {
                        &grib_template_helpers::NonStdLenUint::new(self.#ident, #num_octets)
                    },
                )
            } else {
                (ty.to_token_stream(), quote! { &self.#ident })
            };

        writes.push(quote! {
            pos += <#ty as grib_template_helpers::WriteToBuffer>::write_to_buffer(
                #self_ident,
                &mut buf[pos..],
            )?;
        });

        sizes.push(quote! {
            size += <#ty as grib_template_helpers::WriteToBuffer>::num_bytes_required(
                #self_ident,
            );
        });
    }

    quote! {
        impl #impl_generics grib_template_helpers::WriteToBuffer for #name #type_generics #where_clause {
            fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
                if buf.len() < self.num_bytes_required() {
                    return Err("destination buffer is too small");
                }

                let mut pos = 0;
                #(#writes)*;
                Ok(pos)
            }

            fn num_bytes_required(&self) -> usize {
                let mut size = 0;
                #(#sizes)*;
                size
            }
        }
    }
}

pub(crate) fn impl_for_enum(
    input: &syn::DeriveInput,
    data: &syn::DataEnum,
) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let mut writes = Vec::new();
    let mut sizes = Vec::new();

    for variant in &data.variants {
        let variant_ident = &variant.ident;

        if let syn::Fields::Unnamed(fields) = &variant.fields
            && fields.unnamed.len() == 1
        {
            let inner_ty = &fields.unnamed.first().unwrap().ty;

            writes.push(quote! {
                #name::#variant_ident(inner) => <#inner_ty as grib_template_helpers::WriteToBuffer>::write_to_buffer(
                    inner,
                    buf,
                )
            });

            sizes.push(quote! {
                #name::#variant_ident(inner) => <#inner_ty as grib_template_helpers::WriteToBuffer>::num_bytes_required(
                    inner,
                )
            });
        } else {
            unimplemented!("`WriteToBuffer` only supports single-field tuple variants");
        }
    }

    quote! {
        impl #impl_generics grib_template_helpers::WriteToBuffer for #name #type_generics #where_clause {
            fn write_to_buffer(&self, buf: &mut [u8]) -> Result<usize, &'static str> {
                match self {
                    #(#writes),*,
                }
            }

            fn num_bytes_required(&self) -> usize {
                match self {
                    #(#sizes),*,
                }
            }
        }
    }
}

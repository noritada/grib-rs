use quote::quote;

use super::helpers::attr_value;

pub(crate) fn impl_for_struct(
    input: &syn::DeriveInput,
    data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let Some((kind, fields)) = super::helpers::extract_struct_info(data) else {
        unimplemented!(
            "`TryFromSlice` can only be derived for structs with named fields or with a single unnamed `u8` field"
        )
    };

    if kind == super::helpers::StructKind::TupleStruct {
        let field_reads = fields.iter().map(|field| {
            let ty = &field.ty;
            quote! {
                <#ty as grib_template_helpers::TryFromSlice>::try_from_slice(slice, pos)?
            }
        });

        return quote! {
            impl #impl_generics grib_template_helpers::TryFromSlice for #name #type_generics #where_clause {
                fn try_from_slice(
                    slice: &[u8],
                    pos: &mut usize,
                ) -> grib_template_helpers::TryFromSliceResult<Self> {
                    Ok(Self(#(#field_reads),*))
                }
            }
        };
    }

    let mut field_reads = Vec::new();
    let mut idents = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let num_octets_attr = field.attrs.iter().find_map(|attr| {
            attr_value(attr, "num_octets").and_then(|v| super::helpers::parse_num_octets_attr(&v))
        });
        if let Some(num_octets) = num_octets_attr {
            field_reads.push(quote! {
                let #ident = grib_template_helpers::NonStdLenUint::try_from(
                        <[u8; #num_octets] as grib_template_helpers::TryFromSlice>::try_from_slice(
                            slice,
                            pos,
                        )?,
                    ).map_err(|_| "slice length is too short")?.inner();
            });
            idents.push(ident);
            continue;
        }

        let len_attr = field
            .attrs
            .iter()
            .find_map(|attr| LenKind::try_from(attr).ok());
        if let Some(len) = len_attr {
            if let syn::Type::Path(type_path) = ty
                && let Some((inner_ty, has_option)) = extract_vec_inner(type_path)
            {
                let tokens = quote! {
                    let mut #ident = Vec::with_capacity(#len);
                    for _ in 0..#len {
                        let item =
                            <#inner_ty as grib_template_helpers::TryFromSlice>::try_from_slice(
                                slice,
                                pos,
                            )?;
                        #ident.push(item);
                    }
                };

                let tokens = if has_option {
                    quote! {
                        let #ident = if *pos == slice.len() {
                            None
                        } else {
                            #tokens
                            Some(#ident)
                        };
                    }
                } else {
                    tokens
                };
                field_reads.push(tokens);

                idents.push(ident);
                continue;
            }
            unimplemented!(
                "`#[grib_template(len = N)]` is only available for `Vec<T>` and `Option<Vec<T>>`"
            );
        }

        let disc_attr = field
            .attrs
            .iter()
            .find_map(|attr| attr_value(attr, "variant").and_then(|v| parse_variant_attr(&v)));
        if let Some(disc_ident) = disc_attr {
            field_reads.push(quote! {
                let #ident = <#ty as grib_template_helpers::TryEnumFromSlice>::try_enum_from_slice(
                    #disc_ident,
                    slice,
                    pos,
                )?;
            });
            idents.push(ident);
            continue;
        }

        field_reads.push(quote! {
            let #ident = <#ty as grib_template_helpers::TryFromSlice>::try_from_slice(slice, pos)?;
        });
        idents.push(ident);
    }

    quote! {
        impl #impl_generics grib_template_helpers::TryFromSlice for #name #type_generics #where_clause {
            fn try_from_slice(
                slice: &[u8],
                pos: &mut usize,
            ) -> grib_template_helpers::TryFromSliceResult<Self> {
                #(#field_reads)*
                Ok(Self { #(#idents),* })
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

    let mut arms = Vec::new();

    for variant in &data.variants {
        let variant_ident = &variant.ident;
        let disc_expr = variant
            .discriminant
            .as_ref()
            .expect("`TryFromSlice` requires the enum to have explicit discriminant")
            .1
            .clone();

        if let syn::Fields::Unnamed(fields) = &variant.fields
            && fields.unnamed.len() == 1
        {
            let inner_ty = &fields.unnamed.first().unwrap().ty;
            arms.push(quote! {
                #disc_expr => {
                    let inner = <#inner_ty as grib_template_helpers::TryFromSlice>::try_from_slice(
                        slice,
                        pos
                    )?;
                    Ok(#name::#variant_ident(inner))
                }
            });
        } else {
            unimplemented!("`TryFromSlice` only supports single-field tuple variants");
        }
    }

    quote! {
        impl #impl_generics grib_template_helpers::TryEnumFromSlice for #name #type_generics #where_clause {
            fn try_enum_from_slice(
                discriminant: impl Into<u64>,
                slice: &[u8],
                pos: &mut usize,
            ) -> grib_template_helpers::TryFromSliceResult<Self> {
                let discriminant = discriminant.into();
                match discriminant {
                    #(#arms),*,
                    _ => panic!(
                        "unknown variant for {} (discriminant = {})",
                        stringify!(#name),
                        &discriminant
                    ),
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LenKind {
    Literal(usize),
    Ident(syn::Ident),
}

impl TryFrom<&syn::Attribute> for LenKind {
    type Error = &'static str;

    fn try_from(value: &syn::Attribute) -> Result<Self, Self::Error> {
        attr_value(value, "len")
            .and_then(|v| match v {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(lit_int),
                    ..
                }) => Some(LenKind::Literal(lit_int.base10_parse::<usize>().unwrap())),
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) => Some(LenKind::Ident(syn::Ident::new(
                    &lit_str.value(),
                    lit_str.span(),
                ))),
                _ => None,
            })
            .ok_or(r#"parsing "len" failed"#)
    }
}

impl quote::ToTokens for LenKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            LenKind::Literal(n) => {
                tokens.extend(quote! { #n });
            }
            LenKind::Ident(ident) => {
                tokens.extend(quote! { #ident as usize });
            }
        }
    }
}

pub(crate) fn parse_variant_attr(attr_value: &syn::Expr) -> Option<syn::Ident> {
    match attr_value {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => Some(syn::Ident::new(&lit_str.value(), lit_str.span())),
        _ => None,
    }
}

pub(crate) fn extract_vec_inner(type_path: &syn::TypePath) -> Option<(syn::Type, bool)> {
    if type_path.path.segments.len() == 1 {
        let (type_path, has_option) = if type_path.path.segments[0].ident == "Option"
            && let syn::PathArguments::AngleBracketed(ref args) =
                type_path.path.segments[0].arguments
            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
            && let syn::Type::Path(type_path) = inner_ty
        {
            (type_path, true)
        } else {
            (type_path, false)
        };

        if type_path.path.segments[0].ident == "Vec"
            && let syn::PathArguments::AngleBracketed(ref args) =
                type_path.path.segments[0].arguments
            && let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first()
        {
            return Some((inner_ty.clone(), has_option));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_literal_len_attr() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[grib_template(len = 3)]
        };
        let parsed = LenKind::try_from(&attr);
        assert_eq!(parsed, Ok(LenKind::Literal(3)));
    }

    #[test]
    fn parsing_ident_len_attr() {
        let attr: syn::Attribute = syn::parse_quote! {
            #[grib_template(len = "field1")]
        };
        let parsed = LenKind::try_from(&attr);
        if let Ok(LenKind::Ident(ident)) = parsed
            && ident.to_string() == "field1"
        {
            return;
        }
        panic!(r#"parsing "len" is failure"#);
    }
}

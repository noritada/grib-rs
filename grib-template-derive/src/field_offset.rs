use proc_macro2::TokenStream;
use quote::quote;

fn fields(input: &syn::DeriveInput) -> syn::Result<&syn::Fields> {
    match &input.data {
        syn::Data::Struct(data) => Ok(&data.fields),
        _ => Err(syn::Error::new_spanned(input, "only structs are supported")),
    }
}

// Keep the encoded layout consistent with TryFromSlice, including explicit
// octet counts. Unknown layout attributes must not silently yield an offset.
fn size(field: &syn::Field, generics: &mut syn::Generics) -> syn::Result<TokenStream> {
    let mut num_octets = None;
    for attr in &field.attrs {
        if attr.path().is_ident("grib_template") {
            let meta = attr.parse_args::<syn::Meta>()?;
            match meta {
                syn::Meta::NameValue(nv) if nv.path.is_ident("num_octets") => {
                    let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(n),
                        ..
                    }) = nv.value
                    else {
                        return Err(syn::Error::new_spanned(
                            nv,
                            "expected an integer octet count",
                        ));
                    };
                    if num_octets.replace(n.base10_parse::<usize>()?).is_some() {
                        return Err(syn::Error::new_spanned(attr, "duplicate octet count"));
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "cannot determine a fixed encoded size for this field",
                    ));
                }
            }
        }
    }
    if let Some(n) = num_octets {
        return Ok(quote!(#n));
    }
    let ty = &field.ty;
    generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote!(
            #ty: grib_template_helpers::FixedSize
        ));
    Ok(quote!(<#ty as grib_template_helpers::FixedSize>::SIZE))
}

pub(crate) fn fixed_size(input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let mut generics = input.generics.clone();
    let sizes = fields(input)?
        .iter()
        .map(|f| size(f, &mut generics))
        .collect::<syn::Result<Vec<_>>>()?;
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics grib_template_helpers::FixedSize for #name #type_generics #where_clause {
            const SIZE: usize = 0 #(+ #sizes)*;
        }
    })
}

pub(crate) fn field_offset(input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    if let Some(attr) = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("field_offset"))
    {
        return Err(syn::Error::new_spanned(
            attr,
            "field_offset belongs on a field",
        ));
    }
    let fields = fields(input)?;
    let name = &input.ident;
    let mut output = TokenStream::new();
    for (index, field) in fields.iter().enumerate() {
        for attr in field
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("field_offset"))
        {
            let ty = attr.parse_args::<syn::Type>()?;
            let mut generics = input.generics.clone();
            let sizes = fields
                .iter()
                .take(index)
                .map(|f| size(f, &mut generics))
                .collect::<syn::Result<Vec<_>>>()?;
            let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
            output.extend(quote! {
                impl #impl_generics grib_template_helpers::FieldOffset<#ty> for #name #type_generics #where_clause {
                    const OFFSET: usize = 0 #(+ #sizes)*;
                }
            });
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! rejects_offset {
        ($($name:ident: $input:tt),* $(,)?) => {$(
            #[test]
            fn $name() {
                let input = syn::parse_quote! $input;
                assert!(field_offset(&input).is_err());
            }
        )*};
    }

    rejects_offset! {
        enumeration: { enum Template { A } },
        misplaced_attribute: { #[field_offset(u8)] struct Template { value: u8 } },
        missing_type: { struct Template { #[field_offset] value: u8 } },
        multiple_types: { struct Template { #[field_offset(u8, u16)] value: u8 } },
        variable_length_prefix: {
            struct Template {
                #[grib_template(len = "count")]
                prefix: Vec<u8>,
                #[field_offset(u8)]
                value: u8,
            }
        },
        discriminant_selected_prefix: {
            struct Template {
                #[grib_template(variant = "kind")]
                prefix: Choice,
                #[field_offset(u8)]
                value: u8,
            }
        },
        invalid_octet_count: {
            struct Template {
                #[grib_template(num_octets = "3")]
                prefix: u32,
                #[field_offset(u8)]
                value: u8,
            }
        },
    }
}

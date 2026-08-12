use std::collections::HashMap;

use quote::quote;

pub(crate) fn impl_for_struct(
    input: &syn::DeriveInput,
    data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let Some((kind, fields)) = super::helpers::extract_struct_info(data) else {
        unimplemented!(
            "`Dump` can only be derived for structs with named fields or with a single unnamed `u8` field"
        )
    };

    if kind == super::helpers::StructKind::TupleStruct {
        let doc = get_doc(&fields[0].attrs)
            .map(|s| format!("  // {}", s.trim()))
            .unwrap_or_default();

        return quote! {
            impl #impl_generics grib_template_helpers::Dump for #name #type_generics #where_clause {
                fn dump<W: std::io::Write>(
                    &self,
                    parent: Option<&std::borrow::Cow<str>>,
                    pos: &mut usize,
                    output: &mut W,
                ) -> Result<(), std::io::Error> {
                    let size = 1;
                    grib_template_helpers::write_position_column(output, pos, size)?;
                    if let Some(parent) = parent {
                        write!(output, "{}", parent)?;
                    }
                    writeln!(output, " = {:#010b}{}",
                        self.0,
                        #doc,
                    )
                }
            }
        };
    }

    let mut dumps = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let doc = get_doc(&field.attrs)
            .map(|s| format!("  // {}", s.trim()))
            .unwrap_or_default();

        let num_octets_attr = field
            .attrs
            .iter()
            .find_map(|attr| super::helpers::NumOctets::try_from(attr).ok());
        if let Some(num_octets) = num_octets_attr {
            dumps.push(quote! {
                <grib_template_helpers::NonStdLenUint<#ty> as grib_template_helpers::DumpField>::dump_field(
                    &grib_template_helpers::NonStdLenUint::new(self.#ident, #num_octets),
                    stringify!(#ident),
                    parent,
                    #doc,
                    pos,
                    output,
                )?;
            });
            continue;
        }

        dumps.push(quote! {
            <#ty as grib_template_helpers::DumpField>::dump_field(
                &self.#ident,
                stringify!(#ident),
                parent,
                #doc,
                pos,
                output,
            )?;
        });
    }

    quote! {
        impl #impl_generics grib_template_helpers::Dump for #name #type_generics #where_clause {
            fn dump<W: std::io::Write>(
                &self,
                parent: Option<&std::borrow::Cow<str>>,
                pos: &mut usize,
                output: &mut W,
            ) -> Result<(), std::io::Error> {
                #(#dumps)*;
                Ok(())
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

        if let syn::Fields::Unnamed(fields) = &variant.fields
            && fields.unnamed.len() == 1
        {
            let inner_ty = &fields.unnamed.first().unwrap().ty;
            arms.push(quote! {
                #name::#variant_ident(inner) => <#inner_ty as grib_template_helpers::Dump>::dump(
                    inner,
                    parent,
                    pos,
                    output
                )
            });
        } else {
            unimplemented!("`Dump` only supports single-field tuple variants");
        }
    }

    quote! {
        impl #impl_generics grib_template_helpers::Dump for #name #type_generics #where_clause {
            fn dump<W: std::io::Write>(
                &self,
                parent: Option<&std::borrow::Cow<str>>,
                pos: &mut usize,
                output: &mut W,
            ) -> Result<(), std::io::Error> {
                match self {
                    #(#arms),*,
                }
            }
        }
    }
}

pub(crate) fn get_doc(attrs: &[syn::Attribute]) -> Option<String> {
    let mut doc = String::new();
    for attr in attrs.iter() {
        match attr.meta {
            syn::Meta::NameValue(ref value) if value.path.is_ident("doc") => {
                if let syn::Expr::Lit(lit) = &value.value
                    && let syn::Lit::Str(s) = &lit.lit
                {
                    doc.push_str(&s.value());
                }
            }
            _ => {}
        }
    }
    if doc.is_empty() { None } else { Some(doc) }
}

#[derive(Debug, PartialEq, Eq)]
struct DocOverrides(HashMap<String, String>);

impl TryFrom<&syn::Attribute> for DocOverrides {
    type Error = &'static str;

    fn try_from(value: &syn::Attribute) -> Result<Self, Self::Error> {
        const MESSAGE: &str = "error";
        if !value.path().is_ident("dump") {
            return Err(MESSAGE);
        }
        let meta = value.parse_args::<syn::Meta>().map_err(|_| MESSAGE)?;
        if let syn::Meta::List(list) = meta {
            if !list.path.is_ident("doc") {
                return Err(MESSAGE);
            }
            let items = list
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                )
                .map_err(|_| MESSAGE)?;

            let mut map = HashMap::new();
            for item in items {
                if let syn::Meta::NameValue(nv) = item {
                    let k = nv
                        .path
                        .get_ident()
                        .map(|i| i.to_string())
                        .unwrap_or_default();
                    let v = if let syn::Expr::Lit(lit) = &nv.value
                        && let syn::Lit::Str(s) = &lit.lit
                    {
                        s.value()
                    } else {
                        return Err(MESSAGE);
                    };
                    map.insert(k, v);
                } else {
                    return Err(MESSAGE);
                }
            }
            Ok(Self(map))
        } else {
            Err(MESSAGE)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_doc_attr() -> Result<(), Box<dyn std::error::Error>> {
        let attr: syn::Attribute = syn::parse_quote! {
            #[dump(doc(
                key1 = "val1",
                key2 = "val2",
            ))]
        };
        let actual = DocOverrides::try_from(&attr)?;
        let expected = vec![("key1", "val1"), ("key2", "val2")]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<HashMap<_, _>>();
        let expected = DocOverrides(expected);
        assert_eq!(actual, expected);

        Ok(())
    }
}

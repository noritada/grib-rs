use quote::{ToTokens, quote};

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
                fn dump<'d, W: std::io::Write>(
                    &self,
                    parent: Option<&std::borrow::Cow<str>>,
                    doc_overrides: Option<grib_template_helpers::DocOverrides<'d>>,
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

        let doc = if let Some(s) = get_doc(&field.attrs) {
            let s = s.trim();
            quote! { Some(#s) }
        } else {
            quote! { None }
        };

        dumps.push(quote! {
            let name = stringify!(#ident);
            let doc = if let Some(doc_overrides) = &doc_overrides
                && let Some(val) = doc_overrides.get(name)
            {
                Some(*val)
            } else {
                #doc
            };
        });

        let doc_overrides = if let Ok(inner) = DocOverrides::try_from(field) {
            quote! { Some(grib_template_helpers::DocOverrides::new(#inner)) }
        } else {
            quote! { None }
        };

        let (ty, self_ident) = if let Ok(num_octets) = super::helpers::NumOctets::try_from(field) {
            (
                quote! { grib_template_helpers::NonStdLenUint<#ty> },
                quote! {
                    &grib_template_helpers::NonStdLenUint::new(self.#ident, #num_octets)
                },
            )
        } else {
            (ty.to_token_stream(), quote! { &self.#ident })
        };

        dumps.push(quote! {
            <#ty as grib_template_helpers::DumpField>::dump_field(
                #self_ident,
                name,
                parent,
                doc,
                #doc_overrides,
                pos,
                output,
            )?;
        });
    }

    quote! {
        impl #impl_generics grib_template_helpers::Dump for #name #type_generics #where_clause {
            fn dump<'d, W: std::io::Write>(
                &self,
                parent: Option<&std::borrow::Cow<str>>,
                doc_overrides: Option<grib_template_helpers::DocOverrides<'d>>,
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
                    None,
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
            fn dump<'d, W: std::io::Write>(
                &self,
                parent: Option<&std::borrow::Cow<str>>,
                doc_overrides: Option<grib_template_helpers::DocOverrides<'d>>,
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
struct DocOverrides(Vec<(Vec<String>, String)>);

impl TryFrom<&syn::Field> for DocOverrides {
    type Error = &'static str;

    fn try_from(value: &syn::Field) -> Result<Self, Self::Error> {
        value
            .attrs
            .iter()
            .find_map(|attr| DocOverrides::try_from(attr).ok())
            .ok_or("error")
    }
}

impl TryFrom<&syn::Attribute> for DocOverrides {
    type Error = &'static str;

    fn try_from(value: &syn::Attribute) -> Result<Self, Self::Error> {
        const MESSAGE: &str = "error";
        if !value.path().is_ident("dump") {
            return Err(MESSAGE);
        }
        let meta = value.parse_args::<syn::Meta>().map_err(|_| MESSAGE)?;
        if let syn::Meta::List(list) = meta {
            let mut map = Vec::with_capacity(16);
            parse_tree(list, &Vec::new(), &mut map)?;
            if let Some((k, _v)) = map.first()
                && k[0] != "doc"
            {
                return Err(MESSAGE);
            }
            Ok(Self(map))
        } else {
            Err(MESSAGE)
        }
    }
}

impl ToTokens for DocOverrides {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self(inner) = self;
        let iter = inner
            .iter()
            .map(|(k, v)| {
                // the 0th element should be "doc"
                let k = &k[1..].join(".");
                quote! { (#k, #v) }
            })
            .collect::<Vec<_>>();
        let tok = quote! {
            vec![#(#iter),*]
        };
        tokens.extend(tok);
    }
}

fn parse_tree(
    list: syn::MetaList,
    path: &Vec<String>,
    out: &mut Vec<(Vec<String>, String)>,
) -> Result<(), &'static str> {
    const MESSAGE: &str = "error";

    let mut path = path.clone();
    let name = list
        .path
        .get_ident()
        .map(|i| i.to_string())
        .unwrap_or_default();
    path.push(name);

    let items = list
        .parse_args_with(syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
        .map_err(|_| MESSAGE)?;

    for item in items {
        match item {
            syn::Meta::NameValue(nv) => {
                let mut k = path.clone();
                let name = nv
                    .path
                    .get_ident()
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                k.push(name);

                let v = if let syn::Expr::Lit(lit) = &nv.value
                    && let syn::Lit::Str(s) = &lit.lit
                {
                    s.value()
                } else {
                    return Err(MESSAGE);
                };
                out.push((k, v));
            }
            syn::Meta::List(sublist) => parse_tree(sublist, &path, out)?,
            _ => {
                return Err(MESSAGE);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_doc_attr() -> Result<(), Box<dyn std::error::Error>> {
        let attr: syn::Attribute = syn::parse_quote! {
            #[dump(doc(
                key1 = "val1",
                key2(key1 = "val21"),
                key3 = "val3",
            ))]
        };
        let actual = DocOverrides::try_from(&attr)?;
        let expected = vec![
            (vec!["doc", "key1"], "val1"),
            (vec!["doc", "key2", "key1"], "val21"),
            (vec!["doc", "key3"], "val3"),
        ]
        .into_iter()
        .map(|(k, v)| {
            (
                k.into_iter().map(|s| s.to_owned()).collect::<Vec<_>>(),
                v.to_owned(),
            )
        })
        .collect::<Vec<_>>();
        let expected = DocOverrides(expected);
        assert_eq!(actual, expected);

        let actual_stream = actual.to_token_stream().to_string();
        let expected_stream =
            r#"vec ! [("key1" , "val1") , ("key2.key1" , "val21") , ("key3" , "val3")]"#;
        assert_eq!(actual_stream, expected_stream);

        Ok(())
    }
}

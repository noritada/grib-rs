use quote::{ToTokens, quote};

pub(crate) fn impl_for_struct(
    input: &syn::DeriveInput,
    data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let options = match dump_options(input) {
        Ok(options) => options,
        Err(error) => return error.to_compile_error(),
    };
    let name = &input.ident;
    let (marker_generics, marker_type_generics, marker_where_clause) =
        input.generics.split_for_impl();
    let marker_impl = quote! {
        impl #marker_generics grib_template_helpers::Dumpable for #name #marker_type_generics #marker_where_clause {}
    };
    let mut generics = input.generics.clone();
    let context = if let Some(context) = &options.context {
        context.to_token_stream()
    } else {
        let context = context_ident(&input.generics);
        generics.params.push(syn::parse_quote!(#context: ?Sized));
        context.to_token_stream()
    };
    let (_, type_generics, _) = input.generics.split_for_impl();
    let Some((kind, fields)) = super::helpers::extract_struct_info(data) else {
        unimplemented!(
            "`Dump` can only be derived for structs with named fields or with a single unnamed `u8` or `u16` field"
        )
    };

    if kind == super::helpers::StructKind::TupleStruct {
        if let Err(error) = reject_field_context(&fields[0]) {
            return error.to_compile_error();
        }
        let doc = get_doc(&fields[0].attrs);
        let doc = match doc {
            Some(doc) => {
                let doc = doc.trim();
                quote! { Some(#doc) }
            }
            None => quote! { None },
        };

        if options.code_table {
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(Self: ::grib_template_helpers::CodeTable<#context>));
        }
        let format = if options.flag { " = {:#010b}" } else { " = {}" };
        let description = if options.code_table {
            quote! {
                if let Some(description) = <Self as ::grib_template_helpers::CodeTable<#context>>::code_description(self, context) {
                    write!(output, " [{}]", description)?;
                }
            }
        } else {
            quote! {}
        };

        let (impl_generics, _, where_clause) = generics.split_for_impl();
        return quote! {
            #marker_impl
            impl #impl_generics grib_template_helpers::Dump<#context> for #name #type_generics #where_clause {
                fn dump<'d, W: std::io::Write>(
                    &self,
                    context: &#context,
                    parent: Option<&std::borrow::Cow<str>>,
                    doc_overrides: grib_template_helpers::DocOverrides<'d>,
                    pos: &mut usize,
                    output: &mut W,
                ) -> Result<(), std::io::Error> {
                    let size = std::mem::size_of_val(&self.0);
                    grib_template_helpers::write_position_column(output, pos, size)?;
                    if let Some(parent) = parent {
                        write!(output, "{}", parent)?;
                    }
                    let doc = doc_overrides.get("").or(#doc)
                        .map(|s| format!("  // {}", s))
                        .unwrap_or_default();
                    write!(output, #format, self.0)?;
                    #description
                    writeln!(output, "{}", doc)
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
            let doc = doc_overrides.get(name).or(#doc);
        });

        let (doc_overrides, child_context) = match field_options(field) {
            Ok(options) => options,
            Err(error) => return error.to_compile_error(),
        };
        let doc_overrides = quote! { grib_template_helpers::DocOverrides::new(#doc_overrides) };

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

        let (field_context_type, field_context) = if let Some(expr) = child_context {
            (quote!(_), quote!(&(#expr)))
        } else {
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#ty: grib_template_helpers::DumpField<#context>));
            (context.clone(), quote!(context))
        };
        dumps.push(quote! {
            let mut child_doc_overrides = doc_overrides.get_child_overrides(name);
            child_doc_overrides.merge(&#doc_overrides);

            <#ty as grib_template_helpers::DumpField<#field_context_type>>::dump_field(
                #self_ident,
                #field_context,
                name,
                parent,
                doc,
                child_doc_overrides,
                pos,
                output,
            )?;
        });
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();
    quote! {
        #marker_impl
        impl #impl_generics grib_template_helpers::Dump<#context> for #name #type_generics #where_clause {
            fn dump<'d, W: std::io::Write>(
                &self,
                context: &#context,
                parent: Option<&std::borrow::Cow<str>>,
                doc_overrides: grib_template_helpers::DocOverrides<'d>,
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
    let options = match dump_options(input) {
        Ok(options) => options,
        Err(error) => return error.to_compile_error(),
    };
    let name = &input.ident;
    let (marker_generics, marker_type_generics, marker_where_clause) =
        input.generics.split_for_impl();
    let marker_impl = quote! {
        impl #marker_generics grib_template_helpers::Dumpable for #name #marker_type_generics #marker_where_clause {}
    };
    let mut generics = input.generics.clone();
    let context = if let Some(context) = &options.context {
        context.to_token_stream()
    } else {
        let context = context_ident(&input.generics);
        generics.params.push(syn::parse_quote!(#context: ?Sized));
        context.to_token_stream()
    };
    let (_, type_generics, _) = input.generics.split_for_impl();

    let mut arms = Vec::new();

    for variant in &data.variants {
        let variant_ident = &variant.ident;

        if let syn::Fields::Unnamed(fields) = &variant.fields
            && fields.unnamed.len() == 1
        {
            let field = fields.unnamed.first().unwrap();
            if let Err(error) = reject_field_context(field) {
                return error.to_compile_error();
            }
            let inner_ty = &field.ty;
            generics
                .make_where_clause()
                .predicates
                .push(syn::parse_quote!(#inner_ty: grib_template_helpers::Dump<#context>));
            arms.push(quote! {
                #name::#variant_ident(inner) => <#inner_ty as grib_template_helpers::Dump<#context>>::dump(
                    inner,
                    context,
                    parent,
                    grib_template_helpers::DocOverrides::empty(),
                    pos,
                    output
                )
            });
        } else {
            unimplemented!("`Dump` only supports single-field tuple variants");
        }
    }

    let (impl_generics, _, where_clause) = generics.split_for_impl();
    quote! {
        #marker_impl
        impl #impl_generics grib_template_helpers::Dump<#context> for #name #type_generics #where_clause {
            fn dump<'d, W: std::io::Write>(
                &self,
                context: &#context,
                parent: Option<&std::borrow::Cow<str>>,
                doc_overrides: grib_template_helpers::DocOverrides<'d>,
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

#[derive(Default)]
struct DumpOptions {
    flag: bool,
    code_table: bool,
    context: Option<syn::Type>,
}

fn dump_options(input: &syn::DeriveInput) -> syn::Result<DumpOptions> {
    let mut options = DumpOptions::default();
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("dump"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("context") {
                if options.context.is_some() {
                    return Err(meta.error("duplicate `context` option"));
                }
                options.context = Some(meta.value()?.parse()?);
                return Ok(());
            }
            let (value, name) = if meta.path.is_ident("flag") {
                (&mut options.flag, "flag")
            } else if meta.path.is_ident("code_table") {
                (&mut options.code_table, "code_table")
            } else {
                return Err(meta.error("expected `flag`, `code_table`, or `context`"));
            };
            if *value {
                return Err(meta.error(format!("duplicate `{name}` option")));
            }
            *value = true;
            Ok(())
        })?;
    }
    if options.flag {
        let supported = matches!(
            &input.data,
            syn::Data::Struct(data)
                if matches!(super::helpers::extract_struct_info(data),
                    Some((super::helpers::StructKind::TupleStruct, fields))
                        if super::helpers::is_type_u8(&fields[0].ty))
        );
        if !supported {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "`#[dump(flag)]` requires a single-field `u8` tuple struct",
            ));
        }
    }
    if options.code_table {
        let supported = matches!(
            &input.data,
            syn::Data::Struct(data)
                if matches!(super::helpers::extract_struct_info(data),
                    Some((super::helpers::StructKind::TupleStruct, _)))
        );
        if !supported {
            return Err(syn::Error::new_spanned(
                &input.ident,
                "`#[dump(code_table)]` requires a single-field `u8` or `u16` tuple struct",
            ));
        }
    }
    Ok(options)
}

fn field_options(field: &syn::Field) -> syn::Result<(DocOverrides, Option<syn::Expr>)> {
    let mut docs = Vec::new();
    let mut context = None;
    for attr in field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("dump"))
    {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("context") {
                if context.is_some() {
                    return Err(meta.error("duplicate `context` option"));
                }
                context = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("doc") {
                let content;
                syn::parenthesized!(content in meta.input);
                let tokens: proc_macro2::TokenStream = content.parse()?;
                let list = syn::parse_quote!(doc(#tokens));
                parse_tree(list, &[], &mut docs).map_err(|_| meta.error("invalid `doc` option"))?;
            } else {
                return Err(meta.error("expected `doc` or `context`"));
            }
            Ok(())
        })?;
    }
    Ok((DocOverrides(docs), context))
}

fn reject_field_context(field: &syn::Field) -> syn::Result<()> {
    if let Some(expr) = field_options(field)?.1 {
        return Err(syn::Error::new_spanned(
            expr,
            "field context expressions are only supported on named struct fields",
        ));
    }
    Ok(())
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
    path: &[String],
    out: &mut Vec<(Vec<String>, String)>,
) -> Result<(), &'static str> {
    const MESSAGE: &str = "error";

    let mut path = path.to_owned();
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

// Avoid collisions with type and const parameters supplied by the caller.
pub(crate) fn context_ident(generics: &syn::Generics) -> syn::Ident {
    let mut name = String::from("__DumpContext");
    while generics.params.iter().any(|param| match param {
        syn::GenericParam::Type(param) => param.ident == name,
        syn::GenericParam::Const(param) => param.ident == name,
        syn::GenericParam::Lifetime(_) => false,
    }) {
        name.push('_');
    }
    syn::Ident::new(&name, proc_macro2::Span::call_site())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_flag_option() -> syn::Result<()> {
        let input = syn::parse_quote! {
            #[dump(flag,)]
            struct Flags(u8);
        };
        assert!(dump_options(&input)?.flag);
        let input = syn::parse_quote! { struct Value(u8); };
        assert!(!dump_options(&input)?.flag);
        Ok(())
    }

    #[test]
    fn invalid_flag_options() {
        let inputs: [syn::DeriveInput; 6] = [
            syn::parse_quote! { #[dump(flag)] struct Named { value: u8 } },
            syn::parse_quote! { #[dump(flag)] struct Wide(u16); },
            syn::parse_quote! { #[dump(flag)] enum Value { Byte(u8) } },
            syn::parse_quote! { #[dump(flag, flag)] struct Flags(u8); },
            syn::parse_quote! { #[dump(flag = true)] struct Flags(u8); },
            syn::parse_quote! { #[dump(flga)] struct Flags(u8); },
        ];
        for input in inputs {
            assert!(dump_options(&input).is_err());
        }
    }

    macro_rules! test_invalid_code_table_option {
        ($($name:ident: $input:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() -> syn::Result<()> {
                    let input = syn::parse_str($input)?;
                    assert!(dump_options(&input).is_err());
                    Ok(())
                }
            )*
        };
    }

    test_invalid_code_table_option! {
        code_table_named: "#[dump(code_table)] struct T { value: u8 }",
        code_table_wide: "#[dump(code_table)] struct T(u32);",
        code_table_enum: "#[dump(code_table)] enum T { Value(u8) }",
        code_table_duplicate: "#[dump(code_table, code_table)] struct T(u8);",
        code_table_value: "#[dump(code_table = true)] struct T(u8);",
    }

    test_invalid_code_table_option! {
        context_duplicate: "#[dump(context = A, context = B)] struct T { value: u8 }",
        context_missing_type: "#[dump(context)] struct T { value: u8 }",
        context_expression_as_type: "#[dump(context = context.child())] struct T { value: u8 }",
    }

    #[test]
    fn invalid_field_context_options() -> syn::Result<()> {
        for source in [
            "#[dump(context)] value: u8",
            "#[dump(context = a, context = b)] value: u8",
            "#[dump(context = a)] #[dump(context = b)] value: u8",
            "#[dump(context = )] value: u8",
        ] {
            let input: syn::DeriveInput = syn::parse_str(&format!("struct T {{ {source} }}"))?;
            let syn::Data::Struct(data) = input.data else {
                unreachable!()
            };
            assert!(field_options(data.fields.iter().next().unwrap()).is_err());
        }
        Ok(())
    }

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

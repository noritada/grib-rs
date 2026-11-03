mod wmo_cct;
mod wmo_grib2_tables;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

pub(crate) fn expand(input: &syn::DeriveInput) -> syn::Result<TokenStream> {
    let max_code = validate_input(input)?;
    let options = Options::parse(input)?;
    if !matches!(
        options.format.name.to_string().as_str(),
        "wmo_grib2_tables" | "wmo_cct_c00" | "wmo_cct_c11"
    ) {
        return Err(syn::Error::new_spanned(
            &options.format.name,
            "unsupported format; expected wmo_grib2_tables, wmo_cct_c00, or wmo_cct_c11",
        ));
    }
    if !options.format.options.is_empty() {
        return Err(syn::Error::new_spanned(
            &options.format.options,
            format!("{} does not support options", options.format.name),
        ));
    }
    if options.format.name != "wmo_grib2_tables" && options.key.is_some() {
        return Err(syn::Error::new_spanned(
            &options.format.name,
            "common code tables do not support context and key options",
        ));
    }
    let (path, contents) = read_table(&options.file)?;
    let entries = match options.format.name.to_string().as_str() {
        "wmo_cct_c00" => wmo_cct::parse_c00(&contents, max_code),
        "wmo_cct_c11" => wmo_cct::parse_c11(&contents, max_code),
        _ => wmo_grib2_tables::parse(&contents, max_code),
    }
    .map_err(|error| syn::Error::new_spanned(&options.file, error))?;
    let keys = if options.key.is_some() {
        Some(
            wmo_grib2_tables::parse_keys(&contents)
                .map_err(|error| syn::Error::new_spanned(&options.file, error))?,
        )
    } else {
        None
    };
    let description = lookup_body(&entries, keys.as_deref(), options.key.as_ref(), |entry| {
        entry.description.as_deref()
    });
    let unit = lookup_body(&entries, keys.as_deref(), options.key.as_ref(), |entry| {
        entry.unit.as_deref()
    });
    let name = &input.ident;
    let mut generics = input.generics.clone();
    let context = if let Some(context) = &options.context {
        context.to_token_stream()
    } else {
        let context = crate::dump::context_ident(&input.generics);
        generics.params.push(syn::parse_quote!(#context: ?Sized));
        context.to_token_stream()
    };
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();
    Ok(quote! {
        const _: &str = include_str!(#path);

        impl #impl_generics ::grib_template_helpers::CodeTable<#context> for #name #ty_generics #where_clause {
            fn code_description(&self, context: &#context) -> Option<&'static str> {
                #description
            }

            fn code_unit(&self, context: &#context) -> Option<&'static str> {
                #unit
            }
        }
    })
}

fn lookup_body(
    entries: &[wmo_grib2_tables::Entry],
    keys: Option<&[Vec<u8>]>,
    key: Option<&syn::Expr>,
    value: impl Fn(&wmo_grib2_tables::Entry) -> Option<&str>,
) -> TokenStream {
    if entries.iter().all(|entry| value(entry).is_none()) {
        return quote!(::core::option::Option::None);
    }
    let arms = entries.iter().enumerate().map(|(index, entry)| {
        let start = proc_macro2::Literal::u16_unsuffixed(*entry.codes.start());
        let end = proc_macro2::Literal::u16_unsuffixed(*entry.codes.end());
        let result = match value(entry) {
            Some(text) => quote!(::core::option::Option::Some(#text)),
            None => quote!(::core::option::Option::None),
        };
        if let Some(keys) = keys {
            let keys = keys[index]
                .iter()
                .map(|key| proc_macro2::Literal::u8_unsuffixed(*key));
            quote!(( (#(#keys,)*), #start..=#end) => #result,)
        } else {
            quote!(#start..=#end => #result,)
        }
    });
    let target = key.map_or_else(|| quote!(self.0), |key| quote!((#key, self.0)));
    quote! {
        // The fallback also supports tables that do not cover every code.
        #[allow(unreachable_patterns)]
        match #target {
            #(#arms)*
            _ => ::core::option::Option::None,
        }
    }
}

struct Options {
    file: syn::LitStr,
    format: Format,
    context: Option<syn::Type>,
    key: Option<syn::Expr>,
}

struct Format {
    name: syn::Ident,
    options: syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>,
}

impl syn::parse::Parse for Format {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let options = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            content.parse_terminated(syn::Meta::parse, syn::Token![,])?
        } else {
            syn::punctuated::Punctuated::new()
        };
        Ok(Self { name, options })
    }
}

impl Options {
    fn parse(input: &syn::DeriveInput) -> syn::Result<Self> {
        let mut attributes = input
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("code_table"));
        let attr = attributes.next().ok_or_else(|| {
            syn::Error::new_spanned(
                input,
                "expected #[code_table(file = \"...\", format = ...)]",
            )
        })?;
        if let Some(attr) = attributes.next() {
            return Err(syn::Error::new_spanned(
                attr,
                "duplicate code_table attribute",
            ));
        }
        let mut file = None;
        let mut format = None;
        let mut context = None;
        let mut key = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("file") {
                if file.is_some() {
                    return Err(meta.error("duplicate file option"));
                }
                file = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("format") {
                if format.is_some() {
                    return Err(meta.error("duplicate format option"));
                }
                format = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("context") {
                if context.is_some() {
                    return Err(meta.error("duplicate context option"));
                }
                context = Some(meta.value()?.parse()?);
            } else if meta.path.is_ident("key") {
                if key.is_some() {
                    return Err(meta.error("duplicate key option"));
                }
                key = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error("expected file, format, context, or key"));
            }
            Ok(())
        })?;
        if context.is_some() != key.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "context and key must be specified together",
            ));
        }
        Ok(Self {
            context,
            key,
            file: file.ok_or_else(|| syn::Error::new_spanned(attr, "missing file option"))?,
            format: format.ok_or_else(|| syn::Error::new_spanned(attr, "missing format option"))?,
        })
    }
}

fn validate_input(input: &syn::DeriveInput) -> syn::Result<u16> {
    if let syn::Data::Struct(data) = &input.data
        && let syn::Fields::Unnamed(fields) = &data.fields
        && fields.unnamed.len() == 1
        && let syn::Type::Path(ty) = &fields.unnamed[0].ty
        && ty.qself.is_none()
        && (ty.path.is_ident("u8") || ty.path.is_ident("u16"))
    {
        return Ok(if ty.path.is_ident("u8") {
            u8::MAX.into()
        } else {
            u16::MAX
        });
    }
    Err(syn::Error::new_spanned(
        input,
        "CodeTable requires a tuple struct with a single u8 or u16 field",
    ))
}

fn read_table(file: &syn::LitStr) -> syn::Result<(syn::LitStr, String)> {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .ok_or_else(|| syn::Error::new_spanned(file, "CARGO_MANIFEST_DIR is not set"))?;
    let path = std::path::PathBuf::from(manifest_dir).join(file.value());
    let contents = std::fs::read_to_string(&path).map_err(|error| {
        syn::Error::new_spanned(file, format!("failed to read {}: {error}", path.display()))
    })?;
    let path = path
        .to_str()
        .ok_or_else(|| syn::Error::new_spanned(file, "code table path is not valid UTF-8"))?;
    Ok((syn::LitStr::new(path, file.span()), contents))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_context_options() -> syn::Result<()> {
        for options in [
            "context = Context",
            "key = (context.discipline,)",
            "context = Context, context = Context, key = (0,)",
            "context = Context, key = (0,), key = (1,)",
        ] {
            let input = syn::parse_str(&format!(
                "#[code_table(file = \"unused.csv\", format = wmo_grib2_tables, {options})] struct Table(u8);"
            ))?;
            assert!(Options::parse(&input).is_err());
        }
        Ok(())
    }

    #[test]
    fn common_tables_reject_context_keys() -> syn::Result<()> {
        for format in ["wmo_cct_c00", "wmo_cct_c11"] {
            let input = syn::parse_str(&format!(
                "#[code_table(file = \"unused.csv\", format = {format}, context = Context, key = (0,))] struct Table(u16);"
            ))?;
            let error = expand(&input).expect_err("common table cannot have external keys");
            assert!(error.to_string().contains("do not support context and key"));
        }
        Ok(())
    }

    #[test]
    fn missing_table() -> Result<(), Box<dyn std::error::Error>> {
        let input = syn::parse_str(
            r#"#[code_table(file = "tests/tables/nonexistent.csv", format = wmo_grib2_tables)] struct Table(u8);"#,
        )?;
        let error = expand(&input).expect_err("missing table should fail");
        assert!(error.to_string().contains("failed to read"));
        Ok(())
    }

    #[test]
    fn tracks_manifest_relative_file() -> Result<(), Box<dyn std::error::Error>> {
        let input = syn::parse_str(
            r#"#[code_table(file = "tests/tables/code_table.csv", format = wmo_grib2_tables)] struct Table(u8);"#,
        )?;
        let output = expand(&input)?.to_string();
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/tables/code_table.csv");
        let path = syn::LitStr::new(
            path.to_str().ok_or("invalid path")?,
            proc_macro2::Span::call_site(),
        );
        assert!(
            output.contains(
                &quote!(
                    const _: &str = include_str!(#path);
                )
                .to_string()
            )
        );
        Ok(())
    }

    #[test]
    fn unitless_table_has_no_runtime_lookup() -> Result<(), Box<dyn std::error::Error>> {
        let entries = wmo_grib2_tables::parse(
            include_str!("../tests/tables/code_table_unitless.csv"),
            u16::MAX,
        )?;
        let body = lookup_body(&entries, None, None, |entry| entry.unit.as_deref());
        assert_eq!(
            body.to_string(),
            quote!(::core::option::Option::None).to_string()
        );
        Ok(())
    }

    macro_rules! test_invalid_table {
        ($($name:ident: $format:expr => $message:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() -> Result<(), Box<dyn std::error::Error>> {
                    let input = syn::parse_str(&format!(
                        "#[code_table(file = \"tests/tables/code_table.csv\", format = {})] struct Table(u8);",
                        $format,
                    ))?;
                    let error = expand(&input).expect_err("unsupported format should fail");
                    assert!(error.to_string().contains($message));
                    Ok(())
                }
            )*
        };
    }

    test_invalid_table! {
        unsupported_format: "csv" => "unsupported format",
        unsupported_c00_option: "wmo_cct_c00(extra)" => "does not support options",
        unsupported_c11_option: "wmo_cct_c11(extra)" => "does not support options",
        c00_wrong_header: "wmo_cct_c00" => "expected WMO CCT CSV header",
        c11_wrong_header: "wmo_cct_c11" => "expected WMO CCT CSV header",
        unsupported_wmo_option: r#"wmo_grib2_tables(delimiter = ";")"# => "does not support options",
    }

    macro_rules! test_format {
        ($($name:ident: $format:expr => $options:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() -> Result<(), Box<dyn std::error::Error>> {
                    let input = syn::parse_str(&format!(
                        "#[code_table(file = \"a\", format = {},)] struct Table(u8);", $format,
                    ))?;
                    let format = Options::parse(&input)?.format;
                    assert_eq!(format.name, "csv");
                    let options = format.options;
                    let expected: TokenStream = $options.parse()?;
                    assert_eq!(quote!(#options).to_string(), expected.to_string());
                    Ok(())
                }
            )*
        };
    }

    test_format! {
        bare_format: "csv" => "",
        empty_format_options: "csv()" => "",
        format_with_option: r#"csv(delimiter = ";")"# => r#"delimiter = ";""#,
        format_with_trailing_comma: r#"csv(delimiter = ";",)"# => r#"delimiter = ";","#,
        format_with_nested_options: r#"csv(reader(header = true), delimiter = ";")"#
            => r#"reader(header = true), delimiter = ";""#,
    }

    macro_rules! test_invalid_input {
        ($($name:ident: $input:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() -> Result<(), Box<dyn std::error::Error>> {
                    let input = syn::parse_str($input)?;
                    assert!(validate_input(&input).is_err());
                    Ok(())
                }
            )*
        };
    }

    test_invalid_input! {
        named_struct: "struct Table { value: u8 }",
        unit_struct: "struct Table;",
        empty_tuple: "struct Table();",
        multiple_fields: "struct Table(u8, u16);",
        unsupported_type: "struct Table(u32);",
        enumeration: "enum Table { Value }",
        union: "union Table { value: u8 }",
    }

    macro_rules! test_invalid_options {
        ($($name:ident: $input:expr),* $(,)?) => {
            $(
                #[test]
                fn $name() -> Result<(), Box<dyn std::error::Error>> {
                    let input = syn::parse_str($input)?;
                    assert!(Options::parse(&input).is_err());
                    Ok(())
                }
            )*
        };
    }

    test_invalid_options! {
        string_format: r#"#[code_table(file = "a", format = "csv")] struct Table(u8);"#,
        numeric_format: r#"#[code_table(file = "a", format = 1)] struct Table(u8);"#,
        path_format: r#"#[code_table(file = "a", format = formats::csv)] struct Table(u8);"#,
        expression_format: r#"#[code_table(file = "a", format = csv + other)] struct Table(u8);"#,
        invalid_format_options: r#"#[code_table(file = "a", format = csv(";"))] struct Table(u8);"#,
        missing_attribute: "struct Table(u8);",
        missing_file: "#[code_table(format = csv)] struct Table(u8);",
        missing_format: "#[code_table(file = \"table.csv\")] struct Table(u8);",
        non_string_file: "#[code_table(file = 1, format = csv)] struct Table(u8);",
        duplicate_file: "#[code_table(file = \"a\", file = \"b\", format = csv)] struct Table(u8);",
        duplicate_format: "#[code_table(file = \"a\", format = csv, format = csv)] struct Table(u8);",
        unknown_option: "#[code_table(file = \"a\", format = csv, other = 1)] struct Table(u8);",
        duplicate_attribute: "#[code_table(file = \"a\", format = csv)] #[code_table(file = \"b\", format = csv)] struct Table(u8);",
    }
}

mod param_codes;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, DeriveInput, Lit, Result, Token,
};

#[proc_macro_attribute]
pub fn parameter_codes(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as ParameterCodesArgs);
    let (table_path, span) = &attr_args.path;
    let table_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(table_path);
    let (entries, remapper) = if let Ok(entries) = param_codes::Wgrib2Table::from_file(table_path) {
        entries.enum_variants()
    } else {
        return syn::Error::new(*span, "wrong input file")
            .into_compile_error()
            .into();
    };

    let input = parse_macro_input!(input as DeriveInput);
    if !is_empty_enum(&input) {
        return syn::Error::new(input.ident.span(), "not an empty enum")
            .into_compile_error()
            .into();
    }
    let vis = input.vis;
    let ident = input.ident;

    quote! {
        use std::cell::LazyCell;
        use std::collections::HashMap;

        #vis enum #ident {
            #entries
        }

        impl #ident {
            const REMAPPER: LazyCell<HashMap<u32, u32>> =
                LazyCell::new(|| HashMap::from([#remapper]));

            #vis fn remap(code: &u32) -> Option<u32> {
                Self::REMAPPER.get(code).copied()
            }
        }
    }
    .into()
}

fn is_empty_enum(input: &DeriveInput) -> bool {
    matches!(&input.data, syn::Data::Enum(enum_) if enum_.variants.is_empty())
}

#[derive(Debug)]
struct ParameterCodesArgs {
    path: (String, proc_macro2::Span),
}

impl Parse for ParameterCodesArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.parse::<syn::Ident>()? != "path" {
            return Err(input.error("'path' argument not found"));
        }
        let _: Token![=] = input.parse()?;
        let span = input.span();
        match input.parse::<Lit>()? {
            Lit::Str(s) => Ok(Self {
                path: (s.value(), span),
            }),
            _ => Err(input.error("non-`str` 'path' value")),
        }
    }
}

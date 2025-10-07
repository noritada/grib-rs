mod param_codes;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemEnum, Lit, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

#[proc_macro_attribute]
pub fn parameter_codes(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as ParameterCodesArgs);
    let (table_path, span) = &attr_args.path;
    let table_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(table_path);
    let (entries, mapper) = if let Ok(entries) = param_codes::Wgrib2Table::from_file(table_path) {
        entries.enum_variants()
    } else {
        return syn::Error::new(*span, "wrong input file")
            .into_compile_error()
            .into();
    };

    let input = parse_macro_input!(input as ItemEnum);
    if !input.variants.is_empty() {
        return syn::Error::new(input.ident.span(), "not an empty enum")
            .into_compile_error()
            .into();
    }
    let attrs = input.attrs;
    let vis = input.vis;
    let ident = input.ident;

    quote! {
        #(#attrs)*
        #vis enum #ident {
            #entries
        }

        impl TryFrom<u32> for #ident {
            type Error = &'static str;

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    #mapper,
                    _ => Err("code not found")
                }
            }
        }
    }
    .into()
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

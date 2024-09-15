mod wgrib2;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, DeriveInput, Lit, Result, Token,
};

#[proc_macro_attribute]
pub fn parameter_codes(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ParameterCodesArgs);
    let table_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(&args.path);
    let entries = wgrib2::Wgrib2Table::from_file(table_path).unwrap();
    let entries = entries.enum_variants();

    let input = parse_macro_input!(input as DeriveInput);
    if !is_empty_enum(&input) {
        return syn::Error::new(input.ident.span(), "not an empty enum")
            .into_compile_error()
            .into();
    }
    let vis = input.vis;
    let ident = input.ident;

    quote! {
        #vis enum #ident {
            #entries
        }
    }
    .into()
}

fn is_empty_enum(input: &DeriveInput) -> bool {
    matches!(&input.data, syn::Data::Enum(enum_) if enum_.variants.is_empty())
}

#[derive(Debug)]
struct ParameterCodesArgs {
    path: String,
}

impl Parse for ParameterCodesArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.parse::<syn::Ident>()? != "path" {
            return Err(input.error("'path' argument not found"));
        }
        let _: Token![=] = input.parse()?;
        match input.parse::<Lit>()? {
            Lit::Str(s) => Ok(Self { path: s.value() }),
            _ => Err(input.error("non-`str` 'path' value")),
        }
    }
}

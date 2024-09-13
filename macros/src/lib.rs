mod wgrib2;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn parameter_codes(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let vis = input.vis;
    let ident = input.ident;

    let entries = "\
0:1:0:255:0:0:0:0:TMP:Temperature:K
0:1:0:255:0:0:0:1:VTMP:Virtual Temperature:K
0:1:0:255:0:0:3:5:HGT:Geopotential Height:gpm
";
    let entries = entries.parse::<wgrib2::Wgrib2Table>().unwrap();
    let entries = entries.enum_variants();

    quote! {
        #vis enum #ident {
            #entries
        }
    }
    .into()
}

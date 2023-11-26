use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Dump)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let ident = input.ident;

    let fields = match input.data {
        syn::Data::Struct(ds) => match ds.fields {
            syn::Fields::Named(ref fields) => fields
                .named
                .iter()
                .flat_map(|f| {
                    let ident = &f.ident.clone()?;
                    Some(quote! {
                        writeln!(output, "{}: {}", stringify!(#ident), self.#ident)?;
                    })
                })
                .collect::<Vec<_>>(),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    quote! {
        impl Dump for #ident {
            fn dump<W: std::io::Write>(&self, output: &mut W) -> Result<(), std::io::Error> {
                #(#fields)*;
                Ok(())
            }
        }
    }
    .into()
}

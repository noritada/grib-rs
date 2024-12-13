use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Dump, attributes(doc))]
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
                    let doc = get_doc(&f.attrs)
                        .map(|s| format!(" ({})", s.trim()))
                        .unwrap_or(String::new());
                    Some(quote! {
                        writeln!(output, "{}{}: {}", stringify!(#ident), #doc, self.#ident)?;
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

fn get_doc(attrs: &Vec<syn::Attribute>) -> Option<String> {
    let mut doc = String::new();
    for attr in attrs.iter() {
        match attr.meta {
            syn::Meta::NameValue(ref value) if value.path.is_ident("doc") => {
                if let syn::Expr::Lit(lit) = &value.value {
                    if let syn::Lit::Str(s) = &lit.lit {
                        doc.push_str(&s.value());
                    }
                }
            }
            _ => {}
        }
    }
    if doc.is_empty() {
        None
    } else {
        Some(doc)
    }
}

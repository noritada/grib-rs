use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(TryFromSlice)]
pub fn derive_try_from_slice(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref s) => match &s.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => unimplemented!("`TryFromSlice` can only be derived for structs with named fields"),
        },
        _ => unimplemented!("`TryFromSlice` can only be derived for structs"),
    };

    let mut field_reads = Vec::new();
    let mut idents = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        field_reads.push(quote! {
            let #ident = grib_data_helpers::read_from_slice::<#ty>(slice, &mut pos)?;
        });
        idents.push(ident);
    }

    quote! {
        impl grib_data_helpers::TryFromSlice for #name {
            fn try_from_slice(slice: &[u8]) -> grib_data_helpers::TryFromSliceResult<Self> {
                let mut pos = 0;
                #(#field_reads)*
                Ok(Self { #(#idents),* })
            }
        }
    }
    .into()
}

#[proc_macro_derive(Dump, attributes(doc))]
pub fn derive_dump(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        syn::Data::Struct(ref s) => match &s.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => unimplemented!("`Dump` can only be derived for structs with named fields"),
        },
        _ => unimplemented!("`Dump` can only be derived for structs"),
    };

    let mut dumps = Vec::new();
    let mut start_pos = quote! { 1usize }; // WMO documentation style octet representation

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let end_pos = quote! { #start_pos + std::mem::size_of::<#ty>() };

        let doc = get_doc(&field.attrs)
            .map(|s| format!("  // {}", s.trim()))
            .unwrap_or(String::new());
        dumps.push(quote! {
            let (start, end) = (#start_pos, #end_pos - 1);
            if start == end {
                write!(output, "{}", start)?;
            } else {
                write!(output, "{}-{}", start, end)?;
            }
            writeln!(output, ": {} = {}{}",
                stringify!(#ident),
                self.#ident,
                #doc,
            )?;
        });

        start_pos = end_pos;
    }

    quote! {
        impl Dump for #name {
            fn dump<W: std::io::Write>(&self, output: &mut W) -> Result<(), std::io::Error> {
                #(#dumps)*;
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

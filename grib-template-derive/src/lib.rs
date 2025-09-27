use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(TryFromSlice, attributes(try_from_slice))]
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

        let len_attr = field.attrs.iter().find_map(|attr| parse_len_attr(attr));
        if let Some(len) = len_attr {
            if let syn::Type::Path(type_path) = ty {
                if let Some(inner_ty) = extract_vec_inner(type_path) {
                    field_reads.push(quote! {
                        let mut #ident = Vec::with_capacity(#len);
                        for _ in 0..#len {
                            let item =
                                grib_data_helpers::read_from_slice::<#inner_ty>(slice, &mut pos)?;
                            #ident.push(item);
                        }
                    });
                    idents.push(ident);
                    continue;
                }
            }
            unimplemented!("`#[try_from_slice(len = N)]` is only available for `Vec<T>");
        }

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

enum LenKind {
    Literal(usize),
    Ident(syn::Ident),
}

impl quote::ToTokens for LenKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            LenKind::Literal(n) => {
                tokens.extend(quote! { #n });
            }
            LenKind::Ident(ident) => {
                tokens.extend(quote! { #ident as usize });
            }
        }
    }
}

fn parse_len_attr(attr: &syn::Attribute) -> Option<LenKind> {
    if !attr.path().is_ident("try_from_slice") {
        return None;
    }
    let meta = attr.parse_args::<syn::Meta>().ok()?;
    if let syn::Meta::NameValue(nv) = meta {
        if !nv.path.is_ident("len") {
            return None;
        }
        match nv.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(lit_int),
                ..
            }) => Some(LenKind::Literal(lit_int.base10_parse::<usize>().unwrap())),
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) => Some(LenKind::Ident(syn::Ident::new(
                &lit_str.value(),
                lit_str.span(),
            ))),
            _ => None,
        }
    } else {
        None
    }
}

fn extract_vec_inner(type_path: &syn::TypePath) -> Option<syn::Type> {
    if type_path.path.segments.len() == 1 && type_path.path.segments[0].ident == "Vec" {
        if let syn::PathArguments::AngleBracketed(ref args) = type_path.path.segments[0].arguments {
            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                return Some(inner_ty.clone());
            }
        }
    }
    None
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
    let mut start_pos = quote! { start };

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        let end_pos = quote! { #start_pos + std::mem::size_of::<#ty>() };

        let doc = get_doc(&field.attrs)
            .map(|s| format!("  // {}", s.trim()))
            .unwrap_or_default();
        dumps.push(quote! {
            <#ty as grib_data_helpers::DumpField>::dump_field(
                &self.#ident,
                stringify!(#ident),
                parent,
                #doc,
                #start_pos,
                output,
            )?;
        });

        start_pos = end_pos;
    }

    quote! {
        impl grib_data_helpers::Dump for #name {
            fn dump<W: std::io::Write>(
                &self,
                parent: Option<&std::borrow::Cow<str>>,
                start: usize,
                output: &mut W,
            ) -> Result<(), std::io::Error> {
                #(#dumps)*;
                Ok(())
            }
        }
    }
    .into()
}

fn get_doc(attrs: &[syn::Attribute]) -> Option<String> {
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

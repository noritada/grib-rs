use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(TryFromSlice, attributes(grib_template))]
pub fn derive_try_from_slice(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => impl_try_from_slice_for_struct(&input, data),
        syn::Data::Enum(data) => impl_try_from_slice_for_enum(&input, data),
        _ => unimplemented!("`TryFromSlice` can only be derived for structs/enums"),
    }
    .into()
}

fn impl_try_from_slice_for_struct(
    input: &syn::DeriveInput,
    data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let fields = match &data.fields {
        syn::Fields::Named(fields) => &fields.named,
        _ => unimplemented!("`TryFromSlice` can only be derived for structs with named fields"),
    };

    let mut field_reads = Vec::new();
    let mut idents = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let len_attr = field
            .attrs
            .iter()
            .find_map(|attr| attr_value(attr, "len").map(|v| parse_len_attr(&v)));
        if let Some(len) = len_attr {
            if let syn::Type::Path(type_path) = ty {
                if let Some(inner_ty) = extract_vec_inner(type_path) {
                    field_reads.push(quote! {
                        let mut #ident = Vec::with_capacity(#len);
                        for _ in 0..#len {
                            let item =
                                <#inner_ty as grib_data_helpers::TryFromSlice>::try_from_slice(
                                    slice,
                                    pos,
                                )?;
                            #ident.push(item);
                        }
                    });
                    idents.push(ident);
                    continue;
                }
            }
            unimplemented!("`#[grib_template(len = N)]` is only available for `Vec<T>");
        }

        let enum_attr = field
            .attrs
            .iter()
            .find_map(|attr| attr_value(attr, "variant").map(|v| parse_variant_attr(&v)));
        if let Some(enum_ident) = enum_attr {
            field_reads.push(quote! {
                let #ident = <#ty as grib_data_helpers::TryEnumFromSlice>::try_enum_from_slice(
                    #enum_ident,
                    slice,
                    pos,
                )?;
            });
            idents.push(ident);
            continue;
        }

        field_reads.push(quote! {
            let #ident = <#ty as grib_data_helpers::TryFromSlice>::try_from_slice(slice, pos)?;
        });
        idents.push(ident);
    }

    quote! {
        impl grib_data_helpers::TryFromSlice for #name {
            fn try_from_slice(
                slice: &[u8],
                pos: &mut usize,
            ) -> grib_data_helpers::TryFromSliceResult<Self> {
                #(#field_reads)*
                Ok(Self { #(#idents),* })
            }
        }
    }
}

fn impl_try_from_slice_for_enum(
    input: &syn::DeriveInput,
    data: &syn::DataEnum,
) -> proc_macro2::TokenStream {
    let name = &input.ident;

    let mut arms = Vec::new();

    for variant in &data.variants {
        let variant_ident = &variant.ident;
        let disc_expr = variant
            .discriminant
            .as_ref()
            .expect("`TryFromSlice` requires the enum to have explicit discriminant")
            .1
            .clone();

        if let syn::Fields::Unnamed(fields) = &variant.fields
            && fields.unnamed.len() == 1
        {
            let inner_ty = &fields.unnamed.first().unwrap().ty;
            arms.push(quote! {
                #disc_expr => {
                    let inner =
                        <#inner_ty as grib_data_helpers::TryFromSlice>::try_from_slice(slice, pos)?;
                    Ok(#name::#variant_ident(inner))
                }
            });
        } else {
            unimplemented!("`TryFromSlice` only supports single-field tuple variants");
        }
    }

    quote! {
        impl grib_data_helpers::TryEnumFromSlice for #name {
            fn try_enum_from_slice(
                discriminant: impl Into<u64>,
                slice: &[u8],
                pos: &mut usize,
            ) -> grib_data_helpers::TryFromSliceResult<Self> {
                match discriminant.into() {
                    #(#arms),*,
                    _ => panic!("unknown variant for {}", stringify!(#name)),
                }
            }
        }
    }
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

fn attr_value(attr: &syn::Attribute, ident: &str) -> Option<syn::Expr> {
    if !attr.path().is_ident("grib_template") {
        return None;
    }
    let meta = attr.parse_args::<syn::Meta>().ok()?;
    if let syn::Meta::NameValue(nv) = meta {
        if !nv.path.is_ident(ident) {
            return None;
        }
        Some(nv.value)
    } else {
        None
    }
}

fn parse_len_attr(attr_value: &syn::Expr) -> Option<LenKind> {
    match attr_value {
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
}

fn parse_variant_attr(attr_value: &syn::Expr) -> Option<syn::Ident> {
    match attr_value {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => Some(syn::Ident::new(&lit_str.value(), lit_str.span())),
        _ => None,
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

    match &input.data {
        syn::Data::Struct(data) => impl_dump_for_struct(&input, data),
        syn::Data::Enum(data) => impl_dump_for_enum(&input, data),
        _ => unimplemented!("`Dump` can only be derived for structs/enums"),
    }
    .into()
}

fn impl_dump_for_struct(
    input: &syn::DeriveInput,
    data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let fields = match &data.fields {
        syn::Fields::Named(fields) => &fields.named,
        _ => unimplemented!("`Dump` can only be derived for structs with named fields"),
    };

    let mut dumps = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let doc = get_doc(&field.attrs)
            .map(|s| format!("  // {}", s.trim()))
            .unwrap_or_default();
        dumps.push(quote! {
            <#ty as grib_data_helpers::DumpField>::dump_field(
                &self.#ident,
                stringify!(#ident),
                parent,
                #doc,
                pos,
                output,
            )?;
        });
    }

    quote! {
        impl grib_data_helpers::Dump for #name {
            fn dump<W: std::io::Write>(
                &self,
                parent: Option<&std::borrow::Cow<str>>,
                pos: &mut usize,
                output: &mut W,
            ) -> Result<(), std::io::Error> {
                #(#dumps)*;
                Ok(())
            }
        }
    }
    .into()
}

fn impl_dump_for_enum(input: &syn::DeriveInput, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    let name = &input.ident;

    let mut arms = Vec::new();

    for variant in &data.variants {
        let variant_ident = &variant.ident;

        if let syn::Fields::Unnamed(fields) = &variant.fields
            && fields.unnamed.len() == 1
        {
            let inner_ty = &fields.unnamed.first().unwrap().ty;
            arms.push(quote! {
                #name::#variant_ident(inner) => <#inner_ty as grib_data_helpers::Dump>::dump(
                    inner,
                    parent,
                    pos,
                    output
                )
            });
        } else {
            unimplemented!("`Dump` only supports single-field tuple variants");
        }
    }

    quote! {
        impl grib_data_helpers::Dump for #name {
            fn dump<W: std::io::Write>(
                &self,
                parent: Option<&std::borrow::Cow<str>>,
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

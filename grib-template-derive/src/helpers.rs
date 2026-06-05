#[derive(PartialEq)]
pub(crate) enum StructKind {
    TupleStruct,
    NamedStruct,
}

pub(crate) fn attr_value(attr: &syn::Attribute, ident: &str) -> Option<syn::Expr> {
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NumOctets(usize);

impl TryFrom<&syn::Attribute> for NumOctets {
    type Error = &'static str;

    fn try_from(value: &syn::Attribute) -> Result<Self, Self::Error> {
        attr_value(value, "num_octets")
            .and_then(|v| match v {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(lit_int),
                    ..
                }) => Some(Self(lit_int.base10_parse::<usize>().unwrap())),
                _ => None,
            })
            .ok_or(r#"parsing "num_octets" failed"#)
    }
}

impl quote::ToTokens for NumOctets {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens);
    }
}

pub(crate) fn extract_struct_info(
    data: &syn::DataStruct,
) -> Option<(
    StructKind,
    &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
)> {
    match &data.fields {
        syn::Fields::Named(fields) => Some((StructKind::NamedStruct, &fields.named)),
        syn::Fields::Unnamed(fields) => {
            let fields = &fields.unnamed;
            if fields.len() == 1 && is_type_u8(&fields.first().unwrap().ty) {
                Some((StructKind::TupleStruct, fields))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn is_type_u8(ty: &syn::Type) -> bool {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty
        && let Some(segment) = path.segments.last()
    {
        return segment.ident == "u8";
    }
    false
}

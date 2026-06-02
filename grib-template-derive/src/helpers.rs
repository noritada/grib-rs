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

pub(crate) fn parse_num_octets_attr(attr_value: &syn::Expr) -> Option<usize> {
    match attr_value {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) => Some(lit_int.base10_parse::<usize>().unwrap()),
        _ => None,
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

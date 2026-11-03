use proc_macro::TokenStream;

/// Derive macro generating an impl of the trait
/// `grib_template_helpers::TryFromSlice`.
#[proc_macro_derive(TryFromSlice, attributes(grib_template))]
pub fn derive_try_from_slice(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => try_from_slice::impl_for_struct(&input, data),
        syn::Data::Enum(data) => try_from_slice::impl_for_enum(&input, data),
        _ => unimplemented!("`TryFromSlice` can only be derived for structs/enums"),
    }
    .into()
}

/// Derive macro generating an impl of the trait
/// `grib_template_helpers::WriteToBuffer`.
#[proc_macro_derive(WriteToBuffer, attributes(grib_template))]
pub fn derive_write_to_buffer(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => write_to_buffer::impl_for_struct(&input, data),
        syn::Data::Enum(data) => write_to_buffer::impl_for_enum(&input, data),
        _ => unimplemented!("`WriteToBuffer` can only be derived for structs/enums"),
    }
    .into()
}

/// Derive macro generating an impl of the trait `grib_template_helpers::Dump`.
///
/// The generated implementation accepts a borrowed context of any type
/// supported by all fields and forwards it unchanged to nested values.
///
/// Use `#[dump(context = ContextType)]` on a struct or enum to select its
/// context type instead. On a named struct field, `#[dump(context =
/// expression)]` evaluates the expression once and borrows its result for that
/// child's dump. The expression can use `context` (the parent's borrowed
/// context) and `self`. Other fields continue to receive the original context.
/// The expression should produce the child's context value, rather than a
/// reference to it. Generic fields with a context expression need explicit
/// trait bounds when the compiler cannot prove their `DumpField`
/// implementation.
///
/// For example, a template can supplement an external context with a sibling:
///
/// ```ignore
/// #[derive(Dump)]
/// #[dump(context = Grib2Context)]
/// struct ProductTemplate {
///     parameter_category: ParameterCategory,
///     #[dump(context = context.with_parameter_category(&self.parameter_category))]
///     parameter_number: ParameterNumber,
/// }
/// ```
///
/// Field context and documentation overrides can share an attribute:
/// `#[dump(context = context.for_child(), doc(value = "Description"))]`.
///
/// Single-field `u8` and `u16` tuple structs display their value as a decimal
/// number. Use `#[dump(flag)]` on a single-field `u8` tuple struct to display
/// it as an eight-bit binary number with a `0b` prefix instead.
/// Use `#[dump(code_table)]` on a single-field `u8` or `u16` tuple struct
/// implementing `grib_template_helpers::CodeTable` to append its description
/// in brackets when present. This can be combined with `#[dump(flag)]`.
#[proc_macro_derive(Dump, attributes(grib_template, dump))]
pub fn derive_dump(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match &input.data {
        syn::Data::Struct(data) => dump::impl_for_struct(&input, data),
        syn::Data::Enum(data) => dump::impl_for_enum(&input, data),
        _ => unimplemented!("`Dump` can only be derived for structs/enums"),
    }
    .into()
}

/// Derive macro generating an impl of `grib_template_helpers::CodeTable`.
///
/// Tables without external keys implement `CodeTable<C>` for any context.
/// For WMO tables 4.1 and 4.2, specify `context = ContextType` and
/// `key = (context.discipline,)` or
/// `key = (context.discipline, context.category)`. These tuple keys are matched
/// against the discipline and category in `SubTitle_en`, followed by the
/// wrapped code number. Both options must be specified together.
///
/// Requires a single-field `u8` or `u16` tuple struct with
/// `#[code_table(file = "...", format = ...)]`. The UTF-8 file is read relative
/// to `CARGO_MANIFEST_DIR` and included in the expansion to track file changes.
///
/// The `format` value is an identifier with optional parenthesized options,
/// currently supporting only `wmo_csv` without options. WMO CSV records are
/// used to generate description and unit lookups. If no units are defined,
/// `code_unit` returns `None` without a runtime lookup.
#[proc_macro_derive(CodeTable, attributes(code_table))]
pub fn derive_code_table(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    code_table::expand(&input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

mod code_table;
mod dump;
mod helpers;
mod try_from_slice;
mod write_to_buffer;

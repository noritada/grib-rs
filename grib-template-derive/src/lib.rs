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
/// Single-field `u8` and `u16` tuple structs display their value as a decimal
/// number. Use `#[dump(flag)]` on a single-field `u8` tuple struct to display
/// it as an eight-bit binary number with a `0b` prefix instead.
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

mod dump;
mod helpers;
mod try_from_slice;
mod write_to_buffer;

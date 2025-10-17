#![no_std]
extern crate alloc;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::{contract::contract_macro, table::table_macro};

mod contract;
mod derive_numbytes;
mod derive_read;
mod derive_write;
mod dispatch;
mod internal;
mod name;
mod name_raw;
mod symbol_with_code;
mod table;

#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Keep any tokens for later parsing in #[contract]
    let _ = attr;
    item
}

#[proc_macro_attribute]
pub fn constructor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn destructor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
    contract_macro(_attr, item)
}

#[inline]
#[proc_macro]
pub fn name(input: TokenStream) -> TokenStream {
    use crate::name::PulseName;
    let item = parse_macro_input!(input as PulseName);
    quote!(#item).into()
}

#[inline]
#[proc_macro]
pub fn name_raw(input: TokenStream) -> TokenStream {
    use crate::name_raw::PulseName;
    let item = parse_macro_input!(input as PulseName);
    quote!(#item).into()
}

#[inline]
#[proc_macro]
pub fn dispatch(input: TokenStream) -> TokenStream {
    crate::dispatch::expand(input)
}

#[inline]
#[proc_macro_derive(Read)]
pub fn derive_read(input: TokenStream) -> TokenStream {
    use crate::derive_read::DeriveRead;
    let item = parse_macro_input!(input as DeriveRead);
    quote!(#item).into()
}

#[inline]
#[proc_macro_derive(NumBytes)]
pub fn derive_numbytes(input: TokenStream) -> TokenStream {
    use crate::derive_numbytes::DeriveNumBytes;
    let item = parse_macro_input!(input as DeriveNumBytes);
    quote!(#item).into()
}

#[inline]
#[proc_macro_derive(Write, attributes(pulse))]
pub fn derive_write(input: TokenStream) -> TokenStream {
    use crate::derive_write::DeriveWrite;
    let item = parse_macro_input!(input as DeriveWrite);
    quote!(#item).into()
}

#[inline]
#[proc_macro]
pub fn symbol_with_code(input: TokenStream) -> TokenStream {
    use crate::symbol_with_code::SymbolWithCode;
    let item = parse_macro_input!(input as SymbolWithCode);
    quote!(#item).into()
}

#[inline]
#[proc_macro_attribute]
pub fn table(attr: TokenStream, item: TokenStream) -> TokenStream {
    table_macro(attr, item)
}

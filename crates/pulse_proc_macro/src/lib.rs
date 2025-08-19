#![no_std]
extern crate alloc;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

mod action;
mod derive_numbytes;
mod derive_read;
mod derive_write;
mod dispatch;
mod internal;
mod name;
mod name_raw;
mod symbol_with_code;

#[inline]
#[proc_macro_attribute]
pub fn action(args: TokenStream, input: TokenStream) -> TokenStream {
    use crate::action::{ActionArgs, ActionFn};
    let args = parse_macro_input!(args as ActionArgs);
    let item = parse_macro_input!(input as ItemFn);
    let action = ActionFn::new(args, item);
    quote!(#action).into()
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
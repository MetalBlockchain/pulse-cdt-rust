use proc_macro2::TokenStream;
use pulse_name::name_from_bytes;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result as ParseResult},
    LitStr,
};

pub struct PulseName(u64);

impl Parse for PulseName {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let name = input.parse::<LitStr>()?.value();
        name_from_bytes(name.bytes())
            .map(Self)
            .map_err(|_e| input.error("failed to parse name"))
    }
}

impl ToTokens for PulseName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name_raw = self.0;
        let expanded = quote! {
            pulse_cdt::core::Name::new(#name_raw)
        };

        expanded.to_tokens(tokens);
    }
}

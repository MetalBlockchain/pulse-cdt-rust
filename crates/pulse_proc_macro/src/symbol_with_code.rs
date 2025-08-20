use proc_macro2::TokenStream;
use pulse_bytes::{symbol_code_from_bytes, symbol_from_code};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result as ParseResult},
    LitInt, LitStr, Token,
};

pub struct SymbolWithCode {
    symbol: u64,
}

impl Parse for SymbolWithCode {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let precision: LitInt = input.parse()?;
        input.parse::<Token![,]>()?;
        let sym: LitStr = input.parse()?;
        let symbol_code = symbol_code_from_bytes(sym.value().as_bytes())
            .map_err(|_| input.error("failed to parse symbol code"))?;
        let symbol = symbol_from_code(precision.base10_parse::<u8>()?, symbol_code);

        Ok(SymbolWithCode { symbol })
    }
}

impl ToTokens for SymbolWithCode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let symbol = &self.symbol;

        let expanded = quote! {{
            ::pulse_cdt::core::Symbol::new(#symbol)
        }};

        tokens.extend(expanded);
    }
}

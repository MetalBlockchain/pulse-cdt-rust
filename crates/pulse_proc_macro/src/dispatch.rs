use alloc::vec::Vec;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_macro_input,
    punctuated::Punctuated,
    Expr, Path, Token,
};

struct AbiPair {
    code: Option<Expr>,
    action: Path,
}

impl Parse for AbiPair {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let action: Path = input.parse()?;
        match input.parse::<Token![@]>() {
            Ok(_) => {
                let code: Expr = input.parse()?;
                Ok(AbiPair {
                    code: Some(code),
                    action,
                })
            }
            Err(_) => Ok(AbiPair { code: None, action }),
        }
    }
}

struct AbiPairs(Vec<AbiPair>);

impl Parse for AbiPairs {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let parsed = Punctuated::<AbiPair, Token![,]>::parse_separated_nonempty(input)?;
        let pairs: Vec<AbiPair> = parsed.into_iter().collect();
        Ok(AbiPairs(pairs))
    }
}

pub fn expand(input: TokenStream) -> TokenStream {
    let pairs = parse_macro_input!(input as AbiPairs);
    let actions = pairs.0.into_iter().map(|pair| {
        let code = pair
            .code
            .map(|code| quote!(pulse_cdt::name!(#code)))
            .unwrap_or_else(|| quote!(receiver));
        let action = pair.action;
        quote! {
            else if code == #code && action == <#action as pulse_cdt::contracts::ActionFn>::NAME.as_u64() {
                let data = pulse_cdt::contracts::read_action_data::<#action>().expect("failed to read action data");
                <#action as pulse_cdt::contracts::ActionFn>::call(data)
            }
        }
    });
    let expanded = quote! {
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        #[inline]
        pub extern "C" fn apply(receiver: u64, code: u64, action: u64) {
            if action == pulse_cdt::name!("onerror") {
                pulse_cdt::core::check(false, "onerror action's are only valid from the \"pulse\" system account");
            }
            #(#actions)*
            else if code == receiver {
                pulse_cdt::core::check(false, "unknown action");
            }
        }
    };
    expanded.into()
}

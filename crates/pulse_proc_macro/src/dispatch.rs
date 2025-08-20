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
            .map(|code| quote!(pulse_cdt::name_raw!(#code)))
            .unwrap_or_else(|| quote!(receiver));
        let action = pair.action;
        quote! {
            else if code == #code && action == <#action as pulse_cdt::contracts::ActionFn>::NAME.raw() {
                let data = pulse_cdt::contracts::read_action_data::<#action>().expect("failed to read action data");
                <#action as pulse_cdt::contracts::ActionFn>::call(data)
            }
        }
    });
    let expanded = quote! {
        #[cfg(target_arch = "wasm32")]
        #[global_allocator]
        static ALLOCATOR: ::pulse_cdt::__reexports::lol_alloc::AssumeSingleThreaded<
            ::pulse_cdt::__reexports::lol_alloc::LeakingAllocator
        > = unsafe {
            ::pulse_cdt::__reexports::lol_alloc::AssumeSingleThreaded::new(
                ::pulse_cdt::__reexports::lol_alloc::LeakingAllocator::new()
            )
        };

        #[cfg(target_arch = "wasm32")]
        #[panic_handler]
        fn panic(panic_info: &core::panic::PanicInfo) -> ! {
            let s = panic_info.message().as_str();
            if let Some(s) = s {
                pulse_cdt::core::check(false, s);
            } else {
                pulse_cdt::core::check(false, "panic without message");
            }
            ::core::arch::wasm32::unreachable()
        }

        let mut _self = 0u64;
        fn get_self() -> pulse_cdt::Name {
            pulse_cdt::Name(_self)
        }

        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn apply(receiver: u64, code: u64, action: u64) {
            if action == pulse_cdt::name_raw!("onerror") {
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

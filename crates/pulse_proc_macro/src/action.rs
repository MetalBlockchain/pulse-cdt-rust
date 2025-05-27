use alloc::{boxed::Box, string::ToString};
use heck::{ToLowerCamelCase, ToUpperCamelCase};
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream, Result as ParseResult},
    Block, FnArg, Ident, ItemFn, LitStr, Signature,
};

pub struct ActionArgs {
    name: Option<LitStr>,
}

impl Parse for ActionArgs {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let name = input.parse::<Option<LitStr>>()?;
        Ok(Self { name })
    }
}

pub struct ActionFn {
    sig: Signature,
    block: Box<Block>,
    args: ActionArgs,
}

impl ActionFn {
    pub fn new(args: ActionArgs, item: ItemFn) -> Self {
        Self {
            sig: item.sig,
            block: item.block,
            args,
        }
    }

    pub fn struct_ident(&self) -> Ident {
        let name = self.sig.ident.to_string().as_str().to_lower_camel_case();
        Ident::new(&name, self.sig.ident.span())
    }

    pub fn action_name(&self) -> LitStr {
        if let Some(lit) = &self.args.name {
            lit.clone()
        } else {
            LitStr::new(self.sig.ident.to_string().as_str(), self.sig.ident.span())
        }
    }
}

impl ToTokens for ActionFn {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let inputs = &self.sig.inputs;
        let mut struct_fields = quote!();
        let mut assign_args = quote!();
        for input in inputs.iter() {
            match input {
                FnArg::Typed(input) => {
                    let pat = &input.pat;
                    let ty = &input.ty;
                    struct_fields = quote! {
                        #struct_fields
                        pub #pat: #ty,
                    };
                    assign_args = quote! {
                        #assign_args
                        let #pat = self.#pat;
                    };
                }
                _ => unimplemented!(),
            }
        }
        let block = &self.block;

        let struct_ident_name = self.struct_ident().to_string() + "Wrapper";
        let struct_ident = Ident::new(
            &struct_ident_name.to_upper_camel_case().as_str(),
            self.sig.ident.span(),
        );
        let type_ident = &self.sig.ident;
        let action_name = self.action_name();

        let expanded = quote! {
            #[derive(Clone, pulse_cdt::Read, pulse_cdt::NumBytes)]
            pub struct #struct_ident {
                #struct_fields
            }

            // This makes the abi! macro work nicer
            #[allow(non_camel_case_types)]
            pub type #type_ident = #struct_ident;

            #[automatically_derived]
            impl pulse_cdt::contracts::ActionFn for #struct_ident {
                const NAME: pulse_cdt::core::name::Name = pulse_cdt::core::name::Name::new(pulse_cdt::name!(#action_name));
                fn call(self) {
                    #assign_args
                    #block
                }
            }
        };
        expanded.to_tokens(tokens);
    }
}

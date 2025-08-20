use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, Ident, ItemStruct, Result, Token,
};

/// #[table(primary_key = row.balance.symbol.code().raw())]
/// or
/// #[table(primary_key = |row| row.balance.symbol.code().raw())]
pub struct TableArgs {
    primary_key: Expr,
}

impl Parse for TableArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let key_ident: Ident = input.parse()?;
        if key_ident != "primary_key" {
            return Err(syn::Error::new(
                key_ident.span(),
                "expected `primary_key = <expr>`",
            ));
        }
        input.parse::<Token![=]>()?;
        let primary_key: Expr = input.parse()?;
        // optional trailing comma
        let _ = input.parse::<Token![,]>();
        Ok(Self { primary_key })
    }
}

pub fn table_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Keep the original struct around
    let input: ItemStruct = match syn::parse(item.clone()) {
        Ok(s) => s,
        Err(e) => return e.to_compile_error().into(),
    };

    let args = parse_macro_input!(attr as TableArgs);

    let ident = &input.ident;
    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let pk_expr = args.primary_key;

    // Allow either a plain expression that references `row`, or a closure `|row| ...`
    let body = match pk_expr {
        Expr::Closure(_) => quote! { (#pk_expr)(row) },
        _ => quote! { (#pk_expr) },
    };

    let expanded = quote! {
        // original struct
        #input

        // generated impl
        impl #impl_generics Table for #ident #ty_generics #where_clause {
            type Key = u64;
            type Row = Self;

            #[inline]
            fn primary_key(row: &Self::Row) -> u64 {
                #body
            }
        }
    };

    expanded.into()
}

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Attribute, FnArg, ImplItem, ImplItemMethod, ItemImpl, Lit, Meta, MetaList, MetaNameValue,
    NestedMeta, Path, Result, Type,
};

pub fn contract_macro(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(item as ItemImpl);

    let args = match syn::parse::<ContractArgs>(attr) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    match expand_contract(impl_block, args) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Global options for #[contract]
struct ContractArgs {
    decoder: Option<Path>, // generic fn<T>() -> T
}

impl Parse for ContractArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self { decoder: None });
        }
        // Parse: decoder = <path>
        let meta: Meta = input.parse()?;
        match meta {
            Meta::NameValue(MetaNameValue {
                path,
                lit: Lit::Str(s),
                ..
            }) if path.is_ident("decoder") => {
                let p: Path = s.parse()?;
                Ok(Self { decoder: Some(p) })
            }
            Meta::List(MetaList { path, nested, .. }) if path.is_ident("decoder") => {
                // allow #[contract(decoder(path::to::decode))]
                let mut iter = nested.into_iter();
                let first = iter
                    .next()
                    .ok_or_else(|| syn::Error::new(path.span(), "expected decoder path"))?;
                if iter.next().is_some() {
                    return Err(syn::Error::new(
                        path.span(),
                        "expected a single path for decoder",
                    ));
                }
                let p = match first {
                    NestedMeta::Meta(Meta::Path(p)) => p,
                    other => return Err(syn::Error::new(other.span(), "expected a path")),
                };
                Ok(Self { decoder: Some(p) })
            }
            _ => Err(syn::Error::new(
                meta.span(),
                r#"expected `decoder = "path::to::decode"` or `decoder(path::to::decode)`"#,
            )),
        }
    }
}

fn expand_contract(impl_block: ItemImpl, args: ContractArgs) -> Result<TokenStream2> {
    // Must be inherent impl (no trait)
    if impl_block.trait_.is_some() {
        return Err(syn::Error::new(
            impl_block.span(),
            "#[contract] must be applied to an inherent `impl Type { ... }`",
        ));
    }
    // Keep it non-generic for a clean `apply`
    if !impl_block.generics.params.is_empty() || impl_block.generics.where_clause.is_some() {
        return Err(syn::Error::new(
            impl_block.generics.span(),
            "#[contract] does not support generic impls",
        ));
    }

    let self_ty: &Type = &*impl_block.self_ty;

    // Find constructor (at most one) & actions
    let mut constructor: Option<ImplItemMethod> = None;
    let mut destructor: Option<ImplItemMethod> = None;
    let mut actions: Vec<ActionMeta> = Vec::new();
    let mut notify_handlers: Vec<NotifyMeta> = Vec::new();

    for item in &impl_block.items {
        if let ImplItem::Method(m) = item {
            if has_attr(&m.attrs, "constructor") {
                ensure_no_receiver(m, "constructor")?;
                ensure_arg_count(m, 0, "constructor")?;
                if constructor.is_some() {
                    return Err(syn::Error::new(
                        m.sig.span(),
                        "only one #[constructor] is allowed",
                    ));
                }
                constructor = Some(m.clone());
            }

            if has_attr(&m.attrs, "destructor") {
                // must take `self` by value
                match receiver_kind(m) {
                    ReceiverKind::Value => {} // ok
                    _ => {
                        return Err(syn::Error::new(
                            m.sig.span(),
                            "#[destructor] must take `self` by value",
                        ));
                    }
                }
                ensure_arg_count(m, 0, "destructor")?;
                if destructor.is_some() {
                    return Err(syn::Error::new(
                        m.sig.span(),
                        "only one #[destructor] is allowed",
                    ));
                }
                destructor = Some(m.clone());
            }

            if let Some(cfg) = parse_action_attr(&m.attrs)? {
                // CHANGED: allow &self OR no receiver; forbid &mut self / self
                let rk = receiver_kind(m);
                match rk {
                    ReceiverKind::None | ReceiverKind::Ref | ReceiverKind::MutRef => {}
                    ReceiverKind::Value => {
                        return Err(syn::Error::new(
                            m.sig.span(),
                            "#[action] methods must be static or take `&self` (not `&mut self` or `self`)",
                        ));
                    }
                }

                actions.push(ActionMeta {
                    method: m.clone(),
                    name: cfg.name.unwrap_or_else(|| m.sig.ident.to_string()),
                    decoder: cfg.decoder.or_else(|| args.decoder.clone()), // per-action wins, else global, else default
                    rk,
                });
            }

            // #[on_notify("account::action")]
            if let Some(cfg) = parse_on_notify_attr(&m.attrs)? {
                let rk = receiver_kind(m);
                match rk {
                    ReceiverKind::None | ReceiverKind::Ref | ReceiverKind::MutRef => {}
                    ReceiverKind::Value => {
                        return Err(syn::Error::new(
                            m.sig.span(),
                            "#[on_notify] methods must be static or take `&self` (not `&mut self` or `self`)",
                        ));
                    }
                }

                let (acct_pat, action_pat) = parse_notify_pattern(&cfg.pattern, m.sig.span())?;

                notify_handlers.push(NotifyMeta {
                    method: m.clone(),
                    acct_pat,
                    action_pat,
                    decoder: cfg.decoder.or_else(|| args.decoder.clone()),
                    rk,
                });
            }
        }
    }

    // Generate constructor arm
    let ctor_arm = if let Some(m) = constructor.as_ref() {
        let ident = &m.sig.ident;
        quote! {
            let mut __instance: #self_ty = <#self_ty>::#ident();
        }
    } else {
        quote! {
            let mut __instance: #self_ty = ::core::default::Default::default();
        }
    };

    let dtor_call = if let Some(m) = destructor.as_ref() {
        let ident = &m.sig.ident;
        quote! {
            __instance.#ident();
        }
    } else {
        quote! {}
    };

    // Generate action arms.
    //
    // These run only when `code == receiver` (a self-received action), so the
    // arm conditions no longer re-test that invariant — it's hoisted into the
    // top-level branch in `apply`. The first arm is a plain `if`, the rest are
    // `else if`, so no `if false {}` seed is needed.
    let action_arms = actions.iter().enumerate().map(|(i, a)| {
        let method_ident = &a.method.sig.ident;
        let action_name_str = &a.name;
        let kw = if i == 0 { quote!(if) } else { quote!(else if) };

        // Build tuple type of method parameters
        let arg_types: Vec<&Type> = a
            .method
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let FnArg::Typed(pt) = arg {
                    Some(&*pt.ty)
                } else {
                    None
                }
            })
            .collect();

        let args_len = arg_types.len();

        let tuple_ty = tuple_type_tokens(&arg_types);

        // Choose decoder path:
        // - explicit per-action
        // - else global impl-level
        // - else default pulse_cdt::core::unpack_action_data
        let decoder_path: TokenStream2 = a
            .decoder
            .clone()
            .map(|p| quote!(#p))
            .unwrap_or_else(|| quote!(::pulse_cdt::contracts::read_action_data));

        // Generate the call depending on receiver kind
        let call_no_args = match a.rk {
            ReceiverKind::None => quote! { <#self_ty>::#method_ident() },
            ReceiverKind::Ref => quote! { __instance.#method_ident() },
            ReceiverKind::MutRef => quote! { __instance.#method_ident() },
            _ => unreachable!(),
        };

        if args_len == 0 {
            // no-arg action: no decode needed
            quote! {
                #kw action == pulse_cdt::name_raw!(#action_name_str) {
                    #call_no_args;
                }
            }
        } else {
            // decode tuple, destructure, call
            let tmp_ident = format_ident!("__args");
            let bind_idents: Vec<proc_macro2::Ident> =
                (0..args_len).map(|i| format_ident!("__a{}", i)).collect();

            let call_with_args = match a.rk {
                ReceiverKind::None => {
                    let args = quote! { #(#bind_idents),* };
                    quote! { <#self_ty>::#method_ident( #args ) }
                }

                ReceiverKind::Ref | ReceiverKind::MutRef => {
                    let args = quote! { #(#bind_idents),* };
                    quote! { __instance.#method_ident( #args ) }
                }
                _ => unreachable!(),
            };

            let bind_pat = if args_len == 1 {
                let a0 = &bind_idents[0];
                quote! { ( #a0 , ) } // <-- note the trailing comma
            } else {
                quote! { ( #(#bind_idents),* ) }
            };

            quote! {
                #kw action == pulse_cdt::name_raw!(#action_name_str) {
                    type __Args = #tuple_ty;
                    let #tmp_ident: __Args = #decoder_path::<__Args>();
                    let #bind_pat = #tmp_ident;
                    #call_with_args;
                }
            }
        }
    });

    // Generate on_notify arms.
    //
    // EOSIO semantics: a notify handler fires when the contract is being
    // *notified* of an action it did not directly receive, i.e. `code != receiver`.
    // That invariant is hoisted into the top-level branch in `apply`, so the
    // arms only test `code` (the first receiver / the account that called
    // `require_recipient`) and `action`. Either side of the pattern may be a
    // wildcard `*`, in which case that side collapses to `true`. The first arm
    // is a plain `if`, the rest are `else if`.
    let notify_arms = notify_handlers.iter().enumerate().map(|(i, h)| {
        let method_ident = &h.method.sig.ident;
        let kw = if i == 0 { quote!(if) } else { quote!(else if) };

        let arg_types: Vec<&Type> = h
            .method
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let FnArg::Typed(pt) = arg {
                    Some(&*pt.ty)
                } else {
                    None
                }
            })
            .collect();

        let args_len = arg_types.len();
        let tuple_ty = tuple_type_tokens(&arg_types);

        let decoder_path: TokenStream2 = h
            .decoder
            .clone()
            .map(|p| quote!(#p))
            .unwrap_or_else(|| quote!(::pulse_cdt::contracts::read_action_data));

        // Build the match condition for code/action against the (possibly wildcarded) pattern.
        let code_cond: TokenStream2 = match &h.acct_pat {
            NotifyPat::Wildcard => quote! { true },
            NotifyPat::Name(s) => quote! { code == pulse_cdt::name_raw!(#s) },
        };
        let action_cond: TokenStream2 = match &h.action_pat {
            NotifyPat::Wildcard => quote! { true },
            NotifyPat::Name(s) => quote! { action == pulse_cdt::name_raw!(#s) },
        };

        let call_no_args = match h.rk {
            ReceiverKind::None => quote! { <#self_ty>::#method_ident() },
            ReceiverKind::Ref => quote! { __instance.#method_ident() },
            ReceiverKind::MutRef => quote! { __instance.#method_ident() },
            _ => unreachable!(),
        };

        if args_len == 0 {
            quote! {
                #kw (#code_cond) && (#action_cond) {
                    #call_no_args;
                }
            }
        } else {
            let tmp_ident = format_ident!("__nargs");
            let bind_idents: Vec<proc_macro2::Ident> =
                (0..args_len).map(|i| format_ident!("__n{}", i)).collect();

            let call_with_args = match h.rk {
                ReceiverKind::None => {
                    let args = quote! { #(#bind_idents),* };
                    quote! { <#self_ty>::#method_ident( #args ) }
                }
                ReceiverKind::Ref | ReceiverKind::MutRef => {
                    let args = quote! { #(#bind_idents),* };
                    quote! { __instance.#method_ident( #args ) }
                }
                _ => unreachable!(),
            };

            let bind_pat = if args_len == 1 {
                let a0 = &bind_idents[0];
                quote! { ( #a0 , ) }
            } else {
                quote! { ( #(#bind_idents),* ) }
            };

            quote! {
                #kw (#code_cond) && (#action_cond) {
                    type __NArgs = #tuple_ty;
                    let #tmp_ident: __NArgs = #decoder_path::<__NArgs>();
                    let #bind_pat = #tmp_ident;
                    #call_with_args;
                }
            }
        }
    });

    // Assemble the `code == receiver` branch (self-received actions). If there
    // are no actions, the body is just the "unknown action" check — emitting a
    // bare `else` with no preceding `if` would be a syntax error.
    let action_dispatch = if actions.is_empty() {
        quote! {
            pulse_cdt::core::check(false, "unknown action");
        }
    } else {
        quote! {
            #(#action_arms)*
            else {
                pulse_cdt::core::check(false, "unknown action");
            }
        }
    };

    // Assemble the `code != receiver` branch (notifications). If there are no
    // notify handlers, the notification falls through silently (EOSIO behavior),
    // so emit nothing.
    let notify_dispatch = if notify_handlers.is_empty() {
        quote! {}
    } else {
        quote! {
            #(#notify_arms)*
        }
    };

    let type_ident = match self_ty {
        syn::Type::Path(tp) => tp.path.segments.last().unwrap().ident.clone(),
        _ => {
            return Err(syn::Error::new(
                self_ty.span(),
                "#[contract] expects a plain type name",
            ))
        }
    };
    let ctx_mod_ident = syn::Ident::new(
        &format!("__{}_contract_ctx", type_ident),
        proc_macro2::Span::call_site(),
    );

    let output = quote! {
        #impl_block

        #[cfg(all(target_arch = "wasm32"))]
        #[global_allocator]
        static ALLOCATOR: ::pulse_cdt::__reexports::lol_alloc::AssumeSingleThreaded<
            ::pulse_cdt::__reexports::lol_alloc::LeakingAllocator
        > = unsafe {
            ::pulse_cdt::__reexports::lol_alloc::AssumeSingleThreaded::new(
                ::pulse_cdt::__reexports::lol_alloc::LeakingAllocator::new()
            )
        };

        #[cfg(all(target_arch = "wasm32"))]
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

        // ===== per-call context (receiver) =====
        #[doc(hidden)]
        mod #ctx_mod_ident {
            use core::sync::atomic::{AtomicU64, Ordering};

            static RECEIVER: AtomicU64 = AtomicU64::new(0);

            #[inline]
            pub fn get_self() -> u64 {
                RECEIVER.load(Ordering::Relaxed)
            }

            #[inline]
            pub fn __set_receiver(v: u64) {
                RECEIVER.store(v, Ordering::Relaxed);
            }

            #[inline]
            pub fn __clear_receiver() {
                RECEIVER.store(0, Ordering::Relaxed);
            }

            pub struct ReceiverGuard;

            impl ReceiverGuard {
                #[inline]
                pub fn new(v: u64) -> Self {
                    __set_receiver(v);
                    ReceiverGuard
                }
            }

            impl Drop for ReceiverGuard {
                #[inline]
                fn drop(&mut self) {
                    __clear_receiver();
                }
            }
        }

        #[inline]
        pub fn get_self() -> Name {
            pulse_cdt::core::Name::new(#ctx_mod_ident::get_self())
        }

        #[no_mangle]
        pub extern "C" fn apply(receiver: u64, code: u64, action: u64) {
            // set receiver for the entire call; cleared on all exits (incl. early returns)
            let __guard = #ctx_mod_ident::ReceiverGuard::new(receiver);

            #ctor_arm

            if action == pulse_cdt::name_raw!("onerror") {
                pulse_cdt::core::check(false, "onerror action's are only valid from the \"pulse\" system account");
            }

            // The `code == receiver` invariant is tested once here rather than
            // in every dispatch arm:
            //   - code == receiver  => a self-received action
            //   - code != receiver  => a notification from another contract
            if code == receiver {
                #action_dispatch
            } else {
                #notify_dispatch
            }

            #dtor_call

            // guard drops here, clearing the receiver
            core::mem::drop(__guard);
        }
    };

    Ok(output)
}

struct ActionCfg {
    name: Option<String>,
    decoder: Option<Path>,
}

struct ActionMeta {
    method: ImplItemMethod,
    name: String,
    decoder: Option<Path>,
    rk: ReceiverKind,
}

struct NotifyCfg {
    pattern: String,
    decoder: Option<Path>,
}

struct NotifyMeta {
    method: ImplItemMethod,
    acct_pat: NotifyPat,
    action_pat: NotifyPat,
    decoder: Option<Path>,
    rk: ReceiverKind,
}

#[derive(Clone)]
enum NotifyPat {
    Wildcard,
    Name(String),
}

fn parse_action_attr(attrs: &[Attribute]) -> Result<Option<ActionCfg>> {
    // Accept #[action], #[action(name = "...")], #[action(decoder = path)], or both
    let mut cfg: Option<ActionCfg> = None;

    for a in attrs {
        if is_attr(a, "action") {
            let mut found = ActionCfg {
                name: None,
                decoder: None,
            };

            if let Ok(meta) = a.parse_meta() {
                match meta {
                    Meta::Path(_) => { /* bare #[action] */ }
                    Meta::List(MetaList { nested, .. }) => {
                        for n in nested {
                            match n {
                                NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                    path,
                                    lit: Lit::Str(s),
                                    ..
                                })) if path.is_ident("name") => {
                                    found.name = Some(s.value());
                                }
                                NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                    path,
                                    lit: Lit::Str(s),
                                    ..
                                })) if path.is_ident("decoder") => {
                                    let p: Path = s.parse()?;
                                    found.decoder = Some(p);
                                }
                                NestedMeta::Meta(Meta::List(MetaList { path, nested, .. }))
                                    if path.is_ident("decoder") =>
                                {
                                    // allow #[action(decoder(path::to::decode))]
                                    let mut it = nested.into_iter();
                                    let one = it.next().ok_or_else(|| {
                                        syn::Error::new(path.span(), "expected decoder path")
                                    })?;
                                    if it.next().is_some() {
                                        return Err(syn::Error::new(
                                            path.span(),
                                            "expected a single path for decoder",
                                        ));
                                    }
                                    let p = match one {
                                        NestedMeta::Meta(Meta::Path(p)) => p,
                                        other => {
                                            return Err(syn::Error::new(
                                                other.span(),
                                                "expected a path",
                                            ))
                                        }
                                    };
                                    found.decoder = Some(p);
                                }
                                other => {
                                    return Err(syn::Error::new(
                                        other.span(),
                                        r#"expected `name = "..."` or `decoder = "path::to::decode"` or `decoder(path::to::decode)`"#,
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            cfg = Some(found);
        }
    }

    Ok(cfg)
}

/// Parse `#[on_notify("account::action")]` or
/// `#[on_notify("account::action", decoder = "path::to::decode")]`.
fn parse_on_notify_attr(attrs: &[Attribute]) -> Result<Option<NotifyCfg>> {
    let mut cfg: Option<NotifyCfg> = None;

    for a in attrs {
        if is_attr(a, "on_notify") {
            let meta = a.parse_meta().map_err(|_| {
                syn::Error::new(
                    a.span(),
                    r#"expected `#[on_notify("account::action")]`"#,
                )
            })?;

            let list = match meta {
                Meta::List(list) => list,
                _ => {
                    return Err(syn::Error::new(
                        a.span(),
                        r#"expected `#[on_notify("account::action")]`"#,
                    ));
                }
            };

            let mut pattern: Option<String> = None;
            let mut decoder: Option<Path> = None;

            for n in list.nested {
                match n {
                    // bare string literal: the "account::action" pattern
                    NestedMeta::Lit(Lit::Str(s)) => {
                        if pattern.is_some() {
                            return Err(syn::Error::new(
                                s.span(),
                                "only one notify pattern is allowed",
                            ));
                        }
                        pattern = Some(s.value());
                    }
                    // decoder = "path::to::decode"
                    NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                        path,
                        lit: Lit::Str(s),
                        ..
                    })) if path.is_ident("decoder") => {
                        let p: Path = s.parse()?;
                        decoder = Some(p);
                    }
                    // decoder(path::to::decode)
                    NestedMeta::Meta(Meta::List(MetaList { path, nested, .. }))
                        if path.is_ident("decoder") =>
                    {
                        let mut it = nested.into_iter();
                        let one = it.next().ok_or_else(|| {
                            syn::Error::new(path.span(), "expected decoder path")
                        })?;
                        if it.next().is_some() {
                            return Err(syn::Error::new(
                                path.span(),
                                "expected a single path for decoder",
                            ));
                        }
                        let p = match one {
                            NestedMeta::Meta(Meta::Path(p)) => p,
                            other => {
                                return Err(syn::Error::new(other.span(), "expected a path"))
                            }
                        };
                        decoder = Some(p);
                    }
                    other => {
                        return Err(syn::Error::new(
                            other.span(),
                            r#"expected `"account::action"` and optionally `decoder = "path::to::decode"`"#,
                        ));
                    }
                }
            }

            let pattern = pattern.ok_or_else(|| {
                syn::Error::new(
                    a.span(),
                    r#"#[on_notify] requires a pattern, e.g. #[on_notify("eosio.token::transfer")]"#,
                )
            })?;

            cfg = Some(NotifyCfg { pattern, decoder });
        }
    }

    Ok(cfg)
}

/// Split `"account::action"` into account + action patterns, each of which may
/// be the wildcard `*`. Mirrors EOSIO's `on_notify` matching.
fn parse_notify_pattern(
    pattern: &str,
    span: proc_macro2::Span,
) -> Result<(NotifyPat, NotifyPat)> {
    let mut parts = pattern.splitn(2, "::");
    let acct = parts.next().unwrap_or("");
    let action = parts.next().ok_or_else(|| {
        syn::Error::new(
            span,
            r#"on_notify pattern must be of the form "account::action" (e.g. "eosio.token::transfer")"#,
        )
    })?;

    if acct.is_empty() || action.is_empty() {
        return Err(syn::Error::new(
            span,
            r#"on_notify pattern must be of the form "account::action""#,
        ));
    }

    let acct_pat = if acct == "*" {
        NotifyPat::Wildcard
    } else {
        NotifyPat::Name(acct.to_string())
    };
    let action_pat = if action == "*" {
        NotifyPat::Wildcard
    } else {
        NotifyPat::Name(action.to_string())
    };

    Ok((acct_pat, action_pat))
}

fn is_attr(a: &Attribute, want: &str) -> bool {
    let p = &a.path;
    p.is_ident(want)
        || (p.segments.len() == 2
            && p.segments.first().unwrap().ident == "contract_macros"
            && p.segments.last().unwrap().ident == want)
}

fn has_attr(attrs: &[Attribute], want: &str) -> bool {
    attrs.iter().any(|a| is_attr(a, want))
}

fn ensure_no_receiver(m: &ImplItemMethod, kind: &str) -> Result<()> {
    if m.sig.receiver().is_some() {
        return Err(syn::Error::new(
            m.sig.span(),
            format!("#[{kind}] methods must be static (no self receiver)"),
        ));
    }
    Ok(())
}

fn ensure_arg_count(m: &ImplItemMethod, n: usize, kind: &str) -> Result<()> {
    let count = m
        .sig
        .inputs
        .iter()
        .filter(|arg| matches!(arg, FnArg::Typed(_)))
        .count();
    if count != n {
        return Err(syn::Error::new(
            m.sig.span(),
            format!("#[{kind}] must have exactly {n} arguments"),
        ));
    }
    Ok(())
}

fn tuple_type_tokens(tys: &[&Type]) -> TokenStream2 {
    match tys.len() {
        0 => quote! { () },
        1 => {
            let t0 = tys[0];
            quote! { ( #t0 , ) }
        }
        _ => {
            quote! { ( #(#tys),* ) }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum ReceiverKind {
    None,
    Ref,
    MutRef,
    Value,
}

fn receiver_kind(m: &ImplItemMethod) -> ReceiverKind {
    // Detect receiver form
    if let Some(recv) = m.sig.receiver() {
        match recv {
            FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_none() => {
                ReceiverKind::Ref
            }
            FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_some() => {
                ReceiverKind::MutRef
            }
            FnArg::Receiver(_) => ReceiverKind::Value,
            _ => ReceiverKind::None,
        }
    } else {
        ReceiverKind::None
    }
}
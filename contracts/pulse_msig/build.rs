// build.rs
use serde_json::json;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::{BufWriter, Write},
    path::PathBuf,
};
use syn::{
    Attribute, Expr, ExprCall, ExprCast, ExprField, ExprLit, ExprMacro, ExprMethodCall, ExprParen,
    FnArg, ImplItem, ImplItemMethod, Item, ItemConst, ItemFn, ItemImpl, ItemStruct, Lit, Meta,
    MetaList, MetaNameValue, Pat, PatIdent, PatType, PathArguments, Type, TypePath,
};

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_path = PathBuf::from(&manifest_dir).join("src/lib.rs");

    let code = fs::read_to_string(&src_path).expect("Failed to read src/lib.rs");
    let syntax = syn::parse_file(&code).expect("Failed to parse src/lib.rs");

    let mut actions = vec![];
    let mut tables = vec![];
    let mut struct_map: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    let mut seen_table_names: HashSet<String> = HashSet::new();

    // -----------------------------------------------------------
    // 0) Parse const table/singleton definitions FIRST and map:
    //    row_type -> (table_name, kind)
    // -----------------------------------------------------------
    #[derive(Copy, Clone, Eq, PartialEq)]
    enum TableKind {
        Singleton,
        MultiIndex,
    }
    struct ParsedTableConst {
        kind: TableKind,
        row_type: String,
        name: String,
    }

    fn extract_table_type(ty: &Type) -> Option<(TableKind, String)> {
        let Type::Path(TypePath { path, .. }) = ty else { return None };
        let seg = path.segments.last()?;
        let kind = match seg.ident.to_string().as_str() {
            "SingletonDefinition" => TableKind::Singleton,
            "MultiIndexDefinition" => TableKind::MultiIndex,
            _ => return None,
        };
        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
            for ga in &ab.args {
                if let syn::GenericArgument::Type(Type::Path(tp)) = ga {
                    if let Some(last) = tp.path.segments.last() {
                        return Some((kind, last.ident.to_string()));
                    }
                }
            }
        }
        None
    }

    fn extract_table_name(expr: &Expr) -> Option<String> {
        // DFS to find name!("…") / name_raw!("…") or a plain "…"
        fn dfs(e: &Expr) -> Option<String> {
            match e {
                Expr::Macro(ExprMacro { mac, .. }) => {
                    let last = mac.path.segments.last()?.ident.to_string();
                    if last == "name" || last == "name_raw" {
                        if let Ok(s) = syn::parse2::<syn::LitStr>(mac.tokens.clone()) {
                            return Some(s.value());
                        }
                    }
                    None
                }
                Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) => Some(s.value()),
                Expr::Call(ExprCall { func, args, .. }) => {
                    if let Some(v) = dfs(func) {
                        return Some(v);
                    }
                    for a in args {
                        if let Some(v) = dfs(a) {
                            return Some(v);
                        }
                    }
                    None
                }
                Expr::MethodCall(ExprMethodCall { receiver, args, .. }) => {
                    if let Some(v) = dfs(receiver) {
                        return Some(v);
                    }
                    for a in args {
                        if let Some(v) = dfs(a) {
                            return Some(v);
                        }
                    }
                    None
                }
                Expr::Field(ExprField { base, .. }) => dfs(base),
                Expr::Paren(ExprParen { expr, .. }) => dfs(expr),
                Expr::Cast(ExprCast { expr, .. }) => dfs(expr),
                _ => None,
            }
        }
        dfs(expr)
    }

    let mut const_def_map: HashMap<String, (String, TableKind)> = HashMap::new();
    for item in &syntax.items {
        if let Item::Const(ic) = item {
            if let Some((kind, row_type)) = extract_table_type(&ic.ty) {
                if let Some(name) = extract_table_name(&ic.expr) {
                    const_def_map.insert(row_type, (name, kind));
                }
            }
        }
    }

    // -----------------------------------------------------------
    // 1) Collect user-defined structs (for ABI structs)
    //    and emit tables for those marked #[table], but:
    //    - use table name from const_def_map if present
    //    - else use #[table(name="…")] if provided
    //    - else fallback to struct_name.to_lowercase()
    // -----------------------------------------------------------
    let mut row_types_with_table_attr: HashSet<String> = HashSet::new();

    for item in &syntax.items {
        if let Item::Struct(ItemStruct {
            ident,
            fields,
            attrs,
            ..
        }) = item
        {
            let struct_name = ident.to_string();
            let mut field_entries = vec![];

            for field in fields.iter() {
                let name = field
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "_".into());
                let ty_str = rust_type_to_eos_type(strip_refs(&field.ty));
                field_entries.push(json!({ "name": name, "type": ty_str }));
            }
            struct_map.insert(struct_name.clone(), field_entries);

            if has_table_attr(attrs) {
                row_types_with_table_attr.insert(struct_name.clone());

                // read optional #[table(...)] params (for index_type fallback)
                let cfg = table_cfg_from_attrs(attrs);
                let index_type = cfg.index_type.unwrap_or_else(|| "i64".to_string());

                // Prefer table name from const definition
                let table_name = if let Some((n, _k)) = const_def_map.get(&struct_name) {
                    n.clone()
                } else if let Some(n) = cfg.name {
                    n
                } else {
                    struct_name.to_lowercase()
                };

                if seen_table_names.insert(table_name.clone()) {
                    tables.push(json!({
                        "name": table_name,
                        "type": struct_name,
                        "index_type": index_type,
                        "key_names": [],
                        "key_types": [],
                    }));
                }
            }
        }
    }

    // -----------------------------------------------------------
    // 1.5) Emit tables for const defs whose row type did NOT
    //      have a #[table] attribute (so they aren't missed).
    //      Name always comes from the const def.
    // -----------------------------------------------------------
    for (row_type, (name, _kind)) in &const_def_map {
        if seen_table_names.contains(name) {
            continue;
        }
        if !row_types_with_table_attr.contains(row_type) {
            // make sure the row struct appears in ABI "structs" even if empty
            struct_map.entry(row_type.clone()).or_insert_with(Vec::new);

            tables.push(json!({
                "name": name,
                "type": row_type,
                "index_type": "i64",
                "key_names": [],
                "key_types": [],
            }));
            seen_table_names.insert(name.clone());
        }
    }

    // -----------------------------------------------------------
    // actions (unchanged)
    // -----------------------------------------------------------
    let mut push_action = |action_name: String, params: Vec<(String, String)>| {
        let fields_json: Vec<_> = params
            .into_iter()
            .map(|(name, ty)| json!({ "name": name, "type": ty }))
            .collect();

        struct_map.insert(action_name.clone(), fields_json);

        actions.push(json!({
            "name": action_name,
            "type": action_name,
            "ricardian_contract": ""
        }));
    };

    // #[contract] impl … { #[action] fn … }
    for item in &syntax.items {
        if let Item::Impl(ItemImpl {
            attrs,
            items: impl_items,
            ..
        }) = item
        {
            if !has_contract_attr(attrs) {
                continue;
            }
            for it in impl_items {
                if let ImplItem::Method(m) = it {
                    if !has_action_attr(&m.attrs) {
                        continue;
                    }
                    let action_name = action_name_from_attrs(&m.attrs)
                        .unwrap_or_else(|| m.sig.ident.to_string());

                    let params = method_params_as_abi_fields(m);
                    push_action(action_name, params);
                }
            }
        }
    }

    // top-level #[action] fn …
    for item in &syntax.items {
        if let Item::Fn(ItemFn { attrs, sig, .. }) = item {
            if !has_action_attr(attrs) {
                continue;
            }
            let action_name =
                action_name_from_attrs(attrs).unwrap_or_else(|| sig.ident.to_string());

            let mut params = vec![];
            for (idx, arg) in sig.inputs.iter().enumerate() {
                if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
                    let name = pat_name_or_fallback(pat, idx);
                    let ty_str = rust_type_to_eos_type(strip_refs(ty));
                    params.push((name, ty_str));
                }
            }
            push_action(action_name, params);
        }
    }

    // -----------------------------------------------------------
    // Emit ABI JSON (pretty)
    // -----------------------------------------------------------
    let structs_json: Vec<_> = struct_map
        .iter()
        .map(|(name, fields)| json!({ "name": name, "base": "", "fields": fields }))
        .collect();

    let abi_json = json!({
        "version": "eosio::abi/1.1",
        "types": [],
        "structs": structs_json,
        "actions": actions,
        "tables": tables,
        "ricardian_clauses": [],
        "error_messages": [],
    });

    let out_path = PathBuf::from("./abi.json");
    let file = fs::File::create(&out_path).expect("open abi.json");
    let mut w = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut w, &abi_json).expect("write pretty json");
    w.write_all(b"\n").ok();
    println!("cargo:rustc-env=ABI_PATH={}", out_path.display());
}

/* -------------------- helpers -------------------- */

fn has_action_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| path_is(a, &["action"]))
        || attrs
            .iter()
            .any(|a| path_is(a, &["contract_macros", "action"]))
}

fn has_contract_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| path_is(a, &["contract"]))
        || attrs
            .iter()
            .any(|a| path_is(a, &["contract_macros", "contract"]))
}

fn has_table_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| path_is(a, &["table"]))
        || attrs
            .iter()
            .any(|a| path_is(a, &["contract_macros", "table"]))
}

#[derive(Default)]
struct TableCfg {
    name: Option<String>,
    index_type: Option<String>,
    key_names: Vec<String>,
    key_types: Vec<String>,
}

fn table_cfg_from_attrs(attrs: &[Attribute]) -> TableCfg {
    let mut cfg = TableCfg::default();
    for a in attrs {
        if !(path_is(a, &["table"]) || path_is(a, &["contract_macros", "table"])) {
            continue;
        }
        if let Ok(meta) = a.parse_meta() {
            if let Meta::List(MetaList { nested, .. }) = meta {
                for n in nested {
                    match n {
                        syn::NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(s),
                            ..
                        })) => {
                            if path.is_ident("name") {
                                cfg.name = Some(s.value());
                            } else if path.is_ident("index_type") {
                                cfg.index_type = Some(s.value());
                            } else if path.is_ident("key_names") {
                                cfg.key_names = s
                                    .value()
                                    .split(',')
                                    .map(|x| x.trim().to_string())
                                    .filter(|x| !x.is_empty())
                                    .collect();
                            } else if path.is_ident("key_types") {
                                cfg.key_types = s
                                    .value()
                                    .split(',')
                                    .map(|x| x.trim().to_string())
                                    .filter(|x| !x.is_empty())
                                    .collect();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    cfg
}

fn action_name_from_attrs(attrs: &[Attribute]) -> Option<String> {
    for a in attrs {
        if !(path_is(a, &["action"]) || path_is(a, &["contract_macros", "action"])) {
            continue;
        }
        if let Ok(meta) = a.parse_meta() {
            if let syn::Meta::List(list) = meta {
                for nested in list.nested {
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(kv)) = nested {
                        if kv.path.is_ident("name") {
                            if let syn::Lit::Str(s) = kv.lit {
                                return Some(s.value());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn method_params_as_abi_fields(m: &ImplItemMethod) -> Vec<(String, String)> {
    let mut out = vec![];
    let mut index = 0usize;
    for arg in &m.sig.inputs {
        match arg {
            FnArg::Receiver(_) => {}
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let name = pat_name_or_fallback(pat, index);
                let ty_str = rust_type_to_eos_type(strip_refs(ty));
                out.push((name, ty_str));
                index += 1;
            }
        }
    }
    out
}

fn pat_name_or_fallback(pat: &Pat, idx: usize) -> String {
    if let Pat::Ident(PatIdent { ident, .. }) = pat {
        ident.to_string()
    } else {
        format!("arg{}", idx)
    }
}

fn strip_refs<'a>(ty: &'a Type) -> &'a Type {
    if let Type::Reference(r) = ty {
        &r.elem
    } else {
        ty
    }
}

fn rust_type_to_eos_type(ty: &Type) -> String {
    match ty {
        Type::Path(p) => {
            let name = p.path.segments.first().unwrap().ident.to_string();
            match name.as_str() {
                "String" => "string".into(),
                "Vec" => "bytes".into(),
                "u64" => "uint64".into(),
                "u32" => "uint32".into(),
                "u16" => "uint16".into(),
                "u8" => "uint8".into(),
                "i64" => "int64".into(),
                "i32" => "int32".into(),
                "bool" => "bool".into(),
                // EOSIO-ish domain types if you use them:
                "Name" => "name".into(),
                "Asset" => "asset".into(),
                "Symbol" => "symbol".into(),
                "SymbolCode" => "symbol_code".into(),
                other => other.to_lowercase(),
            }
        }
        Type::Array(_) => "bytes".into(),
        _ => "unknown".into(),
    }
}

fn path_is(attr: &Attribute, want: &[&str]) -> bool {
    let segs: Vec<_> = attr.path.segments.iter().map(|s| s.ident.to_string()).collect();
    segs.as_slice() == want
}

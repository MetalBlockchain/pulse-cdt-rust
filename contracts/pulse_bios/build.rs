// build.rs
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env, fs,
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};
use syn::{
    Attribute, Expr, ExprCall, ExprCast, ExprField, ExprLit, ExprMacro, ExprMethodCall, ExprParen,
    FnArg, GenericArgument, ImplItem, ImplItemMethod, Item, ItemConst, ItemFn, ItemImpl,
    ItemStruct, Lit, Meta, MetaList, MetaNameValue, Pat, PatIdent, PatType, PathArguments, Type,
    TypePath,
};

fn main() {
    // --- watch all .rs files under src ---
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = PathBuf::from(&manifest_dir).join("src");

    fn gather_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                gather_rs_files(&path, out)?;
            } else if path.extension().map_or(false, |e| e == "rs") {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut rs_files = Vec::new();
    gather_rs_files(&src_dir, &mut rs_files).expect("scan src");

    // Make Cargo rebuild when any file changes
    for f in &rs_files {
        println!("cargo:rerun-if-changed={}", f.display());
    }

    // --- parse & merge all files into one syn::File-like structure ---
    let mut all_items = Vec::new();
    for f in &rs_files {
        let code =
            fs::read_to_string(f).unwrap_or_else(|e| panic!("Failed to read {}: {e}", f.display()));
        let parsed = syn::parse_file(&code)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", f.display()));
        all_items.extend(parsed.items);
    }

    // Synthetic "file" that contains the merged items from all src files
    let syntax = syn::File {
        shebang: None,
        attrs: Vec::new(),
        items: all_items,
    };

    let mut actions: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut tables: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut struct_map: BTreeMap<String, (&str, Vec<serde_json::Value>)> = BTreeMap::new();
    let mut type_map: HashMap<String, serde_json::Value> = HashMap::new();
    let mut variant_map: HashMap<String, serde_json::Value> = HashMap::new();
    let mut reciardian_contracts: HashMap<String, serde_json::Value> = HashMap::new();
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
        let Type::Path(TypePath { path, .. }) = ty else {
            return None;
        };
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
            struct_map.insert(struct_name.clone(), ("", field_entries));

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
                    ident.to_string().to_lowercase()
                };

                if seen_table_names.insert(table_name.clone()) {
                    tables.insert(
                        table_name.clone(),
                        json!({
                            "name": table_name,
                            "type": struct_name,
                            "index_type": index_type,
                            "key_names": [],
                            "key_types": [],
                        }),
                    );
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
            struct_map
                .entry(row_type.clone())
                .or_insert_with(|| ("", Vec::new()));

            tables.insert(
                name.clone(),
                json!({
                    "name": name,
                    "type": row_type,
                    "index_type": "i64",
                    "key_names": [],
                    "key_types": [],
                }),
            );
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

        struct_map.insert(action_name.clone(), ("", fields_json));

        actions.insert(
            action_name.clone(),
            json!({
                "name": action_name,
                "type": action_name,
                "ricardian_contract": ""
            }),
        );
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
                    let action_name =
                        action_name_from_attrs(&m.attrs).unwrap_or_else(|| m.sig.ident.to_string());

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

    // Normalize a bit before scanning (optional)
    let referenced = collect_referenced_types(&struct_map);

    // Inject well-known external structs that are referenced but missing
    for head in referenced {
        if !is_builtin_eos_type(&head) && !struct_map.contains_key(&head) {
            let _ = inject_well_known(&head, &mut struct_map, &mut type_map, &mut variant_map);
        }
    }

    // Make sure all referenced types are present in struct_map
    rewrite_generics_to_v1_1(&mut struct_map);

    // -----------------------------------------------------------
    // Emit ABI JSON (pretty)
    // -----------------------------------------------------------
    let types_json: Vec<_> = type_map.iter().map(|(name, value)| value).collect();
    let structs_json: Vec<_> = struct_map
        .iter()
        .map(|(name, fields)| json!({ "name": name, "base": fields.0, "fields": fields.1 }))
        .collect();
    let variants_json: Vec<_> = variant_map.iter().map(|(name, value)| value).collect();
    let actions_json: Vec<_> = actions.iter().map(|(name, value)| value).collect();
    let tables_json: Vec<_> = tables.iter().map(|(name, value)| value).collect();

    let abi_json = json!({
        "____comment": "This file was generated. DO NOT EDIT ",
        "version": "eosio::abi/1.1",
        "types": types_json,
        "structs": structs_json,
        "actions": actions_json,
        "tables": tables_json,
        "ricardian_clauses": [
            {
                "id": "UserAgreement",
                "body": "User agreement for the chain can go here."
            },
            {
                "id": "BlockProducerAgreement",
                "body": "I, {{producer}}, hereby nominate myself for consideration as an elected block producer.\n\nIf {{producer}} is selected to produce blocks by the system contract, I will sign blocks with my registered block signing keys and I hereby attest that I will keep these keys secret and secure.\n\nIf {{producer}} is unable to perform obligations under this contract I will resign my position using the unregprod action.\n\nI acknowledge that a block is 'objectively valid' if it conforms to the deterministic blockchain rules in force at the time of its creation, and is 'objectively invalid' if it fails to conform to those rules.\n\n{{producer}} hereby agrees to only use my registered block signing keys to sign messages under the following scenarios:\n\n* proposing an objectively valid block at the time appointed by the block scheduling algorithm;\n* pre-confirming a block produced by another producer in the schedule when I find said block objectively valid;\n* and, confirming a block for which {{producer}} has received pre-confirmation messages from more than two-thirds of the active block producers.\n\nI hereby accept liability for any and all provable damages that result from my:\n\n* signing two different block proposals with the same timestamp;\n* signing two different block proposals with the same block number;\n* signing any block proposal which builds off of an objectively invalid block;\n* signing a pre-confirmation for an objectively invalid block;\n* or, signing a confirmation for a block for which I do not possess pre-confirmation messages from more than two-thirds of the active block producers.\n\nI hereby agree that double-signing for a timestamp or block number in concert with two or more other block producers shall automatically be deemed malicious and cause {{producer}} to be subject to:\n\n* a fine equal to the past year of compensation received,\n* immediate disqualification from being a producer,\n* and/or other damages.\n\nAn exception may be made if {{producer}} can demonstrate that the double-signing occurred due to a bug in the reference software; the burden of proof is on {{producer}}.\n\nI hereby agree not to interfere with the producer election process. I agree to process all producer election transactions that occur in blocks I create, to sign all objectively valid blocks I create that contain election transactions, and to sign all pre-confirmations and confirmations necessary to facilitate transfer of control to the next set of producers as determined by the system contract.\n\nI hereby acknowledge that more than two-thirds of the active block producers may vote to disqualify {{producer}} in the event {{producer}} is unable to produce blocks or is unable to be reached, according to criteria agreed to among block producers.\n\nIf {{producer}} qualifies for and chooses to collect compensation due to votes received, {{producer}} will provide a public endpoint allowing at least 100 peers to maintain synchronization with the blockchain and/or submit transactions to be included. {{producer}} shall maintain at least one validating node with full state and signature checking and shall report any objectively invalid blocks produced by the active block producers. Reporting shall be via a method to be agreed to among block producers, said method and reports to be made public.\n\nThe community agrees to allow {{producer}} to authenticate peers as necessary to prevent abuse and denial of service attacks; however, {{producer}} agrees not to discriminate against non-abusive peers.\n\nI agree to process transactions on a FIFO (first in, first out) best-effort basis and to honestly bill transactions for measured execution time.\n\nI {{producer}} agree not to manipulate the contents of blocks in order to derive profit from: the order in which transactions are included, or the hash of the block that is produced.\n\nI, {{producer}}, hereby agree to disclose and attest under penalty of perjury all ultimate beneficial owners of my business entity who own more than 10% and all direct shareholders.\n\nI, {{producer}}, hereby agree to cooperate with other block producers to carry out our respective and mutual obligations under this agreement, including but not limited to maintaining network stability and a valid blockchain.\n\nI, {{producer}}, agree to maintain a website hosted at {{url}} which contains up-to-date information on all disclosures required by this contract.\n\nI, {{producer}}, agree to set the location value of {{location}} such that {{producer}} is scheduled with minimal latency between my previous and next peer.\n\nI, {{producer}}, agree to maintain time synchronization within 10 ms of global atomic clock time, using a method agreed to among block producers.\n\nI, {{producer}}, agree not to produce blocks before my scheduled time unless I have received all blocks produced by the prior block producer.\n\nI, {{producer}}, agree not to publish blocks with timestamps more than 500ms in the future unless the prior block is more than 75% full by either NET or CPU bandwidth metrics.\n\nI, {{producer}}, agree not to set the RAM supply to more RAM than my nodes contain and to resign if I am unable to provide the RAM approved by more than two-thirds of active block producers, as shown in the system parameters."
            }
        ],
        "variants": variants_json,
    });

    let out_path = PathBuf::from("./abi.json");
    let file = fs::File::create(&out_path).expect("open abi.json");
    let mut w = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut w, &abi_json).expect("write pretty json");
    w.write_all(b"\n").ok();
    println!("cargo:rustc-env=ABI_PATH={}", out_path.display());
}

/* -------------------- helpers -------------------- */

// Strip trailing v1.1 suffixes repeatedly and return the core type.
fn strip_suffixes(mut s: &str) -> (&str, &str) {
    // returns (core, suffixes) where suffixes preserve original order
    let mut end = s.len();
    let mut suffix = String::new();
    loop {
        if s[..end].ends_with("[]") {
            end -= 2;
            suffix.push_str("[]");
            continue;
        }
        if end > 0 && s[..end].ends_with('?') {
            end -= 1;
            suffix.push('?');
            continue;
        }
        if end > 0 && s[..end].ends_with('$') {
            end -= 1;
            suffix.push('$');
            continue;
        }
        break;
    }
    let core = &s[..end];
    // suffix collected right-to-left; reverse into original right-to-left order
    let rev = suffix.chars().rev().collect::<String>();
    (core, Box::leak(rev.into_boxed_str()))
}

fn is_builtin_eos_type(t: &str) -> bool {
    // Simple check; extend as needed. Also treat suffix containers as builtins.
    const PRIMS: &[&str] = &[
        "string",
        "name",
        "bool",
        "symbol",
        "symbol_code",
        "asset",
        "varuint32",
        "varint32",
        "int8",
        "int16",
        "int32",
        "int64",
        "int128",
        "uint8",
        "uint16",
        "uint32",
        "uint64",
        "uint128",
        "float32",
        "float64",
        "time_point",
        "time_point_sec",
        "block_timestamp_type",
        "checksum160",
        "checksum256",
        "checksum512",
        "public_key",
        "signature",
        "permission_level",
        "bytes",
    ];
    let (core, _suf) = strip_suffixes(t);
    PRIMS.contains(&core)
}

fn inject_well_known(
    name: &str,
    struct_map: &mut BTreeMap<String, (&str, Vec<Value>)>,
    type_map: &mut HashMap<String, Value>,
    variant_map: &mut HashMap<String, Value>,
) -> bool {
    match name {
        // Canonical EOSIO authority family
        "key_weight" if !struct_map.contains_key("key_weight") => {
            struct_map.insert(
                "key_weight".into(),
                (
                    "",
                    vec![
                        json!({"name":"key",    "type":"public_key"}),
                        json!({"name":"weight", "type":"uint16"}),
                    ],
                ),
            );
            true
        }
        "permission_level" if !struct_map.contains_key("permission_level") => {
            struct_map.insert(
                "permission_level".into(),
                (
                    "",
                    vec![
                        json!({"name":"actor",      "type":"name"}),
                        json!({"name":"permission", "type":"name"}),
                    ],
                ),
            );
            true
        }
        "permission_level_weight" if !struct_map.contains_key("permission_level_weight") => {
            // Ensure dependencies exist too
            let _ = inject_well_known("permission_level", struct_map, type_map, variant_map);
            struct_map.insert(
                "permission_level_weight".into(),
                (
                    "",
                    vec![
                        json!({"name":"permission","type":"permission_level"}),
                        json!({"name":"weight",    "type":"uint16"}),
                    ],
                ),
            );
            true
        }
        "wait_weight" if !struct_map.contains_key("wait_weight") => {
            struct_map.insert(
                "wait_weight".into(),
                (
                    "",
                    vec![
                        json!({"name":"wait_sec","type":"uint32"}),
                        json!({"name":"weight",  "type":"uint16"}),
                    ],
                ),
            );
            true
        }
        "authority" if !struct_map.contains_key("authority") => {
            // Ensure dependencies exist too
            let _ = inject_well_known("key_weight", struct_map, type_map, variant_map);
            let _ = inject_well_known("permission_level_weight", struct_map, type_map, variant_map);
            let _ = inject_well_known("wait_weight", struct_map, type_map, variant_map);
            struct_map.insert(
                "authority".into(),
                (
                    "",
                    vec![
                        json!({"name":"threshold","type":"uint32"}),
                        json!({"name":"keys",     "type":"key_weight[]"}),
                        json!({"name":"accounts", "type":"permission_level_weight[]"}),
                        json!({"name":"waits",    "type":"wait_weight[]"}),
                    ],
                ),
            );
            true
        }
        "block_signing_authority" if !struct_map.contains_key("block_signing_authority") => {
            // Ensure dependencies exist too
            let _ = inject_well_known("key_weight", struct_map, type_map, variant_map);
            struct_map.insert(
                "block_signing_authority_v0".into(),
                (
                    "",
                    vec![
                        json!({"name":"threshold","type":"uint32"}),
                        json!({"name":"keys",     "type":"key_weight[]"}),
                    ],
                ),
            );
            variant_map.insert(
                "variant_block_signing_authority_v0".into(),
                json!({
                    "name": "variant_block_signing_authority_v0",
                    "types": ["block_signing_authority_v0"]
                }),
            );
            type_map.insert(
                "variant_block_signing_authority_v0".into(),
                json!({
                    "new_type_name": "block_signing_authority",
                    "type": "variant_block_signing_authority_v0"
                }),
            );
            true
        }
        "producer_key" if !struct_map.contains_key("producer_key") => {
            struct_map.insert(
                "producer_key".into(),
                (
                    "",
                    vec![
                        json!({"name":"producer_name","type":"name"}),
                        json!({"name":"block_signing_key","type":"public_key"}),
                    ],
                ),
            );
            true
        }
        "producer_schedule" if !struct_map.contains_key("producer_schedule") => {
            // Ensure dependencies exist too
            let _ = inject_well_known("producer_key", struct_map, type_map, variant_map);
            struct_map.insert(
                "producer_schedule".into(),
                (
                    "",
                    vec![
                        json!({"name":"version","type":"uint32"}),
                        json!({"name":"producers","type":"producer_key[]"}),
                    ],
                ),
            );
            true
        }
        "block_header" if !struct_map.contains_key("block_header") => {
            // Ensure dependencies exist too
            let _ = inject_well_known("producer_schedule", struct_map, type_map, variant_map);
            struct_map.insert(
                "block_header".into(),
                (
                    "",
                    vec![
                        json!({"name":"timestamp","type":"uint32"}),
                        json!({"name":"producer","type":"name"}),
                        json!({"name":"confirmed","type":"uint16"}),
                        json!({"name":"previous","type":"checksum256"}),
                        json!({"name":"transaction_mroot","type":"checksum256"}),
                        json!({"name":"action_mroot","type":"checksum256"}),
                        json!({"name":"schedule_version","type":"uint32"}),
                        json!({"name":"new_producers","type":"producer_schedule?"}),
                    ],
                ),
            );
            true
        }
        "transaction_header" if !struct_map.contains_key("transaction_header") => {
            struct_map.insert(
                "transaction_header".into(),
                (
                    "",
                    vec![
                        json!({"name":"expiration","type":"time_point_sec"}),
                        json!({"name":"ref_block_num","type":"uint16"}),
                        json!({"name":"ref_block_prefix","type":"uint32"}),
                        json!({"name":"max_net_usage_words","type":"varuint32"}),
                        json!({"name":"max_cpu_usage","type":"uint8"}),
                        json!({"name":"delay_sec","type":"varuint32"}),
                    ],
                ),
            );
            true
        }
        "transaction" if !struct_map.contains_key("transaction") => {
            // Ensure dependencies exist too
            let _ = inject_well_known("transaction_header", struct_map, type_map, variant_map);
            let _ = inject_well_known("action", struct_map, type_map, variant_map);
            let _ = inject_well_known("extension", struct_map, type_map, variant_map);
            struct_map.insert(
                "transaction".into(),
                (
                    "transaction_header",
                    vec![
                        json!({"name":"header","type":"transaction_header"}),
                        json!({"name":"context_free_actions","type":"action[]"}),
                        json!({"name":"actions","type":"action[]"}),
                        json!({"name":"transaction_extensions","type":"extension[]"}),
                    ],
                ),
            );
            true
        }
        "action" if !struct_map.contains_key("action") => {
            // Ensure dependencies exist too
            let _ = inject_well_known("permission_level", struct_map, type_map, variant_map);
            struct_map.insert(
                "action".into(),
                (
                    "",
                    vec![
                        json!({"name":"account","type":"name"}),
                        json!({"name":"name","type":"name"}),
                        json!({"name":"authorization","type":"permission_level[]"}),
                        json!({"name":"data","type":"bytes"}),
                    ],
                ),
            );
            true
        }
        "extension" if !struct_map.contains_key("extension") => {
            struct_map.insert(
                "extension".into(),
                (
                    "",
                    vec![
                        json!({"name":"type","type":"uint16"}),
                        json!({"name":"data","type":"bytes"}),
                    ],
                ),
            );
            true
        }
        _ => false,
    }
}

fn collect_referenced_types(struct_map: &BTreeMap<String, (&str, Vec<Value>)>) -> Vec<String> {
    let mut out = std::collections::BTreeSet::new();
    for fields in struct_map.values() {
        for f in fields.1.clone() {
            if let Some(ts) = f.get("type").and_then(|v| v.as_str()) {
                // Remove suffixes first ([], ?, $). If generic (pair<…>, map<…>) remains,
                // peel at '<' to get the head.
                let (core, _) = strip_suffixes(ts);
                let head = core.split('<').next().unwrap_or(core).trim();
                out.insert(head.to_string());
            }
        }
    }
    out.into_iter().collect()
}

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
        // e.g. String, Vec<T>, Option<T>, HashMap<K,V>, my::types::Id, etc.
        Type::Path(p) => {
            let seg = p.path.segments.last().expect("at least one segment");
            let name = seg.ident.to_string();

            // Helper: pull generic type arguments
            let mut gen_types = || -> Vec<&Type> {
                if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                    ab.args
                        .iter()
                        .filter_map(|a| match a {
                            GenericArgument::Type(t) => Some(t),
                            _ => None,
                        })
                        .collect()
                } else {
                    Vec::new()
                }
            };

            match name.as_str() {
                // Builtins
                "String" => "string".into(),
                "str" => "string".into(),

                "u128" => "uint128".into(),
                "u64" => "uint64".into(),
                "u32" => "uint32".into(),
                "u16" => "uint16".into(),
                "u8" => "uint8".into(),
                "i128" => "int128".into(),
                "i64" => "int64".into(),
                "i32" => "int32".into(),
                "i16" => "int16".into(),
                "i8" => "int8".into(),
                "f32" => "float32".into(),
                "f64" => "float64".into(),
                "bool" => "bool".into(),

                // Containers -> v1.1 suffix grammar
                "Vec" | "BTreeSet" => {
                    let args = gen_types();
                    if let Some(inner) = args.first() {
                        let inner_ty = rust_type_to_eos_type(inner);
                        if inner_ty == "uint8" || inner_ty == "bytes" {
                            "bytes".into() // Vec<u8> or Vec<Bytes> → bytes
                        } else {
                            format!("{inner_ty}[]")
                        }
                    } else {
                        "unknown[]".into()
                    }
                }
                "Option" | "Optional" => {
                    let args = gen_types();
                    if let Some(inner) = args.first() {
                        let inner_ty = rust_type_to_eos_type(inner);
                        format!("{inner_ty}?")
                    } else {
                        "unknown?".into()
                    }
                }
                "HashMap" | "BTreeMap" | "Map" => {
                    // Keep as map<K,V> for now; we'll rewrite to entry[] later (and synthesize the entry struct).
                    let args = gen_types();
                    if args.len() == 2 {
                        let k = rust_type_to_eos_type(args[0]);
                        let v = rust_type_to_eos_type(args[1]);
                        format!("map<{k},{v}>")
                    } else {
                        "map<unknown,unknown>".into()
                    }
                }

                // EOSIO-ish domain types
                "Name" => "name".into(),
                "Asset" => "asset".into(),
                "Symbol" => "symbol".into(),
                "SymbolCode" => "symbol_code".into(),
                "TimePoint" => "time_point".into(),
                "TimePointSec" => "time_point_sec".into(),
                "BlockTimestamp" | "BlockTimestampType" => "block_timestamp_type".into(),
                "Checksum256" => "checksum256".into(),
                "Checksum160" => "checksum160".into(),
                "Checksum512" => "checksum512".into(),
                "PublicKey" => "public_key".into(),
                "Signature" => "signature".into(),
                "VarUint32" => "varuint32".into(),
                "VarInt32" => "varint32".into(),

                // Project-specific aliases
                "Id" => "checksum256".into(), // block id type in your codebase
                "Bytes" => "bytes".into(),

                // Well known types
                "Authority" => "authority".into(),
                "BlockHeader" => "block_header".into(),
                "PermissionLevel" => "permission_level".into(),
                "KeyWeight" => "key_weight".into(),
                "PermissionLevelWeight" => "permission_level_weight".into(),
                "WaitWeight" => "wait_weight".into(),
                "BlockSigningAuthority" => "block_signing_authority".into(),
                "Transaction" => "transaction".into(),

                // Default: KEEP THE NAME (don't lowercase!) so custom structs match exactly.
                other => other.to_string(),
            }
        }

        // &[T] or &str
        Type::Reference(r) => {
            let inner = rust_type_to_eos_type(&r.elem);
            if inner == "uint8" || inner == "bytes" || inner == "string" {
                inner
            } else {
                format!("{inner}[]")
            }
        }

        // [T] slice
        Type::Slice(s) => {
            let inner = rust_type_to_eos_type(&s.elem);
            if inner == "uint8" || inner == "bytes" {
                "bytes".into()
            } else {
                format!("{inner}[]")
            }
        }

        // [T; N] array — no fixed-size arrays in ABI, treat like vector / bytes
        Type::Array(a) => {
            let inner = rust_type_to_eos_type(&a.elem);
            if inner == "uint8" || inner == "bytes" {
                "bytes".into()
            } else {
                format!("{inner}[]")
            }
        }

        // (A, B) → keep as pair<A,B> for now; we'll synthesize a struct later.
        Type::Tuple(t) => {
            if t.elems.len() == 2 {
                let a = rust_type_to_eos_type(&t.elems[0]);
                let b = rust_type_to_eos_type(&t.elems[1]);
                format!("pair<{a},{b}>")
            } else {
                "unknown".into()
            }
        }

        _ => "unknown".into(),
    }
}

fn path_is(attr: &Attribute, want: &[&str]) -> bool {
    let segs: Vec<_> = attr
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect();
    segs.as_slice() == want
}

// ---------- Normalization to ABI v1.1 (suffix grammar) ----------

// Parse "map<K,V>" where K/V may themselves contain angle brackets.
fn parse_map_types(s: &str) -> Option<(String, String)> {
    let s = s.strip_prefix("map<")?.strip_suffix('>')?;
    let mut depth = 0usize;
    let mut split_at = None;
    for (i, ch) in s.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                split_at = Some(i);
                break;
            }
            _ => {}
        }
    }
    let idx = split_at?;
    let (k, v) = s.split_at(idx);
    let v = &v[1..]; // skip comma
    Some((k.trim().to_string(), v.trim().to_string()))
}

// Parse "vector<T>" or "optional<T>" (single generic).
fn parse_single_generic<'a>(s: &'a str, head: &str) -> Option<String> {
    let pref = format!("{head}<");
    if !s.starts_with(&pref) || !s.ends_with('>') {
        return None;
    }
    let mut depth = 0usize;
    let inner = &s[pref.len()..s.len() - 1];
    for ch in inner.chars() {
        match ch {
            '<' => depth += 1,
            '>' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    Some(inner.trim().to_string())
}

fn sanitize_for_ident(t: &str) -> String {
    let mut out = String::with_capacity(t.len());
    for ch in t.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => out.push(ch),
            '<' | '>' | ',' | ' ' | ':' | '&' | '[' | ']' | '-' | '?' | '$' => out.push('_'),
            _ => {}
        }
    }
    // collapse repeated '_'
    let mut collapsed = String::with_capacity(out.len());
    let mut prev_us = false;
    for c in out.chars() {
        if c == '_' {
            if !prev_us {
                collapsed.push(c);
            }
            prev_us = true;
        } else {
            collapsed.push(c);
            prev_us = false;
        }
    }
    collapsed.trim_matches('_').to_string()
}

fn ensure_map_entry_struct(
    have: &HashSet<String>,
    scheduled: &mut HashSet<String>,
    to_add: &mut Vec<(String, Vec<Value>)>,
    k: &str,
    v: &str,
) -> String {
    let name = format!("pair_{}_{}", sanitize_for_ident(k), sanitize_for_ident(v));
    if !have.contains(&name) && scheduled.insert(name.clone()) {
        to_add.push((
            name.clone(),
            vec![
                json!({"name": "key",   "type": k}),
                json!({"name": "value", "type": v}),
            ],
        ));
    }
    name
}

fn ensure_pair_struct(
    have: &HashSet<String>,
    scheduled: &mut HashSet<String>,
    to_add: &mut Vec<(String, Vec<Value>)>,
    a: &str,
    b: &str,
) -> String {
    // Name style: pair_<A>_<B>, e.g. pair_time_point_sec_int64
    let name = format!("pair_{}_{}", sanitize_for_ident(a), sanitize_for_ident(b));
    if !have.contains(&name) && scheduled.insert(name.clone()) {
        to_add.push((
            name.clone(),
            vec![
                json!({"name": "key",   "type": a}),
                json!({"name": "value", "type": b}),
            ],
        ));
    }
    name
}

// Rewrite any lingering generics to v1.1 suffix style and synthesize helper structs.
fn rewrite_generics_to_v1_1(struct_map: &mut BTreeMap<String, (&str, Vec<Value>)>) {
    // Snapshot existing struct names
    let have: HashSet<String> = struct_map.keys().cloned().collect();
    let mut scheduled: HashSet<String> = HashSet::new();
    let mut to_add: Vec<(String, Vec<Value>)> = Vec::new();

    for (_name, fields) in struct_map.iter_mut() {
        for f in fields.1.iter_mut() {
            let Some(ts) = f.get_mut("type") else {
                continue;
            };
            let Some(mut tstr) = ts.as_str().map(|s| s.to_string()) else {
                continue;
            };

            // 1) Normalize single-arg generics first (defensive)
            if let Some(inner) = parse_single_generic(&tstr, "vector") {
                tstr = format!("{inner}[]");
            }
            if let Some(inner) = parse_single_generic(&tstr, "optional") {
                tstr = format!("{inner}?");
            }

            // 2) Pull off trailing suffixes so we can rewrite the core cleanly.
            let (mut core, suf) = {
                // We need owned strings to recombine later
                let (c, s) = strip_suffixes(&tstr);
                (c.to_string(), s.to_string())
            };

            // 3) Rewrite map<…> → map_entry_…[] (preserve existing suffixes; ensure one array)
            if core.starts_with("map<") {
                if let Some((k, v)) = parse_map_types(&core) {
                    let entry = ensure_map_entry_struct(&have, &mut scheduled, &mut to_add, &k, &v);
                    // ensure array on the entry
                    let needs_array = !suf.contains("[]");
                    let mut new_suf = suf.clone();
                    if needs_array {
                        new_suf.push_str("[]");
                    }
                    tstr = format!("{entry}{new_suf}");
                    *ts = json!(tstr);
                    continue;
                }
            }

            // 4) Rewrite pair<…> → pair_A_B (preserve suffixes exactly)
            if core.starts_with("pair<") {
                if let Some(inner) = core.strip_prefix("pair<").and_then(|s| s.strip_suffix('>')) {
                    // split inner at top-level comma
                    let mut depth = 0usize;
                    let mut split_at = None;
                    for (i, ch) in inner.char_indices() {
                        match ch {
                            '<' => depth += 1,
                            '>' => depth = depth.saturating_sub(1),
                            ',' if depth == 0 => {
                                split_at = Some(i);
                                break;
                            }
                            _ => {}
                        }
                    }
                    if let Some(idx) = split_at {
                        let (a, b) = inner.split_at(idx);
                        let a = a.trim().to_string();
                        let b = b[1..].trim().to_string(); // skip comma
                        let pair_name =
                            ensure_pair_struct(&have, &mut scheduled, &mut to_add, &a, &b);
                        tstr = format!("{pair_name}{suf}");
                        *ts = json!(tstr);
                        continue;
                    }
                }
            }

            // 5) Write back if changed by step (1)
            *ts = json!(tstr);
        }
    }

    // Now insert synthesized helper structs
    for (name, fields) in to_add {
        struct_map.insert(name, ("", fields));
    }
}

fn validate_name(name: &str) -> Result<(), &'static str> {
    let bytes = name.as_bytes();

    // EOS name rules:
    // - 1 to 12 characters
    // - lowercase a-z, digits 1-5, and dot (.)
    // - if 13 chars (legacy), only '.' allowed at last position
    // - cannot start or end with '.'
    if bytes.is_empty() || bytes.len() > 13 {
        return Err("name must be 1–13 characters long");
    }

    if bytes[0] == b'.' || bytes[bytes.len() - 1] == b'.' {
        return Err("name cannot start or end with '.'");
    }

    for (i, &c) in bytes.iter().enumerate() {
        let valid = (b'a'..=b'z').contains(&c) || (b'1'..=b'5').contains(&c) || c == b'.';
        if !valid {
            return Err("invalid character in name");
        }
        // last char rule for 13-char names
        if bytes.len() == 13 && i == 12 && c != b'.' {
            return Err("13th character must be '.'");
        }
    }

    Ok(())
}

use std::{env, fs, path::PathBuf};
use syn::{Attribute, FnArg, Item, ItemFn, ItemStruct, PatType, Type};
use serde_json::json;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_path = PathBuf::from(manifest_dir.clone()).join("src/lib.rs");

    let code = fs::read_to_string(&src_path).expect("Failed to read lib.rs");
    let syntax = syn::parse_file(&code).expect("Failed to parse lib.rs");

    let mut actions = vec![];
    let mut struct_map = std::collections::HashMap::new();

    for item in &syntax.items {
        if let Item::Struct(ItemStruct { ident, fields, .. }) = item {
            let struct_name = ident.to_string();
            let mut field_entries = vec![];

            for field in fields.iter() {
                let name = field.ident.as_ref().unwrap().to_string();
                let ty_str = rust_type_to_eos_type(&field.ty);
                field_entries.push(json!({
                    "name": name,
                    "type": ty_str
                }));
            }

            struct_map.insert(struct_name.clone(), field_entries);
        }
    }

    for item in &syntax.items {
        if let Item::Fn(ItemFn { attrs, sig, .. }) = item {
            if !has_action_attribute(attrs) {
                continue;
            }

            let func_name = sig.ident.to_string();

            // Get first argument and ensure it's a struct
            if let Some(FnArg::Typed(PatType { ty, .. })) = sig.inputs.iter().nth(0) {
                if let Type::Path(type_path) = &**ty {
                    let struct_ident = &type_path.path.segments.first().unwrap().ident;
                    let struct_name = struct_ident.to_string();

                    let fields = struct_map.get(&struct_name).cloned().unwrap_or_default();

                    // Add struct ABI entry
                    actions.push(json!({
                        "name": func_name,
                        "type": func_name,
                        "ricardian_contract": ""
                    }));

                    struct_map.insert(func_name.clone(), fields); // associate func_name as struct too
                }
            }
        }
    }

    // Emit structs section
    let structs_json: Vec<_> = struct_map
        .iter()
        .map(|(name, fields)| {
            json!({
                "name": name,
                "base": "",
                "fields": fields
            })
        })
        .collect();

    let abi_json = json!({
        "version": "eosio::abi/1.1",
        "types": [],
        "structs": structs_json,
        "actions": actions,
        "tables": []
    });

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let abi_path = out_dir.join("abi.json");

    fs::write(&abi_path, abi_json.to_string()).expect("Failed to write ABI");

    // Optionally make it available to your crate:
    println!("cargo:rustc-env=ABI_PATH={}", abi_path.display());
}

fn has_action_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path.is_ident("action")
    })
}

fn rust_type_to_eos_type(ty: &Type) -> String {
    if let Type::Path(p) = ty {
        let name = p.path.segments.first().unwrap().ident.to_string();
        match name.as_str() {
            "String" => "string".into(),
            "u64" => "uint64".into(),
            "u32" => "uint32".into(),
            "u16" => "uint16".into(),
            "u8" => "uint8".into(),
            "i64" => "int64".into(),
            "i32" => "int32".into(),
            "bool" => "bool".into(),
            other => other.to_lowercase(),
        }
    } else {
        "unknown".into()
    }
}
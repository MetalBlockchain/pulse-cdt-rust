use alloc::vec::Vec;
use proc_macro2::Span;
use syn::{
    Attribute, Ident, Lit, LitStr,
    Meta::{self, List},
    NestedMeta, Path,
};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

impl PartialEq<Symbol> for Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

pub const PULSE: Symbol = Symbol("pulse");
pub const CRATE_PATH: Symbol = Symbol("crate_path");

pub fn get_pulse_meta_items(attr: &syn::Attribute) -> Result<Vec<syn::NestedMeta>, ()> {
    if attr.path != PULSE {
        return Ok(Vec::new());
    }

    match attr.parse_meta() {
        Ok(List(meta)) => Ok(meta.nested.into_iter().collect()),
        _ => Err(()),
    }
}

pub fn get_root_path(attrs: &[Attribute]) -> Path {
    for meta_item in attrs.iter().flat_map(get_pulse_meta_items).flatten() {
        match meta_item {
            NestedMeta::Meta(Meta::NameValue(m)) if m.path == CRATE_PATH => match m.lit {
                Lit::Str(string) => {
                    if let Ok(path) = string.parse_with(Path::parse_mod_style) {
                        return path;
                    } else {
                        panic!(
                            "`#[pulse(crate_path = \"...\")]` received an \
                                 invalid path"
                        );
                    }
                }
                _ => {
                    panic!("invalid pulse crate path");
                }
            },
            _ => continue,
        }
    }
    LitStr::new("::pulse_cdt", Span::call_site())
        .parse_with(Path::parse_mod_style)
        .unwrap()
}

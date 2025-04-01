use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::{Error as ParseError, Parse, ParseStream, Result as ParseResult},
    Data, DeriveInput, Fields, Generics, Ident, Lit, LitStr, Meta, NestedMeta,
    Path, Type,
};

use crate::internal::{get_pulse_meta_items, Attr, BoolAttr, CRATE_PATH, PRIMARY_KEY, SECONDARY_KEY, SINGLETON, TABLE_NAME};

pub struct PrimaryKey {
    ident: Ident,
}

pub struct SecondaryKey {
    ident: Ident,
    ty: Type,
}

impl SecondaryKey {
    fn by_ident(&self) -> Ident {
        Ident::new(format!("by_{}", self.ident).as_str(), self.ident.span())
    }
}

pub struct Table {
    name: LitStr,
    ident: Ident,
    generics: Generics,
    primary_key: PrimaryKey,
    secondary_keys: Vec<SecondaryKey>,
    crate_path: Option<Path>,
}

pub struct Singleton {
    name: LitStr,
    ident: Ident,
    generics: Generics,
    crate_path: Option<Path>,
}

impl Parse for DeriveTable {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let input = input.parse::<DeriveInput>()?;

        // Container attributes
        let mut table_name: Attr<LitStr> = Attr::none(TABLE_NAME);
        let mut singleton: BoolAttr = BoolAttr::none(SINGLETON);
        let mut crate_path: Attr<Path> = Attr::none(CRATE_PATH);

        // Get container attributes
        for meta_item in
            input.attrs.iter().flat_map(get_pulse_meta_items).flatten()
        {
            match meta_item {
                // Parse `#[pulse(table_name = "test")]`
                NestedMeta::Meta(Meta::NameValue(m))
                    if m.path == TABLE_NAME =>
                {
                    if let Lit::Str(lit) = m.lit {
                        table_name.set(lit.clone(), lit)?;
                    } else {
                        return Err(ParseError::new_spanned(
                            m,
                            "`#[pulse(table_name = \"...\")]` must use a \
                             string literal",
                        ));
                    }
                }
                // Parse `#[pulse(singleton)]
                NestedMeta::Meta(Meta::Path(word)) if word == SINGLETON => {
                    singleton.set_true(word)?;
                }
                // Parse `#[pulse(crate_path = "crate")]`
                NestedMeta::Meta(Meta::NameValue(m))
                    if m.path == CRATE_PATH =>
                {
                    match m.lit {
                        Lit::Str(string) => {
                            match string.parse_with(Path::parse_mod_style) {
                                Ok(path) => {
                                    crate_path.set(path.clone(), path)?;
                                }
                                Err(_) => {
                                    return Err(ParseError::new_spanned(
                                        m.path,
                                        "`#[pulse(crate_path = \"...\")]` \
                                         received an invalid path",
                                    ))
                                }
                            }
                        }
                        _ => {
                            return Err(ParseError::new_spanned(
                                m.path,
                                "`#[pulse(crate_path = \"...\")]` expected a \
                                 string literal",
                            ))
                        }
                    }
                }
                // Error
                NestedMeta::Meta(meta_item) => {
                    let path = meta_item
                        .path()
                        .into_token_stream()
                        .to_string()
                        .replace(' ', "");
                    return Err(ParseError::new_spanned(
                        meta_item,
                        format!("unknown pulse container attribute `{}`", path),
                    ));
                }
                // Error
                NestedMeta::Lit(lit) => {
                    return Err(ParseError::new_spanned(
                        lit,
                        "unexpected literal in pulse container attribute",
                    ));
                }
            }
        }

        let table_name = match table_name.get() {
            Some(t) => t,
            None => {
                return Err(ParseError::new(
                    input.ident.span(),
                    "`#[pulse(table_name = \"...\")]` must be set when \
                     deriving from `pulse::Table`",
                ))
            }
        };

        // Field attributes
        let mut primary_key: Attr<PrimaryKey> = Attr::none(PRIMARY_KEY);
        let mut secondary_keys: Vec<SecondaryKey> = Vec::new();
        match input.data {
            Data::Struct(data) => match data.fields {
                // Structs with named fields
                Fields::Named(fields) => {
                    for field in fields.named.into_iter() {
                        let mut is_primary = false;
                        let mut is_secondary = false;
                        for field_attr in field
                            .attrs
                            .iter()
                            .flat_map(get_pulse_meta_items)
                            .flatten()
                        {
                            match field_attr {
                                // Parse `#[pulse(primary_key)]`
                                NestedMeta::Meta(Meta::Path(word))
                                    if word == PRIMARY_KEY =>
                                {
                                    is_primary = true;
                                }
                                // Parse `#[pulse(secondary_key)]`
                                NestedMeta::Meta(Meta::Path(word))
                                    if word == SECONDARY_KEY =>
                                {
                                    is_secondary = true;
                                }
                                // Error
                                NestedMeta::Meta(meta_item) => {
                                    let path = meta_item
                                        .path()
                                        .into_token_stream()
                                        .to_string()
                                        .replace(' ', "");
                                    return Err(ParseError::new_spanned(
                                        meta_item,
                                        format!(
                                            "unknown pulse field attribute \
                                             `{}`",
                                            path
                                        ),
                                    ));
                                }
                                // Error
                                NestedMeta::Lit(lit) => {
                                    return Err(ParseError::new_spanned(
                                        lit,
                                        "unexpected literal in pulse field \
                                         attribute",
                                    ));
                                }
                            }
                        }
                        let ident = field.ident.clone().unwrap();
                        match (is_primary, is_secondary) {
                            (true, false) => {
                                primary_key.set(field, PrimaryKey { ident })?
                            }
                            (false, true) => {
                                secondary_keys.push(SecondaryKey {
                                    ident,
                                    ty: field.ty,
                                })
                            }
                            (true, true) => {
                                return Err(ParseError::new_spanned(
                                    field,
                                    "cannot use both `#[pulse(primary_key)]` \
                                     and `#[pulse(secondary_key)]` on the \
                                     same field",
                                ));
                            }
                            (false, false) => (),
                        }
                    }
                }
                // Error
                Fields::Unnamed(fields) => {
                    return Err(ParseError::new_spanned(
                        fields,
                        "deriving `pulse::Table` from structs with unnamed \
                         fields is not currently supported",
                    ));
                }
                // Error
                Fields::Unit => {
                    return Err(ParseError::new(
                        input.ident.span(),
                        "deriving `pulse::Table` from unit structs is not \
                         supported",
                    ));
                }
            },
            // Error
            Data::Enum(_data) => {
                return Err(ParseError::new(
                    input.ident.span(),
                    "deriving `pulse::Table` with enums is not currently \
                     supported",
                ));
            }
            // Error
            Data::Union(_data) => {
                return Err(ParseError::new(
                    input.ident.span(),
                    "deriving `pulse::Table` with unions is not supported",
                ));
            }
        }

        let primary_key = primary_key.get_with_tokens();

        if singleton.get() {
            if let Some((ts, _)) = &primary_key {
                return Err(ParseError::new_spanned(
                    ts,
                    "`#[pulse(primary_key)]` cannot be used with \
                     `#[pulse(singleton)]`",
                ));
            }

            return Ok(DeriveTable::Singleton(Singleton {
                ident: input.ident,
                generics: input.generics,
                name: table_name,
                crate_path: None,
            }));
        }

        let primary_key = match primary_key {
            Some((_, pk)) => pk,
            None => {
                return Err(ParseError::new(
                    input.ident.span(),
                    "`#[pulse(primary_key)]` must be set for a field when \
                     deriving from `pulse::Table`",
                ))
            }
        };

        Ok(DeriveTable::Table(Table {
            name: table_name,
            ident: input.ident,
            generics: input.generics,
            primary_key,
            secondary_keys,
            crate_path: None,
        }))
    }
}

pub enum DeriveTable {
    Table(Table),
    Singleton(Singleton),
}

impl ToTokens for Table {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (impl_generics, ty_generics, where_clause) =
            &self.generics.split_for_impl();

        let mut secondary_keys_expanded = quote!();
        let mut secondary_keys_constructors = quote!();
        let default_path = LitStr::new("::pulse", Span::call_site())
            .parse_with(Path::parse_mod_style)
            .unwrap();
        let pulse = &self.crate_path.as_ref().unwrap_or(&default_path);

        for i in 0..16 {
            match self.secondary_keys.get(i) {
                Some(sk) => {
                    let ident = &sk.ident;
                    secondary_keys_expanded = quote! {
                        #secondary_keys_expanded
                        Some(#pulse::SecondaryKey::from(row.#ident)),
                    };
                    let ty = &sk.ty;
                    let by_ident = sk.by_ident();
                    secondary_keys_constructors = quote! {
                        #secondary_keys_constructors

                        #[inline]
                        pub fn #by_ident<C, S>(code: C, scope: S) -> #pulse::SecondaryTableIndex<#ty, Self>
                        where
                            C: Into<#pulse::Name>,
                            S: Into<#pulse::Name>,
                        {
                            #pulse::SecondaryTableIndex::new(code, scope, Self::NAME, #i)
                        }
                    };
                }
                None => {
                    secondary_keys_expanded = quote! {
                        #secondary_keys_expanded
                        None,
                    };
                }
            };
        }

        let table_name = &self.name;
        let name = &self.ident;
        let primary_key = &self.primary_key.ident;

        let expanded = quote! {
            #[automatically_derived]
            impl #impl_generics #pulse::Table for #name #ty_generics #where_clause {
                const NAME: #pulse::Name = #pulse::Name::new(#pulse::name!(#table_name));

                type Row = Self;

                #[inline]
                fn primary_key(row: &Self::Row) -> u64 {
                    row.#primary_key.into()
                }

                #[inline]
                fn secondary_keys(row: &Self::Row) -> #pulse::SecondaryKeys {
                    SecondaryKeys::from([
                        #secondary_keys_expanded
                    ])
                }
            }

            #[automatically_derived]
            impl #impl_generics #name #ty_generics #where_clause {
                #secondary_keys_constructors
            }
        };
        expanded.to_tokens(tokens);
    }
}

impl ToTokens for Singleton {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let table_name = &self.name;
        let name = &self.ident;
        // let pulse = &self.crate_path;

        let default_path = LitStr::new("::pulse", Span::call_site())
            .parse_with(Path::parse_mod_style)
            .unwrap();
        let pulse = &self.crate_path.as_ref().unwrap_or(&default_path);
        let (impl_generics, ty_generics, where_clause) =
            &self.generics.split_for_impl();
        let expanded = quote! {
            #[automatically_derived]
            impl #impl_generics #pulse::Table for #name #ty_generics #where_clause {
                const NAME: #pulse::Name = #pulse::Name::new(#pulse::name!(#table_name));

                type Row = Self;

                #[inline]
                fn primary_key(_row: &Self::Row) -> u64 {
                    Self::NAME.as_u64()
                }
            }

            #[automatically_derived]
            impl #impl_generics #name #ty_generics #where_clause {
                #[inline]
                pub fn singleton<C, S>(code: C, scope: S) -> ::pulse_cdt::SingletonIndex<Self>
                where
                    C: Into<#pulse::Name>,
                    S: Into<#pulse::Name>,
                {
                    ::pulse_cdt::SingletonIndex::new(code, scope)
                }
            }
        };
        expanded.to_tokens(tokens);
    }
}

impl ToTokens for DeriveTable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Table(table) => table.to_tokens(tokens),
            Self::Singleton(singleton) => singleton.to_tokens(tokens),
        }
    }
}
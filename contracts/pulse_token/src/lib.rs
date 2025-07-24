#![no_std]
#![no_main]
extern crate alloc;
use alloc::{borrow::ToOwned, string::String};

use pulse_cdt::{
    action,
    contracts::{
        get_self, has_auth, is_account, require_auth, require_recipient,
    },
    core::{check, Asset, Name, Payer, Symbol, Table, TableCursor, MAX_ASSET_AMOUNT},
    dispatch, name, NumBytes, Read, Write,
};

#[derive(Read, Write, NumBytes, Clone)]
pub struct Account {
    pub balance: Asset,
}

impl Table for Account {
    const NAME: Name = Name::new(name!("accounts"));
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.balance.symbol.code().raw()
    }
}

#[derive(Read, Write, NumBytes, Clone)]
pub struct CurrencyStats {
    pub supply: Asset,
    pub max_supply: Asset,
    pub issuer: Name,
}

impl Table for CurrencyStats {
    const NAME: Name = Name::new(name!("stats"));
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.supply.symbol.code().raw()
    }
}

#[action]
fn create(issuer: Name, max_supply: Asset) {
    require_auth(get_self());

    let sym = max_supply.symbol;
    check(sym.is_valid(), "invalid symbol name");
    check(max_supply.is_valid(), "invalid supply");
    check(max_supply.amount > 0, "max-supply must be positive");

    let stats_table = CurrencyStats::table(get_self(), sym.code().raw());
    check(
        stats_table.find(sym.code().raw()).is_none(),
        "token with symbol already exists",
    );

    stats_table.emplace(
        get_self(),
        CurrencyStats {
            supply: Asset {
                amount: 0,
                symbol: sym,
            },
            max_supply: Asset {
                amount: 0,
                symbol: sym,
            },
            issuer,
        },
    );
}

#[action]
fn issue(to: Name, quantity: Asset, memo: String) {
    let sym = quantity.symbol;
    check(sym.is_valid(), "invalid symbol name");
    check(memo.len() <= 256, "memo has more than 256 bytes");

    let stats_table = CurrencyStats::table(get_self(), sym.code().raw());
    let existing = stats_table
        .find(sym.code().raw())
        .expect("token with symbol does not exist, create token before issue");
    let mut st = existing.get().expect("failed to read stats");
    check( to == st.issuer, "tokens can only be issued to issuer account" );

    require_auth(st.issuer);
    check(quantity.is_valid(), "invalid quantity");
    check(quantity.amount > 0, "must issue positive quantity");

    check(
        quantity.symbol == st.supply.symbol,
        "symbol precision mismatch",
    );
    check( quantity.amount + st.supply.amount <= MAX_ASSET_AMOUNT, "quantity exceeds available supply");

    existing
        .modify(&mut st, Payer::Same, |s| {
            s.supply.amount += quantity.amount;
        })
        .expect("failed to modify stats");

    add_balance(st.issuer, quantity, st.issuer);
}

#[action]
fn retire(quantity: Asset, memo: String) {
    let sym = quantity.symbol;
    check(sym.is_valid(), "invalid symbol name");
    check(memo.len() <= 256, "memo has more than 256 bytes");

    let stats_table = CurrencyStats::table(get_self(), sym.code().raw());
    let existing = stats_table
        .find(sym.code().raw())
        .expect("token with symbol does not exist");
    let mut st = existing.get().unwrap();

    require_auth(st.issuer);
    check(quantity.is_valid(), "invalid quantity");
    check(quantity.amount > 0, "must retire positive quantity");
    check(
        quantity.symbol == st.supply.symbol,
        "symbol precision mismatch",
    );

    existing
        .modify(&mut st, Payer::Same, |s| {
            s.supply -= quantity;
        })
        .expect("failed to modify stats");

    sub_balance(st.issuer, quantity);
}

#[action]
fn transfer(from: Name, to: Name, quantity: Asset, memo: String) {
    check(from != to, "cannot transfer to self");
    require_auth(from);
    check(is_account(to), "to account does not exist");
    let sym = quantity.symbol.code();
    let stats_table = CurrencyStats::table(get_self(), sym.raw());
    let existing = stats_table
        .find(sym.raw())
        .expect("token with symbol does not exist, create token before issue");
    let st = existing.get().expect("failed to read stats");

    require_recipient(from);
    require_recipient(to);

    check(quantity.is_valid(), "invalid quantity");
    check(quantity.amount > 0, "must transfer positive quantity");
    check(
        quantity.symbol == st.supply.symbol,
        "symbol precision mismatch",
    );
    check(memo.len() <= 256, "memo has more than 256 bytes");

    let payer = if has_auth(to) { to } else { from };

    sub_balance(from, quantity);
    add_balance(to, quantity, payer);
}

#[action]
fn open(owner: Name, symbol: Symbol, ram_payer: Name) {
    require_auth(ram_payer);
    check(is_account(owner), "owner account does not exist");

    let sym_code_raw = symbol.code().raw();
    let stats_table = CurrencyStats::table(get_self(), sym_code_raw);
    let st = stats_table
        .find(sym_code_raw)
        .expect("symbol does not exist")
        .get()
        .expect("failed to read stats");
    check(st.supply.symbol == symbol, "symbol precision mismatch");

    let accounts = Account::table(get_self(), owner);
    let it = accounts.find(sym_code_raw);
    if it.is_none() {
        accounts.emplace(
            ram_payer,
            Account {
                balance: Asset { amount: 0, symbol },
            },
        );
    }
}

#[action]
fn close(owner: Name, symbol: Symbol) {
    require_auth(owner);
    let accounts = Account::table(get_self(), owner);
    let it = accounts
        .find(symbol.code().raw())
        .expect("balance row already deleted or never existed. action won't have any effect.");
    let existing = it.get().expect("failed to read account");
    check(
        existing.balance.amount == 0,
        "cannot close because the balance is not zero.",
    );
    it.erase().expect("failed to erase account");
}

fn sub_balance(owner: Name, value: Asset) {
    let accounts = Account::table(get_self(), owner);
    let existing = accounts
        .find(value.symbol.code().raw())
        .expect("no balance object found");

    let mut from = existing.get().expect("failed to read account");
    existing
        .modify(&mut from, Payer::New(owner), |a| {
            a.balance -= value;
        }).expect("failed to subtract account balance");
}

fn add_balance(owner: Name, value: Asset, payer: Name) {
    let accounts = Account::table(get_self(), owner);
    let to = accounts.find(value.symbol.code().raw());

    if to.is_none() {
        accounts.emplace(payer, Account { balance: value });
    } else {
        let existing = to.unwrap();
        let mut account = existing.get().unwrap();
        existing
            .modify(&mut account, Payer::Same, |a| {
                a.balance += value;
            })
            .expect("failed to add account balance");
    }
}

dispatch!(create, issue, retire, transfer, open, close);

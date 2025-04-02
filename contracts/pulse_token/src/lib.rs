use pulse::{action, dispatch, name, Asset, Name, NumBytes, Read, Write};
use pulse_cdt::{assert::check, auth::{get_self, has_auth, is_account, require_auth, require_recipient}, table::{Payer, Table, TableCursor}};

#[derive(Read, Write, NumBytes, Clone)]
pub struct Account {
    pub balance: Asset,
}

impl Table for Account {
    const NAME: Name = Name::new(name!("accounts"));
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.balance.symbol.code().as_u64()
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
        row.supply.symbol.code().as_u64()
    }
}

#[action]
fn create(issuer: Name, max_supply: Asset) {
    require_auth(get_self());

    let sym = max_supply.symbol;
    check(sym.is_valid(), "invalid symbol name");
    check(max_supply.is_valid(), "invalid supply");
    check(max_supply.amount > 0, "max-supply must be positive");

    let stats_table = CurrencyStats::table(get_self(), sym.code().as_u64());
    check(stats_table.find(sym.code().as_u64()).is_none(), "token with symbol already exists");

    stats_table.emplace(get_self(), CurrencyStats {
        supply: Asset { amount: 0, symbol: sym },
        max_supply: Asset{ amount: 0, symbol: sym},
        issuer,
    });
}

#[action]
fn issue(to: Name, quantity: Asset, memo: String) {
    let sym = quantity.symbol;
    check(sym.is_valid(), "invalid symbol name");
    check(memo.len() <= 256, "memo has more than 256 bytes");

    let stats_table = CurrencyStats::table(get_self(), sym.code().as_u64());
    let existing = stats_table.find(sym.code().as_u64()).expect("token with symbol does not exist, create token before issue");
    let mut st = existing.get().expect("failed to read stats");
    check(st.issuer == to, "must be issuer to issue");

    require_auth(st.issuer);
    check(quantity.is_valid(), "invalid quantity");
    check(quantity.amount > 0, "must issue positive quantity");

    check(quantity.symbol == st.supply.symbol, "symbol precision mismatch" );
    check(quantity.amount <= st.max_supply.amount - st.supply.amount, "quantity exceeds available supply");

    st.supply.amount += quantity.amount;
    existing.modify(&mut st, Payer::Same).expect("failed to modify stats");

    add_balance(st.issuer, quantity, st.issuer);
}

#[action]
fn transfer(from: Name, to: Name, quantity: Asset, memo: String) {
    check(from != to, "cannot transfer to self" );
    require_auth(from);
    check(is_account(to), "to account does not exist");
    let sym = quantity.symbol.code();
    let stats_table = CurrencyStats::table(get_self(), sym.as_u64());
    let existing = stats_table.find(sym.as_u64()).expect("token with symbol does not exist, create token before issue");
    let st = existing.get().expect("failed to read stats");

    require_recipient(from);
    require_recipient(to);

    check(quantity.is_valid(), "invalid quantity");
    check(quantity.amount > 0, "must transfer positive quantity");
    check(quantity.symbol == st.supply.symbol, "symbol precision mismatch");
    check(memo.len() <= 256, "memo has more than 256 bytes");

    let payer = if has_auth(to) { to } else { from };

    sub_balance(from, quantity);
    add_balance(to, quantity, payer);
}

fn sub_balance(owner: Name, value: Asset) {
    let accounts = Account::table(get_self(), owner);
    let existing = accounts.find(value.symbol.code().as_u64())
    .expect("no balance object found");

    let mut from = existing.get().expect("failed to read account");
    from.balance.amount -= value.amount;
    existing.modify(&mut from, Payer::New(owner)).expect("failed to modify account");
}

fn add_balance(owner: Name, value: Asset, payer: Name) {
    let accounts = Account::table(get_self(), owner);
    let to = accounts.find(value.symbol.code().as_u64());

    if to.is_none() {
        accounts.emplace(payer, Account {
            balance: value,
        });
    } else {
        let existing = to.unwrap();
        let mut account = existing.get().expect("failed to read account");
        account.balance.amount += value.amount;
        existing.modify(&mut account, Payer::Same).expect("failed to modify account");
    }
}

dispatch!(create, issue, transfer);
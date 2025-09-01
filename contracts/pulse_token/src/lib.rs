#![no_std]
#![no_main]
extern crate alloc;
use alloc::string::String;

use pulse_cdt::{
    NumBytes, Read, SAME_PAYER, Write, action, contract,
    contracts::{has_auth, is_account, require_auth, require_recipient},
    core::{Asset, MAX_ASSET_AMOUNT, MultiIndexDefinition, Name, Symbol, SymbolCode, Table, check},
    name, table,
};

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.balance.symbol.code().raw())]
pub struct Account {
    pub balance: Asset,
}

const ACCOUNTS: MultiIndexDefinition<Account> = MultiIndexDefinition::new(name!("accounts"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.supply.symbol.code().raw())]
pub struct CurrencyStats {
    pub supply: Asset,
    pub max_supply: Asset,
    pub issuer: Name,
}

pub const STATS: MultiIndexDefinition<CurrencyStats> = MultiIndexDefinition::new(name!("stats"));

#[derive(Default)]
struct TokenContract;

#[contract]
impl TokenContract {
    #[action]
    fn create(issuer: Name, max_supply: Asset) {
        require_auth(get_self());

        let sym = max_supply.symbol;
        check(sym.is_valid(), "invalid symbol name");
        check(max_supply.is_valid(), "invalid supply");
        check(max_supply.amount > 0, "max-supply must be positive");

        let stats_table = STATS.index(get_self(), sym.code().raw());
        let existing = stats_table.find(sym.code().raw());
        check(
            existing == stats_table.end(),
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

        let stats_table = STATS.index(get_self(), sym.code().raw());
        let mut st = stats_table.find(sym.code().raw());
        check(
            st != stats_table.end(),
            "token with symbol does not exist, create token before issue",
        );
        check(
            to == st.issuer,
            "tokens can only be issued to issuer account",
        );

        require_auth(st.issuer);
        check(quantity.is_valid(), "invalid quantity");
        check(quantity.amount > 0, "must issue positive quantity");

        check(
            quantity.symbol == st.supply.symbol,
            "symbol precision mismatch",
        );
        check(
            quantity.amount + st.supply.amount <= MAX_ASSET_AMOUNT,
            "quantity exceeds available supply",
        );

        stats_table.modify(&mut st, SAME_PAYER, |s| {
            s.supply += quantity;
        });

        add_balance(st.issuer, quantity, st.issuer);
    }

    #[action]
    fn retire(quantity: Asset, memo: String) {
        let sym = quantity.symbol;
        check(sym.is_valid(), "invalid symbol name");
        check(memo.len() <= 256, "memo has more than 256 bytes");

        let stats_table = STATS.index(get_self(), sym.code().raw());
        let mut st = stats_table.find(sym.code().raw());
        check(st != stats_table.end(), "token with symbol does not exist");

        require_auth(st.issuer);
        check(quantity.is_valid(), "invalid quantity");
        check(quantity.amount > 0, "must retire positive quantity");
        check(
            quantity.symbol == st.supply.symbol,
            "symbol precision mismatch",
        );

        stats_table.modify(&mut st, SAME_PAYER, |s| {
            s.supply -= quantity;
        });

        sub_balance(st.issuer, quantity);
    }

    #[action]
    fn transfer(from: Name, to: Name, quantity: Asset, memo: String) {
        check(from != to, "cannot transfer to self");
        require_auth(from);
        check(is_account(to), "to account does not exist");
        let sym = quantity.symbol.code();
        let stats_table = STATS.index(get_self(), sym.raw());
        let st = stats_table.get(sym.raw(), "symbol does not exist");

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
        let stats_table = STATS.index(get_self(), sym_code_raw);
        let st = stats_table.get(sym_code_raw, "symbol does not exist");
        check(st.supply.symbol == symbol, "symbol precision mismatch");

        let accounts = ACCOUNTS.index(get_self(), owner.raw());
        let it = accounts.find(sym_code_raw);
        if it == accounts.end() {
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
        let accounts = ACCOUNTS.index(get_self(), owner.raw());
        let it = accounts.find(symbol.code().raw());
        check(it != accounts.end(), "balance row already doesn't exist");
        check(
            it.balance.amount == 0,
            "cannot close because the balance is not zero.",
        );
        accounts.erase(it);
    }
}

fn sub_balance(owner: Name, value: Asset) {
    let from_acnts = ACCOUNTS.index(get_self(), owner.raw());
    let mut from = from_acnts.get(value.symbol.code().raw(), "no balance object found");
    check(from.balance.amount >= value.amount, "overdrawn balance");

    from_acnts.modify(&mut from, SAME_PAYER, |a| {
        a.balance -= value;
    });
}

fn add_balance(owner: Name, value: Asset, payer: Name) {
    let to_acnts = ACCOUNTS.index(get_self(), owner.raw());
    let mut to = to_acnts.find(value.symbol.code().raw());

    if to == to_acnts.end() {
        to_acnts.emplace(payer, Account { balance: value });
    } else {
        to_acnts.modify(&mut to, SAME_PAYER, |a| {
            a.balance += value;
        });
    }
}

pub fn get_supply(token_contract_account: Name, sym_code: SymbolCode) -> Asset {
    let stats_table = STATS.index(token_contract_account, sym_code.raw());
    let st = stats_table.get(sym_code.raw(), "symbol does not exist");
    st.supply
}

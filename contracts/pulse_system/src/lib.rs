#![no_std]
#![no_main]
extern crate alloc;

use core::{iter::Map, str::FromStr};

use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use pulse_cdt::{
    NumBytes, Read, Write, action,
    contracts::{get_self, require_auth},
    core::{
        Asset, Name, Symbol, SymbolCode, Table, TimePoint, TimePointSec, check,
    },
    dispatch, name,
};
use pulse_token::get_supply;

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct Connector {
    pub balance: Asset,
    pub weight: f64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct ExchangeState {
    pub supply: Asset,
    pub base: Connector,
    pub quote: Connector,
}

impl Table for ExchangeState {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.supply.symbol.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct NameBid {
    pub new_name: Name,
    pub high_bidder: Name,
    pub high_bid: i64,
    pub last_bid_time: TimePoint,
}

impl Table for NameBid {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.new_name.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct BidRefund {
    pub bidder: Name,
    pub amount: Asset,
}

impl Table for BidRefund {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.bidder.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct ProducerInfo {
    owner: Name,
    total_votes: f64,
    is_active: bool,
    url: String,
    unpaid_blocks: u32,
    last_claim_time: TimePoint,
    location: u16,
}

impl Table for ProducerInfo {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct ProducerInfo2 {
    owner: Name,
    votepay_share: f64,
    last_votepay_share_update: TimePoint,
}

impl Table for ProducerInfo2 {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct VoterInfo {
    owner: Name,
    proxy: Name,
    producers: Vec<Name>,
    staked: i64,
    last_vote_weight: f64,
    proxied_vote_weight: f64,
    is_proxy: bool,
    flags1: u32,
    reserved2: u32,
    reserved3: Asset,
}

impl Table for VoterInfo {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct UserResources {
    owner: Name,
    net_weight: Asset,
    cpu_weight: Asset,
    ram_bytes: i64,
}

impl Table for UserResources {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct DelegatedBandwidth {
    from: Name,
    to: Name,
    net_weight: Asset,
    cpu_weight: Asset,
}

impl Table for DelegatedBandwidth {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.to.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct RefundRequest {
    owner: Name,
    request_time: TimePointSec,
    net_amount: Asset,
    cpu_amount: Asset,
}

impl Table for RefundRequest {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct DelegatedXPR {
    from: Name,
    to: Name,
    quantity: Asset,
}

impl Table for DelegatedXPR {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.to.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct VotersXPR {
    owner: Name,
    staked: u64,
    isqualified: bool,
    claimamount: u64,
    lastclaim: u64,
    startstake: Option<u64>,
    startqualif: Option<bool>,
}

impl Table for VotersXPR {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct XPRRefundRequest {
    owner: Name,
    request_time: TimePointSec,
    quantity: Asset,
}

impl Table for XPRRefundRequest {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct GlobalState {
    max_bp_per_vote: u64,       // Max BPs allowed to vote from one account
    min_bp_reward: u64,         // Min voted BPs to get voter reward
    unstake_period: u64,        // Unstake period for XPR tokens   14 * 24 * 60 * 60 = 14 days
    process_by: u64, // How many accounts process in one step during voters reward sharing
    process_interval: u64, // Time (sec) interval between voter reward sharing //      60 * 60 * 12;   = 12h
    voters_claim_interval: u64, // Time (sec) between voter claim rewards (24h def)
    spare1: u64,
    spare2: u64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct GlobalStateD {
    totalstaked: i64,
    totalrstaked: i64,
    totalrvoters: i64,
    notclaimed: i64,
    pool: i64,
    processtime: i64,
    processtimeupd: i64,
    isprocessing: bool,
    process_from: Name,
    process_quant: u64,
    processrstaked: u64,
    processed: u64,
    spare1: i64,
    spare2: i64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct GlobalStateRAM {
    ram_price_per_byte: Asset,
    max_per_user_bytes: u64,
    ram_fee_percent: u64,
    total_ram: u64,
    total_xpr: u64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct UserRAM {
    account: Name,
    ram: u64,
    quantity: Asset,
    ramlimit: u64,
}

impl Table for UserRAM {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.account.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct RexPool {
    version: u64,
    total_lent: Asset,
    total_unlent: Asset,
    total_rent: Asset,
    total_lendable: Asset,
    total_rex: Asset,
    namebid_proceeds: Asset,
    loan_num: u64,
}

impl Table for RexPool {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        0
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct RexReturnPool {
    version: u64,
    last_dist_time: TimePointSec,
    pending_bucket_time: TimePointSec,
    oldest_bucket_time: TimePointSec,
    pending_bucket_proceeds: i64,
    current_rate_of_proceeds: i64,
    proceeds: i64,
}

impl Table for RexReturnPool {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        0
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct RexReturnBuckets {
    version: u8,
    return_buckets: BTreeMap<TimePointSec, i64>,
}

impl Table for RexReturnBuckets {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        0
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct RexFund {
    version: u8,
    owner: Name,
    balance: Asset,
}

impl Table for RexFund {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct RexBalance {
    version: u8,
    owner: Name,
    vote_stake: Asset,
    rex_balance: Asset,
    matured_rex: i64,
}

impl Table for RexBalance {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct CpuLoan {
    version: u8,
    from: Name,
    receiver: Name,
    payment: Asset,
    balance: Asset,
    total_staked: Asset,
    loan_num: u64,
    expiration: TimePoint,
}

impl Table for CpuLoan {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.version as u64
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct NetLoan {
    version: u8,
    from: Name,
    receiver: Name,
    payment: Asset,
    balance: Asset,
    total_staked: Asset,
    loan_num: u64,
    expiration: TimePoint,
}

impl Table for NetLoan {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.version as u64
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct RexOrder {
    version: u8,
    owner: Name,
    rex_requested: Asset,
    proceeds: Asset,
    stake_change: Asset,
    order_time: TimePoint,
    is_open: bool,
}

impl Table for RexOrder {
    type Key = u64;
    type Row = Self;

    fn primary_key(row: &Self::Row) -> u64 {
        row.owner.raw()
    }
}

const TOKEN_ACCOUNT: Name = name!("pulse.token");

#[action]
fn init(version: u8, core: Symbol) {
    require_auth(get_self());
    check(version == 0, "unsupported version for init action");

    let ramcore_symbol: Symbol = Symbol::new_with_code(4, SymbolCode::from_str("RAMCORE").unwrap());
    let rammarket = ExchangeState::table(get_self(), get_self());
    let itr = rammarket.find(ramcore_symbol.raw());
    check(
        itr.is_none(),
        "system contract has already been initialized",
    );

    let system_token_supply = get_supply(TOKEN_ACCOUNT, core.code());
    check(
        system_token_supply.symbol == core,
        "specified core symbol does not exist (precision mismatch)",
    );
    check(
        system_token_supply.amount > 0,
        "system token supply must be greater than 0",
    );

    let ram_symbol: Symbol = Symbol::new_with_code(0, SymbolCode::from_str("RAM").unwrap());

    rammarket.emplace(
        get_self(),
        ExchangeState {
            supply: Asset {
                amount: 100000000000000,
                symbol: ramcore_symbol,
            },
            base: Connector {
                balance: Asset {
                    amount: (),
                    symbol: ram_symbol,
                },
                weight: 0.5,
            },
            quote: Connector {
                balance: Asset {
                    amount: system_token_supply.amount / 1000,
                    symbol: core,
                },
                weight: 0.5,
            },
        },
    );
}

dispatch!(init);

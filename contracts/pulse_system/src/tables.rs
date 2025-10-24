use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use pulse_cdt::{
    NumBytes, Read, Write,
    core::{
        Asset, BitEnum, BlockTimestamp, MultiIndexDefinition, Name, PublicKey, Symbol, Table,
        TimePoint, TimePointSec, check,
    },
    name, symbol_with_code, table,
};

use crate::exchange_state::get_bancor_output;

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
pub struct Connector {
    pub balance: Asset,
    pub weight: f64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.supply.symbol.raw())]
pub struct ExchangeState {
    pub supply: Asset,
    pub base: Connector,
    pub quote: Connector,
}

impl ExchangeState {
    pub fn direct_convert(&mut self, from: &Asset, to: &Symbol) -> Asset {
        let sell_symbol = from.symbol;
        let base_symbol = self.base.balance.symbol;
        let quote_symbol = self.quote.balance.symbol;
        check(sell_symbol != *to, "cannot convert to the same symbol");

        let mut out = Asset::new(0, to.clone());

        if sell_symbol == base_symbol && *to == quote_symbol {
            out.amount = get_bancor_output(
                self.base.balance.amount,
                self.quote.balance.amount,
                from.amount,
            );
            self.base.balance += *from;
            self.quote.balance -= out;
        } else if sell_symbol == quote_symbol && *to == base_symbol {
            out.amount = get_bancor_output(
                self.quote.balance.amount,
                self.base.balance.amount,
                from.amount,
            );
            self.quote.balance += *from;
            self.base.balance -= out;
        } else {
            check(false, "invalid conversion");
        }

        out
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.new_name.raw())]
pub struct NameBid {
    pub new_name: Name,
    pub high_bidder: Name,
    pub high_bid: i64,
    pub last_bid_time: TimePoint,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.bidder.raw())]
pub struct BidRefund {
    pub bidder: Name,
    pub amount: Asset,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct ProducerInfo {
    pub owner: Name,
    pub total_votes: f64,
    pub producer_key: PublicKey,
    pub is_active: bool,
    pub url: String,
    pub unpaid_blocks: u32,
    pub last_claim_time: TimePoint,
    pub location: u16,
}

impl ProducerInfo {
    #[inline]
    pub fn deactivate(&mut self) {
        self.producer_key = PublicKey::default();
        self.is_active = false;
    }

    pub fn active(&self) -> bool {
        self.is_active
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct ProducerInfo2 {
    pub owner: Name,
    pub votepay_share: f64,
    pub last_votepay_share_update: TimePoint,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct VoterInfo {
    pub owner: Name,
    pub proxy: Name,
    pub producers: Vec<Name>,
    pub staked: i64,
    pub last_vote_weight: f64,
    pub proxied_vote_weight: f64,
    pub is_proxy: bool,
    pub flags1: u32,
    pub reserved2: u32,
    pub reserved3: Asset,
}

#[repr(u32)]
pub enum VoterInfoFlags1Fields {
    RamManaged = 1,
    NetManaged = 2,
    CpuManaged = 4,
}

impl BitEnum for VoterInfoFlags1Fields {
    type Repr = u32;
    #[inline]
    fn to_bits(self) -> Self::Repr {
        self as u32
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct UserResources {
    pub owner: Name,
    pub net_weight: Asset,
    pub cpu_weight: Asset,
    pub ram_bytes: i64,
}

impl UserResources {
    pub fn is_empty(&self) -> bool {
        self.net_weight.amount == 0 && self.cpu_weight.amount == 0 && self.ram_bytes == 0
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.to.raw())]
pub struct DelegatedBandwidth {
    pub from: Name,
    pub to: Name,
    pub net_weight: Asset,
    pub cpu_weight: Asset,
}

impl DelegatedBandwidth {
    pub fn is_empty(&self) -> bool {
        self.net_weight.amount == 0 && self.cpu_weight.amount == 0
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct RefundRequest {
    pub owner: Name,
    pub request_time: TimePointSec,
    pub net_amount: Asset,
    pub cpu_amount: Asset,
}

impl RefundRequest {
    pub fn is_empty(&self) -> bool {
        self.net_amount.amount == 0 && self.cpu_amount.amount == 0
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.to.raw())]
pub struct DelegatedXPR {
    pub from: Name,
    pub to: Name,
    pub quantity: Asset,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct VotersXPR {
    pub owner: Name,
    pub staked: u64,
    pub isqualified: bool,
    pub claimamount: u64,
    pub lastclaim: u64,
    pub startstake: Option<u64>,
    pub startqualif: Option<bool>,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct XPRRefundRequest {
    pub owner: Name,
    pub request_time: TimePointSec,
    pub quantity: Asset,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct GlobalStateXPR {
    pub max_bp_per_vote: u64,       // Max BPs allowed to vote from one account
    pub min_bp_reward: u64,         // Min voted BPs to get voter reward
    pub unstake_period: u64,        // Unstake period for XPR tokens   14 * 24 * 60 * 60 = 14 days
    pub process_by: u64, // How many accounts process in one step during voters reward sharing
    pub process_interval: u64, // Time (sec) interval between voter reward sharing //      60 * 60 * 12;   = 12h
    pub voters_claim_interval: u64, // Time (sec) between voter claim rewards (24h def)
    pub spare1: u64,
    pub spare2: u64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct GlobalStateD {
    pub totalstaked: i64,
    pub totalrstaked: i64,
    pub totalrvoters: i64,
    pub notclaimed: i64,
    pub pool: i64,
    pub processtime: i64,
    pub processtimeupd: i64,
    pub isprocessing: bool,
    pub process_from: Name,
    pub process_quant: u64,
    pub processrstaked: u64,
    pub processed: u64,
    pub spare1: i64,
    pub spare2: i64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct GlobalStateRAM {
    pub ram_price_per_byte: Asset,
    pub max_per_user_bytes: u64,
    pub ram_fee_percent: u64,
    pub total_ram: u64,
    pub total_xpr: u64,
}

impl Default for GlobalStateRAM {
    fn default() -> Self {
        Self {
            ram_price_per_byte: Asset {
                amount: 200,
                symbol: symbol_with_code!(4, "XPR"),
            },
            max_per_user_bytes: 3 * 1024 * 1024, // 3 MB
            ram_fee_percent: 1000,               // 10%
            total_ram: 0,
            total_xpr: 0,
        }
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.account.raw())]
pub struct UserRAM {
    pub account: Name,
    pub ram: u64,
    pub quantity: Asset,
    pub ramlimit: u64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct RexPool {
    pub version: u64,
    pub total_lent: Asset,
    pub total_unlent: Asset,
    pub total_rent: Asset,
    pub total_lendable: Asset,
    pub total_rex: Asset,
    pub namebid_proceeds: Asset,
    pub loan_num: u64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct RexReturnPool {
    pub version: u64,
    pub last_dist_time: TimePointSec,
    pub pending_bucket_time: TimePointSec,
    pub oldest_bucket_time: TimePointSec,
    pub pending_bucket_proceeds: i64,
    pub current_rate_of_proceeds: i64,
    pub proceeds: i64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct RexReturnBuckets {
    pub version: u8,
    pub return_buckets: BTreeMap<TimePointSec, i64>,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct RexFund {
    pub version: u8,
    pub owner: Name,
    pub balance: Asset,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct RexBalance {
    pub version: u8,
    pub owner: Name,
    pub vote_stake: Asset,
    pub rex_balance: Asset,
    pub matured_rex: i64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.version as u64)]
pub struct RexLoan {
    pub version: u8,
    pub from: Name,
    pub receiver: Name,
    pub payment: Asset,
    pub balance: Asset,
    pub total_staked: Asset,
    pub loan_num: u64,
    pub expiration: TimePoint,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq, Default)]
#[table(primary_key = 0)]
pub struct GlobalState {
    pub max_ram_size: u64,
    pub total_ram_bytes_reserved: u64,
    pub total_ram_stake: i64,
    pub last_producer_schedule_update: BlockTimestamp,
    pub last_pervote_bucket_fill: TimePoint,
    pub pervote_bucket: i64,
    pub perblock_bucket: i64,
    pub total_unpaid_blocks: u32,
    pub total_activated_stake: i64,
    pub thresh_activated_stake_time: TimePoint,
    pub last_producer_schedule_size: u16,
    pub total_producer_vote_weight: f64,
    pub last_name_close: BlockTimestamp,
}

impl GlobalState {
    #[inline]
    pub const fn free_ram(&self) -> u64 {
        self.max_ram_size - self.total_ram_bytes_reserved
    }
}

#[derive(Read, Write, NumBytes, Clone, PartialEq, Default)]
#[table(primary_key = 0)]
pub struct GlobalState2 {
    pub new_ram_per_block: u16,
    pub last_ram_increase: BlockTimestamp,
    pub last_block_num: BlockTimestamp,
    pub total_producer_votepay_share: f64,
    pub revision: u8,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq, Default)]
#[table(primary_key = 0)]
pub struct GlobalState3 {
    pub last_vpay_state_update: TimePoint,
    pub total_vpay_share_change_rate: f64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq, Default)]
#[table(primary_key = 0)]
pub struct GlobalState4 {
    pub continuous_rate: f64,
    pub inflation_pay_factor: i64,
    pub votepay_factor: i64,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.supply.symbol.code().raw())]
pub struct CurrencyStats {
    pub supply: Asset,
    pub max_supply: Asset,
    pub issuer: Name,
}

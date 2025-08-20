#![no_std]
#![no_main]
extern crate alloc;

use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};
use pulse_cdt::{
    action, constructor, contract,
    contracts::{require_auth, set_privileged, set_resource_limits, Authority},
    core::{
        check, Asset, MultiIndexDefinition, Name, Singleton, SingletonDefinition, Symbol,
        SymbolCode, Table, TimePoint, TimePointSec,
    },
    dispatch, name, symbol_with_code, table, NumBytes, Read, Write,
};

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

const RAMMARKET: MultiIndexDefinition<ExchangeState> =
    MultiIndexDefinition::new(name!("rammarket"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.new_name.raw())]
pub struct NameBid {
    pub new_name: Name,
    pub high_bidder: Name,
    pub high_bid: i64,
    pub last_bid_time: TimePoint,
}

const NAME_BID_TABLE: MultiIndexDefinition<NameBid> = MultiIndexDefinition::new(name!("namebids"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.bidder.raw())]
pub struct BidRefund {
    pub bidder: Name,
    pub amount: Asset,
}

const BID_REFUND_TABLE: MultiIndexDefinition<BidRefund> =
    MultiIndexDefinition::new(name!("bidrefunds"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct ProducerInfo {
    owner: Name,
    total_votes: f64,
    is_active: bool,
    url: String,
    unpaid_blocks: u32,
    last_claim_time: TimePoint,
    location: u16,
}

const PRODUCERS_TABLE: MultiIndexDefinition<ProducerInfo> =
    MultiIndexDefinition::new(name!("producers"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct ProducerInfo2 {
    owner: Name,
    votepay_share: f64,
    last_votepay_share_update: TimePoint,
}

const PRODUCERS_TABLE2: MultiIndexDefinition<ProducerInfo2> =
    MultiIndexDefinition::new(name!("producers2"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
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

const VOTERS_TABLE: MultiIndexDefinition<VoterInfo> = MultiIndexDefinition::new(name!("voters"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct UserResources {
    owner: Name,
    net_weight: Asset,
    cpu_weight: Asset,
    ram_bytes: i64,
}

const USER_RESOURCES_TABLE: MultiIndexDefinition<UserResources> =
    MultiIndexDefinition::new(name!("userres"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.to.raw())]
pub struct DelegatedBandwidth {
    from: Name,
    to: Name,
    net_weight: Asset,
    cpu_weight: Asset,
}

const DEL_BANDWIDTH_TABLE: MultiIndexDefinition<DelegatedBandwidth> =
    MultiIndexDefinition::new(name!("delband"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct RefundRequest {
    owner: Name,
    request_time: TimePointSec,
    net_amount: Asset,
    cpu_amount: Asset,
}

const REFUNDS_TABLE: MultiIndexDefinition<RefundRequest> =
    MultiIndexDefinition::new(name!("refunds"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.to.raw())]
pub struct DelegatedXPR {
    from: Name,
    to: Name,
    quantity: Asset,
}

const DEL_XPR_TABLE: MultiIndexDefinition<DelegatedXPR> =
    MultiIndexDefinition::new(name!("delxpr"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct VotersXPR {
    owner: Name,
    staked: u64,
    isqualified: bool,
    claimamount: u64,
    lastclaim: u64,
    startstake: Option<u64>,
    startqualif: Option<bool>,
}

const VOTERS_XPR_TABLE: MultiIndexDefinition<VotersXPR> =
    MultiIndexDefinition::new(name!("votersxpr"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct XPRRefundRequest {
    owner: Name,
    request_time: TimePointSec,
    quantity: Asset,
}

const XPR_REFUNDS_TABLE: MultiIndexDefinition<XPRRefundRequest> =
    MultiIndexDefinition::new(name!("refundsxpr"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct GlobalStateXPR {
    max_bp_per_vote: u64,       // Max BPs allowed to vote from one account
    min_bp_reward: u64,         // Min voted BPs to get voter reward
    unstake_period: u64,        // Unstake period for XPR tokens   14 * 24 * 60 * 60 = 14 days
    process_by: u64, // How many accounts process in one step during voters reward sharing
    process_interval: u64, // Time (sec) interval between voter reward sharing //      60 * 60 * 12;   = 12h
    voters_claim_interval: u64, // Time (sec) between voter claim rewards (24h def)
    spare1: u64,
    spare2: u64,
}

const GLOBAL_STATEXPR_SINGLETON: MultiIndexDefinition<GlobalStateXPR> =
    MultiIndexDefinition::new(name!("globalsxpr"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
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

const GLOBAL_STATESD_SINGLETON: MultiIndexDefinition<GlobalStateD> =
    MultiIndexDefinition::new(name!("globalsd"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct GlobalStateRAM {
    ram_price_per_byte: Asset,
    max_per_user_bytes: u64,
    ram_fee_percent: u64,
    total_ram: u64,
    total_xpr: u64,
}

const GLOBAL_STATE_RAM_SINGLETON: MultiIndexDefinition<GlobalStateRAM> =
    MultiIndexDefinition::new(name!("globalram"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.account.raw())]
pub struct UserRAM {
    account: Name,
    ram: u64,
    quantity: Asset,
    ramlimit: u64,
}

const USERRAM_TABLE: MultiIndexDefinition<UserRAM> = MultiIndexDefinition::new(name!("usersram"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
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

const REX_POOL_TABLE: MultiIndexDefinition<RexPool> = MultiIndexDefinition::new(name!("rexpool"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct RexReturnPool {
    version: u64,
    last_dist_time: TimePointSec,
    pending_bucket_time: TimePointSec,
    oldest_bucket_time: TimePointSec,
    pending_bucket_proceeds: i64,
    current_rate_of_proceeds: i64,
    proceeds: i64,
}

const REX_RETURN_POOL_TABLE: MultiIndexDefinition<RexReturnPool> =
    MultiIndexDefinition::new(name!("retpool"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct RexReturnBuckets {
    version: u8,
    return_buckets: BTreeMap<TimePointSec, i64>,
}

const REX_RETURN_BUCKETS_TABLE: MultiIndexDefinition<RexReturnBuckets> =
    MultiIndexDefinition::new(name!("retbuckets"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct RexFund {
    version: u8,
    owner: Name,
    balance: Asset,
}

const REX_FUND_TABLE: MultiIndexDefinition<RexFund> = MultiIndexDefinition::new(name!("rexfund"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct RexBalance {
    version: u8,
    owner: Name,
    vote_stake: Asset,
    rex_balance: Asset,
    matured_rex: i64,
}

const REX_BALANCE_TABLE: MultiIndexDefinition<RexBalance> =
    MultiIndexDefinition::new(name!("rexbal"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.version as u64)]
pub struct RexLoan {
    version: u8,
    from: Name,
    receiver: Name,
    payment: Asset,
    balance: Asset,
    total_staked: Asset,
    loan_num: u64,
    expiration: TimePoint,
}

const REX_CPU_LOAN_TABLE: MultiIndexDefinition<RexLoan> =
    MultiIndexDefinition::new(name!("cpuloan"));
const REX_NET_LOAN_TABLE: MultiIndexDefinition<RexLoan> =
    MultiIndexDefinition::new(name!("netloan"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct RexOrder {
    version: u8,
    owner: Name,
    rex_requested: Asset,
    proceeds: Asset,
    stake_change: Asset,
    order_time: TimePoint,
    is_open: bool,
}

const TOKEN_ACCOUNT: Name = name!("pulse.token");
const RAM_SYMBOL: Symbol = symbol_with_code!(0, "RAM");
const RAMCORE_SYMBOL: Symbol = symbol_with_code!(4, "RAMCORE");
const REX_SYMBOL: Symbol = symbol_with_code!(4, "REX");

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = 0)]
pub struct GlobalState {
    max_ram_size: u64,
    total_ram_bytes_reserved: u64,
    total_ram_stake: i64,
}

impl GlobalState {
    #[inline]
    pub const fn free_ram(&self) -> u64 {
        self.max_ram_size - self.total_ram_bytes_reserved
    }
}

const GLOBAL_STATE_SINGLETON: SingletonDefinition<GlobalState> =
    SingletonDefinition::new(name!("global"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.supply.symbol.code().raw())]
pub struct CurrencyStats {
    pub supply: Asset,
    pub max_supply: Asset,
    pub issuer: Name,
}

const STATS: MultiIndexDefinition<CurrencyStats> = MultiIndexDefinition::new(name!("stats"));

/* #[inline]
fn get_supply(token_contract_account: Name, sym_code: SymbolCode) -> Asset {
    let stats_table = STATS.index(token_contract_account, sym_code.raw());
    let st = stats_table.get(sym_code.raw(), "symbol does not exist");
    st.supply
} */

#[inline]
fn get_core_symbol(system_account: Option<Name>) -> Symbol {
    let system_account = {
        if system_account.is_some() {
            system_account.unwrap()
        } else {
            name!("pulse")
        }
    };
    let rm = RAMMARKET.index(system_account, system_account.raw());
    let itr = rm.find(RAMCORE_SYMBOL.raw());
    check(itr != rm.end(), "system contract must first be initialized");
    itr.quote.balance.symbol
}

struct SystemContract {
    gstate: GlobalState,
}

#[contract]
impl SystemContract {
    #[constructor]
    fn constructor() -> Self {
        let global = GLOBAL_STATE_SINGLETON.get_instance(get_self(), get_self().raw());

        Self {
            gstate: if (global.exists()) {
                global.get()
            } else {
                global.get()
            },
        }
    }

    #[action]
    fn setpriv(account: Name, ispriv: u8) {
        require_auth(get_self());
        set_privileged(account, ispriv == 1);
    }

    #[action]
    fn newaccount(creator: Name, newact: Name, owner: Authority, active: Authority) {
        if creator != get_self() && creator != name!("proton") {
            let mut tmp = newact.raw() >> 4;
            let mut has_dot_or_less_than_12_chars = false;

            for _ in 0..12 {
                has_dot_or_less_than_12_chars |= (tmp & 0x1f) == 0;
                tmp >>= 5;
            }

            if has_dot_or_less_than_12_chars {
                let suffix = newact.suffix();
                let has_dot = suffix != newact;
                if has_dot {
                    // PROTON: only the suffix account may create names with dots/short
                    check(creator == suffix, "only suffix may create this account");
                    // or: check(creator == suffix, "only suffix may create this account");
                }
            }

            check(
                newact.to_string().chars().count() > 3,
                "minimum 4 character length",
            );
        }

        let userres = USER_RESOURCES_TABLE.index(get_self(), newact.raw());
        let core_symbol = get_core_symbol(None);
        userres.emplace(
            newact,
            UserResources {
                owner: newact,
                net_weight: Asset {
                    amount: 0,
                    symbol: core_symbol,
                },
                cpu_weight: Asset {
                    amount: 0,
                    symbol: core_symbol,
                },
                ram_bytes: 0,
            },
        );

        set_resource_limits(newact, 0, 0, 0);
    }

    #[action]
    fn setcode(account: Name, vmtype: u8, vmversion: u8, code: Vec<u8>) {
        // Set code is open for all
    }

    #[action]
    fn init(&self, version: u8, core: Symbol) {
        require_auth(get_self());
        check(version == 0, "unsupported version for init action");

        let rammarket = RAMMARKET.index(get_self(), get_self().raw());
        let itr = rammarket.find(RAMCORE_SYMBOL.raw());
        check(
            itr != rammarket.end(),
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

        rammarket.emplace(
            get_self(),
            ExchangeState {
                supply: Asset {
                    amount: 100000000000000,
                    symbol: RAMCORE_SYMBOL,
                },
                base: Connector {
                    balance: Asset {
                        amount: self.gstate.free_ram() as i64,
                        symbol: RAM_SYMBOL,
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
}

#![no_std]
#![no_main]
extern crate alloc;

mod exchange_state;
mod native;

use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use pulse_cdt::{
    action, constructor, contract, contracts::{
        current_block_time, get_resource_limits, require_auth, set_privileged, set_resource_limits, sha256, ActionWrapper, Authority, PermissionLevel
    }, core::{
        check, has_field, Asset, BitEnum, BlockTimestamp, MultiIndexDefinition, Name, SingletonDefinition, Symbol, SymbolCode, Table, TimePoint, TimePointSec
    }, destructor, name, symbol_with_code, table, NumBytes, Read, Write, SAME_PAYER
};

use crate::{
    __SystemContract_contract_ctx::get_self, exchange_state::{get_bancor_input, get_bancor_output}, native::{AbiHash, ABI_HASH_TABLE}
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

impl ExchangeState {
    pub fn direct_convert(&mut self, from: &Asset, to: &Symbol) -> Asset {
        let sell_symbol = from.symbol;
        let base_symbol = self.base.balance.symbol;
        let quote_symbol = self.quote.balance.symbol;
        check(sell_symbol != *to, "cannot convert to the same symbol");

        let mut out = Asset::new(0, to.clone());

        if sell_symbol == base_symbol && *to == quote_symbol {
            out.amount = get_bancor_output(self.base.balance.amount, self.quote.balance.amount, from.amount);
            self.base.balance += *from;
            self.quote.balance -= out;
        } else if sell_symbol == quote_symbol && *to == base_symbol {
            out.amount = get_bancor_output(self.quote.balance.amount, self.base.balance.amount, from.amount);
            self.quote.balance += *from;
            self.base.balance -= out;
        } else {
            check(false, "invalid conversion");
        }

        out
    }
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

#[repr(u32)]
pub enum VoterInfoFlags1Fields {
    RAM_MANAGED = 1,
    NET_MANAGED = 2,
    CPU_MANAGED = 4,
}

impl BitEnum for VoterInfoFlags1Fields {
    type Repr = u32;
    #[inline]
    fn to_bits(self) -> Self::Repr { self as u32 }
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

impl Default for GlobalStateRAM {
    fn default() -> Self {
        Self {
            ram_price_per_byte: Asset {
                amount: 200,
                symbol: symbol_with_code!(4, "XPR"),
            },
            max_per_user_bytes: 3 * 1024 * 1024, // 3 MB
            ram_fee_percent: 1000,            // 10%
            total_ram: 0,
            total_xpr: 0,
        }
    }
}

const GLOBAL_STATE_RAM_SINGLETON: SingletonDefinition<GlobalStateRAM> =
    SingletonDefinition::new(name!("globalram"));

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

const ACTIVE_PERMISSION: Name = name!("active");
const TOKEN_ACCOUNT: Name = name!("pulse.token");
const RAM_ACCOUNT: Name = name!("pulse.ram");
const RAMFEE_ACCOUNT: Name = name!("pulse.ramfee");
const RAM_SYMBOL: Symbol = symbol_with_code!(0, "RAM");
const RAMCORE_SYMBOL: Symbol = symbol_with_code!(4, "RAMCORE");
const REX_SYMBOL: Symbol = symbol_with_code!(4, "REX");
const REX_ACCOUNT: Name = name!("pulse.rex");

const RAM_GIFT_BYTES: i64 = 1400;

const INFLATION_PRECISION: i64 = 100; // 2 decimals
const DEFAULT_ANNUAL_RATE: i64 = 500; // 5% annual rate
const DEFAULT_INFLATION_PAY_FACTOR: i64 = 50000; // producers pay share = 10000 / 50000 = 20% of the inflation
const DEFAULT_VOTEPAY_FACTOR: i64 = 40000; // per-block pay share = 10000 / 40000 = 25% of the producer pay

#[derive(Read, Write, NumBytes, Clone, PartialEq, Default)]
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

#[derive(Read, Write, NumBytes, Clone, PartialEq, Default)]
#[table(primary_key = 0)]
pub struct GlobalState2 {
    new_ram_per_block: u16,
    last_ram_increase: BlockTimestamp,
    last_block_num: BlockTimestamp,
    total_producer_votepay_share: f64,
    revision: u8,
}

const GLOBAL_STATE2_SINGLETON: SingletonDefinition<GlobalState2> =
    SingletonDefinition::new(name!("global2"));

#[derive(Read, Write, NumBytes, Clone, PartialEq, Default)]
#[table(primary_key = 0)]
pub struct GlobalState3 {
    last_vpay_state_update: TimePoint,
    total_vpay_share_change_rate: f64,
}

const GLOBAL_STATE3_SINGLETON: SingletonDefinition<GlobalState3> =
    SingletonDefinition::new(name!("global3"));

#[derive(Read, Write, NumBytes, Clone, PartialEq, Default)]
#[table(primary_key = 0)]
pub struct GlobalState4 {
    continuous_rate: f64,
    inflation_pay_factor: i64,
    votepay_factor: i64,
}

const GLOBAL_STATE4_SINGLETON: SingletonDefinition<GlobalState4> =
    SingletonDefinition::new(name!("global4"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.supply.symbol.code().raw())]
pub struct CurrencyStats {
    pub supply: Asset,
    pub max_supply: Asset,
    pub issuer: Name,
}

const STATS: MultiIndexDefinition<CurrencyStats> = MultiIndexDefinition::new(name!("stats"));

#[inline]
fn get_supply(token_contract_account: Name, sym_code: SymbolCode) -> Asset {
    let stats_table = STATS.index(token_contract_account, sym_code.raw());
    let st = stats_table.get(sym_code.raw(), "symbol does not exist");
    st.supply
}

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

#[inline]
fn get_default_inflation_parameters() -> GlobalState4 {
    let mut gs4 = GlobalState4::default();
    gs4.continuous_rate = get_continuous_rate(DEFAULT_ANNUAL_RATE);
    gs4.inflation_pay_factor = DEFAULT_INFLATION_PAY_FACTOR;
    gs4.votepay_factor = DEFAULT_VOTEPAY_FACTOR;
    gs4
}

#[inline]
fn get_continuous_rate(annual_rate: i64) -> f64 {
    let x = (annual_rate as f64) / (100.0 * INFLATION_PRECISION as f64);
    libm::log1p(x)
}

struct SystemContract {
    gstate: GlobalState,
    gstate2: GlobalState2,
    gstate3: GlobalState3,
    gstate4: GlobalState4,

    gstateram: GlobalStateRAM,
}

const OPEN_ACTION: ActionWrapper<(Name, Symbol, Name)> = ActionWrapper::new(name!("open"));
const TRANSFER_ACTION: ActionWrapper<(Name, Name, Asset, String)> =
    ActionWrapper::new(name!("transfer"));

#[contract]
impl SystemContract {
    #[constructor]
    fn constructor() -> Self {
        let global = GLOBAL_STATE_SINGLETON.get_instance(get_self(), get_self().raw());
        let global2 = GLOBAL_STATE2_SINGLETON.get_instance(get_self(), get_self().raw());
        let global3 = GLOBAL_STATE3_SINGLETON.get_instance(get_self(), get_self().raw());
        let global4 = GLOBAL_STATE4_SINGLETON.get_instance(get_self(), get_self().raw());
        let gstateram = GLOBAL_STATE_RAM_SINGLETON.get_instance(get_self(), get_self().raw());

        Self {
            gstate: if global.exists() {
                global.get()
            } else {
                GlobalState::default()
            },
            gstate2: if global2.exists() {
                global2.get()
            } else {
                GlobalState2::default()
            },
            gstate3: if global3.exists() {
                global3.get()
            } else {
                GlobalState3::default()
            },
            gstate4: if global4.exists() {
                global4.get()
            } else {
                get_default_inflation_parameters()
            },
            gstateram: if gstateram.exists() {
                gstateram.get()
            } else {
                GlobalStateRAM::default()
            },
        }
    }

    #[destructor]
    fn destructor(self) {
        let global = GLOBAL_STATE_SINGLETON.get_instance(get_self(), get_self().raw());
        let global2 = GLOBAL_STATE2_SINGLETON.get_instance(get_self(), get_self().raw());
        let global3 = GLOBAL_STATE3_SINGLETON.get_instance(get_self(), get_self().raw());
        let global4 = GLOBAL_STATE4_SINGLETON.get_instance(get_self(), get_self().raw());
        let gstateram = GLOBAL_STATE_RAM_SINGLETON.get_instance(get_self(), get_self().raw());

        global.set(self.gstate, get_self());
        global2.set(self.gstate2, get_self());
        global3.set(self.gstate3, get_self());
        global4.set(self.gstate4, get_self());
        gstateram.set(self.gstateram, get_self());
    }

    #[action]
    fn setpriv(account: Name, is_priv: u8) {
        require_auth(get_self());
        set_privileged(account, is_priv == 1);
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
    fn setabi(account: Name, abi: Vec<u8>) {
        let table = ABI_HASH_TABLE.index(get_self(), get_self().raw());
        let mut itr = table.find(account.raw());

        if itr == table.end() {
            table.emplace(
                account,
                AbiHash {
                    owner: account,
                    hash: sha256(&abi, abi.len() as u32),
                },
            );
        } else {
            table.modify(&mut itr, SAME_PAYER, |t| {
                t.hash = sha256(&abi, abi.len() as u32);
            });
        }
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
            itr == rammarket.end(),
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

        OPEN_ACTION
            .to_action(
                TOKEN_ACCOUNT,
                vec![PermissionLevel::new(get_self(), ACTIVE_PERMISSION)],
                (REX_ACCOUNT, core, get_self()),
            )
            .send();
    }

    #[action]
    fn buyrambsys(&mut self, payer: Name, receiver: Name, bytes: u32) {
        let rammarket = RAMMARKET.index(get_self(), get_self().raw());
        let itr = rammarket.find(RAMCORE_SYMBOL.raw());
        let ram_reserve = itr.base.balance.amount;
        let eos_reserve = itr.quote.balance.amount;
        let cost = get_bancor_input(ram_reserve, eos_reserve, bytes as i64);
        let cost_plus_fee = cost as f64 / 0.995; // ram fee 0.5%
        
        self.buyramsys(
            payer,
            receiver,
            Asset {
                amount: cost_plus_fee as i64,
                symbol: get_core_symbol(None),
            },
        );
    }

    #[action]
    pub fn buyramsys(&mut self, payer: Name, receiver: Name, quant: Asset) {
        require_auth(payer);
        //checkPermission(receiver, ACTIVE_PERMISSION);

        self.update_ram_supply();

        check(
            quant.symbol == get_core_symbol(None),
            "must buy ram with core token",
        );
        check(quant.amount > 0, "must purchase a positive amount");

        let fee = Asset {
            amount: quant.amount + 199 / 200,
            symbol: quant.symbol,
        }; // ram fee 0.5%
        let quant_after_fee = Asset {
            amount: quant.amount - fee.amount,
            symbol: quant.symbol,
        };

        TRANSFER_ACTION
            .to_action(
                TOKEN_ACCOUNT,
                vec![
                    PermissionLevel::new(payer, ACTIVE_PERMISSION),
                    PermissionLevel::new(RAM_ACCOUNT, ACTIVE_PERMISSION),
                ],
                (payer, RAM_ACCOUNT, quant_after_fee, "buy ram".to_string()),
            )
            .send();

        if fee.amount > 0 {
            TRANSFER_ACTION
                .to_action(
                    TOKEN_ACCOUNT,
                    vec![PermissionLevel::new(payer, ACTIVE_PERMISSION)],
                    (payer, RAMFEE_ACCOUNT, fee, "ram fee".to_string()),
                )
                .send();
        }

        let mut bytes_out = 0i64;
        let rammarket = RAMMARKET.index(get_self(), get_self().raw());
        let mut market = rammarket.get(RAMCORE_SYMBOL.raw(), "ram market does not exist");
        rammarket.modify(&mut market, SAME_PAYER, |es| {
            bytes_out = es.direct_convert(&quant_after_fee, &RAM_SYMBOL).amount
        });

        check(bytes_out > 0, "must reserve a positive amount");

        self.gstate.total_ram_bytes_reserved += bytes_out as u64;
        self.gstate.total_ram_stake += quant_after_fee.amount;

        let userres = USER_RESOURCES_TABLE.index(get_self(), receiver.raw());
        let mut res_itr = userres.find(receiver.raw());
        let core_symbol = get_core_symbol(None);
        if res_itr == userres.end() {
            userres.emplace(
                receiver,
                UserResources {
                    owner: receiver,
                    net_weight: Asset {
                        amount: 0,
                        symbol: core_symbol,
                    },
                    cpu_weight: Asset {
                        amount: 0,
                        symbol: core_symbol,
                    },
                    ram_bytes: bytes_out,
                },
            );
        } else {
            userres.modify(&mut res_itr, receiver, |res| {
                res.ram_bytes += bytes_out;
            });
        }

        let voters = VOTERS_TABLE.index(get_self(), get_self().raw());
        let voter_itr = voters.find(receiver.raw());
        if voter_itr == voters.end() || !has_field(voter_itr.flags1, VoterInfoFlags1Fields::RAM_MANAGED) {
            let (ram, net, cpu) = get_resource_limits(receiver);
            set_resource_limits(receiver, ram + RAM_GIFT_BYTES, net, cpu);
        }
    }

    fn update_ram_supply(&mut self) {
        let cbt = current_block_time();

        if cbt <= self.gstate2.last_ram_increase {
            return;
        }

        let rammarket = RAMMARKET.index(get_self(), get_self().raw());
        let mut itr = rammarket.find(RAMCORE_SYMBOL.raw());
        let new_ram: u32 = (cbt.slot - self.gstate2.last_ram_increase.slot) * self.gstate2.new_ram_per_block as u32;
        self.gstate.max_ram_size += new_ram as u64;

        rammarket.modify(&mut itr, SAME_PAYER, |m| {
            m.base.balance.amount += new_ram as i64;
        });
        self.gstate2.last_ram_increase = cbt;
    }
}

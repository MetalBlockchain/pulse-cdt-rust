#![no_std]
#![no_main]
extern crate alloc;

mod exchange_state;
mod native;
mod tables;

use core::cmp;

use alloc::{
    borrow::ToOwned,
    collections::btree_map::BTreeMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use libm::pow;
use pulse_cdt::{
    SAME_PAYER, Write, action, constructor, contract,
    contracts::{
        Action, ActionWrapper, Authority, KeyWeight, PermissionLevel, current_block_time,
        current_time_point, get_resource_limits, require_auth, set_privileged, set_resource_limits,
        sha256,
    },
    core::{
        Asset, BlockHeader, BlockSigningAuthority, BlockTimestamp, ConstIterator, Microseconds,
        MultiIndexDefinition, Name, PublicKey, SingletonDefinition, Symbol, SymbolCode, TimePoint,
        check, has_field,
    },
    destructor, name, symbol_with_code,
};

use crate::{
    exchange_state::get_bancor_input,
    native::{ABI_HASH_TABLE, AbiHash},
    tables::{
        BidRefund, Connector, CurrencyStats, DelegatedBandwidth, DelegatedXPR, ExchangeState,
        GlobalState, GlobalState2, GlobalState3, GlobalState4, GlobalStateD, GlobalStateRAM,
        GlobalStateXPR, NameBid, ProducerInfo, ProducerInfo2, RefundRequest, RexBalance, RexFund,
        RexLoan, RexPool, RexReturnBuckets, RexReturnPool, UserRAM, UserResources, VoterInfo,
        VoterInfoFlags1Fields, VotersXPR, XPRRefundRequest,
    },
};

// Table definitions
const RAMMARKET: MultiIndexDefinition<ExchangeState> =
    MultiIndexDefinition::new(name!("rammarket"));
const NAME_BID_TABLE: MultiIndexDefinition<NameBid> = MultiIndexDefinition::new(name!("namebids"));
const BID_REFUND_TABLE: MultiIndexDefinition<BidRefund> =
    MultiIndexDefinition::new(name!("bidrefunds"));
const PRODUCERS_TABLE: MultiIndexDefinition<ProducerInfo> =
    MultiIndexDefinition::new(name!("producers"));
const PRODUCERS_TABLE2: MultiIndexDefinition<ProducerInfo2> =
    MultiIndexDefinition::new(name!("producers2"));
const VOTERS_TABLE: MultiIndexDefinition<VoterInfo> = MultiIndexDefinition::new(name!("voters"));
const USER_RESOURCES_TABLE: MultiIndexDefinition<UserResources> =
    MultiIndexDefinition::new(name!("userres"));
const DEL_BANDWIDTH_TABLE: MultiIndexDefinition<DelegatedBandwidth> =
    MultiIndexDefinition::new(name!("delband"));
const REFUNDS_TABLE: MultiIndexDefinition<RefundRequest> =
    MultiIndexDefinition::new(name!("refunds"));
const DEL_XPR_TABLE: MultiIndexDefinition<DelegatedXPR> =
    MultiIndexDefinition::new(name!("delxpr"));
const VOTERS_XPR_TABLE: MultiIndexDefinition<VotersXPR> =
    MultiIndexDefinition::new(name!("votersxpr"));
const XPR_REFUNDS_TABLE: MultiIndexDefinition<XPRRefundRequest> =
    MultiIndexDefinition::new(name!("refundsxpr"));
const GLOBAL_STATEXPR_SINGLETON: MultiIndexDefinition<GlobalStateXPR> =
    MultiIndexDefinition::new(name!("globalsxpr"));
const GLOBAL_STATESD_SINGLETON: MultiIndexDefinition<GlobalStateD> =
    MultiIndexDefinition::new(name!("globalsd"));
const GLOBAL_STATE_RAM_SINGLETON: SingletonDefinition<GlobalStateRAM> =
    SingletonDefinition::new(name!("globalram"));
const USERRAM_TABLE: MultiIndexDefinition<UserRAM> = MultiIndexDefinition::new(name!("usersram"));
const REX_POOL_TABLE: MultiIndexDefinition<RexPool> = MultiIndexDefinition::new(name!("rexpool"));
const REX_RETURN_POOL_TABLE: MultiIndexDefinition<RexReturnPool> =
    MultiIndexDefinition::new(name!("retpool"));
const REX_RETURN_BUCKETS_TABLE: MultiIndexDefinition<RexReturnBuckets> =
    MultiIndexDefinition::new(name!("retbuckets"));
const REX_FUND_TABLE: MultiIndexDefinition<RexFund> = MultiIndexDefinition::new(name!("rexfund"));
const REX_BALANCE_TABLE: MultiIndexDefinition<RexBalance> =
    MultiIndexDefinition::new(name!("rexbal"));
const REX_CPU_LOAN_TABLE: MultiIndexDefinition<RexLoan> =
    MultiIndexDefinition::new(name!("cpuloan"));
const REX_NET_LOAN_TABLE: MultiIndexDefinition<RexLoan> =
    MultiIndexDefinition::new(name!("netloan"));
const GLOBAL_STATE_SINGLETON: SingletonDefinition<GlobalState> =
    SingletonDefinition::new(name!("global"));
const GLOBAL_STATE2_SINGLETON: SingletonDefinition<GlobalState2> =
    SingletonDefinition::new(name!("global2"));
const GLOBAL_STATE3_SINGLETON: SingletonDefinition<GlobalState3> =
    SingletonDefinition::new(name!("global3"));
const GLOBAL_STATE4_SINGLETON: SingletonDefinition<GlobalState4> =
    SingletonDefinition::new(name!("global4"));
const STATS: MultiIndexDefinition<CurrencyStats> = MultiIndexDefinition::new(name!("stats"));

// General variables
const ACTIVE_PERMISSION: Name = name!("active");
const TOKEN_ACCOUNT: Name = name!("pulse.token");
const RAM_ACCOUNT: Name = name!("pulse.ram");
const RAMFEE_ACCOUNT: Name = name!("pulse.ramfee");
const RAM_SYMBOL: Symbol = symbol_with_code!(0, "RAM");
const RAMCORE_SYMBOL: Symbol = symbol_with_code!(4, "RAMCORE");
const REX_SYMBOL: Symbol = symbol_with_code!(4, "REX");
const REX_ACCOUNT: Name = name!("pulse.rex");
const STAKE_ACCOUNT: Name = name!("pulse.stake");

const SECONDS_PER_DAY: u32 = 24 * 3600;
const USECONDS_PER_DAY: u64 = SECONDS_PER_DAY as u64 * 1000_000;

const MIN_ACTIVATED_STAKE: i64 = 150_000_000_0000;
const RAM_GIFT_BYTES: i64 = 1400;

const INFLATION_PRECISION: i64 = 100; // 2 decimals
const DEFAULT_ANNUAL_RATE: i64 = 500; // 5% annual rate
const DEFAULT_INFLATION_PAY_FACTOR: i64 = 50000; // producers pay share = 10000 / 50000 = 20% of the inflation
const DEFAULT_VOTEPAY_FACTOR: i64 = 40000; // per-block pay share = 10000 / 40000 = 25% of the producer pay

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
    fn newaccount(creator: Name, name: Name, owner: Authority, active: Authority) {
        if creator != get_self() && creator != name!("proton") {
            let mut tmp = name.raw() >> 4;
            let mut has_dot_or_less_than_12_chars = false;

            for _ in 0..12 {
                has_dot_or_less_than_12_chars |= (tmp & 0x1f) == 0;
                tmp >>= 5;
            }

            if has_dot_or_less_than_12_chars {
                let suffix = name.suffix();
                let has_dot = suffix != name;
                if has_dot {
                    // PROTON: only the suffix account may create names with dots/short
                    check(creator == suffix, "only suffix may create this account");
                    // or: check(creator == suffix, "only suffix may create this account");
                }
            }

            check(
                name.to_string().chars().count() > 3,
                "minimum 4 character length",
            );
        }

        let userres = USER_RESOURCES_TABLE.index(get_self(), name.raw());
        let core_symbol = get_core_symbol(None);
        userres.emplace(
            name,
            UserResources {
                owner: name,
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

        set_resource_limits(name, 0, 0, 0);
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

        let producers = PRODUCERS_TABLE.index(get_self(), get_self().raw());
        producers.emplace(
            get_self(),
            ProducerInfo {
                owner: get_self(),
                total_votes: 0.0,
                producer_key: PublicKey::new(
                    [
                        0, 3, 12, 214, 195, 138, 50, 120, 73, 105, 11, 101, 80, 3, 197, 12, 167,
                        129, 208, 247, 2, 12, 193, 126, 41, 66, 141, 239, 25, 163, 66, 69, 137, 97,
                    ]
                    .as_ref(),
                ),
                is_active: true,
                url: "https://metalblockchain.org".to_string(),
                unpaid_blocks: 0,
                last_claim_time: current_time_point(),
                location: 840,
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
            amount: (quant.amount + 199) / 200,
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
        if voter_itr == voters.end()
            || !has_field(voter_itr.flags1, VoterInfoFlags1Fields::RamManaged)
        {
            let (ram, net, cpu) = get_resource_limits(receiver);
            set_resource_limits(receiver, ram + RAM_GIFT_BYTES, net, cpu);
        }
    }

    pub fn changebw(
        &mut self,
        from: Name,
        receiver: Name,
        stake_net_delta: Asset,
        stake_cpu_delta: Asset,
        transfer: bool,
    ) {
        require_auth(from);
        check(
            stake_net_delta.amount != 0 || stake_cpu_delta.amount != 0,
            "should stake non-zero amount",
        );
        check(
            (stake_net_delta + stake_cpu_delta).amount.abs()
                >= stake_net_delta
                    .amount
                    .abs()
                    .max(stake_cpu_delta.amount.abs()),
            "net and cpu deltas cannot be opposite signs",
        );

        let source_stake_from = from.clone();
        let from = if transfer { receiver } else { from };

        // update stake delegated from "from" to "receiver"
        {
            let del_tbl = DEL_BANDWIDTH_TABLE.index(get_self(), from.raw());
            let mut itr = del_tbl.find(receiver.raw());
            if itr == del_tbl.end() {
                itr = del_tbl.emplace(
                    from,
                    DelegatedBandwidth {
                        from: from,
                        to: receiver,
                        net_weight: stake_net_delta,
                        cpu_weight: stake_cpu_delta,
                    },
                );
            } else {
                del_tbl.modify(&mut itr, SAME_PAYER, |dbo| {
                    dbo.net_weight += stake_net_delta;
                    dbo.cpu_weight += stake_cpu_delta;
                });
            }

            check(
                0 <= itr.net_weight.amount,
                "insufficient staked net bandwidth",
            );
            check(
                0 <= itr.cpu_weight.amount,
                "insufficient staked cpu bandwidth",
            );
            if itr.is_empty() {
                del_tbl.erase(itr);
            }
        }

        // update totals of "receiver"
        {
            let totals_tbl = USER_RESOURCES_TABLE.index(get_self(), receiver.raw());
            let mut tot_itr = totals_tbl.find(receiver.raw());
            if tot_itr == totals_tbl.end() {
                tot_itr = totals_tbl.emplace(
                    from,
                    UserResources {
                        owner: receiver,
                        net_weight: stake_net_delta,
                        cpu_weight: stake_cpu_delta,
                        ram_bytes: 0,
                    },
                );
            } else {
                let payer = if from == receiver { from } else { SAME_PAYER };
                totals_tbl.modify(&mut tot_itr, payer, |tot| {
                    tot.net_weight += stake_net_delta;
                    tot.cpu_weight += stake_cpu_delta;
                });
            }
            check(
                0 <= tot_itr.net_weight.amount,
                "insufficient staked total net bandwidth",
            );
            check(
                0 <= tot_itr.cpu_weight.amount,
                "insufficient staked total cpu bandwidth",
            );

            let mut ram_managed = false;
            let mut net_managed = false;
            let mut cpu_managed = false;

            let voters = VOTERS_TABLE.index(get_self(), get_self().raw());
            let voter_itr = voters.find(receiver.raw());
            if voter_itr != voters.end() {
                ram_managed = has_field(voter_itr.flags1, VoterInfoFlags1Fields::RamManaged);
                net_managed = has_field(voter_itr.flags1, VoterInfoFlags1Fields::NetManaged);
                cpu_managed = has_field(voter_itr.flags1, VoterInfoFlags1Fields::CpuManaged);
            }

            if !(net_managed && cpu_managed) {
                let (ram_bytes, net, cpu) = get_resource_limits(receiver);
                let new_ram = if ram_managed {
                    ram_bytes
                } else {
                    cmp::max(tot_itr.ram_bytes + RAM_GIFT_BYTES, ram_bytes)
                };
                let new_net = if net_managed {
                    net
                } else {
                    tot_itr.net_weight.amount
                };
                let new_cpu = if cpu_managed {
                    cpu
                } else {
                    tot_itr.cpu_weight.amount
                };
                set_resource_limits(receiver, new_ram, new_net, new_cpu);
            }

            if tot_itr.is_empty() {
                totals_tbl.erase(tot_itr);
            }
        }

        // create refund or update from existing refund
        if STAKE_ACCOUNT != source_stake_from {
            let refunds_tbl = REFUNDS_TABLE.index(get_self(), from.raw());
            let mut req = refunds_tbl.find(from.raw());

            let mut net_balance = stake_net_delta;
            let mut cpu_balance = stake_cpu_delta;
            let mut need_deferred_trx = false;

            // net and cpu are same sign by assertions in delegatebw and undelegatebw
            // redundant assertion also at start of changebw to protect against misuse of changebw
            let is_undelegating = (net_balance.amount + cpu_balance.amount) < 0;
            let is_delegating_to_self = !transfer && from == receiver;

            if is_delegating_to_self || is_undelegating {
                if req != refunds_tbl.end() {
                    //need to update refund
                    refunds_tbl.modify(&mut req, SAME_PAYER, |r| {
                        if net_balance.amount < 0 || cpu_balance.amount < 0 {
                            r.request_time = current_time_point().into();
                        }
                        r.net_amount -= net_balance;
                        if r.net_amount.amount < 0 {
                            net_balance = -r.net_amount;
                            r.net_amount.amount = 0;
                        } else {
                            net_balance.amount = 0;
                        }
                        r.cpu_amount -= cpu_balance;
                        if r.cpu_amount.amount < 0 {
                            cpu_balance = -r.cpu_amount;
                            r.cpu_amount.amount = 0;
                        } else {
                            cpu_balance.amount = 0;
                        }
                    });

                    check(0 <= req.net_amount.amount, "negative net refund amount"); //should never happen
                    check(0 <= req.cpu_amount.amount, "negative cpu refund amount"); //should never happen

                    if req.is_empty() {
                        refunds_tbl.erase(req);
                        need_deferred_trx = false;
                    } else {
                        need_deferred_trx = true;
                    }
                } else if net_balance.amount < 0 || cpu_balance.amount < 0 {
                    refunds_tbl.emplace(
                        from,
                        RefundRequest {
                            owner: from,
                            net_amount: if net_balance.amount < 0 {
                                let result = -net_balance;
                                net_balance.amount = 0;
                                result
                            } else {
                                Asset::new(0, get_core_symbol(None))
                            },
                            cpu_amount: if cpu_balance.amount < 0 {
                                let result = -cpu_balance;
                                cpu_balance.amount = 0;
                                result
                            } else {
                                Asset::new(0, get_core_symbol(None))
                            },
                            request_time: current_time_point().into(),
                        },
                    );
                    need_deferred_trx = true;
                }

                if need_deferred_trx {
                    Action::new(
                        vec![PermissionLevel::new(from, ACTIVE_PERMISSION)],
                        get_self(),
                        name!("refund"),
                        from.pack().unwrap(),
                    )
                    .send();
                }
            }

            let transfer_amount = net_balance + cpu_balance;
            if 0 < transfer_amount.amount {
                TRANSFER_ACTION
                    .to_action(
                        TOKEN_ACCOUNT,
                        vec![PermissionLevel::new(source_stake_from, ACTIVE_PERMISSION)],
                        (
                            source_stake_from,
                            STAKE_ACCOUNT,
                            transfer_amount,
                            "stake bandwidth".to_owned(),
                        ),
                    )
                    .send();
            }
        }

        self.update_voting_power(from, stake_net_delta + stake_cpu_delta);
    }

    #[action]
    pub fn delegatebw(
        &mut self,
        from: Name,
        receiver: Name,
        stake_net_quantity: Asset,
        stake_cpu_quantity: Asset,
        transfer: bool,
    ) {
        let zero_asset = Asset::new(0, get_core_symbol(None));
        check(
            stake_cpu_quantity >= zero_asset,
            "must stake a positive amount",
        );
        check(
            stake_net_quantity >= zero_asset,
            "must stake a positive amount",
        );
        check(
            stake_net_quantity.amount + stake_cpu_quantity.amount > 0,
            "must stake a positive amount",
        );
        check(
            !transfer || from != receiver,
            "cannot use transfer flag if delegating to self",
        );

        //check (system_contract::checkPermission(from, "delegate")==1, "You are not authorised to delegate.");  // PROTON Check Permissions

        self.changebw(
            from,
            receiver,
            stake_net_quantity,
            stake_cpu_quantity,
            transfer,
        );
    }

    #[action]
    pub fn undelegatebw(
        &mut self,
        from: Name,
        receiver: Name,
        unstake_net_quantity: Asset,
        unstake_cpu_quantity: Asset,
    ) {
        let zero_asset = Asset::new(0, get_core_symbol(None));
        check(
            unstake_net_quantity >= zero_asset,
            "must unstake a positive amount",
        );
        check(
            unstake_cpu_quantity >= zero_asset,
            "must unstake a positive amount",
        );
        check(
            unstake_cpu_quantity.amount + unstake_net_quantity.amount > 0,
            "must unstake a positive amount",
        );
        check(
            self.gstate.thresh_activated_stake_time != TimePoint::default(),
            "cannot undelegate bandwidth until the chain is activated (at least 15% of all tokens participate in voting)",
        );

        //check (system_contract::checkPermission(from, "undelegate")==1, "You are not authorised to undelegate.");  // PROTON Check Permissions

        self.changebw(
            from,
            receiver,
            -unstake_net_quantity,
            -unstake_cpu_quantity,
            false,
        );
    }

    #[action]
    pub fn refund(owner: Name) {
        require_auth(owner);

        let refunds_tbl = REFUNDS_TABLE.index(get_self(), owner.raw());
        let req = refunds_tbl.find(owner.raw());
        check(req != refunds_tbl.end(), "refund request not found");
        check(
            req.request_time <= current_time_point().into(),
            "refund is not available yet",
        );
        TRANSFER_ACTION
            .to_action(
                TOKEN_ACCOUNT,
                vec![
                    PermissionLevel::new(STAKE_ACCOUNT, ACTIVE_PERMISSION),
                    PermissionLevel::new(req.owner, ACTIVE_PERMISSION),
                ],
                (
                    STAKE_ACCOUNT,
                    req.owner,
                    req.net_amount,
                    "unstake".to_owned(),
                ),
            )
            .send();
        refunds_tbl.erase(req);
    }

    fn update_ram_supply(&mut self) {
        let cbt = current_block_time();

        if cbt <= self.gstate2.last_ram_increase {
            return;
        }

        let rammarket = RAMMARKET.index(get_self(), get_self().raw());
        let mut itr = rammarket.find(RAMCORE_SYMBOL.raw());
        let new_ram: u32 = (cbt.slot - self.gstate2.last_ram_increase.slot)
            * self.gstate2.new_ram_per_block as u32;
        self.gstate.max_ram_size += new_ram as u64;

        rammarket.modify(&mut itr, SAME_PAYER, |m| {
            m.base.balance.amount += new_ram as i64;
        });
        self.gstate2.last_ram_increase = cbt;
    }

    #[action]
    pub fn onblock(&mut self, block_header: BlockHeader) {
        require_auth(get_self());

        self.gstate2.last_block_num = block_header.timestamp;
    }

    fn register_producer(
        &mut self,
        producer: Name,
        producer_authority: BlockSigningAuthority,
        url: String,
        location: u16,
    ) {
        let producers = PRODUCERS_TABLE.index(get_self(), get_self().raw());
        let producers2 = PRODUCERS_TABLE2.index(get_self(), get_self().raw());
        let mut prod = producers.find(producer.raw());
        let ct = current_time_point();
        let mut producer_key = PublicKey::default();

        if producer_authority.keys.len() == 1 {
            producer_key = producer_authority.keys[0].key.clone()
        }

        if prod != producers.end() {
            producers.modify(&mut prod, producer, |info| {
                info.producer_key = producer_key;
                info.is_active = true;
                info.url = url;
                info.location = location;

                if info.last_claim_time == TimePoint::default() {
                    info.last_claim_time = ct;
                }
            });

            let prod2 = producers2.find(producer.raw());

            if prod2 == producers2.end() {
                producers2.emplace(
                    producer,
                    ProducerInfo2 {
                        owner: producer,
                        last_votepay_share_update: ct,
                        votepay_share: 0.0,
                    },
                );

                self.update_total_votepay_share(&ct, 0.0, prod.total_votes);
            }
        } else {
            producers.emplace(
                producer,
                ProducerInfo {
                    owner: producer,
                    total_votes: 0.0,
                    producer_key: producer_key,
                    is_active: true,
                    url: url,
                    location: location,
                    last_claim_time: ct,
                    unpaid_blocks: 0,
                },
            );
            producers2.emplace(
                producer,
                ProducerInfo2 {
                    owner: producer,
                    last_votepay_share_update: ct,
                    votepay_share: 0.0,
                },
            );
        }
    }

    #[action]
    pub fn regproducer(
        &mut self,
        producer: Name,
        producer_key: PublicKey,
        url: String,
        location: u16,
    ) {
        require_auth(producer);

        // In XPR the permission check below is done but for now it is open
        // check (checkPermission(producer, "regprod") == 1, "You are not authorised to register as producer");  // PROTON Check Permissions

        check(url.len() < 512, "url too long");

        self.register_producer(
            producer,
            convert_to_block_signing_authority(&producer_key),
            url,
            location,
        );
    }

    #[action]
    pub fn regproducer2(
        &mut self,
        producer: Name,
        producer_authority: BlockSigningAuthority,
        url: String,
        location: u16,
    ) {
        require_auth(producer);

        // In XPR the permission check below is done but for now it is open
        // check (checkPermission(producer, "regprod") == 1, "You are not authorised to register as producer");  // PROTON Check Permissions

        check(url.len() < 512, "url too long");
        check(producer_authority.is_valid(), "invalid producer authority");

        self.register_producer(producer, producer_authority, url, location);
    }

    #[action]
    pub fn unregprod(producer: Name) {
        require_auth(producer);

        let producers = PRODUCERS_TABLE.index(get_self(), get_self().raw());
        let mut prod = producers.get(producer.raw(), "producer not found");
        producers.modify(&mut prod, SAME_PAYER, |info| {
            info.deactivate();
        });
    }

    fn update_total_votepay_share(
        &mut self,
        ct: &TimePoint,
        additional_shares_delta: f64,
        shares_rate_delta: f64,
    ) -> f64 {
        let mut delta_total_votepay_share = 0.0;
        if *ct > self.gstate3.last_vpay_state_update {
            delta_total_votepay_share = self.gstate3.total_vpay_share_change_rate
                * ((*ct - self.gstate3.last_vpay_state_update).count() / 1000000) as f64;
        }

        delta_total_votepay_share += additional_shares_delta;
        if delta_total_votepay_share < 0.0
            && self.gstate2.total_producer_votepay_share < -delta_total_votepay_share
        {
            self.gstate2.total_producer_votepay_share = 0.0;
        } else {
            self.gstate2.total_producer_votepay_share += delta_total_votepay_share;
        }

        if shares_rate_delta < 0.0 && self.gstate3.total_vpay_share_change_rate < -shares_rate_delta
        {
            self.gstate3.total_vpay_share_change_rate = 0.0;
        } else {
            self.gstate3.total_vpay_share_change_rate += shares_rate_delta;
        }

        self.gstate3.last_vpay_state_update = *ct;

        return self.gstate2.total_producer_votepay_share;
    }

    fn update_voting_power(&mut self, voter: Name, total_update: Asset) {
        let voters = VOTERS_TABLE.index(get_self(), get_self().raw());
        let mut voter_itr = voters.find(voter.raw());
        if voter_itr == voters.end() {
            voter_itr = voters.emplace(
                voter,
                VoterInfo {
                    owner: voter,
                    staked: total_update.amount,
                    proxy: Name::default(),
                    producers: vec![],
                    last_vote_weight: 0.0,
                    proxied_vote_weight: 0.0,
                    is_proxy: false,
                    flags1: 0,
                    reserved2: 0,
                    reserved3: Asset::default(),
                },
            );
        } else {
            voters.modify(&mut voter_itr, SAME_PAYER, |v| {
                v.staked += total_update.amount;
            });
        }

        check(0 <= voter_itr.staked, "stake for voting cannot be negative");

        if voter_itr.producers.len() > 0 || voter_itr.proxy != Name::default() {
            self.update_votes(voter, voter_itr.proxy, &voter_itr.producers, false);
        }
    }

    fn update_votes(&mut self, voter_name: Name, proxy: Name, producers: &Vec<Name>, voting: bool) {
        if proxy != Name::default() {
            check(
                producers.len() == 0,
                "cannot vote for producers and proxy at same time",
            );
            check(voter_name != proxy, "cannot proxy to self");
        } else {
            check(
                producers.len() <= 30,
                "attempt to vote for too many producers",
            );
            for i in 1..producers.len() {
                check(
                    producers[i - 1] < producers[i],
                    "producer votes must be unique and sorted",
                );
            }
        }

        let voters = VOTERS_TABLE.index(get_self(), get_self().raw());
        let mut voter = voters.find(voter_name.raw());
        check(
            voter != voters.end(),
            "user must stake before they can vote",
        );
        /// staking creates voter object
        check(
            proxy == Name::default() || !voter.is_proxy,
            "account registered as a proxy is not allowed to use a proxy",
        );

        if self.gstate.thresh_activated_stake_time == TimePoint::default()
            && voter.last_vote_weight <= 0.0
        {
            self.gstate.total_activated_stake += voter.staked;
            if self.gstate.total_activated_stake >= MIN_ACTIVATED_STAKE {
                self.gstate.thresh_activated_stake_time = current_time_point();
            }
        }

        let mut new_vote_weight = stake_to_vote(voter.staked);
        if voter.is_proxy {
            new_vote_weight += voter.proxied_vote_weight;
        }

        let mut producer_deltas: BTreeMap<Name, (f64, bool)> = BTreeMap::new();
        if voter.last_vote_weight > 0.0 {
            if voter.proxy != Name::default() {
                let mut old_proxy = voters.find(voter.proxy.raw());
                check(old_proxy != voters.end(), "old proxy not found");
                voters.modify(&mut old_proxy, SAME_PAYER, |vp| {
                    vp.proxied_vote_weight -= voter.last_vote_weight;
                });
                self.propagate_weight_change(&mut old_proxy);
            } else {
                for p in voter.producers.iter() {
                    let entry = producer_deltas.entry(p.clone()).or_insert((0.0, true));
                    entry.0 -= voter.last_vote_weight;
                    entry.1 = false;
                }
            }
        }

        if proxy != Name::default() {
            let mut new_proxy = voters.find(proxy.raw());
            check(new_proxy != voters.end(), "invalid proxy specified");
            check(!voting || new_proxy.is_proxy, "proxy not found");
            if new_vote_weight >= 0.0 {
                voters.modify(&mut new_proxy, SAME_PAYER, |vp| {
                    vp.proxied_vote_weight += new_vote_weight;
                });
                self.propagate_weight_change(&mut new_proxy);
            }
        } else {
            if new_vote_weight >= 0.0 {
                for p in producers.iter() {
                    let entry: &mut (f64, bool) =
                        producer_deltas.entry(p.clone()).or_insert((0.0, true));
                    entry.0 += new_vote_weight;
                    entry.1 = true;
                }
            }
        }

        let ct = current_time_point();
        let mut delta_change_rate = 0.0;
        let mut total_inactive_vpay_share = 0.0;
        let producers_table = PRODUCERS_TABLE.index(get_self(), get_self().raw());
        for pd in producer_deltas.iter() {
            let mut pitr = producers_table.find(pd.0.raw());
            if voting && !pitr.active() && pd.1.1 {
                check(
                    false,
                    format!(
                        "producer {} is not currently registered",
                        pitr.owner.to_string()
                    )
                    .as_str(),
                );
            }
            let init_total_votes = pitr.total_votes;
            producers_table.modify(&mut pitr, SAME_PAYER, |p| {
                p.total_votes += pd.1.0;
                if p.total_votes < 0.0 {
                    p.total_votes = 0.0;
                }
                self.gstate.total_producer_vote_weight += pd.1.0;
            });
            let producers_table2 = PRODUCERS_TABLE2.index(get_self(), get_self().raw());
            let mut prod2 = producers_table2.find(pd.0.raw());
            if prod2 != producers_table2.end() {
                let last_claim_plus_3days =
                    pitr.last_claim_time + Microseconds(3 * USECONDS_PER_DAY as i64);
                let crossed_threshold = last_claim_plus_3days <= ct;
                let updated_after_threshold =
                    last_claim_plus_3days <= prod2.last_votepay_share_update;

                let new_votepay_share = self.update_producer_votepay_share(
                    &mut prod2,
                    &ct,
                    if updated_after_threshold {
                        0.0
                    } else {
                        init_total_votes
                    },
                    crossed_threshold && !updated_after_threshold, // only reset votepay_share once after threshold
                );

                if !crossed_threshold {
                    delta_change_rate += pd.1.0;
                } else if !updated_after_threshold {
                    total_inactive_vpay_share += new_votepay_share;
                    delta_change_rate -= init_total_votes;
                }
            } else {
                if pd.1.1 {
                    check(
                        false,
                        format!("producer {} is not currently registered", pd.0.to_string())
                            .as_str(),
                    );
                }
            }
        }

        self.update_total_votepay_share(&ct, -total_inactive_vpay_share, delta_change_rate);

        voters.modify(&mut voter, SAME_PAYER, |av| {
            av.last_vote_weight = new_vote_weight;
            av.producers = producers.clone();
            av.proxy = proxy;
        });
    }

    pub fn update_producer_votepay_share(
        &self,
        prod_itr: &mut ConstIterator<ProducerInfo2>,
        ct: &TimePoint,
        shares_rate: f64,
        reset_to_zero: bool,
    ) -> f64 {
        let mut delta_votepay_share = 0.0;
        if shares_rate > 0.0 && *ct > prod_itr.last_votepay_share_update {
            delta_votepay_share =
                shares_rate * ((*ct - prod_itr.last_votepay_share_update).count() / 1000000) as f64;
        }

        let producers2 = PRODUCERS_TABLE2.index(get_self(), get_self().raw());
        let new_votepay_share = prod_itr.votepay_share + delta_votepay_share;
        producers2.modify(prod_itr, SAME_PAYER, |p| {
            if reset_to_zero {
                p.votepay_share = 0.0;
            } else {
                p.votepay_share = new_votepay_share;
            }

            p.last_votepay_share_update = *ct;
        });

        return new_votepay_share;
    }

    pub fn propagate_weight_change(&mut self, voter: &mut ConstIterator<VoterInfo>) {
        check(
            voter.proxy == Name::default(),
            "account registered as a proxy is not allowed to use a proxy",
        );
        let mut new_weight = stake_to_vote(voter.staked);
        if voter.is_proxy {
            new_weight += voter.proxied_vote_weight;
        }

        let voters = VOTERS_TABLE.index(get_self(), get_self().raw());

        if new_weight - voter.last_vote_weight > 1.0 {
            if !!voter.proxy {
                let mut proxy = voters.get(voter.proxy.raw(), "proxy not found");
                voters.modify(&mut proxy, SAME_PAYER, |p| {
                    p.proxied_vote_weight += new_weight - voter.last_vote_weight;
                });
                self.propagate_weight_change(&mut proxy);
            } else {
                let producers = PRODUCERS_TABLE.index(get_self(), get_self().raw());
                let delta = new_weight - voter.last_vote_weight;
                let ct = current_time_point();
                let mut delta_change_rate = 0.0;
                let mut total_inactive_vpay_share = 0.0;
                for acnt in voter.producers.iter() {
                    let mut prod = producers.get(acnt.raw(), "producer not found");
                    let init_total_votes = prod.total_votes;
                    producers.modify(&mut prod, SAME_PAYER, |p| {
                        p.total_votes += delta;
                    });
                    self.gstate.total_producer_vote_weight += delta;
                    let producers2 = PRODUCERS_TABLE2.index(get_self(), get_self().raw());
                    let mut prod2 = producers2.find(acnt.raw());
                    if prod2 != producers2.end() {
                        let last_claim_plus_3days =
                            prod.last_claim_time + Microseconds(3 * USECONDS_PER_DAY as i64);
                        let crossed_threshold = last_claim_plus_3days <= ct;
                        let updated_after_threshold =
                            last_claim_plus_3days <= prod2.last_votepay_share_update;

                        let new_votepay_share = self.update_producer_votepay_share(
                            &mut prod2,
                            &ct,
                            if updated_after_threshold {
                                0.0
                            } else {
                                init_total_votes
                            },
                            crossed_threshold && !updated_after_threshold, // only reset votepay_share once after threshold
                        );

                        if !crossed_threshold {
                            delta_change_rate += delta
                        } else if !updated_after_threshold {
                            total_inactive_vpay_share += new_votepay_share;
                            delta_change_rate -= init_total_votes;
                        }
                    }
                }

                self.update_total_votepay_share(&ct, -total_inactive_vpay_share, delta_change_rate);
            }
        }

        voters.modify(voter, SAME_PAYER, |v| {
            v.last_vote_weight = new_weight;
        });
    }

    #[action]
    pub fn regproxy(&mut self, proxy: Name, is_proxy: bool) {
        require_auth(proxy);

        //check (checkPermission(proxy, "regproxy")==1, "You are not authorised to register as proxy");  //

        let voters = VOTERS_TABLE.index(get_self(), get_self().raw());
        let mut pitr = voters.find(proxy.raw());
        if pitr != voters.end() {
            check(is_proxy != pitr.is_proxy, "action has no effect");
            check(
                !is_proxy || !pitr.is_proxy,
                "account that uses a proxy is not allowed to become a proxy",
            );
            voters.modify(&mut pitr, SAME_PAYER, |p| {
                p.is_proxy = is_proxy;
            });
            self.propagate_weight_change(&mut pitr);
        } else {
            voters.emplace(
                proxy,
                VoterInfo {
                    owner: proxy,
                    is_proxy: is_proxy,
                    proxy: Name::default(),
                    producers: vec![],
                    staked: 0,
                    last_vote_weight: 0.0,
                    proxied_vote_weight: 0.0,
                    flags1: 0,
                    reserved2: 0,
                    reserved3: Asset::default(),
                },
            );
        }
    }

    #[action]
    pub fn setram(&mut self, max_ram_size: u64) {
        require_auth(get_self());

        check(
            self.gstate.max_ram_size < max_ram_size,
            "ram may only be increased",
        );
        /// decreasing ram might result market maker issues
        check(
            max_ram_size < 1024 * 1024 * 1024 * 1024 * 1024,
            "ram size is unrealistic",
        );
        check(
            max_ram_size > self.gstate.total_ram_bytes_reserved,
            "attempt to set max below reserved",
        );

        let delta = max_ram_size as i64 - self.gstate.max_ram_size as i64;
        let rammarket = RAMMARKET.index(get_self(), get_self().raw());
        let mut itr = rammarket.find(RAMCORE_SYMBOL.raw());

        rammarket.modify(&mut itr, SAME_PAYER, |m| {
            m.base.balance.amount += delta;
        });

        self.gstate.max_ram_size = max_ram_size;
    }
}

fn convert_to_block_signing_authority(producer_key: &PublicKey) -> BlockSigningAuthority {
    BlockSigningAuthority::new(
        1,
        vec![KeyWeight {
            key: producer_key.clone(),
            weight: 1,
        }],
    )
}

fn stake_to_vote(staked: i64) -> f64 {
    let epoch_offset = BlockTimestamp::BLOCK_TIMESTAMP_EPOCH / 1000;

    let weight = ((current_time_point().sec_since_epoch() as i64 - epoch_offset)
        / (SECONDS_PER_DAY * 7) as i64) as f64
        / 52.0;
    (staked as f64) * pow(2.0, weight)
}

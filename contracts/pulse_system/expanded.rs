#![feature(prelude_import)]
#![no_std]
#![no_main]
#[macro_use]
extern crate core;
#[prelude_import]
use core::prelude::rust_2024::*;
extern crate alloc;
mod exchange_state {
    use pulse_cdt::core::{Asset, Symbol};
    #[inline(always)]
    pub fn get_bancor_input(out_reserve: i64, inp_reserve: i64, out: i64) -> i64 {
        let ob = out_reserve as f64;
        let ib = inp_reserve as f64;
        if ob <= out as f64 {
            return 0;
        }
        let mut inp = (ib * (out as f64)) / (ob - (out as f64));
        if inp < 0.0 {
            inp = 0.0;
        }
        inp as i64
    }
    pub fn get_bancor_output(inp_reserve: i64, out_reserve: i64, inp: i64) -> i64 {
        let ib = inp_reserve as f64;
        let ob = out_reserve as f64;
        let inn = inp as f64;
        let mut out = ((inn * ob) / (ib + inn)) as i64;
        if out < 0 {
            out = 0;
        }
        out
    }
}
mod native {
    use pulse_cdt::{
        NumBytes, Read, Write, core::{Checksum256, MultiIndexDefinition, Name, Table},
        name, table,
    };
    pub struct AbiHash {
        pub owner: Name,
        pub hash: Checksum256,
    }
    impl Table for AbiHash {
        type Key = u64;
        type Row = Self;
        #[inline]
        fn primary_key(row: &Self::Row) -> u64 {
            (row.owner.raw())
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::pulse_cdt::Read for AbiHash {
        #[inline]
        fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
            let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
            let hash = <Checksum256 as ::pulse_cdt::Read>::read(bytes, pos)?;
            let item = AbiHash { owner, hash };
            Ok(item)
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::pulse_cdt::Write for AbiHash {
        #[inline]
        fn write(
            &self,
            bytes: &mut [u8],
            pos: &mut usize,
        ) -> Result<(), ::pulse_cdt::WriteError> {
            ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
            ::pulse_cdt::Write::write(&self.hash, bytes, pos)?;
            Ok(())
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl ::pulse_cdt::NumBytes for AbiHash {
        #[inline]
        fn num_bytes(&self) -> usize {
            let mut count = 0;
            count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
            count += ::pulse_cdt::NumBytes::num_bytes(&self.hash);
            count
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for AbiHash {
        #[inline]
        fn clone(&self) -> AbiHash {
            AbiHash {
                owner: ::core::clone::Clone::clone(&self.owner),
                hash: ::core::clone::Clone::clone(&self.hash),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for AbiHash {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for AbiHash {
        #[inline]
        fn eq(&self, other: &AbiHash) -> bool {
            self.owner == other.owner && self.hash == other.hash
        }
    }
    pub const ABI_HASH_TABLE: MultiIndexDefinition<AbiHash> = MultiIndexDefinition::new(
        pulse_cdt::core::Name::new(3592979018984456192u64),
    );
}
use core::cmp;
use alloc::{
    borrow::ToOwned, collections::btree_map::BTreeMap, format,
    string::{String, ToString},
    vec::Vec, vec,
};
use libm::pow;
use pulse_cdt::{
    action, constructor, contract,
    contracts::{
        current_block_time, current_time_point, get_resource_limits, require_auth,
        set_privileged, set_resource_limits, sha256, Action, ActionWrapper, Authority,
        KeyWeight, PermissionLevel,
    },
    core::{
        check, has_field, seconds, Asset, BitEnum, BlockHeader, BlockSigningAuthority,
        BlockTimestamp, ConstIterator, Microseconds, MultiIndexDefinition, Name,
        PublicKey, SingletonDefinition, Symbol, SymbolCode, Table, TimePoint,
        TimePointSec,
    },
    destructor, name, symbol_with_code, table, NumBytes, Read, Write, SAME_PAYER,
};
use crate::{
    exchange_state::{get_bancor_input, get_bancor_output},
    native::{AbiHash, ABI_HASH_TABLE},
};
pub struct Connector {
    pub balance: Asset,
    pub weight: f64,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for Connector {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let balance = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let weight = <f64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = Connector { balance, weight };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for Connector {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.balance, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.weight, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for Connector {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.balance);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.weight);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Connector {
    #[inline]
    fn clone(&self) -> Connector {
        Connector {
            balance: ::core::clone::Clone::clone(&self.balance),
            weight: ::core::clone::Clone::clone(&self.weight),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for Connector {}
#[automatically_derived]
impl ::core::cmp::PartialEq for Connector {
    #[inline]
    fn eq(&self, other: &Connector) -> bool {
        self.weight == other.weight && self.balance == other.balance
    }
}
pub struct ExchangeState {
    pub supply: Asset,
    pub base: Connector,
    pub quote: Connector,
}
impl Table for ExchangeState {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.supply.symbol.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for ExchangeState {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let supply = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let base = <Connector as ::pulse_cdt::Read>::read(bytes, pos)?;
        let quote = <Connector as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = ExchangeState {
            supply,
            base,
            quote,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for ExchangeState {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.supply, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.base, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.quote, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for ExchangeState {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.supply);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.base);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.quote);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ExchangeState {
    #[inline]
    fn clone(&self) -> ExchangeState {
        ExchangeState {
            supply: ::core::clone::Clone::clone(&self.supply),
            base: ::core::clone::Clone::clone(&self.base),
            quote: ::core::clone::Clone::clone(&self.quote),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ExchangeState {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ExchangeState {
    #[inline]
    fn eq(&self, other: &ExchangeState) -> bool {
        self.supply == other.supply && self.base == other.base
            && self.quote == other.quote
    }
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
const RAMMARKET: MultiIndexDefinition<ExchangeState> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(13377137154988703744u64),
);
pub struct NameBid {
    pub new_name: Name,
    pub high_bidder: Name,
    pub high_bid: i64,
    pub last_bid_time: TimePoint,
}
impl Table for NameBid {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.new_name.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for NameBid {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let new_name = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let high_bidder = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let high_bid = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let last_bid_time = <TimePoint as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = NameBid {
            new_name,
            high_bidder,
            high_bid,
            last_bid_time,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for NameBid {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.new_name, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.high_bidder, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.high_bid, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_bid_time, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for NameBid {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.new_name);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.high_bidder);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.high_bid);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_bid_time);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for NameBid {
    #[inline]
    fn clone(&self) -> NameBid {
        NameBid {
            new_name: ::core::clone::Clone::clone(&self.new_name),
            high_bidder: ::core::clone::Clone::clone(&self.high_bidder),
            high_bid: ::core::clone::Clone::clone(&self.high_bid),
            last_bid_time: ::core::clone::Clone::clone(&self.last_bid_time),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for NameBid {}
#[automatically_derived]
impl ::core::cmp::PartialEq for NameBid {
    #[inline]
    fn eq(&self, other: &NameBid) -> bool {
        self.high_bid == other.high_bid && self.new_name == other.new_name
            && self.high_bidder == other.high_bidder
            && self.last_bid_time == other.last_bid_time
    }
}
const NAME_BID_TABLE: MultiIndexDefinition<NameBid> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(11071153799887323136u64),
);
pub struct BidRefund {
    pub bidder: Name,
    pub amount: Asset,
}
impl Table for BidRefund {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.bidder.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for BidRefund {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let bidder = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let amount = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = BidRefund { bidder, amount };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for BidRefund {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.bidder, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.amount, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for BidRefund {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.bidder);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.amount);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for BidRefund {
    #[inline]
    fn clone(&self) -> BidRefund {
        BidRefund {
            bidder: ::core::clone::Clone::clone(&self.bidder),
            amount: ::core::clone::Clone::clone(&self.amount),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for BidRefund {}
#[automatically_derived]
impl ::core::cmp::PartialEq for BidRefund {
    #[inline]
    fn eq(&self, other: &BidRefund) -> bool {
        self.bidder == other.bidder && self.amount == other.amount
    }
}
const BID_REFUND_TABLE: MultiIndexDefinition<BidRefund> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(4292903715935748096u64),
);
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
impl Table for ProducerInfo {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for ProducerInfo {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_votes = <f64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let producer_key = <PublicKey as ::pulse_cdt::Read>::read(bytes, pos)?;
        let is_active = <bool as ::pulse_cdt::Read>::read(bytes, pos)?;
        let url = <String as ::pulse_cdt::Read>::read(bytes, pos)?;
        let unpaid_blocks = <u32 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let last_claim_time = <TimePoint as ::pulse_cdt::Read>::read(bytes, pos)?;
        let location = <u16 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = ProducerInfo {
            owner,
            total_votes,
            producer_key,
            is_active,
            url,
            unpaid_blocks,
            last_claim_time,
            location,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for ProducerInfo {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_votes, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.producer_key, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.is_active, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.url, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.unpaid_blocks, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_claim_time, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.location, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for ProducerInfo {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_votes);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.producer_key);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.is_active);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.url);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.unpaid_blocks);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_claim_time);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.location);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ProducerInfo {
    #[inline]
    fn clone(&self) -> ProducerInfo {
        ProducerInfo {
            owner: ::core::clone::Clone::clone(&self.owner),
            total_votes: ::core::clone::Clone::clone(&self.total_votes),
            producer_key: ::core::clone::Clone::clone(&self.producer_key),
            is_active: ::core::clone::Clone::clone(&self.is_active),
            url: ::core::clone::Clone::clone(&self.url),
            unpaid_blocks: ::core::clone::Clone::clone(&self.unpaid_blocks),
            last_claim_time: ::core::clone::Clone::clone(&self.last_claim_time),
            location: ::core::clone::Clone::clone(&self.location),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ProducerInfo {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ProducerInfo {
    #[inline]
    fn eq(&self, other: &ProducerInfo) -> bool {
        self.total_votes == other.total_votes && self.is_active == other.is_active
            && self.unpaid_blocks == other.unpaid_blocks
            && self.location == other.location && self.owner == other.owner
            && self.producer_key == other.producer_key && self.url == other.url
            && self.last_claim_time == other.last_claim_time
    }
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
const PRODUCERS_TABLE: MultiIndexDefinition<ProducerInfo> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(12531438729690087424u64),
);
pub struct ProducerInfo2 {
    owner: Name,
    votepay_share: f64,
    last_votepay_share_update: TimePoint,
}
impl Table for ProducerInfo2 {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for ProducerInfo2 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let votepay_share = <f64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let last_votepay_share_update = <TimePoint as ::pulse_cdt::Read>::read(
            bytes,
            pos,
        )?;
        let item = ProducerInfo2 {
            owner,
            votepay_share,
            last_votepay_share_update,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for ProducerInfo2 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.votepay_share, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_votepay_share_update, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for ProducerInfo2 {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.votepay_share);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_votepay_share_update);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ProducerInfo2 {
    #[inline]
    fn clone(&self) -> ProducerInfo2 {
        ProducerInfo2 {
            owner: ::core::clone::Clone::clone(&self.owner),
            votepay_share: ::core::clone::Clone::clone(&self.votepay_share),
            last_votepay_share_update: ::core::clone::Clone::clone(
                &self.last_votepay_share_update,
            ),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for ProducerInfo2 {}
#[automatically_derived]
impl ::core::cmp::PartialEq for ProducerInfo2 {
    #[inline]
    fn eq(&self, other: &ProducerInfo2) -> bool {
        self.votepay_share == other.votepay_share && self.owner == other.owner
            && self.last_votepay_share_update == other.last_votepay_share_update
    }
}
const PRODUCERS_TABLE2: MultiIndexDefinition<ProducerInfo2> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(12531438729690120192u64),
);
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
impl Table for VoterInfo {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for VoterInfo {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let proxy = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let producers = <Vec<Name> as ::pulse_cdt::Read>::read(bytes, pos)?;
        let staked = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let last_vote_weight = <f64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let proxied_vote_weight = <f64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let is_proxy = <bool as ::pulse_cdt::Read>::read(bytes, pos)?;
        let flags1 = <u32 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let reserved2 = <u32 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let reserved3 = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = VoterInfo {
            owner,
            proxy,
            producers,
            staked,
            last_vote_weight,
            proxied_vote_weight,
            is_proxy,
            flags1,
            reserved2,
            reserved3,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for VoterInfo {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.proxy, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.producers, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.staked, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_vote_weight, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.proxied_vote_weight, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.is_proxy, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.flags1, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.reserved2, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.reserved3, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for VoterInfo {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.proxy);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.producers);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.staked);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_vote_weight);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.proxied_vote_weight);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.is_proxy);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.flags1);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.reserved2);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.reserved3);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for VoterInfo {
    #[inline]
    fn clone(&self) -> VoterInfo {
        VoterInfo {
            owner: ::core::clone::Clone::clone(&self.owner),
            proxy: ::core::clone::Clone::clone(&self.proxy),
            producers: ::core::clone::Clone::clone(&self.producers),
            staked: ::core::clone::Clone::clone(&self.staked),
            last_vote_weight: ::core::clone::Clone::clone(&self.last_vote_weight),
            proxied_vote_weight: ::core::clone::Clone::clone(&self.proxied_vote_weight),
            is_proxy: ::core::clone::Clone::clone(&self.is_proxy),
            flags1: ::core::clone::Clone::clone(&self.flags1),
            reserved2: ::core::clone::Clone::clone(&self.reserved2),
            reserved3: ::core::clone::Clone::clone(&self.reserved3),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for VoterInfo {}
#[automatically_derived]
impl ::core::cmp::PartialEq for VoterInfo {
    #[inline]
    fn eq(&self, other: &VoterInfo) -> bool {
        self.staked == other.staked && self.last_vote_weight == other.last_vote_weight
            && self.proxied_vote_weight == other.proxied_vote_weight
            && self.is_proxy == other.is_proxy && self.flags1 == other.flags1
            && self.reserved2 == other.reserved2 && self.owner == other.owner
            && self.proxy == other.proxy && self.producers == other.producers
            && self.reserved3 == other.reserved3
    }
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
    fn to_bits(self) -> Self::Repr {
        self as u32
    }
}
const VOTERS_TABLE: MultiIndexDefinition<VoterInfo> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(15938991009778630656u64),
);
pub struct UserResources {
    owner: Name,
    net_weight: Asset,
    cpu_weight: Asset,
    ram_bytes: i64,
}
impl Table for UserResources {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for UserResources {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let net_weight = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let cpu_weight = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let ram_bytes = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = UserResources {
            owner,
            net_weight,
            cpu_weight,
            ram_bytes,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for UserResources {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.net_weight, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.cpu_weight, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.ram_bytes, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for UserResources {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.net_weight);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.cpu_weight);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.ram_bytes);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for UserResources {
    #[inline]
    fn clone(&self) -> UserResources {
        UserResources {
            owner: ::core::clone::Clone::clone(&self.owner),
            net_weight: ::core::clone::Clone::clone(&self.net_weight),
            cpu_weight: ::core::clone::Clone::clone(&self.cpu_weight),
            ram_bytes: ::core::clone::Clone::clone(&self.ram_bytes),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for UserResources {}
#[automatically_derived]
impl ::core::cmp::PartialEq for UserResources {
    #[inline]
    fn eq(&self, other: &UserResources) -> bool {
        self.ram_bytes == other.ram_bytes && self.owner == other.owner
            && self.net_weight == other.net_weight && self.cpu_weight == other.cpu_weight
    }
}
impl UserResources {
    pub fn is_empty(&self) -> bool {
        self.net_weight.amount == 0 && self.cpu_weight.amount == 0 && self.ram_bytes == 0
    }
}
const USER_RESOURCES_TABLE: MultiIndexDefinition<UserResources> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(15426372072997126144u64),
);
pub struct DelegatedBandwidth {
    from: Name,
    to: Name,
    net_weight: Asset,
    cpu_weight: Asset,
}
impl Table for DelegatedBandwidth {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.to.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for DelegatedBandwidth {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let from = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let to = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let net_weight = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let cpu_weight = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = DelegatedBandwidth {
            from,
            to,
            net_weight,
            cpu_weight,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for DelegatedBandwidth {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.from, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.to, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.net_weight, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.cpu_weight, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for DelegatedBandwidth {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.from);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.to);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.net_weight);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.cpu_weight);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for DelegatedBandwidth {
    #[inline]
    fn clone(&self) -> DelegatedBandwidth {
        DelegatedBandwidth {
            from: ::core::clone::Clone::clone(&self.from),
            to: ::core::clone::Clone::clone(&self.to),
            net_weight: ::core::clone::Clone::clone(&self.net_weight),
            cpu_weight: ::core::clone::Clone::clone(&self.cpu_weight),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for DelegatedBandwidth {}
#[automatically_derived]
impl ::core::cmp::PartialEq for DelegatedBandwidth {
    #[inline]
    fn eq(&self, other: &DelegatedBandwidth) -> bool {
        self.from == other.from && self.to == other.to
            && self.net_weight == other.net_weight && self.cpu_weight == other.cpu_weight
    }
}
impl DelegatedBandwidth {
    pub fn is_empty(&self) -> bool {
        self.net_weight.amount == 0 && self.cpu_weight.amount == 0
    }
}
const DEL_BANDWIDTH_TABLE: MultiIndexDefinition<DelegatedBandwidth> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(5377987680120340480u64),
);
pub struct RefundRequest {
    owner: Name,
    request_time: TimePointSec,
    net_amount: Asset,
    cpu_amount: Asset,
}
impl Table for RefundRequest {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for RefundRequest {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let request_time = <TimePointSec as ::pulse_cdt::Read>::read(bytes, pos)?;
        let net_amount = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let cpu_amount = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = RefundRequest {
            owner,
            request_time,
            net_amount,
            cpu_amount,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for RefundRequest {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.request_time, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.net_amount, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.cpu_amount, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for RefundRequest {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.request_time);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.net_amount);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.cpu_amount);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for RefundRequest {
    #[inline]
    fn clone(&self) -> RefundRequest {
        RefundRequest {
            owner: ::core::clone::Clone::clone(&self.owner),
            request_time: ::core::clone::Clone::clone(&self.request_time),
            net_amount: ::core::clone::Clone::clone(&self.net_amount),
            cpu_amount: ::core::clone::Clone::clone(&self.cpu_amount),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for RefundRequest {}
#[automatically_derived]
impl ::core::cmp::PartialEq for RefundRequest {
    #[inline]
    fn eq(&self, other: &RefundRequest) -> bool {
        self.owner == other.owner && self.request_time == other.request_time
            && self.net_amount == other.net_amount && self.cpu_amount == other.cpu_amount
    }
}
impl RefundRequest {
    pub fn is_empty(&self) -> bool {
        self.net_amount.amount == 0 && self.cpu_amount.amount == 0
    }
}
const REFUNDS_TABLE: MultiIndexDefinition<RefundRequest> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(13445401747262537728u64),
);
pub struct DelegatedXPR {
    from: Name,
    to: Name,
    quantity: Asset,
}
impl Table for DelegatedXPR {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.to.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for DelegatedXPR {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let from = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let to = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let quantity = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = DelegatedXPR { from, to, quantity };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for DelegatedXPR {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.from, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.to, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.quantity, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for DelegatedXPR {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.from);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.to);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.quantity);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for DelegatedXPR {
    #[inline]
    fn clone(&self) -> DelegatedXPR {
        DelegatedXPR {
            from: ::core::clone::Clone::clone(&self.from),
            to: ::core::clone::Clone::clone(&self.to),
            quantity: ::core::clone::Clone::clone(&self.quantity),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for DelegatedXPR {}
#[automatically_derived]
impl ::core::cmp::PartialEq for DelegatedXPR {
    #[inline]
    fn eq(&self, other: &DelegatedXPR) -> bool {
        self.from == other.from && self.to == other.to && self.quantity == other.quantity
    }
}
const DEL_XPR_TABLE: MultiIndexDefinition<DelegatedXPR> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(5378383018438164480u64),
);
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
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for VotersXPR {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let staked = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let isqualified = <bool as ::pulse_cdt::Read>::read(bytes, pos)?;
        let claimamount = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let lastclaim = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let startstake = <Option<u64> as ::pulse_cdt::Read>::read(bytes, pos)?;
        let startqualif = <Option<bool> as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = VotersXPR {
            owner,
            staked,
            isqualified,
            claimamount,
            lastclaim,
            startstake,
            startqualif,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for VotersXPR {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.staked, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.isqualified, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.claimamount, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.lastclaim, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.startstake, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.startqualif, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for VotersXPR {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.staked);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.isqualified);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.claimamount);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.lastclaim);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.startstake);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.startqualif);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for VotersXPR {
    #[inline]
    fn clone(&self) -> VotersXPR {
        VotersXPR {
            owner: ::core::clone::Clone::clone(&self.owner),
            staked: ::core::clone::Clone::clone(&self.staked),
            isqualified: ::core::clone::Clone::clone(&self.isqualified),
            claimamount: ::core::clone::Clone::clone(&self.claimamount),
            lastclaim: ::core::clone::Clone::clone(&self.lastclaim),
            startstake: ::core::clone::Clone::clone(&self.startstake),
            startqualif: ::core::clone::Clone::clone(&self.startqualif),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for VotersXPR {}
#[automatically_derived]
impl ::core::cmp::PartialEq for VotersXPR {
    #[inline]
    fn eq(&self, other: &VotersXPR) -> bool {
        self.staked == other.staked && self.isqualified == other.isqualified
            && self.claimamount == other.claimamount && self.lastclaim == other.lastclaim
            && self.owner == other.owner && self.startstake == other.startstake
            && self.startqualif == other.startqualif
    }
}
const VOTERS_XPR_TABLE: MultiIndexDefinition<VotersXPR> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(15938991025712267264u64),
);
pub struct XPRRefundRequest {
    owner: Name,
    request_time: TimePointSec,
    quantity: Asset,
}
impl Table for XPRRefundRequest {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for XPRRefundRequest {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let request_time = <TimePointSec as ::pulse_cdt::Read>::read(bytes, pos)?;
        let quantity = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = XPRRefundRequest {
            owner,
            request_time,
            quantity,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for XPRRefundRequest {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.request_time, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.quantity, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for XPRRefundRequest {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.request_time);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.quantity);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for XPRRefundRequest {
    #[inline]
    fn clone(&self) -> XPRRefundRequest {
        XPRRefundRequest {
            owner: ::core::clone::Clone::clone(&self.owner),
            request_time: ::core::clone::Clone::clone(&self.request_time),
            quantity: ::core::clone::Clone::clone(&self.quantity),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for XPRRefundRequest {}
#[automatically_derived]
impl ::core::cmp::PartialEq for XPRRefundRequest {
    #[inline]
    fn eq(&self, other: &XPRRefundRequest) -> bool {
        self.owner == other.owner && self.request_time == other.request_time
            && self.quantity == other.quantity
    }
}
const XPR_REFUNDS_TABLE: MultiIndexDefinition<XPRRefundRequest> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(13445401747760463872u64),
);
pub struct GlobalStateXPR {
    max_bp_per_vote: u64,
    min_bp_reward: u64,
    unstake_period: u64,
    process_by: u64,
    process_interval: u64,
    voters_claim_interval: u64,
    spare1: u64,
    spare2: u64,
}
impl Table for GlobalStateXPR {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for GlobalStateXPR {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let max_bp_per_vote = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let min_bp_reward = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let unstake_period = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let process_by = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let process_interval = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let voters_claim_interval = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let spare1 = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let spare2 = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = GlobalStateXPR {
            max_bp_per_vote,
            min_bp_reward,
            unstake_period,
            process_by,
            process_interval,
            voters_claim_interval,
            spare1,
            spare2,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for GlobalStateXPR {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.max_bp_per_vote, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.min_bp_reward, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.unstake_period, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.process_by, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.process_interval, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.voters_claim_interval, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.spare1, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.spare2, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for GlobalStateXPR {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.max_bp_per_vote);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.min_bp_reward);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.unstake_period);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.process_by);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.process_interval);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.voters_claim_interval);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.spare1);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.spare2);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for GlobalStateXPR {
    #[inline]
    fn clone(&self) -> GlobalStateXPR {
        GlobalStateXPR {
            max_bp_per_vote: ::core::clone::Clone::clone(&self.max_bp_per_vote),
            min_bp_reward: ::core::clone::Clone::clone(&self.min_bp_reward),
            unstake_period: ::core::clone::Clone::clone(&self.unstake_period),
            process_by: ::core::clone::Clone::clone(&self.process_by),
            process_interval: ::core::clone::Clone::clone(&self.process_interval),
            voters_claim_interval: ::core::clone::Clone::clone(
                &self.voters_claim_interval,
            ),
            spare1: ::core::clone::Clone::clone(&self.spare1),
            spare2: ::core::clone::Clone::clone(&self.spare2),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GlobalStateXPR {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GlobalStateXPR {
    #[inline]
    fn eq(&self, other: &GlobalStateXPR) -> bool {
        self.max_bp_per_vote == other.max_bp_per_vote
            && self.min_bp_reward == other.min_bp_reward
            && self.unstake_period == other.unstake_period
            && self.process_by == other.process_by
            && self.process_interval == other.process_interval
            && self.voters_claim_interval == other.voters_claim_interval
            && self.spare1 == other.spare1 && self.spare2 == other.spare2
    }
}
const GLOBAL_STATEXPR_SINGLETON: MultiIndexDefinition<GlobalStateXPR> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(7235159550648500224u64),
);
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
impl Table for GlobalStateD {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for GlobalStateD {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let totalstaked = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let totalrstaked = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let totalrvoters = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let notclaimed = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let pool = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let processtime = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let processtimeupd = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let isprocessing = <bool as ::pulse_cdt::Read>::read(bytes, pos)?;
        let process_from = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let process_quant = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let processrstaked = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let processed = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let spare1 = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let spare2 = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = GlobalStateD {
            totalstaked,
            totalrstaked,
            totalrvoters,
            notclaimed,
            pool,
            processtime,
            processtimeupd,
            isprocessing,
            process_from,
            process_quant,
            processrstaked,
            processed,
            spare1,
            spare2,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for GlobalStateD {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.totalstaked, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.totalrstaked, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.totalrvoters, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.notclaimed, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.pool, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.processtime, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.processtimeupd, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.isprocessing, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.process_from, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.process_quant, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.processrstaked, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.processed, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.spare1, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.spare2, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for GlobalStateD {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.totalstaked);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.totalrstaked);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.totalrvoters);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.notclaimed);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.pool);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.processtime);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.processtimeupd);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.isprocessing);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.process_from);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.process_quant);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.processrstaked);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.processed);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.spare1);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.spare2);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for GlobalStateD {
    #[inline]
    fn clone(&self) -> GlobalStateD {
        GlobalStateD {
            totalstaked: ::core::clone::Clone::clone(&self.totalstaked),
            totalrstaked: ::core::clone::Clone::clone(&self.totalrstaked),
            totalrvoters: ::core::clone::Clone::clone(&self.totalrvoters),
            notclaimed: ::core::clone::Clone::clone(&self.notclaimed),
            pool: ::core::clone::Clone::clone(&self.pool),
            processtime: ::core::clone::Clone::clone(&self.processtime),
            processtimeupd: ::core::clone::Clone::clone(&self.processtimeupd),
            isprocessing: ::core::clone::Clone::clone(&self.isprocessing),
            process_from: ::core::clone::Clone::clone(&self.process_from),
            process_quant: ::core::clone::Clone::clone(&self.process_quant),
            processrstaked: ::core::clone::Clone::clone(&self.processrstaked),
            processed: ::core::clone::Clone::clone(&self.processed),
            spare1: ::core::clone::Clone::clone(&self.spare1),
            spare2: ::core::clone::Clone::clone(&self.spare2),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GlobalStateD {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GlobalStateD {
    #[inline]
    fn eq(&self, other: &GlobalStateD) -> bool {
        self.totalstaked == other.totalstaked && self.totalrstaked == other.totalrstaked
            && self.totalrvoters == other.totalrvoters
            && self.notclaimed == other.notclaimed && self.pool == other.pool
            && self.processtime == other.processtime
            && self.processtimeupd == other.processtimeupd
            && self.isprocessing == other.isprocessing
            && self.process_quant == other.process_quant
            && self.processrstaked == other.processrstaked
            && self.processed == other.processed && self.spare1 == other.spare1
            && self.spare2 == other.spare2 && self.process_from == other.process_from
    }
}
const GLOBAL_STATESD_SINGLETON: MultiIndexDefinition<GlobalStateD> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(7235159550301569024u64),
);
pub struct GlobalStateRAM {
    ram_price_per_byte: Asset,
    max_per_user_bytes: u64,
    ram_fee_percent: u64,
    total_ram: u64,
    total_xpr: u64,
}
impl Table for GlobalStateRAM {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for GlobalStateRAM {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let ram_price_per_byte = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let max_per_user_bytes = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let ram_fee_percent = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_ram = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_xpr = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = GlobalStateRAM {
            ram_price_per_byte,
            max_per_user_bytes,
            ram_fee_percent,
            total_ram,
            total_xpr,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for GlobalStateRAM {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.ram_price_per_byte, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.max_per_user_bytes, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.ram_fee_percent, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_ram, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_xpr, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for GlobalStateRAM {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.ram_price_per_byte);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.max_per_user_bytes);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.ram_fee_percent);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_ram);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_xpr);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for GlobalStateRAM {
    #[inline]
    fn clone(&self) -> GlobalStateRAM {
        GlobalStateRAM {
            ram_price_per_byte: ::core::clone::Clone::clone(&self.ram_price_per_byte),
            max_per_user_bytes: ::core::clone::Clone::clone(&self.max_per_user_bytes),
            ram_fee_percent: ::core::clone::Clone::clone(&self.ram_fee_percent),
            total_ram: ::core::clone::Clone::clone(&self.total_ram),
            total_xpr: ::core::clone::Clone::clone(&self.total_xpr),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GlobalStateRAM {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GlobalStateRAM {
    #[inline]
    fn eq(&self, other: &GlobalStateRAM) -> bool {
        self.max_per_user_bytes == other.max_per_user_bytes
            && self.ram_fee_percent == other.ram_fee_percent
            && self.total_ram == other.total_ram && self.total_xpr == other.total_xpr
            && self.ram_price_per_byte == other.ram_price_per_byte
    }
}
impl Default for GlobalStateRAM {
    fn default() -> Self {
        Self {
            ram_price_per_byte: Asset {
                amount: 200,
                symbol: { ::pulse_cdt::core::Symbol::new(1380997124u64) },
            },
            max_per_user_bytes: 3 * 1024 * 1024,
            ram_fee_percent: 1000,
            total_ram: 0,
            total_xpr: 0,
        }
    }
}
const GLOBAL_STATE_RAM_SINGLETON: SingletonDefinition<GlobalStateRAM> = SingletonDefinition::new(
    pulse_cdt::core::Name::new(7235159549723803648u64),
);
pub struct UserRAM {
    account: Name,
    ram: u64,
    quantity: Asset,
    ramlimit: u64,
}
impl Table for UserRAM {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.account.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for UserRAM {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let account = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let ram = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let quantity = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let ramlimit = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = UserRAM {
            account,
            ram,
            quantity,
            ramlimit,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for UserRAM {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.account, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.ram, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.quantity, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.ramlimit, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for UserRAM {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.account);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.ram);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.quantity);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.ramlimit);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for UserRAM {
    #[inline]
    fn clone(&self) -> UserRAM {
        UserRAM {
            account: ::core::clone::Clone::clone(&self.account),
            ram: ::core::clone::Clone::clone(&self.ram),
            quantity: ::core::clone::Clone::clone(&self.quantity),
            ramlimit: ::core::clone::Clone::clone(&self.ramlimit),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for UserRAM {}
#[automatically_derived]
impl ::core::cmp::PartialEq for UserRAM {
    #[inline]
    fn eq(&self, other: &UserRAM) -> bool {
        self.ram == other.ram && self.ramlimit == other.ramlimit
            && self.account == other.account && self.quantity == other.quantity
    }
}
const USERRAM_TABLE: MultiIndexDefinition<UserRAM> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(15426372836729552896u64),
);
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
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for RexPool {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let version = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_lent = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_unlent = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_rent = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_lendable = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_rex = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let namebid_proceeds = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let loan_num = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = RexPool {
            version,
            total_lent,
            total_unlent,
            total_rent,
            total_lendable,
            total_rex,
            namebid_proceeds,
            loan_num,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for RexPool {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.version, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_lent, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_unlent, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_rent, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_lendable, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_rex, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.namebid_proceeds, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.loan_num, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for RexPool {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.version);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_lent);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_unlent);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_rent);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_lendable);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_rex);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.namebid_proceeds);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.loan_num);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for RexPool {
    #[inline]
    fn clone(&self) -> RexPool {
        RexPool {
            version: ::core::clone::Clone::clone(&self.version),
            total_lent: ::core::clone::Clone::clone(&self.total_lent),
            total_unlent: ::core::clone::Clone::clone(&self.total_unlent),
            total_rent: ::core::clone::Clone::clone(&self.total_rent),
            total_lendable: ::core::clone::Clone::clone(&self.total_lendable),
            total_rex: ::core::clone::Clone::clone(&self.total_rex),
            namebid_proceeds: ::core::clone::Clone::clone(&self.namebid_proceeds),
            loan_num: ::core::clone::Clone::clone(&self.loan_num),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for RexPool {}
#[automatically_derived]
impl ::core::cmp::PartialEq for RexPool {
    #[inline]
    fn eq(&self, other: &RexPool) -> bool {
        self.version == other.version && self.loan_num == other.loan_num
            && self.total_lent == other.total_lent
            && self.total_unlent == other.total_unlent
            && self.total_rent == other.total_rent
            && self.total_lendable == other.total_lendable
            && self.total_rex == other.total_rex
            && self.namebid_proceeds == other.namebid_proceeds
    }
}
const REX_POOL_TABLE: MultiIndexDefinition<RexPool> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(13455447620470177792u64),
);
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
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for RexReturnPool {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let version = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let last_dist_time = <TimePointSec as ::pulse_cdt::Read>::read(bytes, pos)?;
        let pending_bucket_time = <TimePointSec as ::pulse_cdt::Read>::read(bytes, pos)?;
        let oldest_bucket_time = <TimePointSec as ::pulse_cdt::Read>::read(bytes, pos)?;
        let pending_bucket_proceeds = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let current_rate_of_proceeds = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let proceeds = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = RexReturnPool {
            version,
            last_dist_time,
            pending_bucket_time,
            oldest_bucket_time,
            pending_bucket_proceeds,
            current_rate_of_proceeds,
            proceeds,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for RexReturnPool {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.version, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_dist_time, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.pending_bucket_time, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.oldest_bucket_time, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.pending_bucket_proceeds, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.current_rate_of_proceeds, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.proceeds, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for RexReturnPool {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.version);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_dist_time);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.pending_bucket_time);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.oldest_bucket_time);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.pending_bucket_proceeds);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.current_rate_of_proceeds);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.proceeds);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for RexReturnPool {
    #[inline]
    fn clone(&self) -> RexReturnPool {
        RexReturnPool {
            version: ::core::clone::Clone::clone(&self.version),
            last_dist_time: ::core::clone::Clone::clone(&self.last_dist_time),
            pending_bucket_time: ::core::clone::Clone::clone(&self.pending_bucket_time),
            oldest_bucket_time: ::core::clone::Clone::clone(&self.oldest_bucket_time),
            pending_bucket_proceeds: ::core::clone::Clone::clone(
                &self.pending_bucket_proceeds,
            ),
            current_rate_of_proceeds: ::core::clone::Clone::clone(
                &self.current_rate_of_proceeds,
            ),
            proceeds: ::core::clone::Clone::clone(&self.proceeds),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for RexReturnPool {}
#[automatically_derived]
impl ::core::cmp::PartialEq for RexReturnPool {
    #[inline]
    fn eq(&self, other: &RexReturnPool) -> bool {
        self.version == other.version
            && self.pending_bucket_proceeds == other.pending_bucket_proceeds
            && self.current_rate_of_proceeds == other.current_rate_of_proceeds
            && self.proceeds == other.proceeds
            && self.last_dist_time == other.last_dist_time
            && self.pending_bucket_time == other.pending_bucket_time
            && self.oldest_bucket_time == other.oldest_bucket_time
    }
}
const REX_RETURN_POOL_TABLE: MultiIndexDefinition<RexReturnPool> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(13453195820656492544u64),
);
pub struct RexReturnBuckets {
    version: u8,
    return_buckets: BTreeMap<TimePointSec, i64>,
}
impl Table for RexReturnBuckets {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for RexReturnBuckets {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let version = <u8 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let return_buckets = <BTreeMap<
            TimePointSec,
            i64,
        > as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = RexReturnBuckets {
            version,
            return_buckets,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for RexReturnBuckets {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.version, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.return_buckets, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for RexReturnBuckets {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.version);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.return_buckets);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for RexReturnBuckets {
    #[inline]
    fn clone(&self) -> RexReturnBuckets {
        RexReturnBuckets {
            version: ::core::clone::Clone::clone(&self.version),
            return_buckets: ::core::clone::Clone::clone(&self.return_buckets),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for RexReturnBuckets {}
#[automatically_derived]
impl ::core::cmp::PartialEq for RexReturnBuckets {
    #[inline]
    fn eq(&self, other: &RexReturnBuckets) -> bool {
        self.version == other.version && self.return_buckets == other.return_buckets
    }
}
const REX_RETURN_BUCKETS_TABLE: MultiIndexDefinition<RexReturnBuckets> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(13452952622072725504u64),
);
pub struct RexFund {
    version: u8,
    owner: Name,
    balance: Asset,
}
impl Table for RexFund {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for RexFund {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let version = <u8 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let balance = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = RexFund { version, owner, balance };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for RexFund {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.version, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.balance, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for RexFund {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.version);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.balance);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for RexFund {
    #[inline]
    fn clone(&self) -> RexFund {
        RexFund {
            version: ::core::clone::Clone::clone(&self.version),
            owner: ::core::clone::Clone::clone(&self.owner),
            balance: ::core::clone::Clone::clone(&self.balance),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for RexFund {}
#[automatically_derived]
impl ::core::cmp::PartialEq for RexFund {
    #[inline]
    fn eq(&self, other: &RexFund) -> bool {
        self.version == other.version && self.owner == other.owner
            && self.balance == other.balance
    }
}
const REX_FUND_TABLE: MultiIndexDefinition<RexFund> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(13455274975669780480u64),
);
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
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for RexBalance {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let version = <u8 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let vote_stake = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let rex_balance = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let matured_rex = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = RexBalance {
            version,
            owner,
            vote_stake,
            rex_balance,
            matured_rex,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for RexBalance {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.version, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.vote_stake, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.rex_balance, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.matured_rex, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for RexBalance {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.version);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.vote_stake);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.rex_balance);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.matured_rex);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for RexBalance {
    #[inline]
    fn clone(&self) -> RexBalance {
        RexBalance {
            version: ::core::clone::Clone::clone(&self.version),
            owner: ::core::clone::Clone::clone(&self.owner),
            vote_stake: ::core::clone::Clone::clone(&self.vote_stake),
            rex_balance: ::core::clone::Clone::clone(&self.rex_balance),
            matured_rex: ::core::clone::Clone::clone(&self.matured_rex),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for RexBalance {}
#[automatically_derived]
impl ::core::cmp::PartialEq for RexBalance {
    #[inline]
    fn eq(&self, other: &RexBalance) -> bool {
        self.version == other.version && self.matured_rex == other.matured_rex
            && self.owner == other.owner && self.vote_stake == other.vote_stake
            && self.rex_balance == other.rex_balance
    }
}
const REX_BALANCE_TABLE: MultiIndexDefinition<RexBalance> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(13455193572617748480u64),
);
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
impl Table for RexLoan {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.version as u64)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for RexLoan {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let version = <u8 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let from = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let receiver = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let payment = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let balance = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_staked = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let loan_num = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let expiration = <TimePoint as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = RexLoan {
            version,
            from,
            receiver,
            payment,
            balance,
            total_staked,
            loan_num,
            expiration,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for RexLoan {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.version, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.from, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.receiver, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.payment, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.balance, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_staked, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.loan_num, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.expiration, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for RexLoan {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.version);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.from);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.receiver);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.payment);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.balance);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_staked);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.loan_num);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.expiration);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for RexLoan {
    #[inline]
    fn clone(&self) -> RexLoan {
        RexLoan {
            version: ::core::clone::Clone::clone(&self.version),
            from: ::core::clone::Clone::clone(&self.from),
            receiver: ::core::clone::Clone::clone(&self.receiver),
            payment: ::core::clone::Clone::clone(&self.payment),
            balance: ::core::clone::Clone::clone(&self.balance),
            total_staked: ::core::clone::Clone::clone(&self.total_staked),
            loan_num: ::core::clone::Clone::clone(&self.loan_num),
            expiration: ::core::clone::Clone::clone(&self.expiration),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for RexLoan {}
#[automatically_derived]
impl ::core::cmp::PartialEq for RexLoan {
    #[inline]
    fn eq(&self, other: &RexLoan) -> bool {
        self.version == other.version && self.loan_num == other.loan_num
            && self.from == other.from && self.receiver == other.receiver
            && self.payment == other.payment && self.balance == other.balance
            && self.total_staked == other.total_staked
            && self.expiration == other.expiration
    }
}
const REX_CPU_LOAN_TABLE: MultiIndexDefinition<RexLoan> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(5004935261474258944u64),
);
const REX_NET_LOAN_TABLE: MultiIndexDefinition<RexLoan> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(11147282203254194176u64),
);
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
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.owner.raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for RexOrder {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let version = <u8 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let owner = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let rex_requested = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let proceeds = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let stake_change = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let order_time = <TimePoint as ::pulse_cdt::Read>::read(bytes, pos)?;
        let is_open = <bool as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = RexOrder {
            version,
            owner,
            rex_requested,
            proceeds,
            stake_change,
            order_time,
            is_open,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for RexOrder {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.version, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.owner, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.rex_requested, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.proceeds, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.stake_change, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.order_time, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.is_open, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for RexOrder {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.version);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.owner);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.rex_requested);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.proceeds);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.stake_change);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.order_time);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.is_open);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for RexOrder {
    #[inline]
    fn clone(&self) -> RexOrder {
        RexOrder {
            version: ::core::clone::Clone::clone(&self.version),
            owner: ::core::clone::Clone::clone(&self.owner),
            rex_requested: ::core::clone::Clone::clone(&self.rex_requested),
            proceeds: ::core::clone::Clone::clone(&self.proceeds),
            stake_change: ::core::clone::Clone::clone(&self.stake_change),
            order_time: ::core::clone::Clone::clone(&self.order_time),
            is_open: ::core::clone::Clone::clone(&self.is_open),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for RexOrder {}
#[automatically_derived]
impl ::core::cmp::PartialEq for RexOrder {
    #[inline]
    fn eq(&self, other: &RexOrder) -> bool {
        self.version == other.version && self.is_open == other.is_open
            && self.owner == other.owner && self.rex_requested == other.rex_requested
            && self.proceeds == other.proceeds && self.stake_change == other.stake_change
            && self.order_time == other.order_time
    }
}
const ACTIVE_PERMISSION: Name = pulse_cdt::core::Name::new(3617214756542218240u64);
const TOKEN_ACCOUNT: Name = pulse_cdt::core::Name::new(12584048032615671296u64);
const RAM_ACCOUNT: Name = pulse_cdt::core::Name::new(12584048031307923456u64);
const RAMFEE_ACCOUNT: Name = pulse_cdt::core::Name::new(12584048031308108960u64);
const RAM_SYMBOL: Symbol = { ::pulse_cdt::core::Symbol::new(1296126464u64) };
const RAMCORE_SYMBOL: Symbol = {
    ::pulse_cdt::core::Symbol::new(4995142087184830980u64)
};
const REX_SYMBOL: Symbol = { ::pulse_cdt::core::Symbol::new(1480937988u64) };
const REX_ACCOUNT: Name = pulse_cdt::core::Name::new(12584048031380799488u64);
const STAKE_ACCOUNT: Name = pulse_cdt::core::Name::new(12584048032157537280u64);
const SECONDS_PER_DAY: u32 = 24 * 3600;
const USECONDS_PER_DAY: u64 = SECONDS_PER_DAY as u64 * 1000_000;
const MIN_ACTIVATED_STAKE: i64 = 150_000_000_0000;
const RAM_GIFT_BYTES: i64 = 1400;
const INFLATION_PRECISION: i64 = 100;
const DEFAULT_ANNUAL_RATE: i64 = 500;
const DEFAULT_INFLATION_PAY_FACTOR: i64 = 50000;
const DEFAULT_VOTEPAY_FACTOR: i64 = 40000;
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
impl Table for GlobalState {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for GlobalState {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let max_ram_size = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_ram_bytes_reserved = <u64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_ram_stake = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let last_producer_schedule_update = <BlockTimestamp as ::pulse_cdt::Read>::read(
            bytes,
            pos,
        )?;
        let last_pervote_bucket_fill = <TimePoint as ::pulse_cdt::Read>::read(
            bytes,
            pos,
        )?;
        let pervote_bucket = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let perblock_bucket = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_unpaid_blocks = <u32 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_activated_stake = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let thresh_activated_stake_time = <TimePoint as ::pulse_cdt::Read>::read(
            bytes,
            pos,
        )?;
        let last_producer_schedule_size = <u16 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_producer_vote_weight = <f64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let last_name_close = <BlockTimestamp as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = GlobalState {
            max_ram_size,
            total_ram_bytes_reserved,
            total_ram_stake,
            last_producer_schedule_update,
            last_pervote_bucket_fill,
            pervote_bucket,
            perblock_bucket,
            total_unpaid_blocks,
            total_activated_stake,
            thresh_activated_stake_time,
            last_producer_schedule_size,
            total_producer_vote_weight,
            last_name_close,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for GlobalState {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.max_ram_size, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_ram_bytes_reserved, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_ram_stake, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_producer_schedule_update, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_pervote_bucket_fill, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.pervote_bucket, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.perblock_bucket, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_unpaid_blocks, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_activated_stake, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.thresh_activated_stake_time, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_producer_schedule_size, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_producer_vote_weight, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_name_close, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for GlobalState {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.max_ram_size);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_ram_bytes_reserved);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_ram_stake);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_producer_schedule_update);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_pervote_bucket_fill);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.pervote_bucket);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.perblock_bucket);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_unpaid_blocks);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_activated_stake);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.thresh_activated_stake_time);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_producer_schedule_size);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_producer_vote_weight);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_name_close);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for GlobalState {
    #[inline]
    fn clone(&self) -> GlobalState {
        GlobalState {
            max_ram_size: ::core::clone::Clone::clone(&self.max_ram_size),
            total_ram_bytes_reserved: ::core::clone::Clone::clone(
                &self.total_ram_bytes_reserved,
            ),
            total_ram_stake: ::core::clone::Clone::clone(&self.total_ram_stake),
            last_producer_schedule_update: ::core::clone::Clone::clone(
                &self.last_producer_schedule_update,
            ),
            last_pervote_bucket_fill: ::core::clone::Clone::clone(
                &self.last_pervote_bucket_fill,
            ),
            pervote_bucket: ::core::clone::Clone::clone(&self.pervote_bucket),
            perblock_bucket: ::core::clone::Clone::clone(&self.perblock_bucket),
            total_unpaid_blocks: ::core::clone::Clone::clone(&self.total_unpaid_blocks),
            total_activated_stake: ::core::clone::Clone::clone(
                &self.total_activated_stake,
            ),
            thresh_activated_stake_time: ::core::clone::Clone::clone(
                &self.thresh_activated_stake_time,
            ),
            last_producer_schedule_size: ::core::clone::Clone::clone(
                &self.last_producer_schedule_size,
            ),
            total_producer_vote_weight: ::core::clone::Clone::clone(
                &self.total_producer_vote_weight,
            ),
            last_name_close: ::core::clone::Clone::clone(&self.last_name_close),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GlobalState {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GlobalState {
    #[inline]
    fn eq(&self, other: &GlobalState) -> bool {
        self.max_ram_size == other.max_ram_size
            && self.total_ram_bytes_reserved == other.total_ram_bytes_reserved
            && self.total_ram_stake == other.total_ram_stake
            && self.pervote_bucket == other.pervote_bucket
            && self.perblock_bucket == other.perblock_bucket
            && self.total_unpaid_blocks == other.total_unpaid_blocks
            && self.total_activated_stake == other.total_activated_stake
            && self.last_producer_schedule_size == other.last_producer_schedule_size
            && self.total_producer_vote_weight == other.total_producer_vote_weight
            && self.last_producer_schedule_update == other.last_producer_schedule_update
            && self.last_pervote_bucket_fill == other.last_pervote_bucket_fill
            && self.thresh_activated_stake_time == other.thresh_activated_stake_time
            && self.last_name_close == other.last_name_close
    }
}
#[automatically_derived]
impl ::core::default::Default for GlobalState {
    #[inline]
    fn default() -> GlobalState {
        GlobalState {
            max_ram_size: ::core::default::Default::default(),
            total_ram_bytes_reserved: ::core::default::Default::default(),
            total_ram_stake: ::core::default::Default::default(),
            last_producer_schedule_update: ::core::default::Default::default(),
            last_pervote_bucket_fill: ::core::default::Default::default(),
            pervote_bucket: ::core::default::Default::default(),
            perblock_bucket: ::core::default::Default::default(),
            total_unpaid_blocks: ::core::default::Default::default(),
            total_activated_stake: ::core::default::Default::default(),
            thresh_activated_stake_time: ::core::default::Default::default(),
            last_producer_schedule_size: ::core::default::Default::default(),
            total_producer_vote_weight: ::core::default::Default::default(),
            last_name_close: ::core::default::Default::default(),
        }
    }
}
impl GlobalState {
    #[inline]
    pub const fn free_ram(&self) -> u64 {
        self.max_ram_size - self.total_ram_bytes_reserved
    }
}
const GLOBAL_STATE_SINGLETON: SingletonDefinition<GlobalState> = SingletonDefinition::new(
    pulse_cdt::core::Name::new(7235159537265672192u64),
);
pub struct GlobalState2 {
    new_ram_per_block: u16,
    last_ram_increase: BlockTimestamp,
    last_block_num: BlockTimestamp,
    total_producer_votepay_share: f64,
    revision: u8,
}
impl Table for GlobalState2 {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for GlobalState2 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let new_ram_per_block = <u16 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let last_ram_increase = <BlockTimestamp as ::pulse_cdt::Read>::read(bytes, pos)?;
        let last_block_num = <BlockTimestamp as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_producer_votepay_share = <f64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let revision = <u8 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = GlobalState2 {
            new_ram_per_block,
            last_ram_increase,
            last_block_num,
            total_producer_votepay_share,
            revision,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for GlobalState2 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.new_ram_per_block, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_ram_increase, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.last_block_num, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_producer_votepay_share, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.revision, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for GlobalState2 {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.new_ram_per_block);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_ram_increase);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_block_num);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_producer_votepay_share);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.revision);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for GlobalState2 {
    #[inline]
    fn clone(&self) -> GlobalState2 {
        GlobalState2 {
            new_ram_per_block: ::core::clone::Clone::clone(&self.new_ram_per_block),
            last_ram_increase: ::core::clone::Clone::clone(&self.last_ram_increase),
            last_block_num: ::core::clone::Clone::clone(&self.last_block_num),
            total_producer_votepay_share: ::core::clone::Clone::clone(
                &self.total_producer_votepay_share,
            ),
            revision: ::core::clone::Clone::clone(&self.revision),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GlobalState2 {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GlobalState2 {
    #[inline]
    fn eq(&self, other: &GlobalState2) -> bool {
        self.new_ram_per_block == other.new_ram_per_block
            && self.total_producer_votepay_share == other.total_producer_votepay_share
            && self.revision == other.revision
            && self.last_ram_increase == other.last_ram_increase
            && self.last_block_num == other.last_block_num
    }
}
#[automatically_derived]
impl ::core::default::Default for GlobalState2 {
    #[inline]
    fn default() -> GlobalState2 {
        GlobalState2 {
            new_ram_per_block: ::core::default::Default::default(),
            last_ram_increase: ::core::default::Default::default(),
            last_block_num: ::core::default::Default::default(),
            total_producer_votepay_share: ::core::default::Default::default(),
            revision: ::core::default::Default::default(),
        }
    }
}
const GLOBAL_STATE2_SINGLETON: SingletonDefinition<GlobalState2> = SingletonDefinition::new(
    pulse_cdt::core::Name::new(7235159538339414016u64),
);
pub struct GlobalState3 {
    last_vpay_state_update: TimePoint,
    total_vpay_share_change_rate: f64,
}
impl Table for GlobalState3 {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for GlobalState3 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let last_vpay_state_update = <TimePoint as ::pulse_cdt::Read>::read(bytes, pos)?;
        let total_vpay_share_change_rate = <f64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = GlobalState3 {
            last_vpay_state_update,
            total_vpay_share_change_rate,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for GlobalState3 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.last_vpay_state_update, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.total_vpay_share_change_rate, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for GlobalState3 {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.last_vpay_state_update);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.total_vpay_share_change_rate);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for GlobalState3 {
    #[inline]
    fn clone(&self) -> GlobalState3 {
        GlobalState3 {
            last_vpay_state_update: ::core::clone::Clone::clone(
                &self.last_vpay_state_update,
            ),
            total_vpay_share_change_rate: ::core::clone::Clone::clone(
                &self.total_vpay_share_change_rate,
            ),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GlobalState3 {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GlobalState3 {
    #[inline]
    fn eq(&self, other: &GlobalState3) -> bool {
        self.total_vpay_share_change_rate == other.total_vpay_share_change_rate
            && self.last_vpay_state_update == other.last_vpay_state_update
    }
}
#[automatically_derived]
impl ::core::default::Default for GlobalState3 {
    #[inline]
    fn default() -> GlobalState3 {
        GlobalState3 {
            last_vpay_state_update: ::core::default::Default::default(),
            total_vpay_share_change_rate: ::core::default::Default::default(),
        }
    }
}
const GLOBAL_STATE3_SINGLETON: SingletonDefinition<GlobalState3> = SingletonDefinition::new(
    pulse_cdt::core::Name::new(7235159538876284928u64),
);
pub struct GlobalState4 {
    continuous_rate: f64,
    inflation_pay_factor: i64,
    votepay_factor: i64,
}
impl Table for GlobalState4 {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (0)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for GlobalState4 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let continuous_rate = <f64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let inflation_pay_factor = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let votepay_factor = <i64 as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = GlobalState4 {
            continuous_rate,
            inflation_pay_factor,
            votepay_factor,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for GlobalState4 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.continuous_rate, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.inflation_pay_factor, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.votepay_factor, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for GlobalState4 {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.continuous_rate);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.inflation_pay_factor);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.votepay_factor);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for GlobalState4 {
    #[inline]
    fn clone(&self) -> GlobalState4 {
        GlobalState4 {
            continuous_rate: ::core::clone::Clone::clone(&self.continuous_rate),
            inflation_pay_factor: ::core::clone::Clone::clone(
                &self.inflation_pay_factor,
            ),
            votepay_factor: ::core::clone::Clone::clone(&self.votepay_factor),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for GlobalState4 {}
#[automatically_derived]
impl ::core::cmp::PartialEq for GlobalState4 {
    #[inline]
    fn eq(&self, other: &GlobalState4) -> bool {
        self.continuous_rate == other.continuous_rate
            && self.inflation_pay_factor == other.inflation_pay_factor
            && self.votepay_factor == other.votepay_factor
    }
}
#[automatically_derived]
impl ::core::default::Default for GlobalState4 {
    #[inline]
    fn default() -> GlobalState4 {
        GlobalState4 {
            continuous_rate: ::core::default::Default::default(),
            inflation_pay_factor: ::core::default::Default::default(),
            votepay_factor: ::core::default::Default::default(),
        }
    }
}
const GLOBAL_STATE4_SINGLETON: SingletonDefinition<GlobalState4> = SingletonDefinition::new(
    pulse_cdt::core::Name::new(7235159539413155840u64),
);
pub struct CurrencyStats {
    pub supply: Asset,
    pub max_supply: Asset,
    pub issuer: Name,
}
impl Table for CurrencyStats {
    type Key = u64;
    type Row = Self;
    #[inline]
    fn primary_key(row: &Self::Row) -> u64 {
        (row.supply.symbol.code().raw())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Read for CurrencyStats {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ::pulse_cdt::ReadError> {
        let supply = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let max_supply = <Asset as ::pulse_cdt::Read>::read(bytes, pos)?;
        let issuer = <Name as ::pulse_cdt::Read>::read(bytes, pos)?;
        let item = CurrencyStats {
            supply,
            max_supply,
            issuer,
        };
        Ok(item)
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::Write for CurrencyStats {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), ::pulse_cdt::WriteError> {
        ::pulse_cdt::Write::write(&self.supply, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.max_supply, bytes, pos)?;
        ::pulse_cdt::Write::write(&self.issuer, bytes, pos)?;
        Ok(())
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::pulse_cdt::NumBytes for CurrencyStats {
    #[inline]
    fn num_bytes(&self) -> usize {
        let mut count = 0;
        count += ::pulse_cdt::NumBytes::num_bytes(&self.supply);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.max_supply);
        count += ::pulse_cdt::NumBytes::num_bytes(&self.issuer);
        count
    }
}
#[automatically_derived]
impl ::core::clone::Clone for CurrencyStats {
    #[inline]
    fn clone(&self) -> CurrencyStats {
        CurrencyStats {
            supply: ::core::clone::Clone::clone(&self.supply),
            max_supply: ::core::clone::Clone::clone(&self.max_supply),
            issuer: ::core::clone::Clone::clone(&self.issuer),
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for CurrencyStats {}
#[automatically_derived]
impl ::core::cmp::PartialEq for CurrencyStats {
    #[inline]
    fn eq(&self, other: &CurrencyStats) -> bool {
        self.supply == other.supply && self.max_supply == other.max_supply
            && self.issuer == other.issuer
    }
}
const STATS: MultiIndexDefinition<CurrencyStats> = MultiIndexDefinition::new(
    pulse_cdt::core::Name::new(14289248716530384896u64),
);
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
            pulse_cdt::core::Name::new(12584048018849792000u64)
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
const OPEN_ACTION: ActionWrapper<(Name, Symbol, Name)> = ActionWrapper::new(
    pulse_cdt::core::Name::new(11913481165836648448u64),
);
const TRANSFER_ACTION: ActionWrapper<(Name, Name, Asset, String)> = ActionWrapper::new(
    pulse_cdt::core::Name::new(14829575313431724032u64),
);
impl SystemContract {
    fn constructor() -> Self {
        let global = GLOBAL_STATE_SINGLETON.get_instance(get_self(), get_self().raw());
        let global2 = GLOBAL_STATE2_SINGLETON.get_instance(get_self(), get_self().raw());
        let global3 = GLOBAL_STATE3_SINGLETON.get_instance(get_self(), get_self().raw());
        let global4 = GLOBAL_STATE4_SINGLETON.get_instance(get_self(), get_self().raw());
        let gstateram = GLOBAL_STATE_RAM_SINGLETON
            .get_instance(get_self(), get_self().raw());
        Self {
            gstate: if global.exists() { global.get() } else { GlobalState::default() },
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
    fn destructor(self) {
        let global = GLOBAL_STATE_SINGLETON.get_instance(get_self(), get_self().raw());
        let global2 = GLOBAL_STATE2_SINGLETON.get_instance(get_self(), get_self().raw());
        let global3 = GLOBAL_STATE3_SINGLETON.get_instance(get_self(), get_self().raw());
        let global4 = GLOBAL_STATE4_SINGLETON.get_instance(get_self(), get_self().raw());
        let gstateram = GLOBAL_STATE_RAM_SINGLETON
            .get_instance(get_self(), get_self().raw());
        global.set(self.gstate, get_self());
        global2.set(self.gstate2, get_self());
        global3.set(self.gstate3, get_self());
        global4.set(self.gstate4, get_self());
        gstateram.set(self.gstateram, get_self());
    }
    fn setpriv(account: Name, is_priv: u8) {
        require_auth(get_self());
        set_privileged(account, is_priv == 1);
    }
    fn newaccount(creator: Name, newact: Name, owner: Authority, active: Authority) {
        if creator != get_self()
            && creator != pulse_cdt::core::Name::new(12531717089343307776u64)
        {
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
                    check(creator == suffix, "only suffix may create this account");
                }
            }
            check(newact.to_string().chars().count() > 3, "minimum 4 character length");
        }
        let userres = USER_RESOURCES_TABLE.index(get_self(), newact.raw());
        let core_symbol = get_core_symbol(None);
        userres
            .emplace(
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
    fn setabi(account: Name, abi: Vec<u8>) {
        let table = ABI_HASH_TABLE.index(get_self(), get_self().raw());
        let mut itr = table.find(account.raw());
        if itr == table.end() {
            table
                .emplace(
                    account,
                    AbiHash {
                        owner: account,
                        hash: sha256(&abi, abi.len() as u32),
                    },
                );
        } else {
            table
                .modify(
                    &mut itr,
                    SAME_PAYER,
                    |t| {
                        t.hash = sha256(&abi, abi.len() as u32);
                    },
                );
        }
    }
    fn setcode(account: Name, vmtype: u8, vmversion: u8, code: Vec<u8>) {}
    fn init(&self, version: u8, core: Symbol) {
        require_auth(get_self());
        check(version == 0, "unsupported version for init action");
        let rammarket = RAMMARKET.index(get_self(), get_self().raw());
        let itr = rammarket.find(RAMCORE_SYMBOL.raw());
        check(itr == rammarket.end(), "system contract has already been initialized");
        let system_token_supply = get_supply(TOKEN_ACCOUNT, core.code());
        check(
            system_token_supply.symbol == core,
            "specified core symbol does not exist (precision mismatch)",
        );
        check(
            system_token_supply.amount > 0,
            "system token supply must be greater than 0",
        );
        rammarket
            .emplace(
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
                <[_]>::into_vec(
                    ::alloc::boxed::box_new([
                        PermissionLevel::new(get_self(), ACTIVE_PERMISSION),
                    ]),
                ),
                (REX_ACCOUNT, core, get_self()),
            )
            .send();
    }
    fn buyrambsys(&mut self, payer: Name, receiver: Name, bytes: u32) {
        let rammarket = RAMMARKET.index(get_self(), get_self().raw());
        let itr = rammarket.find(RAMCORE_SYMBOL.raw());
        let ram_reserve = itr.base.balance.amount;
        let eos_reserve = itr.quote.balance.amount;
        let cost = get_bancor_input(ram_reserve, eos_reserve, bytes as i64);
        let cost_plus_fee = cost as f64 / 0.995;
        self.buyramsys(
            payer,
            receiver,
            Asset {
                amount: cost_plus_fee as i64,
                symbol: get_core_symbol(None),
            },
        );
    }
    pub fn buyramsys(&mut self, payer: Name, receiver: Name, quant: Asset) {
        require_auth(payer);
        self.update_ram_supply();
        check(quant.symbol == get_core_symbol(None), "must buy ram with core token");
        check(quant.amount > 0, "must purchase a positive amount");
        let fee = Asset {
            amount: quant.amount + 199 / 200,
            symbol: quant.symbol,
        };
        let quant_after_fee = Asset {
            amount: quant.amount - fee.amount,
            symbol: quant.symbol,
        };
        TRANSFER_ACTION
            .to_action(
                TOKEN_ACCOUNT,
                <[_]>::into_vec(
                    ::alloc::boxed::box_new([
                        PermissionLevel::new(payer, ACTIVE_PERMISSION),
                        PermissionLevel::new(RAM_ACCOUNT, ACTIVE_PERMISSION),
                    ]),
                ),
                (payer, RAM_ACCOUNT, quant_after_fee, "buy ram".to_string()),
            )
            .send();
        if fee.amount > 0 {
            TRANSFER_ACTION
                .to_action(
                    TOKEN_ACCOUNT,
                    <[_]>::into_vec(
                        ::alloc::boxed::box_new([
                            PermissionLevel::new(payer, ACTIVE_PERMISSION),
                        ]),
                    ),
                    (payer, RAMFEE_ACCOUNT, fee, "ram fee".to_string()),
                )
                .send();
        }
        let mut bytes_out = 0i64;
        let rammarket = RAMMARKET.index(get_self(), get_self().raw());
        let mut market = rammarket
            .get(RAMCORE_SYMBOL.raw(), "ram market does not exist");
        rammarket
            .modify(
                &mut market,
                SAME_PAYER,
                |es| {
                    bytes_out = es.direct_convert(&quant_after_fee, &RAM_SYMBOL).amount
                },
            );
        check(bytes_out > 0, "must reserve a positive amount");
        self.gstate.total_ram_bytes_reserved += bytes_out as u64;
        self.gstate.total_ram_stake += quant_after_fee.amount;
        let userres = USER_RESOURCES_TABLE.index(get_self(), receiver.raw());
        let mut res_itr = userres.find(receiver.raw());
        let core_symbol = get_core_symbol(None);
        if res_itr == userres.end() {
            userres
                .emplace(
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
            userres
                .modify(
                    &mut res_itr,
                    receiver,
                    |res| {
                        res.ram_bytes += bytes_out;
                    },
                );
        }
        let voters = VOTERS_TABLE.index(get_self(), get_self().raw());
        let voter_itr = voters.find(receiver.raw());
        if voter_itr == voters.end()
            || !has_field(voter_itr.flags1, VoterInfoFlags1Fields::RAM_MANAGED)
        {
            let (ram, net, cpu) = get_resource_limits(receiver);
            set_resource_limits(receiver, ram + RAM_GIFT_BYTES, net, cpu);
        }
    }
    pub fn changebw(
        &self,
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
                >= stake_net_delta.amount.abs().max(stake_cpu_delta.amount.abs()),
            "net and cpu deltas cannot be opposite signs",
        );
        let source_stake_from = from.clone();
        let from = if transfer { receiver } else { from };
        {
            let del_tbl = DEL_BANDWIDTH_TABLE.index(get_self(), from.raw());
            let mut itr = del_tbl.find(receiver.raw());
            if itr == del_tbl.end() {
                itr = del_tbl
                    .emplace(
                        from,
                        DelegatedBandwidth {
                            from: from,
                            to: receiver,
                            net_weight: stake_net_delta,
                            cpu_weight: stake_cpu_delta,
                        },
                    );
            } else {
                del_tbl
                    .modify(
                        &mut itr,
                        SAME_PAYER,
                        |dbo| {
                            dbo.net_weight += stake_net_delta;
                            dbo.cpu_weight += stake_cpu_delta;
                        },
                    );
            }
            check(0 <= itr.net_weight.amount, "insufficient staked net bandwidth");
            check(0 <= itr.cpu_weight.amount, "insufficient staked cpu bandwidth");
            if itr.is_empty() {
                del_tbl.erase(itr);
            }
        }
        {
            let totals_tbl = USER_RESOURCES_TABLE.index(get_self(), receiver.raw());
            let mut tot_itr = totals_tbl.find(receiver.raw());
            if tot_itr == totals_tbl.end() {
                tot_itr = totals_tbl
                    .emplace(
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
                totals_tbl
                    .modify(
                        &mut tot_itr,
                        payer,
                        |tot| {
                            tot.net_weight += stake_net_delta;
                            tot.cpu_weight += stake_cpu_delta;
                        },
                    );
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
                ram_managed = has_field(
                    voter_itr.flags1,
                    VoterInfoFlags1Fields::RAM_MANAGED,
                );
                net_managed = has_field(
                    voter_itr.flags1,
                    VoterInfoFlags1Fields::NET_MANAGED,
                );
                cpu_managed = has_field(
                    voter_itr.flags1,
                    VoterInfoFlags1Fields::CPU_MANAGED,
                );
            }
            if !(net_managed && cpu_managed) {
                let (ram_bytes, net, cpu) = get_resource_limits(receiver);
                let new_ram = if ram_managed {
                    ram_bytes
                } else {
                    cmp::max(tot_itr.ram_bytes + RAM_GIFT_BYTES, ram_bytes)
                };
                let new_net = if net_managed { net } else { tot_itr.net_weight.amount };
                let new_cpu = if cpu_managed { cpu } else { tot_itr.cpu_weight.amount };
                set_resource_limits(receiver, new_ram, new_net, new_cpu);
            }
            if tot_itr.is_empty() {
                totals_tbl.erase(tot_itr);
            }
        }
        if STAKE_ACCOUNT != source_stake_from {
            let refunds_tbl = REFUNDS_TABLE.index(get_self(), from.raw());
            let mut req = refunds_tbl.find(from.raw());
            let mut net_balance = stake_net_delta;
            let mut cpu_balance = stake_cpu_delta;
            let mut need_deferred_trx = false;
            let is_undelegating = (net_balance.amount + cpu_balance.amount) < 0;
            let is_delegating_to_self = !transfer && from == receiver;
            if is_delegating_to_self || is_undelegating {
                if req != refunds_tbl.end() {
                    refunds_tbl
                        .modify(
                            &mut req,
                            SAME_PAYER,
                            |r| {
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
                            },
                        );
                    check(0 <= req.net_amount.amount, "negative net refund amount");
                    check(0 <= req.cpu_amount.amount, "negative cpu refund amount");
                    if req.is_empty() {
                        refunds_tbl.erase(req);
                        need_deferred_trx = false;
                    } else {
                        need_deferred_trx = true;
                    }
                } else if net_balance.amount < 0 || cpu_balance.amount < 0 {
                    refunds_tbl
                        .emplace(
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
                            <[_]>::into_vec(
                                ::alloc::boxed::box_new([
                                    PermissionLevel::new(from, ACTIVE_PERMISSION),
                                ]),
                            ),
                            get_self(),
                            pulse_cdt::core::Name::new(13445401734377635840u64),
                            from.pack().unwrap(),
                        )
                        .send();
                }
                let transfer_amount = net_balance + cpu_balance;
                if 0 < transfer_amount.amount {
                    TRANSFER_ACTION
                        .to_action(
                            TOKEN_ACCOUNT,
                            <[_]>::into_vec(
                                ::alloc::boxed::box_new([
                                    PermissionLevel::new(source_stake_from, ACTIVE_PERMISSION),
                                ]),
                            ),
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
        }
    }
    pub fn delegatebw(
        &self,
        from: Name,
        receiver: Name,
        stake_net_quantity: Asset,
        stake_cpu_quantity: Asset,
        transfer: bool,
    ) {
        let zero_asset = Asset::new(0, get_core_symbol(None));
        check(stake_cpu_quantity >= zero_asset, "must stake a positive amount");
        check(stake_net_quantity >= zero_asset, "must stake a positive amount");
        check(
            stake_net_quantity.amount + stake_cpu_quantity.amount > 0,
            "must stake a positive amount",
        );
        check(
            !transfer || from != receiver,
            "cannot use transfer flag if delegating to self",
        );
        self.changebw(from, receiver, stake_net_quantity, stake_cpu_quantity, transfer);
    }
    pub fn undelegatebw(
        &self,
        from: Name,
        receiver: Name,
        unstake_net_quantity: Asset,
        unstake_cpu_quantity: Asset,
    ) {
        let zero_asset = Asset::new(0, get_core_symbol(None));
        check(unstake_net_quantity >= zero_asset, "must unstake a positive amount");
        check(unstake_cpu_quantity >= zero_asset, "must unstake a positive amount");
        check(
            unstake_cpu_quantity.amount + unstake_net_quantity.amount > 0,
            "must unstake a positive amount",
        );
        check(
            self.gstate.thresh_activated_stake_time != TimePoint::default(),
            "cannot undelegate bandwidth until the chain is activated (at least 15% of all tokens participate in voting)",
        );
        self.changebw(
            from,
            receiver,
            -unstake_net_quantity,
            -unstake_cpu_quantity,
            false,
        );
    }
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
                <[_]>::into_vec(
                    ::alloc::boxed::box_new([
                        PermissionLevel::new(STAKE_ACCOUNT, ACTIVE_PERMISSION),
                        PermissionLevel::new(req.owner, ACTIVE_PERMISSION),
                    ]),
                ),
                (STAKE_ACCOUNT, req.owner, req.net_amount, "unstake".to_owned()),
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
        rammarket
            .modify(
                &mut itr,
                SAME_PAYER,
                |m| {
                    m.base.balance.amount += new_ram as i64;
                },
            );
        self.gstate2.last_ram_increase = cbt;
    }
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
            producer_key = producer_authority.keys[0].key.clone();
        }
        if prod != producers.end() {
            producers
                .modify(
                    &mut prod,
                    producer,
                    |info| {
                        info.producer_key = producer_key;
                        info.is_active = true;
                        info.url = url;
                        info.location = location;
                        if info.last_claim_time == TimePoint::default() {
                            info.last_claim_time = ct;
                        }
                    },
                );
            let prod2 = producers2.find(producer.raw());
            if prod2 == producers2.end() {
                producers2
                    .emplace(
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
            producers
                .emplace(
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
            producers2
                .emplace(
                    producer,
                    ProducerInfo2 {
                        owner: producer,
                        last_votepay_share_update: ct,
                        votepay_share: 0.0,
                    },
                );
        }
    }
    pub fn regproducer(
        &mut self,
        producer: Name,
        producer_key: PublicKey,
        url: String,
        location: u16,
    ) {
        require_auth(producer);
        check(url.len() < 512, "url too long");
        self.register_producer(
            producer,
            convert_to_block_signing_authority(&producer_key),
            url,
            location,
        );
    }
    pub fn regproducer2(
        &mut self,
        producer: Name,
        producer_authority: BlockSigningAuthority,
        url: String,
        location: u16,
    ) {
        require_auth(producer);
        check(url.len() < 512, "url too long");
        check(producer_authority.is_valid(), "invalid producer authority");
        self.register_producer(producer, producer_authority, url, location);
    }
    pub fn unregprod(producer: Name) {
        require_auth(producer);
        let producers = PRODUCERS_TABLE.index(get_self(), get_self().raw());
        let mut prod = producers.get(producer.raw(), "producer not found");
        producers
            .modify(
                &mut prod,
                SAME_PAYER,
                |info| {
                    info.deactivate();
                },
            );
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
        if shares_rate_delta < 0.0
            && self.gstate3.total_vpay_share_change_rate < -shares_rate_delta
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
            voter_itr = voters
                .emplace(
                    voter,
                    VoterInfo {
                        owner: voter,
                        staked: total_update.amount,
                        proxy: Name::default(),
                        producers: ::alloc::vec::Vec::new(),
                        last_vote_weight: 0.0,
                        proxied_vote_weight: 0.0,
                        is_proxy: false,
                        flags1: 0,
                        reserved2: 0,
                        reserved3: Asset::default(),
                    },
                );
        } else {
            voters
                .modify(
                    &mut voter_itr,
                    SAME_PAYER,
                    |v| {
                        v.staked += total_update.amount;
                    },
                );
        }
        check(0 <= voter_itr.staked, "stake for voting cannot be negative");
        if voter_itr.producers.len() > 0 || voter_itr.proxy != Name::default() {
            self.update_votes(voter, voter_itr.proxy, &voter_itr.producers, false);
        }
    }
    fn update_votes(
        &mut self,
        voter_name: Name,
        proxy: Name,
        producers: &Vec<Name>,
        voting: bool,
    ) {
        if proxy != Name::default() {
            check(
                producers.len() == 0,
                "cannot vote for producers and proxy at same time",
            );
            check(voter_name != proxy, "cannot proxy to self");
        } else {
            check(producers.len() <= 30, "attempt to vote for too many producers");
            for i in 1..producers.len() {
                check(
                    producers[i - 1] < producers[i],
                    "producer votes must be unique and sorted",
                );
            }
        }
        let voters = VOTERS_TABLE.index(get_self(), get_self().raw());
        let mut voter = voters.find(voter_name.raw());
        check(voter != voters.end(), "user must stake before they can vote");
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
                voters
                    .modify(
                        &mut old_proxy,
                        SAME_PAYER,
                        |vp| {
                            vp.proxied_vote_weight -= voter.last_vote_weight;
                        },
                    );
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
                voters
                    .modify(
                        &mut new_proxy,
                        SAME_PAYER,
                        |vp| {
                            vp.proxied_vote_weight += new_vote_weight;
                        },
                    );
                self.propagate_weight_change(&mut new_proxy);
            }
        } else {
            if new_vote_weight >= 0.0 {
                for p in producers.iter() {
                    let entry: &mut (f64, bool) = producer_deltas
                        .entry(p.clone())
                        .or_insert((0.0, true));
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
                    ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!(
                                    "producer {0} is not currently registered",
                                    pitr.owner.to_string(),
                                ),
                            )
                        })
                        .as_str(),
                );
            }
            let init_total_votes = pitr.total_votes;
            producers_table
                .modify(
                    &mut pitr,
                    SAME_PAYER,
                    |p| {
                        p.total_votes += pd.1.0;
                        if p.total_votes < 0.0 {
                            p.total_votes = 0.0;
                        }
                        self.gstate.total_producer_vote_weight += pd.1.0;
                    },
                );
            let producers_table2 = PRODUCERS_TABLE2.index(get_self(), get_self().raw());
            let mut prod2 = producers_table2.find(pd.0.raw());
            if prod2 != producers_table2.end() {
                let last_claim_plus_3days = pitr.last_claim_time
                    + Microseconds(3 * USECONDS_PER_DAY as i64);
                let crossed_threshold = last_claim_plus_3days <= ct;
                let updated_after_threshold = last_claim_plus_3days
                    <= prod2.last_votepay_share_update;
                let new_votepay_share = self
                    .update_producer_votepay_share(
                        &mut prod2,
                        &ct,
                        if updated_after_threshold { 0.0 } else { init_total_votes },
                        crossed_threshold && !updated_after_threshold,
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
                        ::alloc::__export::must_use({
                                ::alloc::fmt::format(
                                    format_args!(
                                        "producer {0} is not currently registered",
                                        pd.0.to_string(),
                                    ),
                                )
                            })
                            .as_str(),
                    );
                }
            }
        }
        self.update_total_votepay_share(
            &ct,
            -total_inactive_vpay_share,
            delta_change_rate,
        );
        voters
            .modify(
                &mut voter,
                SAME_PAYER,
                |av| {
                    av.last_vote_weight = new_vote_weight;
                    av.producers = producers.clone();
                    av.proxy = proxy;
                },
            );
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
            delta_votepay_share = shares_rate
                * ((*ct - prod_itr.last_votepay_share_update).count() / 1000000) as f64;
        }
        let producers2 = PRODUCERS_TABLE2.index(get_self(), get_self().raw());
        let new_votepay_share = prod_itr.votepay_share + delta_votepay_share;
        producers2
            .modify(
                prod_itr,
                SAME_PAYER,
                |p| {
                    if reset_to_zero {
                        p.votepay_share = 0.0;
                    } else {
                        p.votepay_share = new_votepay_share;
                    }
                    p.last_votepay_share_update = *ct;
                },
            );
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
                voters
                    .modify(
                        &mut proxy,
                        SAME_PAYER,
                        |p| {
                            p.proxied_vote_weight += new_weight - voter.last_vote_weight;
                        },
                    );
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
                    producers
                        .modify(
                            &mut prod,
                            SAME_PAYER,
                            |p| {
                                p.total_votes += delta;
                            },
                        );
                    self.gstate.total_producer_vote_weight += delta;
                    let producers2 = PRODUCERS_TABLE2
                        .index(get_self(), get_self().raw());
                    let mut prod2 = producers2.find(acnt.raw());
                    if prod2 != producers2.end() {
                        let last_claim_plus_3days = prod.last_claim_time
                            + Microseconds(3 * USECONDS_PER_DAY as i64);
                        let crossed_threshold = last_claim_plus_3days <= ct;
                        let updated_after_threshold = last_claim_plus_3days
                            <= prod2.last_votepay_share_update;
                        let new_votepay_share = self
                            .update_producer_votepay_share(
                                &mut prod2,
                                &ct,
                                if updated_after_threshold {
                                    0.0
                                } else {
                                    init_total_votes
                                },
                                crossed_threshold && !updated_after_threshold,
                            );
                        if !crossed_threshold {
                            delta_change_rate += delta
                        } else if !updated_after_threshold {
                            total_inactive_vpay_share += new_votepay_share;
                            delta_change_rate -= init_total_votes;
                        }
                    }
                }
                self.update_total_votepay_share(
                    &ct,
                    -total_inactive_vpay_share,
                    delta_change_rate,
                );
            }
        }
        voters
            .modify(
                voter,
                SAME_PAYER,
                |v| {
                    v.last_vote_weight = new_weight;
                },
            );
    }
    pub fn regproxy(&mut self, proxy: Name, is_proxy: bool) {
        require_auth(proxy);
        let voters = VOTERS_TABLE.index(get_self(), get_self().raw());
        let mut pitr = voters.find(proxy.raw());
        if pitr != voters.end() {
            check(is_proxy != pitr.is_proxy, "action has no effect");
            check(
                !is_proxy || !pitr.is_proxy,
                "account that uses a proxy is not allowed to become a proxy",
            );
            voters
                .modify(
                    &mut pitr,
                    SAME_PAYER,
                    |p| {
                        p.is_proxy = is_proxy;
                    },
                );
            self.propagate_weight_change(&mut pitr);
        } else {
            voters
                .emplace(
                    proxy,
                    VoterInfo {
                        owner: proxy,
                        is_proxy: is_proxy,
                        proxy: Name::default(),
                        producers: ::alloc::vec::Vec::new(),
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
}
#[doc(hidden)]
mod __SystemContract_contract_ctx {
    use core::sync::atomic::{AtomicU64, Ordering};
    static RECEIVER: AtomicU64 = AtomicU64::new(0);
    #[inline]
    pub fn get_self() -> u64 {
        RECEIVER.load(Ordering::Relaxed)
    }
    #[inline]
    pub fn __set_receiver(v: u64) {
        RECEIVER.store(v, Ordering::Relaxed);
    }
    #[inline]
    pub fn __clear_receiver() {
        RECEIVER.store(0, Ordering::Relaxed);
    }
    pub struct ReceiverGuard;
    impl ReceiverGuard {
        #[inline]
        pub fn new(v: u64) -> Self {
            __set_receiver(v);
            ReceiverGuard
        }
    }
    impl Drop for ReceiverGuard {
        #[inline]
        fn drop(&mut self) {
            __clear_receiver();
        }
    }
}
#[inline]
pub fn get_self() -> Name {
    pulse_cdt::core::Name::new(__SystemContract_contract_ctx::get_self())
}
#[no_mangle]
pub extern "C" fn apply(receiver: u64, code: u64, action: u64) {
    let __guard = __SystemContract_contract_ctx::ReceiverGuard::new(receiver);
    let mut __instance: SystemContract = <SystemContract>::constructor();
    if action == 11877535737890996224u64 {
        pulse_cdt::core::check(
            false,
            "onerror action's are only valid from the \"pulse\" system account",
        );
    } else if code == receiver && action == 14029658124516851712u64 {
        type __Args = (Name, u8);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1) = __args;
        <SystemContract>::setpriv(__a0, __a1);
    } else if code == receiver && action == 11148770977341390848u64 {
        type __Args = (Name, Name, Authority, Authority);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1, __a2, __a3) = __args;
        <SystemContract>::newaccount(__a0, __a1, __a2, __a3);
    } else if code == receiver && action == 14029385431137648640u64 {
        type __Args = (Name, Vec<u8>);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1) = __args;
        <SystemContract>::setabi(__a0, __a1);
    } else if code == receiver && action == 14029427681804681216u64 {
        type __Args = (Name, u8, u8, Vec<u8>);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1, __a2, __a3) = __args;
        <SystemContract>::setcode(__a0, __a1, __a2, __a3);
    } else if code == receiver && action == 8421045207927095296u64 {
        type __Args = (u8, Symbol);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1) = __args;
        __instance.init(__a0, __a1);
    } else if code == receiver && action == 4520896358201556992u64 {
        type __Args = (Name, Name, u32);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1, __a2) = __args;
        __instance.buyrambsys(__a0, __a1, __a2);
    } else if code == receiver && action == 4520896367425486848u64 {
        type __Args = (Name, Name, Asset);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1, __a2) = __args;
        __instance.buyramsys(__a0, __a1, __a2);
    } else if code == receiver && action == 5378043540636893184u64 {
        type __Args = (Name, Name, Asset, Asset, bool);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1, __a2, __a3, __a4) = __args;
        __instance.delegatebw(__a0, __a1, __a2, __a3, __a4);
    } else if code == receiver && action == 15335505127214321600u64 {
        type __Args = (Name, Name, Asset, Asset);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1, __a2, __a3) = __args;
        __instance.undelegatebw(__a0, __a1, __a2, __a3);
    } else if code == receiver && action == 13445401734377635840u64 {
        type __Args = (Name,);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0) = __args;
        <SystemContract>::refund(__a0);
    } else if code == receiver && action == 11875739475730497536u64 {
        type __Args = (BlockHeader,);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0) = __args;
        __instance.onblock(__a0);
    } else if code == receiver && action == 13445879116675067392u64 {
        type __Args = (Name, PublicKey, String, u16);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1, __a2, __a3) = __args;
        __instance.regproducer(__a0, __a1, __a2, __a3);
    } else if code == receiver && action == 13445879116675067424u64 {
        type __Args = (Name, BlockSigningAuthority, String, u16);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1, __a2, __a3) = __args;
        __instance.regproducer2(__a0, __a1, __a2, __a3);
    } else if code == receiver && action == 15343383872893616128u64 {
        type __Args = (Name,);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0) = __args;
        <SystemContract>::unregprod(__a0);
    } else if code == receiver && action == 13445879127475224576u64 {
        type __Args = (Name, bool);
        let __args: __Args = ::pulse_cdt::contracts::read_action_data::<__Args>();
        let (__a0, __a1) = __args;
        __instance.regproxy(__a0, __a1);
    } else if code == receiver {
        pulse_cdt::core::check(false, "unknown action");
    }
    __instance.destructor();
    core::mem::drop(__guard);
}
fn convert_to_block_signing_authority(
    producer_key: &PublicKey,
) -> BlockSigningAuthority {
    BlockSigningAuthority::new(
        1,
        <[_]>::into_vec(
            ::alloc::boxed::box_new([
                KeyWeight {
                    key: producer_key.clone(),
                    weight: 1,
                },
            ]),
        ),
    )
}
fn stake_to_vote(staked: i64) -> f64 {
    let epoch_offset = BlockTimestamp::BLOCK_TIMESTAMP_EPOCH / 1000;
    let weight = ((current_time_point().sec_since_epoch() as i64 - epoch_offset)
        / (SECONDS_PER_DAY * 7) as i64) as f64 / 52.0;
    (staked as f64) * pow(2.0, weight)
}

#![no_std]
#![no_main]
extern crate alloc;

use alloc::vec::Vec;
use pulse_cdt::{
    action, contract, contracts::{has_auth, is_account, require_auth, require_recipient, PermissionLevel}, core::{check, Asset, Checksum256, MultiIndexDefinition, Name, Symbol, SymbolCode, Table, TimePoint, MAX_ASSET_AMOUNT}, name, table, NumBytes, Read, Write, SAME_PAYER
};

use crate::__MsigContract_contract_ctx::get_self;

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.proposal_name.raw())]
struct Proposal {
    proposal_name: Name,
    packed_transaction: Vec<u8>,
    earliest_exec_time: Option<TimePoint>,
}

const PROPOSALS: MultiIndexDefinition<Proposal> = MultiIndexDefinition::new(name!("proposal"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
struct Approval {
    level: PermissionLevel,
    time: TimePoint,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.proposal_name.raw())]
struct ApprovalsInfo {
    version: u8,
    proposal_name: Name,
    requested_approvals: Vec<Approval>,
    provided_approvals: Vec<Approval>,
}

const APPROVALS: MultiIndexDefinition<ApprovalsInfo> = MultiIndexDefinition::new(name!("approvals2"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.account.raw())]
struct Invalidation {
    account: Name,
    last_invalidation_time: TimePoint,
}

const INVALIDATIONS: MultiIndexDefinition<Invalidation> = MultiIndexDefinition::new(name!("invals"));

#[derive(Default)]
struct MsigContract;

#[contract]
impl MsigContract {
    #[action]
    fn propose(proposer: Name, proposal_name: Name, requested: Vec<PermissionLevel>) {
        require_auth(proposer);
    }

    #[action]
    fn approve(proposer: Name, proposal_name: Name, level: PermissionLevel, proposal_hash: Checksum256 ) {

    }

    #[action]
    fn unapprove(proposer: Name, proposal_name: Name, level: PermissionLevel) {

    }

    #[action]
    fn cancel(proposer: Name, proposal_name: Name, canceler: Name ) {

    }

    #[action]
    fn exec(proposer: Name, proposal_name: Name, executer: Name) {

    }

    #[action]
    fn invalidate(account: Name) {
        require_auth(account);

        let inv_table = INVALIDATIONS.index(get_self(), get_self().raw());
        let it = inv_table.find(account.raw());
    }
}
#![no_std]
#![no_main]
extern crate alloc;

use alloc::{collections::btree_set::BTreeSet, vec::Vec};
use pulse_cdt::{
    NumBytes, Read, Write, action, contract,
    contracts::{
        PermissionLevel, check_transaction_authorization, current_time_point, require_auth,
        require_auth2,
    },
    core::{
        Checksum256, Microseconds, MultiIndexDefinition, Name, Table, TimePoint, Transaction,
        TransactionHeader, check,
    },
    name, table,
};

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.proposal_name.raw())]
struct Proposal {
    proposal_name: Name,
    packed_transaction: Vec<u8>,
    earliest_exec_time: Option<TimePoint>,
}

const PROPOSALS: MultiIndexDefinition<Proposal> = MultiIndexDefinition::new(name!("proposal"));

#[derive(Read, Write, NumBytes, Clone, PartialEq, PartialOrd, Eq, Ord)]
struct Approval {
    level: PermissionLevel,
    time: TimePoint,
}

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.proposal_name.raw())]
struct ApprovalsInfo {
    version: u8,
    proposal_name: Name,
    requested_approvals: BTreeSet<Approval>,
    provided_approvals: BTreeSet<Approval>,
}

impl ApprovalsInfo {
    pub fn requested_approvals(&self) -> &BTreeSet<Approval> {
        &self.requested_approvals
    }
}

const APPROVALS: MultiIndexDefinition<ApprovalsInfo> =
    MultiIndexDefinition::new(name!("approvals2"));

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.account.raw())]
struct Invalidation {
    account: Name,
    last_invalidation_time: TimePoint,
}

const INVALIDATIONS: MultiIndexDefinition<Invalidation> =
    MultiIndexDefinition::new(name!("invals"));

#[derive(Default)]
struct MsigContract;

#[contract]
impl MsigContract {
    #[action]
    fn propose(
        proposer: Name,
        proposal_name: Name,
        requested: BTreeSet<PermissionLevel>,
        trx: Transaction,
    ) {
        require_auth(proposer);
        check(
            trx.header.expiration >= current_time_point().into(),
            "transaction expired",
        );
        check(
            trx.context_free_actions.is_empty(),
            "not allowed to `propose` a transaction with context-free actions",
        );

        let proptable = PROPOSALS.index(get_self(), proposer.raw());
        check(
            proptable.find(proposal_name.raw()) == proptable.end(),
            "proposal with the same name exists",
        );

        let res = check_transaction_authorization(&trx, &BTreeSet::new(), &requested);
        check(res > 0, "transaction authorization failed");

        let packed_transaction = trx.pack().expect("failed to pack trx");
        proptable.emplace(
            proposer,
            Proposal {
                proposal_name,
                packed_transaction,
                earliest_exec_time: None,
            },
        );

        let mut requested_approvals: BTreeSet<Approval> = BTreeSet::new();
        for level in requested.iter() {
            requested_approvals.insert(Approval {
                level: level.clone(),
                time: TimePoint::new(Microseconds::new(0)),
            });
        }

        let apptable = APPROVALS.index(get_self(), proposer.raw());
        apptable.emplace(
            proposer,
            ApprovalsInfo {
                version: 0,
                proposal_name,
                requested_approvals,
                provided_approvals: BTreeSet::new(),
            },
        );
    }

    #[action]
    fn approve(
        proposer: Name,
        proposal_name: Name,
        level: PermissionLevel,
        proposal_hash: Checksum256,
    ) {
        require_auth2(level.actor, level.permission);

        let apptable = APPROVALS.index(get_self(), proposer.raw());
        let mut apps_it = apptable.find(proposal_name.raw());
        check(apps_it != apptable.end(), "proposal not found");
        let itr = {
            apps_it
                .requested_approvals()
                .iter()
                .find(|ra| ra.level == level)
                .expect("approval is not on the list of requested approvals")
                .clone()
        };
        apptable.modify(&mut apps_it, proposer, |a| {
            a.requested_approvals.remove(&itr);
            a.provided_approvals.insert(Approval {
                level: level,
                time: current_time_point(),
            });
        });
    }

    #[action]
    fn unapprove(proposer: Name, proposal_name: Name, level: PermissionLevel) {
        require_auth2(level.actor, level.permission);

        let apptable = APPROVALS.index(get_self(), proposer.raw());
        let mut apps_it = apptable.find(proposal_name.raw());
        check(apps_it != apptable.end(), "proposal not found");
        let itr = {
            apps_it
                .provided_approvals
                .iter()
                .find(|a| a.level == level)
                .expect("no approval previously granted")
                .clone()
        };
        apptable.modify(&mut apps_it, proposer, |a| {
            a.provided_approvals.remove(&itr);
            a.requested_approvals.insert(Approval {
                level: level,
                time: current_time_point(),
            });
        });
    }

    #[action]
    fn cancel(proposer: Name, proposal_name: Name, canceler: Name) {
        require_auth(canceler);

        let proptable = PROPOSALS.index(get_self(), proposer.raw());
        let prop = proptable.get(proposal_name.raw(), "proposal not found");

        if canceler != proposer {
            let trx = TransactionHeader::read(&prop.packed_transaction, &mut 0)
                .expect("failed to read trx");
            check(
                trx.expiration < current_time_point().into(),
                "cannot cancel until expiration",
            );
        }

        proptable.erase(prop);

        let apptable = APPROVALS.index(get_self(), proposer.raw());
        let apps_it = apptable.find(proposal_name.raw());
        check(apps_it != apptable.end(), "proposal not found");
        apptable.erase(apps_it);
    }

    #[action]
    fn exec(proposer: Name, proposal_name: Name, executer: Name) {
        require_auth(executer);

        let proptable = PROPOSALS.index(get_self(), proposer.raw());
        let prop = proptable.get(proposal_name.raw(), "proposal not found");
        let trx = Transaction::read(&prop.packed_transaction, &mut 0).expect("failed to read trx");
        check(
            trx.header.expiration >= current_time_point().into(),
            "transaction expired",
        );
        check(
            trx.context_free_actions.is_empty(),
            "not allowed to `exec` a transaction with context-free actions",
        );
        let approval_table = APPROVALS.index(get_self(), proposer.raw());
        let invalidations_table = INVALIDATIONS.index(get_self(), get_self().raw());
        let approval_table_iter = approval_table.find(proposal_name.raw());
        let mut approvals_vector: BTreeSet<PermissionLevel> = BTreeSet::new();

        if approval_table_iter != approval_table.end() {
            for permission in approval_table_iter.provided_approvals.iter() {
                let iter = invalidations_table.find(permission.level.actor.raw());
                if iter == invalidations_table.end()
                    || iter.last_invalidation_time < permission.time
                {
                    approvals_vector.insert(permission.level.clone());
                }
            }
            approval_table.erase(approval_table_iter);
        }

        let res = check_transaction_authorization(&trx, &BTreeSet::new(), &approvals_vector);
        check(res > 0, "transaction authorization failed");

        if let Some(earliest_exec_time) = prop.earliest_exec_time {
            check(
                earliest_exec_time <= current_time_point(),
                "too early to execute",
            );
        } else {
            check(
                trx.header.delay_sec.0 == 0,
                "old proposals are not allowed to have non-zero `delay_sec`; cancel and retry",
            );
        }

        for act in trx.actions.iter() {
            act.send();
        }

        proptable.erase(prop);
    }

    #[action]
    fn invalidate(account: Name) {
        require_auth(account);

        let inv_table = INVALIDATIONS.index(get_self(), get_self().raw());
        let mut it = inv_table.find(account.raw());

        if it == inv_table.end() {
            inv_table.emplace(
                account,
                Invalidation {
                    account,
                    last_invalidation_time: current_time_point(),
                },
            );
        } else {
            inv_table.modify(&mut it, account, |i| {
                i.last_invalidation_time = current_time_point();
            });
        }
    }
}

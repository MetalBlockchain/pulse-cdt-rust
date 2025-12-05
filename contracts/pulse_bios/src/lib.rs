#![no_std]
#![no_main]
extern crate alloc;

mod native;

use alloc::vec::Vec;
use pulse_cdt::contracts::{require_auth, set_privileged, set_resource_limits, sha256, Authority};
use pulse_cdt::{action, contract, SAME_PAYER};
use pulse_cdt::core::Name;

use crate::native::{AbiHash, ABI_HASH_TABLE};

#[derive(Default)]
struct BiosContract;

#[contract]
impl BiosContract {
    #[action]
    fn setpriv(account: Name, is_priv: u8) {
        require_auth(get_self());
        set_privileged(account, is_priv == 1);
    }

    #[action]
    fn setalimits(account: Name, ram_bytes: i64, net_weight: i64, cpu_weight: i64) {
        require_auth(get_self());
        set_resource_limits(account, ram_bytes, net_weight, cpu_weight);
    }

    #[action]
    fn reqauth(from: Name) {
        require_auth(from);
    }

    #[action]
    fn setcode(account: Name, vmtype: u8, vmversion: u8, code: Vec<u8>) {
        // Set code is open for all
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
    fn newaccount(creator: Name, name: Name, owner: Authority, active: Authority) {
        // No action required
    }

    #[action]
    fn updateauth(account: Name, permission: Name, parent: Name, auth: Authority) {
        // No action required
    }

    #[action]
    fn deleteauth(account: Name, permission: Name) {
        // No action required
    }

    #[action]
    fn linkauth(account: Name, code: Name, message_type: Name, requirement: Name) {
        // No action required
    }

    #[action]
    fn unlinkauth(account: Name, code: Name, message_type: Name) {
        // No action required
    }
}
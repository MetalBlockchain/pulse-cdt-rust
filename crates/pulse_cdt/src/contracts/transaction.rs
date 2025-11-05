use alloc::{collections::btree_set::BTreeSet, vec::Vec};
use pulse_serialization::Write;

use crate::{
    contracts::PermissionLevel,
    core::{PublicKey, Transaction},
};

mod transaction_impl {
    extern "C" {
        #[link_name = "check_transaction_authorization"]
        pub fn check_transaction_authorization(
            trx_msg: *mut crate::c_void,
            trx_len: usize,
            pubkeys_msg: *mut crate::c_void,
            pubkeys_len: usize,
            perms_msg: *mut crate::c_void,
            perms_len: usize,
        ) -> u32;
    }
}

#[inline]
pub fn check_transaction_authorization(
    transaction: &Transaction,
    provided_keys: &BTreeSet<PublicKey>,
    provided_permissions: &BTreeSet<PermissionLevel>,
) -> u32 {
    let packed_trx = transaction.pack().expect("failed to pack transaction");
    let packed_keys: Vec<u8> = if provided_keys.len() > 0 {
        provided_keys.pack().expect("failed to pack provided keys")
    } else {
        Vec::new()
    };
    let packed_perms: Vec<u8> = if provided_permissions.len() > 0 {
        provided_permissions
            .pack()
            .expect("failed to pack provided permissions")
    } else {
        Vec::new()
    };

    unsafe {
        transaction_impl::check_transaction_authorization(
            packed_trx.as_ptr() as *mut _,
            packed_trx.len(),
            packed_keys.as_ptr() as *mut _,
            packed_keys.len(),
            packed_perms.as_ptr() as *mut _,
            packed_perms.len(),
        )
    }
}

use alloc::{collections::btree_set::BTreeSet, vec::Vec};
use hashbrown::{hash_map::DefaultHashBuilder, HashSet};
use pulse_proc_macro::{NumBytes, Read, Write};

use crate::{
    contracts::KeyWeight,
    core::{BlockTimestamp, FixedBytes, Name, PublicKey},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, NumBytes, Read, Write)]
#[pulse(crate_path = "pulse_serialization")]
pub struct BlockHeader {
    pub timestamp: BlockTimestamp,
    pub producer: Name,
    pub confirmed: u16,
    pub previous: FixedBytes<32>,
    pub transaction_mroot: FixedBytes<32>,
    pub action_mroot: FixedBytes<32>,
}

#[derive(Debug, Clone, PartialEq, Eq, NumBytes, Read, Write)]
#[pulse(crate_path = "pulse_serialization")]
pub struct BlockSigningAuthority {
    variant: u8,
    pub threshold: u32,
    pub keys: Vec<KeyWeight>,
}

impl BlockSigningAuthority {
    pub fn new(threshold: u32, keys: Vec<KeyWeight>) -> Self {
        Self {
            variant: 0,
            threshold,
            keys,
        }
    }

    pub fn is_valid(&self) -> bool {
        let mut sum_weights = 0u32;
        let mut unique_keys: BTreeSet<PublicKey> = BTreeSet::new();

        for kw in self.keys.iter() {
            if u32::MAX - sum_weights <= kw.weight.into() {
                sum_weights = u32::MAX;
            } else {
                sum_weights += kw.weight as u32;
            }

            unique_keys.insert(kw.key.clone());
        }

        if self.keys.len() != unique_keys.len() {
            return false;
        }

        if self.threshold == 0 {
            return false;
        }

        if sum_weights < self.threshold {
            return false;
        }

        return true;
    }
}

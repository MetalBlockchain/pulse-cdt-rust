use alloc::vec::Vec;
use pulse_proc_macro::{NumBytes, Read, Write};

use crate::core::{Name, PublicKey};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct Authority {
    threshold: u32,
    keys: Vec<KeyWeight>,
    accounts: Vec<PermissionLevelWeight>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct KeyWeight {
    key: PublicKey,
    weight: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct PermissionLevelWeight {
    permission: PermissionLevel,
    weight: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct PermissionLevel {
    actor: Name,
    permission: Name,
}

impl PermissionLevel {
    pub fn new(actor: Name, permission: Name) -> Self {
        Self { actor, permission }
    }
}

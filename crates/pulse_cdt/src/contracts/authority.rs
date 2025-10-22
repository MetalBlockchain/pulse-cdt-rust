use alloc::vec::Vec;
use pulse_proc_macro::{NumBytes, Read, Write};

use crate::core::{Name, PublicKey};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct Authority {
    pub threshold: u32,
    pub keys: Vec<KeyWeight>,
    pub accounts: Vec<PermissionLevelWeight>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct KeyWeight {
    pub key: PublicKey,
    pub weight: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct PermissionLevelWeight {
    pub permission: PermissionLevel,
    pub weight: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct PermissionLevel {
    pub actor: Name,
    pub permission: Name,
}

impl PermissionLevel {
    pub fn new(actor: Name, permission: Name) -> Self {
        Self { actor, permission }
    }
}

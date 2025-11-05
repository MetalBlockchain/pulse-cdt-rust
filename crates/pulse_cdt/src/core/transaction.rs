use alloc::vec::Vec;
use pulse_proc_macro::{NumBytes, Read, Write};
use pulse_serialization::VarUint32;

use crate::{contracts::Action, core::TimePointSec};

#[derive(Debug, Clone, PartialEq, Eq, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct TransactionHeader {
    pub expiration: TimePointSec,
    pub ref_block_num: u16,
    pub ref_block_prefix: u32,
    pub max_net_usage_words: VarUint32,
    pub max_cpu_usage: u8,
    pub delay_sec: VarUint32,
}

#[derive(Debug, Clone, PartialEq, Eq, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct Transaction {
    pub header: TransactionHeader,
    pub context_free_actions: Vec<Action>, // Context-free actions, if any
    pub actions: Vec<Action>,              // Actions to be executed in this transaction
    pub transaction_extensions: Vec<(u16, Vec<u8>)>, // We don't use this for now
}

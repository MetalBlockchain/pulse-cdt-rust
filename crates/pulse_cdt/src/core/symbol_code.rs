use core::str::FromStr;

use pulse_bytes::{symbol_code_from_bytes, symbol_code_to_bytes, ParseSymbolCodeError};
use pulse_proc_macro::{NumBytes, Read, Write};

/// The maximum allowed length of Pulse symbol codes.
pub const SYMBOL_CODE_MAX_LEN: usize = 7;

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, Default, Read, Write, NumBytes, Hash, PartialOrd, Ord,
)]
#[pulse(crate_path = "pulse_serialization")]
pub struct SymbolCode(u64);

impl From<u64> for SymbolCode {
    #[inline]
    fn from(n: u64) -> Self {
        Self(n)
    }
}

impl From<SymbolCode> for u64 {
    #[inline]
    fn from(s: SymbolCode) -> Self {
        s.0
    }
}

impl From<SymbolCode> for [u8; 7] {
    #[inline]
    fn from(s: SymbolCode) -> Self {
        symbol_code_to_bytes(s.0)
    }
}

impl FromStr for SymbolCode {
    type Err = ParseSymbolCodeError;

    #[inline]
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        symbol_code_from_bytes(value.as_bytes()).map(Into::into)
    }
}

impl SymbolCode {
    /// TODO docs
    #[inline]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// TODO docs
    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let chars = symbol_code_to_bytes(self.0);
        for &c in &chars {
            if c == b' ' {
                continue;
            }
            if !(b'A' <= c && c <= b'Z') {
                return false;
            }
        }
        true
    }

    /// TODO docs
    #[inline]
    #[must_use]
    pub const fn raw(&self) -> u64 {
        self.0
    }
}
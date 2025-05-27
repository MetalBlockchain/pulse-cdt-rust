use pulse_proc_macro::{NumBytes, Read, Write};

use super::symbol_code::SymbolCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Read, NumBytes, Write)]
#[pulse(crate_path = "pulse_serialization")]
pub struct Symbol(u64);

impl Symbol {
    #[inline]
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    #[inline]
    #[must_use]
    pub const fn new_with_code(precision: u8, code: SymbolCode) -> Self {
        Self(symbol_from_code(precision, code.raw()))
    }

    #[inline]
    #[must_use]
    pub const fn precision(&self) -> u8 {
        symbol_to_precision(self.as_u64())
    }

    #[inline]
    #[must_use]
    pub const fn code(&self) -> SymbolCode {
        SymbolCode::new(symbol_to_code(self.as_u64()))
    }

    #[inline]
    #[must_use]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.code().is_valid()
    }
}

#[inline]
#[must_use]
pub const fn symbol_to_code(value: u64) -> u64 {
    value >> 8
}

#[inline]
#[must_use]
pub const fn symbol_from_code(precision: u8, code: u64) -> u64 {
    (code << 8) | (precision as u64)
}

#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn symbol_to_precision(value: u64) -> u8 {
    (value & 0xFF) as u8
}

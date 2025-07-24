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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseSymbolCodeError {
    /// The symbol is too long. Symbols must be 7 characters or less.
    TooLong,
    /// The symbol contains an invalid character. Symbols can only contain
    /// uppercase letters A-Z.
    BadChar(u8),
}

#[inline]
pub fn symbol_code_from_bytes<I>(iter: I) -> Result<u64, ParseSymbolCodeError>
where
    I: DoubleEndedIterator<Item = u8> + ExactSizeIterator,
{
    let mut value = 0_u64;
    for (i, c) in iter.enumerate().rev() {
        if i == SYMBOL_CODE_MAX_LEN {
            return Err(ParseSymbolCodeError::TooLong);
        } else if c < b'A' || c > b'Z' {
            return Err(ParseSymbolCodeError::BadChar(c));
        } else {
            value <<= 8;
            value |= u64::from(c);
        }
    }
    Ok(value)
}

#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn symbol_code_to_bytes(value: u64) -> [u8; SYMBOL_CODE_MAX_LEN] {
    let mut chars = [b' '; SYMBOL_CODE_MAX_LEN];
    let mut v = value;
    for c in &mut chars {
        if v == 0 {
            break;
        }
        *c = (v & 0xFF) as u8;
        v >>= 8;
    }
    chars
}

/// The maximum allowed length of Pulse symbol codes.
pub const SYMBOL_CODE_MAX_LEN: usize = 7;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ParseSymbolCodeError {
    /// The symbol is too long. Symbols must be 7 characters or less.
    TooLong,
    /// The symbol contains an invalid character. Symbols can only contain
    /// uppercase letters A-Z.
    BadChar(u8),
}

#[inline]
pub const fn symbol_code_from_bytes(bytes: &[u8]) -> Result<u64, ParseSymbolCodeError> {
    let mut value: u64 = 0;

    // length check first (you used ExactSizeIterator before)
    if bytes.len() > SYMBOL_CODE_MAX_LEN {
        return Err(ParseSymbolCodeError::TooLong);
    }

    // process from end to start (equivalent to .enumerate().rev())
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        let c = bytes[i];
        if c < b'A' || c > b'Z' {
            return Err(ParseSymbolCodeError::BadChar(c));
        }
        value <<= 8;
        value |= c as u64;
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

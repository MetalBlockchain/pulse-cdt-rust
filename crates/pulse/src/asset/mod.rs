use pulse_serialization::{NumBytes, Read, Write};

use crate::Symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, NumBytes, Read, Write)]
#[pulse(crate_path = "pulse_serialization")]
pub struct Asset {
    /// The amount of the asset
    pub amount: i64,
    /// The symbol name of the asset
    pub symbol: Symbol,
}

impl Asset {
    pub fn zero<T: Into<Symbol>>(symbol: T) -> Self {
        Self {
            amount: 0,
            symbol: symbol.into(),
        }
    }

    /// Check if the asset is valid. A valid asset has its amount <=
    /// `max_amount` and its symbol name valid
    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.is_amount_within_range() && self.symbol.is_valid()
    }

    #[inline]
    fn is_amount_within_range(&self) -> bool {
        self.amount >= i64::MIN && self.amount <= i64::MAX
    }
}
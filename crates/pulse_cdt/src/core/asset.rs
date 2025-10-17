use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use pulse_proc_macro::{NumBytes, Read, Write};

use super::{check::check, symbol::Symbol};

pub const MAX_ASSET_AMOUNT: i64 = (1i64 << 62) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, NumBytes, Read, Write)]
#[pulse(crate_path = "pulse_serialization")]
pub struct Asset {
    /// The amount of the asset
    pub amount: i64,
    /// The symbol name of the asset
    pub symbol: Symbol,
}

impl Asset {
    #[inline(always)]
    pub fn new<T: Into<Symbol>>(amount: i64, symbol: T) -> Self {
        Self {
            amount,
            symbol: symbol.into(),
        }
    }

    #[inline(always)]
    pub fn zero<T: Into<Symbol>>(symbol: T) -> Self {
        Self {
            amount: 0,
            symbol: symbol.into(),
        }
    }

    /// Check if the asset is valid. A valid asset has its amount <=
    /// `max_amount` and its symbol name valid
    #[inline(always)]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.is_amount_within_range() && self.symbol.is_valid()
    }

    #[inline(always)]
    fn is_amount_within_range(&self) -> bool {
        self.amount >= i64::MIN && self.amount <= i64::MAX
    }
}

impl Add for Asset {
    type Output = Self;

    #[inline(always)]
    fn add(self, other: Self) -> Self::Output {
        let mut result = self;
        result += other;
        return result;
    }
}

impl AddAssign for Asset {
    #[inline(always)]
    fn add_assign(&mut self, other: Self) {
        check(
            self.symbol == other.symbol,
            "attempt to add asset with different symbol",
        );

        let result = self.amount.checked_add(other.amount);

        match result {
            Some(value) => {
                self.amount = value;
            }
            None => {
                check(false, "addition overflow");
            }
        }
    }
}

impl Sub for Asset {
    type Output = Self;

    #[inline(always)]
    fn sub(self, other: Self) -> Self::Output {
        let mut result = self;
        result -= other;
        return result;
    }
}

impl SubAssign for Asset {
    #[inline(always)]
    fn sub_assign(&mut self, other: Self) {
        check(
            self.symbol == other.symbol,
            "attempt to subtract asset with different symbol",
        );

        let result = self.amount.checked_sub(other.amount);

        match result {
            Some(value) => {
                self.amount = value;
            }
            None => {
                check(false, "subtraction overflow");
            }
        }
    }
}

impl Mul for Asset {
    type Output = Self;

    #[inline(always)]
    fn mul(self, other: Self) -> Self::Output {
        let mut result = self;
        result *= other;
        return result;
    }
}

impl MulAssign for Asset {
    #[inline(always)]
    fn mul_assign(&mut self, other: Self) {
        check(
            self.symbol == other.symbol,
            "attempt to multiply asset with different symbol",
        );

        let result = self.amount.checked_mul(other.amount);

        match result {
            Some(value) => {
                self.amount = value;
            }
            None => {
                check(false, "multiplication overflow");
            }
        }
    }
}

impl Div for Asset {
    type Output = Self;

    #[inline(always)]
    fn div(self, other: Self) -> Self::Output {
        let mut result = self;
        result /= other;
        return result;
    }
}

impl DivAssign for Asset {
    #[inline(always)]
    fn div_assign(&mut self, other: Self) {
        check(
            self.symbol == other.symbol,
            "attempt to divide asset with different symbol",
        );
        check(other.amount != 0, "division by zero");

        let result = self.amount.checked_div(other.amount);

        match result {
            Some(value) => {
                self.amount = value;
            }
            None => {
                check(false, "signed division overflow");
            }
        }
    }
}

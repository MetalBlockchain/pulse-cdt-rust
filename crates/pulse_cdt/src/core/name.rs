use core::{cmp::PartialEq, ops::Not};

use alloc::string::{String, ToString};
use pulse_name::{name_to_bytes, NAME_MAX_LEN};
use pulse_serialization::{NumBytes, Read, Write};

use crate::core::check;

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, Default, Hash, PartialOrd, Ord, Read, NumBytes, Write,
)]
#[pulse(crate_path = "pulse_serialization")]
pub struct Name(u64);

impl Name {
    /// Creates a new name
    #[inline(always)]
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// TODO docs
    #[inline(always)]
    #[must_use]
    pub const fn raw(&self) -> u64 {
        self.0
    }

    pub fn as_bytes(&self) -> [u8; NAME_MAX_LEN] {
        name_to_bytes(self.0)
    }

    pub fn to_string(&self) -> String {
        let bytes = self.as_bytes();
        let value = str::from_utf8(&bytes).map(|s| s.trim_end_matches('.'));
        check(value.is_ok(), "invalid name");
        value.unwrap().to_string()
    }

    #[inline(always)]
    #[must_use]
    pub const fn suffix(self) -> Self {
        let mut remaining_bits_after_last_actual_dot: u32 = 0;
        let mut tmp: u32 = 0;

        // Walk 12 five-bit chars (bits 59..=4), left to right.
        // Note: signed loop variable in C++; we mirror with i32 here.
        let mut remaining_bits: i32 = 59;
        while remaining_bits >= 4 {
            let c = ((self.0 >> remaining_bits) & 0x1f) as u64;
            if c == 0 {
                // dot
                tmp = remaining_bits as u32;
            } else {
                // non-dot: remember the last dot position seen before this
                remaining_bits_after_last_actual_dot = tmp;
            }
            remaining_bits -= 5;
        }

        // 13th character (lowest 4 bits)
        let thirteenth_character = (self.0 & 0x0f) as u64;
        if thirteenth_character != 0 {
            // if 13th char is not a dot, a preceding dot counts as "actual"
            remaining_bits_after_last_actual_dot = tmp;
        }

        // No actual dot (other than possibly leading dots)
        if remaining_bits_after_last_actual_dot == 0 {
            return self;
        }

        // Mask bits for chars after the last actual dot, excluding the lowest 4 bits.
        let mask: u64 = (1u64 << remaining_bits_after_last_actual_dot) - 16;
        let shift: u32 = 64 - remaining_bits_after_last_actual_dot;

        // Move the suffix up and reattach the 13th character.
        Self(((self.0 & mask) << shift) | (thirteenth_character << (shift - 1)))
    }
}

impl From<u64> for Name {
    #[inline(always)]
    fn from(n: u64) -> Self {
        Self(n)
    }
}

impl From<Name> for u64 {
    #[inline(always)]
    fn from(i: Name) -> Self {
        i.0
    }
}

impl PartialEq<u64> for Name {
    #[inline(always)]
    fn eq(&self, other: &u64) -> bool {
        &self.0 == other
    }
}

impl Not for Name {
    type Output = bool;

    #[inline(always)]
    fn not(self) -> Self::Output {
        self.0 == 0
    }
}
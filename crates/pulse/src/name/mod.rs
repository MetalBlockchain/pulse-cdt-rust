use core::{
    cmp::PartialEq,
    str::{self, FromStr},
};

use pulse_name::{name_from_bytes, ParseNameError};
use pulse_serialization::{NumBytes, Read, Write};

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Default,
    Hash,
    PartialOrd,
    Ord,
    Read,
    NumBytes,
    Write
)]
#[pulse(crate_path = "pulse_serialization")]
pub struct Name(u64);

impl Name {
    /// Creates a new name
    #[inline]
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// TODO docs
    #[inline]
    #[must_use]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u64> for Name {
    #[inline]
    #[must_use]
    fn from(n: u64) -> Self {
        Self(n)
    }
}

impl From<Name> for u64 {
    #[inline]
    #[must_use]
    fn from(i: Name) -> Self {
        i.0
    }
}

impl FromStr for Name {
    type Err = ParseNameError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let name = name_from_bytes(s.bytes())?;
        Ok(name.into())
    }
}

impl PartialEq<u64> for Name {
    fn eq(&self, other: &u64) -> bool {
        &self.0 == other
    }
}
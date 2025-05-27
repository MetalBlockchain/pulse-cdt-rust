use core::ops::Deref;

use alloc::vec::Vec;

use super::{Read, ReadError, Write, WriteError};

/// A stream of bytes
pub struct DataStream {
    /// TODO docs
    bytes: Vec<u8>,
    /// TODO docs
    pos: usize,
}

impl DataStream {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            pos: 0,
        }
    }
    /// Read something from the stream
    ///
    /// # Errors
    ///
    /// Will return `Err` if there was a problem reading the data.
    #[inline(always)]
    pub fn read<T: Read>(&mut self) -> Result<T, ReadError> {
        T::read(&self.bytes, &mut self.pos)
    }

    /// Write something to the stream
    ///
    /// # Errors
    ///
    /// Will return `Err` if there was a problem writing the data.
    #[allow(clippy::needless_pass_by_value)]
    #[inline(always)]
    pub fn write<T: Write>(&mut self, thing: T) -> Result<(), WriteError> {
        // Ensure the buffer is large enough
        if self.bytes.len() < self.pos + thing.num_bytes() {
            self.bytes.resize(self.pos + thing.num_bytes(), 0);
        }
        thing.write(&mut self.bytes, &mut self.pos)
    }

    /// Gets the remaining number of bytes
    #[inline(always)]
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Gets remaining bytes as slice
    #[inline(always)]
    #[must_use]
    pub fn as_remaining_bytes(&self) -> Option<&[u8]> {
        self.bytes.get(self.pos..)
    }

    /// Resets the data stream position
    #[inline(always)]
    pub fn reset(&mut self) {
        self.pos = 0;
    }

    /// Get the current position
    #[inline(always)]
    #[must_use]
    pub const fn position(&self) -> usize {
        self.pos
    }

    /// Gets the remaining number of bytes
    #[inline(always)]
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }
}

impl From<Vec<u8>> for DataStream {
    #[inline(always)]
    #[must_use]
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes, pos: 0 }
    }
}

impl From<&[u8]> for DataStream {
    #[inline(always)]
    #[must_use]
    fn from(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
            pos: 0,
        }
    }
}

impl Deref for DataStream {
    type Target = [u8];

    #[inline(always)]
    #[must_use]
    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl AsRef<[u8]> for DataStream {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

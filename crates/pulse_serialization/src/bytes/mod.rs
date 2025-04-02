mod data_stream;
mod primitives;

pub use self::data_stream::DataStream;
use alloc::vec::Vec;
use alloc::vec;
pub use pulse_proc_macro::{Read, Write, NumBytes};

/// Count the number of bytes a type is expected to use.
pub trait NumBytes {
    /// Count the number of bytes a type is expected to use.
    fn num_bytes(&self) -> usize;
}

/// Read bytes.
pub trait Read: Sized + NumBytes {
    /// Read bytes.
    ///
    /// # Errors
    ///
    /// Will return `Err` if there was a problem reading the data.
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ReadError>;

    /// Deserializes a byte array into a data type.
    ///
    /// # Errors
    ///
    /// Will return `Err` if there was a problem reading the data.
    #[inline]
    fn unpack<T: AsRef<[u8]>>(bytes: T) -> Result<Self, ReadError> {
        Self::read(bytes.as_ref(), &mut 0)
    }
}

/// Error that can be returned when reading bytes.
#[derive(Debug, Clone, Copy)]
pub enum ReadError {
    /// Not enough bytes.
    NotEnoughBytes,
    ParseError,
}

/// Write bytes.
pub trait Write: Sized + NumBytes {
    /// Write bytes.
    ///
    /// # Errors
    ///
    /// Will return `Err` if there was a problem writing the data.
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), WriteError>;

    /// Serializes data into a byte vector.
    ///
    /// # Errors
    ///
    /// Will return `Err` if there was a problem writing the data.
    #[inline]
    fn pack(&self) -> Result<Vec<u8>, WriteError> {
        let num_bytes = self.num_bytes();
        let mut bytes = vec![0_u8; num_bytes];
        self.write(&mut bytes, &mut 0)?;
        Ok(bytes)
    }
}

/// Error that can be returned when writing bytes.
#[derive(Debug, Clone, Copy)]
pub enum WriteError {
    /// Not enough space in the vector.
    NotEnoughSpace,
    /// Failed to parse an integer.
    TryFromIntError,
    /// Not enough bytes to read.
    NotEnoughBytes,
}
use pulse_serialization::{NumBytes, Read, ReadError, Write, WriteError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixedBytes<const N: usize>(pub [u8; N]);

impl<const N: usize> FixedBytes<N> {
    pub const fn new(bytes: [u8; N]) -> Self {
        Self(bytes)
    }
}

impl<const N: usize> NumBytes for FixedBytes<N> {
    fn num_bytes(&self) -> usize {
        N
    }
}

impl<const N: usize> Read for FixedBytes<N> {
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ReadError> {
        if bytes.len() < *pos + N {
            return Err(ReadError::ParseError);
        }
        let mut arr = [0u8; N];
        arr.copy_from_slice(&bytes[*pos..*pos + N]);
        *pos += N;
        Ok(Self(arr))
    }
}

impl<const N: usize> Write for FixedBytes<N> {
    fn write(&self, bytes: &mut [u8], pos: &mut usize) -> Result<(), WriteError> {
        if bytes.len() < *pos + N {
            return Err(WriteError::NotEnoughSpace);
        }
        bytes[*pos..*pos + N].copy_from_slice(&self.0);
        *pos += N;
        Ok(())
    }
}
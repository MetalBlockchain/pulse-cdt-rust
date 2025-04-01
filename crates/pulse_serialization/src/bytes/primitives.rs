use super::{NumBytes, Read, ReadError, Write, WriteError};

impl NumBytes for u16 {
    #[inline]
    fn num_bytes(&self) -> usize {
        core::mem::size_of::<u16>()
    }
}

impl NumBytes for i16 {
    #[inline]
    fn num_bytes(&self) -> usize {
        core::mem::size_of::<u16>()
    }
}

impl NumBytes for u32 {
    #[inline]
    fn num_bytes(&self) -> usize {
        core::mem::size_of::<u32>()
    }
}

impl NumBytes for i32 {
    #[inline]
    fn num_bytes(&self) -> usize {
        core::mem::size_of::<u32>()
    }
}

impl NumBytes for u64 {
    #[inline]
    fn num_bytes(&self) -> usize {
        core::mem::size_of::<u64>()
    }
}

impl NumBytes for i64 {
    #[inline]
    fn num_bytes(&self) -> usize {
        core::mem::size_of::<u64>()
    }
}

impl NumBytes for String {
    #[inline]
    fn num_bytes(&self) -> usize {
        self.as_bytes().len()
    }
}

impl Read for u16 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ReadError> {
        if bytes.len() < *pos + core::mem::size_of::<u16>() {
            return Err(ReadError::NotEnoughBytes);
        }
        let value = u16::from_le_bytes([bytes[*pos], bytes[*pos + 1]]);
        *pos += core::mem::size_of::<u16>();
        Ok(value)
    }
}

impl Read for i16 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ReadError> {
        let result = u16::read(bytes, pos).unwrap();
        Ok(result as i16)
    }
}

impl Read for u32 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ReadError> {
        if bytes.len() < *pos + core::mem::size_of::<u32>() {
            return Err(ReadError::NotEnoughBytes);
        }
        let value = u32::from_le_bytes([
            bytes[*pos],
            bytes[*pos + 1],
            bytes[*pos + 2],
            bytes[*pos + 3],
        ]);
        *pos += core::mem::size_of::<u32>();
        Ok(value)
    }
}

impl Read for i32 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ReadError> {
        let result = u32::read(bytes, pos).unwrap();
        Ok(result as i32)
    }
}

impl Read for u64 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ReadError> {
        if bytes.len() < *pos + core::mem::size_of::<u64>() {
            return Err(ReadError::NotEnoughBytes);
        }
        let value = u64::from_le_bytes([
            bytes[*pos],
            bytes[*pos + 1],
            bytes[*pos + 2],
            bytes[*pos + 3],
            bytes[*pos + 4],
            bytes[*pos + 5],
            bytes[*pos + 6],
            bytes[*pos + 7],
        ]);
        *pos += core::mem::size_of::<u64>();
        Ok(value)
    }
}

impl Read for i64 {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ReadError> {
        let result = u64::read(bytes, pos).unwrap();
        Ok(result as i64)
    }
}

impl Read for String {
    #[inline]
    fn read(bytes: &[u8], pos: &mut usize) -> Result<Self, ReadError> {
        let len = bytes.len();
        if len < *pos {
            return Err(ReadError::NotEnoughBytes);
        }
        let end = bytes[*pos..]
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(len - *pos);
        let result = String::from_utf8_lossy(&bytes[*pos..*pos + end]);
        *pos += end + 1;
        Ok(result.to_string())
    }
}

impl Write for u16 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), WriteError> {
        if bytes.len() < *pos + core::mem::size_of::<u16>() {
            return Err(WriteError::NotEnoughBytes);
        }
        let value = self.to_le_bytes();
        bytes[*pos] = value[0];
        bytes[*pos + 1] = value[1];
        *pos += core::mem::size_of::<u16>();
        Ok(())
    }
}

impl Write for i16 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), WriteError> {
        let result = u16::write(&(*self as u16), bytes, pos).unwrap();
        Ok(result)
    }
}

impl Write for u32 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), WriteError> {
        if bytes.len() < *pos + core::mem::size_of::<u32>() {
            return Err(WriteError::NotEnoughBytes);
        }
        let value = self.to_le_bytes();
        bytes[*pos] = value[0];
        bytes[*pos + 1] = value[1];
        bytes[*pos + 2] = value[2];
        bytes[*pos + 3] = value[3];
        *pos += core::mem::size_of::<u32>();
        Ok(())
    }
}

impl Write for i32 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), WriteError> {
        let result = u32::write(&(*self as u32), bytes, pos).unwrap();
        Ok(result)
    }
}

impl Write for u64 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), WriteError> {
        if bytes.len() < *pos + core::mem::size_of::<u64>() {
            return Err(WriteError::NotEnoughBytes);
        }
        let value = self.to_le_bytes();
        bytes[*pos] = value[0];
        bytes[*pos + 1] = value[1];
        bytes[*pos + 2] = value[2];
        bytes[*pos + 3] = value[3];
        bytes[*pos + 4] = value[4];
        bytes[*pos + 5] = value[5];
        bytes[*pos + 6] = value[6];
        bytes[*pos + 7] = value[7];
        *pos += core::mem::size_of::<u64>();
        Ok(())
    }
}

impl Write for i64 {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), WriteError> {
        let result = u64::write(&(*self as u64), bytes, pos).unwrap();
        Ok(result)
    }
}

impl Write for String {
    #[inline]
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), WriteError> {
        let len = self.as_bytes().len();
        if bytes.len() < *pos + len + 1 {
            return Err(WriteError::NotEnoughBytes);
        }
        bytes[*pos..*pos + len].copy_from_slice(self.as_bytes());
        bytes[*pos + len] = 0;
        *pos += len + 1;
        Ok(())
    }
}
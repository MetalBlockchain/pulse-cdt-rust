use pulse_serialization::{NumBytes, Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicKey(pub [u8; 33]);

impl Read for PublicKey {
    fn read(data: &[u8], pos: &mut usize) -> Result<Self, pulse_serialization::ReadError> {
        if *pos + 33 > data.len() {
            return Err(pulse_serialization::ReadError::NotEnoughBytes);
        }
        let mut id = [0u8; 33];
        id.copy_from_slice(&data[*pos..*pos + 33]);
        *pos += 33;
        Ok(PublicKey(id))
    }
}

impl NumBytes for PublicKey {
    fn num_bytes(&self) -> usize {
        33 // Compressed public key size
    }
}

impl Write for PublicKey {
    fn write(
        &self,
        bytes: &mut [u8],
        pos: &mut usize,
    ) -> Result<(), pulse_serialization::WriteError> {
        if *pos + 33 > bytes.len() {
            return Err(pulse_serialization::WriteError::NotEnoughSpace);
        }
        bytes[*pos..*pos + 33].copy_from_slice(&self.0);
        *pos += 33;
        Ok(())
    }
}

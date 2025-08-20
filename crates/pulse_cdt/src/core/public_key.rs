use pulse_serialization::{NumBytes, Read, Write};
use secp256k1::{PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicKey(pub secp256k1::PublicKey);

impl Read for PublicKey {
    fn read(data: &[u8], pos: &mut usize) -> Result<Self, pulse_serialization::ReadError> {
        if *pos + 33 > data.len() {
            return Err(pulse_serialization::ReadError::NotEnoughBytes);
        }
        let mut id = [0u8; 33];
        id.copy_from_slice(&data[*pos..*pos + 33]);
        *pos += 33;
        let key = secp256k1::PublicKey::from_byte_array_compressed(&id)
            .map_err(|_| pulse_serialization::ReadError::ParseError)?;
        Ok(PublicKey(key))
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
        let compressed = self.0.serialize();
        bytes[*pos..*pos + 33].copy_from_slice(&compressed);
        *pos += 33;
        Ok(())
    }
}

impl Default for PublicKey {
    fn default() -> Self {
        let secp = Secp256k1::new();
        let secret_key =
            SecretKey::from_byte_array(&[0xcd; 32]).expect("32 bytes, within curve order");
        let public_key = Secp256k1PublicKey::from_secret_key(&secp, &secret_key);
        PublicKey(public_key)
    }
}

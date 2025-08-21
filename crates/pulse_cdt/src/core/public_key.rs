use pulse_serialization::{NumBytes, Read, Write};

use crate::core::{check, FixedBytes};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct PublicKey(pub FixedBytes<33>);

impl PublicKey {
    /// Create a new `PublicKey` from a byte slice
    #[inline]
    pub fn new(slice: &[u8]) -> Self {
        check(slice.len() == 33, "public key must be 33 bytes long");
        let data: [u8; 33] = slice
            .try_into()
            .expect("slice length is guaranteed to be 33 bytes");
        Self(FixedBytes::new(data))
    }
}

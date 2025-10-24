use pulse_serialization::{NumBytes, Read, Write};

use crate::core::{check, FixedBytes};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Read, Write, NumBytes, Default)]
#[pulse(crate_path = "pulse_serialization")]
pub struct PublicKey(pub FixedBytes<34>);

impl PublicKey {
    /// Create a new `PublicKey` from a byte slice
    #[inline]
    pub fn new(slice: &[u8]) -> Self {
        check(slice.len() == 34, "public key must be 34 bytes long");
        let data: [u8; 34] = slice
            .try_into()
            .expect("slice length is guaranteed to be 34 bytes");
        Self(FixedBytes::new(data))
    }
}
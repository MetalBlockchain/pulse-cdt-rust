use pulse_serialization::{NumBytes, Read, Write};

use crate::core::FixedBytes;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct Signature(pub FixedBytes<65>);

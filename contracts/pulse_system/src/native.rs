use pulse_cdt::{core::{Checksum256, MultiIndexDefinition, Name, Table}, name, table, NumBytes, Read, Write};

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct AbiHash {
    pub owner: Name,
    pub hash: Checksum256,
}

pub const ABI_HASH_TABLE: MultiIndexDefinition<AbiHash> =
    MultiIndexDefinition::new(name!("abihash"));
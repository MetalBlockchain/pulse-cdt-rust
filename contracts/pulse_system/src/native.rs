use pulse_cdt::{
    NumBytes, Read, Write,
    core::{Checksum256, MultiIndexDefinition, Name, Table},
    name, table,
};

#[derive(Read, Write, NumBytes, Clone, PartialEq)]
#[table(primary_key = row.owner.raw())]
pub struct AbiHash {
    pub owner: Name,
    pub hash: Checksum256,
}

pub const ABI_HASH_TABLE: MultiIndexDefinition<AbiHash> =
    MultiIndexDefinition::new(name!("abihash"));

mod bytes;

use bincode::{Decode, Encode};

pub use self::bytes::{
    DataStream, NumBytes, Read, ReadError, Write, WriteError,
};

#[derive(Encode, Decode, Debug)]
struct Test {
    a: u16,
    b: u32,
    c: u64,
    d: u128,
    e: String,
    f: Vec<u8>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        
    }
}
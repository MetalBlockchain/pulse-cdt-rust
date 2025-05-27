#![no_std]
extern crate alloc;

mod bytes;

pub use self::bytes::{DataStream, NumBytes, Read, ReadError, Write, WriteError};

#[cfg(test)]
mod tests {
    use super::bytes::Read;
    use alloc::{borrow::ToOwned, string::String};

    #[test]
    fn test_string() {
        let mut ds = super::DataStream::new();
        let s = "Hello, world!".to_owned();
        ds.write(s).expect("Failed to write");
        assert_eq!(hex::encode(ds.as_bytes()), "000d48656c6c6f2c20776f726c6421");
        let result = String::read(&ds.as_bytes(), &mut 0).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_u16() {
        let mut ds = super::DataStream::new();
        let s = 1u16;
        ds.write(s).expect("Failed to write");
        assert_eq!(hex::encode(ds.as_bytes()), "0001");
        let result = u16::read(&ds.as_bytes(), &mut 0).unwrap();
        assert_eq!(result, 1u16);
    }

    #[test]
    fn test_i16() {
        let mut ds = super::DataStream::new();
        let s = 1i16;
        ds.write(s).expect("Failed to write");
        assert_eq!(hex::encode(ds.as_bytes()), "0001");
        let result = i16::read(&ds.as_bytes(), &mut 0).unwrap();
        assert_eq!(result, 1i16);
    }

    #[test]
    fn test_u32() {
        let mut ds = super::DataStream::new();
        let s = 1u32;
        ds.write(s).expect("Failed to write");
        assert_eq!(hex::encode(ds.as_bytes()), "00000001");
        let result = u32::read(&ds.as_bytes(), &mut 0).unwrap();
        assert_eq!(result, 1u32);
    }

    #[test]
    fn test_i32() {
        let mut ds = super::DataStream::new();
        let s = 1i32;
        ds.write(s).expect("Failed to write");
        assert_eq!(hex::encode(ds.as_bytes()), "00000001");
        let result = i32::read(&ds.as_bytes(), &mut 0).unwrap();
        assert_eq!(result, 1i32);
    }

    #[test]
    fn test_u64() {
        let mut ds = super::DataStream::new();
        let s = 1u64;
        ds.write(s).expect("Failed to write");
        assert_eq!(hex::encode(ds.as_bytes()), "0000000000000001");
        let result = u64::read(&ds.as_bytes(), &mut 0).unwrap();
        assert_eq!(result, 1u64);
    }

    #[test]
    fn test_i64() {
        let mut ds = super::DataStream::new();
        let s = 1i64;
        ds.write(s).expect("Failed to write");
        assert_eq!(hex::encode(ds.as_bytes()), "0000000000000001");
        let result = i64::read(&ds.as_bytes(), &mut 0).unwrap();
        assert_eq!(result, 1i64);
    }

    #[test]
    fn test_bool() {
        let mut ds = super::DataStream::new();
        let s = true;
        ds.write(s).expect("Failed to write");
        assert_eq!(hex::encode(ds.as_bytes()), "01");
        let result = bool::read(&ds.as_bytes(), &mut 0).unwrap();
        assert_eq!(result, true);
    }
}

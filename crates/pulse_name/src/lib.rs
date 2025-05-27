#![no_std]
extern crate alloc;

mod name;
pub use name::{name_from_bytes, name_to_bytes, ParseNameError, NAME_CHARS, NAME_MAX_LEN};

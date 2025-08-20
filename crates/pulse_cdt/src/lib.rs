#![no_std]
extern crate alloc;

pub mod contracts;
pub mod core;

pub use ::core::ffi::c_char;
pub use ::core::ffi::c_void;

pub use pulse_proc_macro::{action, contract, dispatch, name, name_raw, symbol_with_code, table};
pub use pulse_serialization::{DataStream, NumBytes, Read, ReadError, Write, WriteError};

pub mod __reexports {
    pub use dlmalloc;
    pub use lol_alloc;
}

#![no_std]
extern crate alloc;

#[cfg(target_arch = "wasm32")]
use ::core::panic::PanicInfo;

pub mod contracts;
pub mod core;

pub use ::core::ffi::c_char;
pub use ::core::ffi::c_void;

pub use pulse_proc_macro::{action, dispatch, name};
pub use pulse_serialization::{DataStream, NumBytes, Read, ReadError, Write, WriteError};

#[cfg(target_arch = "wasm32")]
use lol_alloc::{AssumeSingleThreaded, LeakingAllocator};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<LeakingAllocator> =
    unsafe { AssumeSingleThreaded::new(LeakingAllocator::new()) };

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    ::core::arch::wasm32::unreachable()
}

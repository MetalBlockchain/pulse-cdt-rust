use alloc::vec;
use pulse::{Read, ReadError};

mod action_impl {
    extern "C" {
        #[link_name = "action_data_size"]
        pub fn action_data_size() -> u32;

        #[link_name = "read_action_data"]
        pub fn read_action_data(msg: *mut crate::c_void, len: u32) -> u32;
    }
}

#[inline]
pub fn action_data_size() -> u32 {
    unsafe { action_impl::action_data_size() }
}

#[inline]
pub fn read_action_data<T: Read>() -> Result<T, ReadError> {
    let num_bytes = action_data_size();
    let mut bytes = vec![0_u8; num_bytes as usize];
    let ptr: *mut crate::c_void = &mut bytes[..] as *mut _ as *mut crate::c_void;
    unsafe {
        action_impl::read_action_data(ptr, num_bytes);
    }
    let mut pos = 0;
    T::read(&bytes, &mut pos)
}
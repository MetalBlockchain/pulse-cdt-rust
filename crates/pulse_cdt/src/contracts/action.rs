use alloc::vec;
use alloc::vec::Vec;
use pulse_serialization::{Read, ReadError};

use crate::core::Name;

mod action_impl {
    extern "C" {
        #[link_name = "action_data_size"]
        pub fn action_data_size() -> u32;

        #[link_name = "read_action_data"]
        pub fn read_action_data(msg: *mut crate::c_void, len: u32) -> u32;

        #[link_name = "require_auth"]
        pub fn require_auth(name: u64);

        #[link_name = "has_auth"]
        pub fn has_auth(name: u64) -> bool;

        #[link_name = "require_recipient"]
        pub fn require_recipient(recipient: u64);

        #[link_name = "is_account"]
        pub fn is_account(name: u64) -> bool;

        #[link_name = "get_self"]
        pub fn get_self() -> u64;

        #[link_name = "current_receiver"]
        pub fn current_receiver() -> u64;
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

#[inline]
pub fn require_auth(name: Name) {
    unsafe { action_impl::require_auth(name.raw()) }
}

#[inline]
pub fn has_auth(name: Name) -> bool {
    unsafe { action_impl::has_auth(name.raw()) }
}

#[inline]
pub fn require_recipient(recipient: Name) {
    unsafe { action_impl::require_recipient(recipient.raw()) }
}

#[inline]
pub fn is_account(recipient: Name) -> bool {
    unsafe { action_impl::is_account(recipient.raw()) }
}

#[inline]
pub fn get_self() -> Name {
    let result = unsafe { action_impl::get_self() };
    Name::new(result)
}

#[inline]
pub fn current_receiver() -> Name {
    let receiver = unsafe { action_impl::current_receiver() };
    Name::new(receiver)
}

#[derive(Clone, Debug, Default)]
pub struct Action<T> {
    /// Name of the account the action is intended for
    pub account: Name,
    /// Name of the action
    pub name: Name,
    /// List of permissions that authorize this action
    pub authorization: Vec<PermissionLevel>,
    /// Payload data
    pub data: T,
}

pub trait ActionFn: Clone {
    /// TODO docs
    const NAME: Name;
    /// TODO docs.
    fn call(self);
}
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash, PartialOrd, Ord)]
pub struct PermissionLevel {
    /// TODO docs
    pub actor: Name,
    /// TODO docs
    pub permission: Name,
}

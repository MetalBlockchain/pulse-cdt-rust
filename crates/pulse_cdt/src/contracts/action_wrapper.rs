use core::marker::PhantomData;

use alloc::vec::Vec;
use pulse_serialization::Write;

use crate::{
    contracts::{Action, PermissionLevel},
    core::Name,
};

pub struct ActionWrapper<T>
where
    T: Write,
{
    _data: PhantomData<T>,
    name: Name,
}

impl<T> ActionWrapper<T>
where
    T: Write,
{
    #[inline]
    pub const fn new(name: Name) -> Self {
        Self {
            _data: PhantomData,
            name,
        }
    }

    pub fn to_action(&self, account: Name, authorization: Vec<PermissionLevel>, data: T) -> Action {
        Action {
            account: account,
            name: self.name.clone(),
            authorization: authorization,
            data: data.pack().expect("failed to serialize action data"),
        }
    }

    pub fn send(&self, action: &Action) {
        action.send();
    }
}

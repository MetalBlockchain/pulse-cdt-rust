use super::{Table, TableCursor};
use crate::{
    contracts::{db_get_i64, db_next_i64, db_remove_i64, db_update_i64},
    core::name::Name,
};
use alloc::vec;
use alloc::vec::Vec;
use core::{borrow::BorrowMut, ffi::c_void, marker::PhantomData, ptr::null_mut};
use pulse_serialization::{NumBytes, ReadError, Write, WriteError};

#[derive(Copy, Clone, Debug)]
pub struct PrimaryTableCursor<T>
where
    T: Table,
{
    value: i32,
    code: Name,
    scope: u64,
    data: PhantomData<T>,
}

impl<T> PrimaryTableCursor<T>
where
    T: Table,
{
    pub const fn new(value: i32, code: Name, scope: u64) -> Self {
        Self {
            value,
            code,
            scope,
            data: PhantomData,
        }
    }
}

impl<T> TableCursor<T> for PrimaryTableCursor<T>
where
    T: Table,
{
    #[inline]
    fn bytes(&self) -> Vec<u8> {
        let nullptr: *mut c_void = null_mut() as *mut _ as *mut c_void;
        let size = db_get_i64(self.value, nullptr, 0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut bytes = vec![0u8; size as usize];
        let ptr: *mut c_void = &mut bytes[..] as *mut _ as *mut c_void;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        db_get_i64(self.value, ptr, size as u32);
        bytes
    }

    #[inline]
    fn erase(&self) -> Result<T::Row, ReadError> {
        let item = self.get()?;
        db_remove_i64(self.value);
        Ok(item)
    }

    #[inline]
    fn modify<I, F>(&self, mut item: I, payer: Name, modifier: F) -> Result<usize, WriteError>
    where
        I: BorrowMut<T::Row>,
        F: FnOnce(&mut T::Row),
    {
        let item = item.borrow_mut();
        modifier(item);
        let size = item.num_bytes();
        let mut bytes = vec![0_u8; size];
        let mut pos = 0;
        item.write(&mut bytes, &mut pos)?;
        let bytes_ptr: *const c_void = &bytes[..] as *const _ as *const c_void;
        #[allow(clippy::cast_possible_truncation)]
        db_update_i64(self.value, payer, bytes_ptr, pos as u32);

        Ok(pos)
    }
}

impl<'a, T> IntoIterator for PrimaryTableCursor<T>
where
    T: Table,
{
    type IntoIter = PrimaryTableIterator<T>;
    type Item = Self;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        PrimaryTableIterator {
            value: self.value,
            code: self.code,
            scope: self.scope,
            data: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PrimaryTableIterator<T>
where
    T: Table,
{
    value: i32,
    code: Name,
    scope: u64,
    data: PhantomData<T>,
}

impl<T> Iterator for PrimaryTableIterator<T>
where
    T: Table,
{
    type Item = PrimaryTableCursor<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.value == -1 {
            return None;
        }

        let cursor = PrimaryTableCursor {
            value: self.value,
            code: self.code,
            scope: self.scope,
            data: PhantomData,
        };

        let mut pk = 0_u64;
        let ptr: *mut u64 = &mut pk;
        self.value = db_next_i64(self.value, ptr);

        Some(cursor)
    }
}

use core::{
    borrow::{Borrow, BorrowMut},
    ffi::c_void,
    marker::PhantomData,
    ops::Deref,
    ptr::null,
};

use crate::{
    contracts::{db_find_i64, db_get_i64, db_remove_i64, db_store_i64, db_update_i64},
    core::{check, name::Name},
    DataStream, NumBytes, Read, ReadError, Write, WriteError,
};
use alloc::vec;
use alloc::vec::Vec;

pub struct MultiIndexDefinition<T>
where
    T: Table,
{
    table: Name,
    _data: PhantomData<T>,
}

impl<T> MultiIndexDefinition<T>
where
    T: Table,
{
    #[inline]
    pub const fn new(table: Name) -> Self {
        Self {
            table,
            _data: PhantomData,
        }
    }

    #[inline]
    pub const fn index(&self, code: Name, scope: u64) -> MultiIndex<T> {
        MultiIndex::new(code, scope, self.table)
    }
}

#[derive(Clone, PartialEq)]
pub struct MultiIndex<T>
where
    T: Table,
{
    code: Name,
    scope: u64,
    table: Name,
    _data: PhantomData<T>,
}

impl<T> MultiIndex<T>
where
    T: Table,
{
    #[inline]
    pub const fn new(code: Name, scope: u64, table: Name) -> Self {
        Self {
            code,
            scope,
            table,
            _data: PhantomData,
        }
    }

    #[inline]
    pub fn find(&self, key: u64) -> ConstIterator<T> {
        let itr = db_find_i64(self.code, self.scope, self.table.into(), key.into());
        if itr < 0 {
            return self.end();
        } else {
            let item = self.load_object_by_primary_iterator(itr);
            ConstIterator::new(self.clone(), Some(Item::new(self.clone(), itr, item)))
        }
    }

    #[inline]
    pub fn get(&self, key: u64, error_msg: &str) -> ConstIterator<T> {
        let result = self.find(key);
        check(result != self.end(), error_msg);
        result
    }

    #[inline]
    pub fn load_object_by_primary_iterator(&self, itr: i32) -> T::Row {
        let size = db_get_i64(itr, null(), 0);
        check(size >= 0, "error reading iterator");

        let mut buffer = vec![0_u8; size as usize];
        db_get_i64(itr, buffer.as_mut_ptr() as *mut c_void, size as u32);
        T::Row::read(&buffer, &mut 0).expect("failed to read row")
    }

    #[inline]
    pub fn emplace(&self, payer: Name, item: T::Row) -> ConstIterator<T> {
        let item = item.borrow();
        let id = T::primary_key(item);
        let size = item.num_bytes();
        let mut bytes = vec![0_u8; size];
        let mut pos = 0;
        item.write(&mut bytes, &mut pos)
            .expect("failed to write item");
        let itr = db_store_i64(
            self.scope,
            self.table.into(),
            payer,
            id.into(),
            &bytes[..],
            pos as u32,
        );
        ConstIterator::new(self.clone(), Some(Item::new(self.clone(), itr, item.clone())))
    }

    #[inline]
    pub fn modify<F>(&self, item: &mut ConstIterator<T>, payer: Name, modifier: F)
    where
        F: FnOnce(&mut T::Row),
    {
        let item = item.borrow_mut();
        modifier(item);
        let size = item.num_bytes();
        let mut bytes = vec![0_u8; size];
        let mut pos = 0;
        item.write(&mut bytes, &mut pos)
            .expect("failed to write item");
        let bytes_ptr: *const c_void = &bytes[..] as *const _ as *const c_void;
        #[allow(clippy::cast_possible_truncation)]
        db_update_i64(item.primary_itr, payer, bytes_ptr, pos as u32);
    }

    #[inline]
    pub fn erase(&self, item: ConstIterator<T>) {
        db_remove_i64(item.primary_itr);
    }

    pub fn end(&self) -> ConstIterator<T> {
        ConstIterator::new(self.clone(), None)
    }
}

pub trait Table: Sized + Clone + PartialEq {
    type Key: Read + Write + NumBytes + Into<u64>;
    /// TODO docs
    type Row: Read + Write + NumBytes + Sized + PartialEq + Clone;
    /// TODO docs
    fn primary_key(row: &Self::Row) -> Self::Key;
}

pub enum Payer {
    Same,
    New(Name),
}

pub trait TableCursor<T>: IntoIterator
where
    T: Table,
{
    fn bytes(&self) -> Vec<u8>;

    #[inline]
    fn stream(&self) -> DataStream {
        self.bytes().into()
    }

    /// Read and deserialize the current table row
    ///
    /// # Errors
    ///
    /// Will return `Err` if there was an issue reading the stored value.
    #[inline]
    fn get(&self) -> Result<T::Row, ReadError> {
        self.stream().read()
    }

    /// Erase the current row
    ///
    /// # Errors
    ///
    /// Will return `Err` if there was an issue reading the stored value. Stored
    /// values must be read in order to erase secondary indexes.
    fn erase(&self) -> Result<T::Row, ReadError>;

    /// Modify the current row
    ///
    /// # Errors
    ///
    /// Will return `Err` if there was an issue serializing the value.
    fn modify<I, F>(&self, item: I, payer: Payer, modifier: F) -> Result<usize, WriteError>
    where
        I: BorrowMut<T::Row>,
        F: FnOnce(&mut T::Row);
}

pub struct ConstIterator<T>
where
    T: Table,
{
    idx: MultiIndex<T>,
    item: Option<Item<T>>,
}

impl<T> ConstIterator<T>
where
    T: Table,
{
    #[inline]
    pub const fn new(idx: MultiIndex<T>, item: Option<Item<T>>) -> Self {
        Self { idx, item }
    }

    pub fn value(&self) -> T::Row {
        self.item.as_ref().expect("iterator is empty").inner.clone()
    }
}

impl<T> core::ops::DerefMut for ConstIterator<T>
where
    T: Table,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item.as_mut().expect("iterator is empty")
    }
}

impl<T> Deref for ConstIterator<T>
where
    T: Table,
{
    type Target = Item<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.item.as_ref().expect("iterator is empty")
    }
}

impl<T> PartialEq for ConstIterator<T>
where
    T: Table,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.inner == other.inner
    }
}

#[derive(Clone, PartialEq)]
pub struct Item<T>
where
    T: Table,
{
    idx: MultiIndex<T>,
    primary_itr: i32,
    inner: T::Row,
}

impl<T> Item<T>
where
    T: Table,
{
    #[inline]
    pub const fn new(idx: MultiIndex<T>, primary_itr: i32, inner: T::Row) -> Self {
        Self {
            idx,
            primary_itr,
            inner,
        }
    }
}

impl<T> Deref for Item<T>
where
    T: Table,
{
    type Target = T::Row;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> core::ops::DerefMut for Item<T>
where
    T: Table,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

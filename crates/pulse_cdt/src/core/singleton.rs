use core::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

use alloc::rc::Rc;

use crate::core::{check, MultiIndex, MultiIndexDefinition, Name, Table};

pub struct SingletonDefinition<T>
where
    T: Table,
{
    singleton_name: Name,
    _marker: core::marker::PhantomData<T>,
}

impl<T> SingletonDefinition<T>
where
    T: Table,
{
    #[inline]
    pub const fn new(singleton_name: Name) -> Self {
        Self {
            singleton_name,
            _marker: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn get_instance(&self, code: Name, scope: u64) -> Singleton<T> {
        Singleton::new(self, code, scope)
    }
}

pub struct Singleton<T>
where
    T: Table,
{
    pk_value: u64,
    table: MultiIndex<T>,
}

impl<T> Singleton<T>
where
    T: Table,
{
    #[inline]
    pub fn new(def: &SingletonDefinition<T>, code: Name, scope: u64) -> Self {
        Self {
            pk_value: def.singleton_name.raw(),
            table: MultiIndexDefinition::new(def.singleton_name).index(code, scope),
        }
    }

    #[inline]
    pub fn exists(&self) -> bool {
        self.table.find(self.pk_value) != self.table.end()
    }

    #[inline]
    pub fn get(&self) -> T::Row {
        let itr = self.table.find(self.pk_value);
        check(itr != self.table.end(), "singleton does not exist");
        itr.value()
    }

    #[inline]
    pub fn get_or_default(&self, def: T::Row) -> T::Row {
        let itr = self.table.find(self.pk_value);
        if itr != self.table.end() {
            itr.value()
        } else {
            def
        }
    }

    #[inline]
    pub fn get_or_create(&self, bill_to_account: Name, def: T::Row) -> T::Row {
        let itr = self.table.find(self.pk_value);
        if itr != self.table.end() {
            itr.value()
        } else {
            self.table.emplace(bill_to_account, def).value()
        }
    }

    #[inline]
    pub fn set(&self, value: T::Row, bill_to_account: Name) {
        let mut itr = self.table.find(self.pk_value);
        if itr != self.table.end() {
            self.table.modify(&mut itr, bill_to_account, |s| *s = value);
        } else {
            self.table.emplace(bill_to_account, value);
        }
    }

    #[inline]
    pub fn remove(&self) {
        let itr = self.table.find(self.pk_value);
        if itr != self.table.end() {
            self.table.erase(itr);
        }
    }
}

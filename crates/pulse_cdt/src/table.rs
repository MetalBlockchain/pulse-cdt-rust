use std::borrow::Borrow;

use pulse::{DataStream, Name, NumBytes, Read, ReadError, Write, WriteError};
use table_primary_index::PrimaryTableIndex;

mod table_primary_index;

pub trait Table: Sized {
    /// TODO docs
    const NAME: Name;
    type Key: Read + Write + NumBytes + Into<u64>;
    /// TODO docs
    type Row: Read + Write + NumBytes;
    /// TODO docs
    fn primary_key(row: &Self::Row) -> Self::Key;
    /// TODO docs
    #[inline]
    fn table<C, S>(code: C, scope: S) -> PrimaryTableIndex<Self>
    where
        C: Into<Name>,
        S: Into<Name>,
    {
        PrimaryTableIndex::new(code, scope)
    }
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
    fn modify<I: Borrow<T::Row>>(
        &self,
        item: I,
        payer: Payer,
    ) -> Result<usize, WriteError>;
}
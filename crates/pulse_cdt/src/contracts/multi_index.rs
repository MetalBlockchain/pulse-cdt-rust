use crate::core::Name;

mod database_impl {
    extern "C" {
        #[link_name = "db_get_i64"]
        pub fn db_get_i64(iterator: i32, data: *const crate::c_void, len: u32) -> i32;

        #[link_name = "db_remove_i64"]
        pub fn db_remove_i64(iterator: i32);

        #[link_name = "db_next_i64"]
        pub fn db_next_i64(iterator: i32, primary: *mut u64) -> i32;

        #[link_name = "db_previous_i64"]
        pub fn db_previous_i64(iterator: i32, primary: *mut u64) -> i32;

        #[link_name = "db_update_i64"]
        pub fn db_update_i64(iterator: i32, payer: u64, data: *const crate::c_void, len: u32);

        #[link_name = "db_store_i64"]
        pub fn db_store_i64(
            scope: u64,
            table: u64,
            payer: u64,
            id: u64,
            data: *const crate::c_void,
            len: u32,
        ) -> i32;

        #[link_name = "db_find_i64"]
        pub fn db_find_i64(code: u64, scope: u64, table: u64, id: u64) -> i32;

        #[link_name = "db_end_i64"]
        pub fn db_end_i64(code: u64, scope: u64, table: u64) -> i32;

        #[link_name = "db_lowerbound_i64"]
        pub fn db_lowerbound_i64(code: u64, scope: u64, table: u64, id: u64) -> i32;

        #[link_name = "db_upperbound_i64"]
        pub fn db_upperbound_i64(code: u64, scope: u64, table: u64, id: u64) -> i32;
    }
}

#[inline]
pub fn db_get_i64(iterator: i32, data: *const crate::c_void, len: u32) -> i32 {
    unsafe { database_impl::db_get_i64(iterator, data, len) }
}

#[inline]
pub fn db_remove_i64(iterator: i32) {
    unsafe { database_impl::db_remove_i64(iterator) }
}

#[inline]
pub fn db_next_i64(iterator: i32, primary: *mut u64) -> i32 {
    unsafe { database_impl::db_next_i64(iterator, primary) }
}

#[inline]
pub fn db_previous_i64(iterator: i32, primary: *mut u64) -> i32 {
    unsafe { database_impl::db_previous_i64(iterator, primary) }
}

#[inline]
pub fn db_update_i64(iterator: i32, payer: Name, data: *const crate::c_void, len: u32) {
    unsafe { database_impl::db_update_i64(iterator, payer.raw(), data, len) }
}

#[inline]
pub fn db_store_i64(scope: u64, table: Name, payer: Name, id: u64, data: &[u8], len: u32) -> i32 {
    unsafe {
        database_impl::db_store_i64(
            scope,
            table.raw(),
            payer.raw(),
            id,
            data as *const _ as *const crate::c_void,
            len,
        )
    }
}

// Tries to find an item by its primary key.
// Returns the iterator if found, or -1 if not found.
#[inline]
pub fn db_find_i64(code: Name, scope: u64, table: Name, id: u64) -> i32 {
    unsafe { database_impl::db_find_i64(code.raw(), scope, table.raw(), id) }
}

#[inline]
pub fn db_end_i64(code: Name, scope: u64, table: Name) -> i32 {
    unsafe { database_impl::db_end_i64(code.raw(), scope, table.raw()) }
}

#[inline]
pub fn db_lowerbound_i64(code: Name, scope: u64, table: Name, id: u64) -> i32 {
    unsafe { database_impl::db_lowerbound_i64(code.raw(), scope, table.raw(), id) }
}

#[inline]
pub fn db_upperbound_i64(code: Name, scope: u64, table: Name, id: u64) -> i32 {
    unsafe { database_impl::db_upperbound_i64(code.raw(), scope, table.raw(), id) }
}

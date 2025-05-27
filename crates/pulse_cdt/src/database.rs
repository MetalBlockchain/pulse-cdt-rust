use crate::core::name::Name;

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
        );

        #[link_name = "db_find_i64"]
        pub fn db_find_i64(code: u64, scope: u64, table: u64, id: u64) -> i32;

        #[link_name = "db_end_i64"]
        pub fn db_end_i64(code: u64, scope: u64, table: u64) -> i32;

        #[link_name = "db_lowerbound_i64"]
        pub fn db_lowerbound_i64(code: u64, scope: u64, table: u64, id: u64) -> i32;
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
    unsafe { database_impl::db_update_i64(iterator, payer.as_u64(), data, len) }
}

#[inline]
pub fn db_store_i64(
    scope: Name,
    table: Name,
    payer: Name,
    id: u64,
    data: *const crate::c_void,
    len: u32,
) {
    unsafe {
        database_impl::db_store_i64(
            scope.as_u64(),
            table.as_u64(),
            payer.as_u64(),
            id,
            data,
            len,
        )
    }
}

// Tries to find an item by its primary key.
// Returns the iterator if found, or -1 if not found.
#[inline]
pub fn db_find_i64(code: Name, scope: Name, table: Name, id: u64) -> i32 {
    unsafe { database_impl::db_find_i64(code.as_u64(), scope.as_u64(), table.as_u64(), id) }
}

#[inline]
pub fn db_end_i64(code: Name, scope: Name, table: Name) -> i32 {
    unsafe { database_impl::db_end_i64(code.as_u64(), scope.as_u64(), table.as_u64()) }
}

#[inline]
pub fn db_lowerbound_i64(code: Name, scope: Name, table: Name, id: u64) -> i32 {
    unsafe { database_impl::db_lowerbound_i64(code.as_u64(), scope.as_u64(), table.as_u64(), id) }
}

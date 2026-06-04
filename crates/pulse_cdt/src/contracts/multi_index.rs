use crate::core::Name;

mod database_impl {
    unsafe extern "C" {
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
        pub unsafe fn db_upperbound_i64(code: u64, scope: u64, table: u64, id: u64) -> i32;

        #[link_name = "db_idx64_store"]
        pub unsafe fn db_idx64_store(
            scope: u64,
            table: u64,
            payer: u64,
            id: u64,
            secondary: *const u64,
        ) -> i32;

        #[link_name = "db_idx64_update"]
        pub unsafe fn db_idx64_update(
            iterator: i32,
            payer: u64,
            secondary: *const u64,
        );

        #[link_name = "db_idx64_remove"]
        pub unsafe fn db_idx64_remove(iterator: i32);

        #[link_name = "db_idx64_find_secondary"]
        pub unsafe fn db_idx64_find_secondary(
            code: u64,
            scope: u64,
            table: u64,
            secondary: *const u64,
            primary: *mut u64,
        ) -> i32;

        #[link_name = "db_idx64_find_primary"]
        pub unsafe fn db_idx64_find_primary(
            code: u64,
            scope: u64,
            table: u64,
            secondary: *mut u64,
            primary: u64,
        ) -> i32;

        #[link_name = "db_idx64_lowerbound"]
        pub unsafe fn db_idx64_lowerbound(
            code: u64,
            scope: u64,
            table: u64,
            secondary: *mut u64,
            primary: *mut u64,
        ) -> i32;

        #[link_name = "db_idx64_upperbound"]
        pub unsafe fn db_idx64_upperbound(
            code: u64,
            scope: u64,
            table: u64,
            secondary: *mut u64,
            primary: *mut u64,
        ) -> i32;

        #[link_name = "db_idx64_end"]
        pub unsafe fn db_idx64_end(code: u64, scope: u64, table: u64) -> i32;

        #[link_name = "db_idx64_next"]
        pub unsafe fn db_idx64_next(iterator: i32, primary: *mut u64) -> i32;

        #[link_name = "db_idx64_previous"]
        pub unsafe fn db_idx64_previous(iterator: i32, primary: *mut u64) -> i32;
    }
}

#[inline]
pub fn db_get_i64(iterator: i32, data: &[u8], len: u32) -> i32 {
    unsafe { database_impl::db_get_i64(iterator, data.as_ptr() as *const crate::c_void, len) }
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
pub fn db_update_i64(iterator: i32, payer: Name, data: &[u8], len: u32) {
    unsafe { database_impl::db_update_i64(iterator, payer.raw(), data.as_ptr() as *const crate::c_void, len) }
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

// Stores a secondary index of type `u64`.
#[inline]
pub fn db_idx64_store(
    scope: u64,
    table: Name,
    payer: Name,
    id: u64,
    secondary: &u64,
) -> i32 {
    unsafe {
        database_impl::db_idx64_store(
            scope,
            table.raw(),
            payer.raw(),
            id,
            secondary,
        )
    }
}

#[inline]
pub fn db_idx64_update(iterator: i32, payer: Name, secondary: &u64) {
    unsafe { database_impl::db_idx64_update(iterator, payer.raw(), secondary) }
}

#[inline]
pub fn db_idx64_remove(iterator: i32) {
    unsafe { database_impl::db_idx64_remove(iterator) }
}

#[inline]
pub fn db_idx64_find_secondary(
    code: Name,
    scope: u64,
    table: Name,
    secondary: &u64,
    primary: *mut u64,
) -> i32 {
    unsafe {
        database_impl::db_idx64_find_secondary(
            code.raw(),
            scope,
            table.raw(),
            secondary,
            primary,
        )
    }
}

#[inline]
pub fn db_idx64_find_primary(
    code: Name,
    scope: u64,
    table: Name,
    secondary: *mut u64,
    primary: u64,
) -> i32 {
    unsafe {
        database_impl::db_idx64_find_primary(
            code.raw(),
            scope,
            table.raw(),
            secondary,
            primary,
        )
    }
}

#[inline]
pub fn db_idx64_lowerbound(
    code: Name,
    scope: u64,
    table: Name,
    secondary: *mut u64,
    primary: *mut u64,
) -> i32 {
    unsafe {
        database_impl::db_idx64_lowerbound(
            code.raw(),
            scope,
            table.raw(),
            secondary,
            primary,
        )
    }
}

#[inline]
pub fn db_idx64_upperbound(
    code: Name,
    scope: u64,
    table: Name,
    secondary: *mut u64,
    primary: *mut u64,
) -> i32 {
    unsafe {
        database_impl::db_idx64_upperbound(
            code.raw(),
            scope,
            table.raw(),
            secondary,
            primary,
        )
    }
}

#[inline]
pub fn db_idx64_end(code: Name, scope: u64, table: Name) -> i32 {
    unsafe { database_impl::db_idx64_end(code.raw(), scope, table.raw()) }
}

#[inline]
pub fn db_idx64_next(iterator: i32, primary: *mut u64) -> i32 {
    unsafe { database_impl::db_idx64_next(iterator, primary) }
}

#[inline]
pub fn db_idx64_previous(iterator: i32, primary: *mut u64) -> i32 {
    unsafe { database_impl::db_idx64_previous(iterator, primary) }
}
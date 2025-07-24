#![no_std]
#![no_main]
extern crate alloc;

use pulse_cdt::{action, contracts::{db_find_i64, db_next_i64, db_previous_i64, db_remove_i64, db_store_i64, get_self}, core::check, dispatch, name};

#[action]
fn pg() {
    let receiver = get_self().as_u64();
    let table1 = name!("table1");

    let alice_itr = db_store_i64(receiver.into(), table1.into(), receiver.into(), name!("alice"), b"alice's info" , b"alice's info".len() as u32);
    db_store_i64(receiver.into(), table1.into(), receiver.into(), name!("bob"), b"bob's info" , b"bob's info".len() as u32);
    db_store_i64(receiver.into(), table1.into(), receiver.into(), name!("charlie"), b"charlie's info" , b"charlie's info".len() as u32);
    db_store_i64(receiver.into(), table1.into(), receiver.into(), name!("allyson"), b"allyson's info" , b"allyson's info".len() as u32);

    // find
    {
        let mut prim = 0u64;
        let mut itr_next = db_next_i64(alice_itr, &mut prim as *mut u64);
        let mut itr_next_expected = db_find_i64(receiver.into(), receiver.into(), table1.into(), name!("allyson"));
        check(itr_next == itr_next_expected && prim == name!("allyson"), "primary_i64_general - db_find_i64"  );
        itr_next = db_next_i64(itr_next, &mut prim as *mut u64);
        itr_next_expected = db_find_i64(receiver.into(), receiver.into(), table1.into(), name!("bob"));
        check(itr_next == itr_next_expected && prim == name!("bob"), "primary_i64_general - db_next_i64" );
    }

    // next
    {
        let charlie_itr = db_find_i64(receiver.into(), receiver.into(), table1.into(), name!("charlie"));
        let mut prim = 0u64;
        let end_itr = db_next_i64(charlie_itr, &mut prim as *mut u64);
        check(end_itr < 0, "primary_i64_general - db_next_i64" );
        // prim didn't change
        check(prim == 0, "primary_i64_general - db_next_i64" );
    }

    // previous
    {
        let charlie_itr = db_find_i64(receiver.into(), receiver.into(), table1.into(), name!("charlie"));
        let mut prim = 0u64;
        let mut itr_prev = db_previous_i64(charlie_itr, &mut prim as *mut u64);
        let mut itr_prev_expected = db_find_i64(receiver.into(), receiver.into(), table1.into(), name!("bob"));
        check(itr_prev == itr_prev_expected && prim == name!("bob"), "primary_i64_general - db_previous_i64" );

        itr_prev = db_previous_i64(itr_prev, &mut prim as *mut u64);
        itr_prev_expected = db_find_i64(receiver.into(), receiver.into(), table1.into(), name!("allyson"));
        check(itr_prev == itr_prev_expected && prim == name!("allyson"), "primary_i64_general - db_previous_i64" );

        itr_prev = db_previous_i64(itr_prev, &mut prim as *mut u64);
        itr_prev_expected = db_find_i64(receiver.into(), receiver.into(), table1.into(), name!("alice"));
        check(itr_prev == itr_prev_expected && prim == name!("alice"), "primary_i64_general - db_previous_i64" );

        itr_prev = db_previous_i64(itr_prev, &mut prim as *mut u64);
        check(itr_prev < 0 && prim == name!("alice"), "primary_i64_general - db_previous_i64" );
    }

    // remove
    {
        let mut itr = db_find_i64(receiver.into(), receiver.into(), table1.into(), name!("alice"));
        check(itr >= 0, "primary_i64_general - db_find_i64");
        db_remove_i64(itr);
        itr = db_find_i64(receiver.into(), receiver.into(), table1.into(), name!("alice"));
        check(itr < 0, "primary_i64_general - db_find_i64" );
    }
}

dispatch!(pg);
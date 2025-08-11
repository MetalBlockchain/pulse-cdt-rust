#![no_std]
#![no_main]
extern crate alloc;

use pulse_cdt::{
    action,
    contracts::{
        db_find_i64, db_lowerbound_i64, db_next_i64, db_previous_i64, db_remove_i64, db_store_i64,
        db_upperbound_i64, get_self,
    },
    core::check,
    dispatch, name, name_raw,
};

#[action]
fn pg() {
    let receiver = get_self().raw();
    let table1 = name!("table1");

    let alice_itr = db_store_i64(
        receiver.into(),
        table1.into(),
        receiver.into(),
        name_raw!("alice"),
        b"alice's info",
        b"alice's info".len() as u32,
    );
    db_store_i64(
        receiver.into(),
        table1.into(),
        receiver.into(),
        name_raw!("bob"),
        b"bob's info",
        b"bob's info".len() as u32,
    );
    db_store_i64(
        receiver.into(),
        table1.into(),
        receiver.into(),
        name_raw!("charlie"),
        b"charlie's info",
        b"charlie's info".len() as u32,
    );
    db_store_i64(
        receiver.into(),
        table1.into(),
        receiver.into(),
        name_raw!("allyson"),
        b"allyson's info",
        b"allyson's info".len() as u32,
    );

    // find
    {
        let mut prim = 5u64;
        let mut itr_next = db_next_i64(alice_itr, &mut prim as *mut u64);
        let mut itr_next_expected = db_find_i64(
            receiver.into(),
            receiver.into(),
            table1.into(),
            name_raw!("allyson"),
        );
        check(
            itr_next == itr_next_expected && prim == name_raw!("allyson"),
            "find: itr_next == itr_next_expected && prim == name!(\"allyson\")",
        );
        itr_next = db_next_i64(itr_next, &mut prim as *mut u64);
        itr_next_expected = db_find_i64(
            receiver.into(),
            receiver.into(),
            table1.into(),
            name_raw!("bob"),
        );
        check(
            itr_next == itr_next_expected && prim == name_raw!("bob"),
            "find: primary_i64_general - db_next_i64",
        );
    }

    // next
    {
        let charlie_itr = db_find_i64(
            receiver.into(),
            receiver.into(),
            table1.into(),
            name_raw!("charlie"),
        );
        let mut prim = 0u64;
        let end_itr = db_next_i64(charlie_itr, &mut prim as *mut u64);
        check(end_itr < 0, "next: primary_i64_general - db_next_i64");
        // prim didn't change
        check(prim == 0, "next: primary_i64_general - db_next_i64");
    }

    // previous
    {
        let charlie_itr = db_find_i64(
            receiver.into(),
            receiver.into(),
            table1.into(),
            name_raw!("charlie"),
        );
        let mut prim = 0u64;
        let mut itr_prev = db_previous_i64(charlie_itr, &mut prim as *mut u64);
        let mut itr_prev_expected = db_find_i64(
            receiver.into(),
            receiver.into(),
            table1.into(),
            name_raw!("bob"),
        );
        check(
            itr_prev == itr_prev_expected && prim == name_raw!("bob"),
            "previous: primary_i64_general - db_previous_i64 - bob",
        );

        itr_prev = db_previous_i64(itr_prev, &mut prim as *mut u64);
        itr_prev_expected = db_find_i64(
            receiver.into(),
            receiver.into(),
            table1.into(),
            name_raw!("allyson"),
        );
        check(
            itr_prev == itr_prev_expected && prim == name_raw!("allyson"),
            "previous: primary_i64_general - db_previous_i64 - allyson",
        );

        itr_prev = db_previous_i64(itr_prev, &mut prim as *mut u64);
        itr_prev_expected = db_find_i64(
            receiver.into(),
            receiver.into(),
            table1.into(),
            name_raw!("alice"),
        );
        check(
            itr_prev == itr_prev_expected && prim == name_raw!("alice"),
            "previous: primary_i64_general - db_previous_i64 - alice",
        );

        itr_prev = db_previous_i64(itr_prev, &mut prim as *mut u64);
        check(
            itr_prev < 0 && prim == name_raw!("alice"),
            "previous: primary_i64_general - db_previous_i64 - alice 2",
        );
    }

    // remove
    {
        let mut itr = db_find_i64(
            receiver.into(),
            receiver.into(),
            table1.into(),
            name_raw!("alice"),
        );
        check(itr >= 0, "remove: primary_i64_general - db_find_i64");
        db_remove_i64(itr);
        itr = db_find_i64(
            receiver.into(),
            receiver.into(),
            table1.into(),
            name_raw!("alice"),
        );
        check(itr < 0, "remove: primary_i64_general - db_find_i64");
    }
}

#[action]
fn pl() {
    let receiver = get_self().raw();
    let table = name_raw!("mytable");
    db_store_i64(
        receiver.into(),
        table.into(),
        receiver.into(),
        name_raw!("alice"),
        b"alice's info",
        b"alice's info".len() as u32,
    );
    db_store_i64(
        receiver.into(),
        table.into(),
        receiver.into(),
        name_raw!("bob"),
        b"bob's info",
        b"bob's info".len() as u32,
    );
    db_store_i64(
        receiver.into(),
        table.into(),
        receiver.into(),
        name_raw!("charlie"),
        b"charlie's info",
        b"charlie's info".len() as u32,
    );
    db_store_i64(
        receiver.into(),
        table.into(),
        receiver.into(),
        name_raw!("emily"),
        b"emily's info",
        b"emily's info".len() as u32,
    );
    db_store_i64(
        receiver.into(),
        table.into(),
        receiver.into(),
        name_raw!("allyson"),
        b"allyson's info",
        b"allyson's info".len() as u32,
    );
    db_store_i64(
        receiver.into(),
        table.into(),
        receiver.into(),
        name_raw!("joe"),
        b"nothing here",
        b"nothing here".len() as u32,
    );

    {
        let lb = db_lowerbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name_raw!("alice").into(),
        );
        check(
            lb == db_find_i64(
                receiver.into(),
                receiver.into(),
                table.into(),
                name_raw!("alice").into(),
            ),
            "lowerbound: primary_i64_general - db_lowerbound_i64 - alice",
        );
    }
    {
        let lb = db_lowerbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name_raw!("billy").into(),
        );
        check(
            lb == db_find_i64(
                receiver.into(),
                receiver.into(),
                table.into(),
                name_raw!("bob").into(),
            ),
            "lowerbound: primary_i64_general - db_lowerbound_i64 - bob",
        );
    }
    {
        let lb = db_lowerbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name!("frank").into(),
        );
        check(
            lb == db_find_i64(
                receiver.into(),
                receiver.into(),
                table.into(),
                name!("joe").into(),
            ),
            "lowerbound: primary_i64_general - db_lowerbound_i64 - joe",
        );
    }
    {
        let lb = db_lowerbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name!("joe").into(),
        );
        check(
            lb == db_find_i64(
                receiver.into(),
                receiver.into(),
                table.into(),
                name!("joe").into(),
            ),
            "lowerbound: primary_i64_general - db_lowerbound_i64 - joe 2",
        );
    }
    {
        let lb = db_lowerbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name!("kevin").into(),
        );
        check(
            lb < 0,
            "lowerbound: primary_i64_general - db_lowerbound_i64 - kevin",
        );
    }
}

#[action]
fn pu() {
    let receiver = get_self().raw();
    let table = name!("mytable");

    {
        let ub = db_upperbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name!("alice").into(),
        );
        check(
            ub == db_find_i64(
                receiver.into(),
                receiver.into(),
                table.into(),
                name!("allyson").into(),
            ),
            "upperbound: primary_i64_general - db_upperbound_i64 - allyson",
        );
    }
    {
        let ub = db_upperbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name!("billy").into(),
        );
        check(
            ub == db_find_i64(
                receiver.into(),
                receiver.into(),
                table.into(),
                name!("bob").into(),
            ),
            "upperbound: primary_i64_general - db_upperbound_i64 - bob",
        );
    }
    {
        let ub = db_upperbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name!("frank").into(),
        );
        check(
            ub == db_find_i64(
                receiver.into(),
                receiver.into(),
                table.into(),
                name!("joe").into(),
            ),
            "upperbound: primary_i64_general - db_upperbound_i64 - joe",
        );
    }
    {
        let ub = db_upperbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name!("joe").into(),
        );
        check(
            ub < 0,
            "upperbound: primary_i64_general - db_upperbound_i64 - joe",
        );
    }
    {
        let ub = db_upperbound_i64(
            receiver.into(),
            receiver.into(),
            table.into(),
            name!("kevin").into(),
        );
        check(
            ub < 0,
            "upperbound: primary_i64_general - db_upperbound_i64 - kevin",
        );
    }
}

dispatch!(pg, pl, pu);

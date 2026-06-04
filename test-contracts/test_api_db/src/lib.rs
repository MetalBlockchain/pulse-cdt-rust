#![no_std]
#![no_main]
extern crate alloc;

use alloc::format;
use pulse_cdt::{
    action, contract,
    contracts::{
        db_find_i64, db_get_i64, db_idx64_find_primary, db_idx64_find_secondary, db_idx64_lowerbound, db_idx64_next, db_idx64_previous, db_idx64_remove, db_idx64_store, db_idx64_update, db_idx64_upperbound, db_lowerbound_i64, db_next_i64, db_previous_i64, db_remove_i64, db_store_i64, db_update_i64, db_upperbound_i64
    },
    core::{Name, check},
    name, name_raw,
};

#[derive(Default)]
struct TestContract;

#[contract]
impl TestContract {
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

    #[action]
    fn s1g() {
        let table: Name = name!("myindextable");
        let receiver: Name = get_self();

        struct Record {
            ssn: u64,
            name: u64,
        }

        let records = [
            Record {
                ssn: 265,
                name: name_raw!("alice"),
            },
            Record {
                ssn: 781,
                name: name_raw!("bob"),
            },
            Record {
                ssn: 234,
                name: name_raw!("charlie"),
            },
            Record {
                ssn: 650,
                name: name_raw!("allyson"),
            },
            Record {
                ssn: 540,
                name: name_raw!("bob"),
            },
            Record {
                ssn: 976,
                name: name_raw!("emily"),
            },
            Record {
                ssn: 110,
                name: name_raw!("joe"),
            },
        ];

        for record in records {
            db_idx64_store(
                receiver.into(),
                table.into(),
                receiver.into(),
                record.ssn,
                &record.name,
            );
        }

        // find primary
        {
            let mut sec: u64 = 0;
            let mut itr = db_idx64_find_primary(receiver.into(), receiver.into(), table.into(), &mut sec, 999);
            check( itr < 0 && sec == 0, "1 idx64_general - db_idx64_find_primary" );
            itr = db_idx64_find_primary(receiver.into(), receiver.into(), table.into(), &mut sec, 110);
            check( itr >= 0 && sec == name!("joe").into(), "2 idx64_general - db_idx64_find_primary" );
            let mut prim_next: u64 = 0;
            let itr_next = db_idx64_next(itr, &mut prim_next);
            check( itr_next < 0 && prim_next == 0, "3 idx64_general - db_idx64_find_primary" );
        }

        // iterate forward starting with charlie
        {
            let mut sec: u64 = 0;
            let itr = db_idx64_find_primary( receiver.into(), receiver.into(), table.into(), &mut sec, 234 );
            check( itr >= 0 && sec == name!("charlie").into(), "4 idx64_general - db_idx64_find_primary" );

            let mut prim_next: u64 = 0;
            let mut itr_next = db_idx64_next( itr, &mut prim_next );
            check( itr_next >= 0 && prim_next == 976, "5 idx64_general - db_idx64_next" );
            let mut sec_next: u64 = 0;
            let mut itr_next_expected = db_idx64_find_primary( receiver.into(), receiver.into(), table.into(), &mut sec_next, prim_next );
            check( itr_next == itr_next_expected && sec_next == name!("emily").into(), "6 idx64_general - db_idx64_next" );

            itr_next = db_idx64_next( itr_next, &mut prim_next );
            check( itr_next >= 0 && prim_next == 110, "7 idx64_general - db_idx64_next" );
            itr_next_expected = db_idx64_find_primary( receiver.into(), receiver.into(), table.into(), &mut sec_next, prim_next );
            check( itr_next == itr_next_expected && sec_next == name!("joe").into(), "8 idx64_general - db_idx64_next" );

            itr_next = db_idx64_next( itr_next, &mut prim_next );
            check( itr_next < 0 && prim_next == 110, "9 idx64_general - db_idx64_next "  );
        }

        // iterate backward staring with second bob
        {
            let mut sec: u64 = 0;
            let itr = db_idx64_find_primary( receiver.into(), receiver.into(), table.into(), &mut sec, 781 );
            check( itr >= 0 && sec == name!("bob").into(), "10 idx64_general - db_idx64_find_primary" );

            let mut prim_prev: u64 = 0;
            let mut itr_prev = db_idx64_previous( itr, &mut prim_prev );
            check( itr_prev >= 0 && prim_prev == 540, "11 idx64_general - db_idx64_previous" );

            let mut sec_prev: u64 = 0;
            let mut itr_prev_expected = db_idx64_find_primary( receiver.into(), receiver.into(), table.into(), &mut sec_prev, prim_prev );
            check( itr_prev == itr_prev_expected && sec_prev == name!("bob").into(), "12 idx64_general - db_idx64_previous" );

            itr_prev = db_idx64_previous( itr_prev, &mut prim_prev );
            check( itr_prev >= 0 && prim_prev == 650, "13 idx64_general - db_idx64_previous" );
            itr_prev_expected = db_idx64_find_primary( receiver.into(), receiver.into(), table.into(), &mut sec_prev, prim_prev );
            check( itr_prev == itr_prev_expected && sec_prev == name!("allyson").into(), "14 idx64_general - db_idx64_previous" );

            itr_prev = db_idx64_previous( itr_prev, &mut prim_prev );
            check( itr_prev >= 0 && prim_prev == 265, "15 idx64_general - db_idx64_previous" );
            itr_prev_expected = db_idx64_find_primary( receiver.into(), receiver.into(), table.into(), &mut sec_prev, prim_prev );
            check( itr_prev == itr_prev_expected && sec_prev == name!("alice").into(), "16 idx64_general - db_idx64_previous" );

            itr_prev = db_idx64_previous( itr_prev, &mut prim_prev );
            check( itr_prev < 0 && prim_prev == 265, "17 idx64_general - db_idx64_previous" );
        }

        // find_secondary
        {
            let mut prim: u64 = 0;
            let mut sec = name!("bob").into();
            let mut itr = db_idx64_find_secondary( receiver.into(), receiver.into(), table.into(), &sec, &mut prim );
            check( itr >= 0 && prim == 540, "18 idx64_general - db_idx64_find_secondary" );

            sec = name!("emily").into();
            itr = db_idx64_find_secondary( receiver.into(), receiver.into(), table.into(), &sec, &mut prim );
            check( itr >= 0 && prim == 976, "19 idx64_general - db_idx64_find_secondary" );

            sec = name!("frank").into();
            itr = db_idx64_find_secondary( receiver.into(), receiver.into(), table.into(), &sec, &mut prim );
            check( itr < 0 && prim == 976, "20 idx64_general - db_idx64_find_secondary" );
        }

        // update and remove
        {
            let one_more_bob = name!("bob").into();
            let ssn = 421;
            let itr = db_idx64_store( receiver.into(), table.into(), receiver.into(), ssn, &one_more_bob );
            let new_name = name!("billy").into();
            db_idx64_update( itr, receiver.into(), &new_name );
            let mut sec = 0;
            let sec_itr = db_idx64_find_primary( receiver.into(), receiver.into(), table.into(), &mut sec, ssn );
            check( sec_itr == itr && sec == new_name, "21 idx64_general - db_idx64_update" );
            db_idx64_remove(itr);
            let itrf = db_idx64_find_primary( receiver.into(), receiver.into(), table.into(), &mut sec, ssn );
            check( itrf < 0, "22 idx64_general - db_idx64_remove" );
        }
    }

    #[action]
    fn s1l() {
        let table: Name = name!("myindextable");
        let receiver: Name = get_self();

        {
            let mut lb_sec = name!("alice").into();
            let mut lb_prim: u64 = 0;
            let ssn: u64 = 265;
            let lb = db_idx64_lowerbound( receiver.into(), receiver.into(), table.into(), &mut lb_sec, &mut lb_prim );
            check( lb_prim == ssn && lb_sec == name!("alice").into(), "1 idx64_lowerbound" );
            check( lb == db_idx64_find_primary(receiver.into(), receiver.into(), table.into(), &mut lb_sec, ssn), "2 idx64_lowerbound" );
        }

        {
            let mut lb_sec = name!("billy").into();
            let mut lb_prim: u64 = 0;
            let ssn: u64 = 540;
            let lb = db_idx64_lowerbound( receiver.into(), receiver.into(), table.into(), &mut lb_sec, &mut lb_prim );
            check( lb_prim == ssn && lb_sec == name!("bob").into(), "3 idx64_lowerbound" );
            check( lb == db_idx64_find_primary(receiver.into(), receiver.into(), table.into(), &mut lb_sec, ssn), "4 idx64_lowerbound" );
        }

        {
            let mut lb_sec = name!("joe").into();
            let mut lb_prim: u64 = 0;
            let ssn: u64 = 110;
            let lb = db_idx64_lowerbound( receiver.into(), receiver.into(), table.into(), &mut lb_sec, &mut lb_prim );
            check( lb_prim == ssn && lb_sec == name!("joe").into(), "5 idx64_lowerbound" );
            check( lb == db_idx64_find_primary(receiver.into(), receiver.into(), table.into(), &mut lb_sec, ssn), "6 idx64_lowerbound" );
        }

        {
            let mut lb_sec = name!("kevin").into();
            let mut lb_prim: u64 = 0;
            let lb = db_idx64_lowerbound( receiver.into(), receiver.into(), table.into(), &mut lb_sec, &mut lb_prim );
            check( lb_prim == 0 && lb_sec == name!("kevin").into(), "7 idx64_lowerbound" );
            check( lb < 0, "8 idx64_lowerbound" );
        }

        { // unaligned
            let sec_off: usize = 4;
            let prim_off: usize = 4;
            let mut lb_sec: u64 = name_raw!("alice");
            let mut lb_prim: u64 = 0;
            let ssn: u64 = 265;
            let mut buf = [0u8; 16];

            unsafe {
                let lb_sec_ptr = buf.as_mut_ptr().add(sec_off) as *mut u64;
                let lb_prim_ptr = buf.as_mut_ptr().add(prim_off) as *mut u64;

                // memcpy(&lb_sec) → write unaligned into buf
                core::ptr::write_unaligned(lb_sec_ptr, lb_sec);

                let _lb = db_idx64_lowerbound(
                    receiver.into(),
                    receiver.into(),
                    table.into(),
                    &mut *lb_sec_ptr,
                    &mut *lb_prim_ptr,
                );

                // memcpy back from buf
                lb_sec = core::ptr::read_unaligned(lb_sec_ptr);
                lb_prim = core::ptr::read_unaligned(lb_prim_ptr);
            }

            check(
                lb_prim == ssn && lb_sec != name_raw!("alice"),
                "idx64_general - db_idx64_lowerbound (unaligned)",
            );
        }
    }

    #[action]
    fn s1u() {
        let table: Name = name!("myindextable");
        let receiver: Name = get_self();

        {
            let mut ub_sec = name_raw!("alice");
            let mut ub_prim = 0;
            let allyson_ssn = 650;
            let ub = db_idx64_upperbound( receiver.into(), receiver.into(), table.into(), &mut ub_sec, &mut ub_prim );
            check( ub_prim == allyson_ssn && ub_sec == name_raw!("allyson"), "" );
            check( ub == db_idx64_find_primary(receiver.into(), receiver.into(), table.into(), &mut ub_sec, allyson_ssn), "idx64_upperbound" );
        }

        {
            let mut ub_sec = name_raw!("billy");
            let mut ub_prim = 0;
            let bob_ssn = 540;
            let ub = db_idx64_upperbound( receiver.into(), receiver.into(), table.into(), &mut ub_sec, &mut ub_prim );
            check( ub_prim == bob_ssn && ub_sec == name_raw!("bob"), "" );
            check( ub == db_idx64_find_primary(receiver.into(), receiver.into(), table.into(), &mut ub_sec, bob_ssn), "idx64_upperbound" );
        }

        {
            let mut ub_sec = name_raw!("joe");
            let mut ub_prim = 0;
            let ub = db_idx64_upperbound( receiver.into(), receiver.into(), table.into(), &mut ub_sec, &mut ub_prim );
            check( ub_prim == 0 && ub_sec == name_raw!("joe"), "" );
            check( ub < 0, "idx64_upperbound" );
        }

        {
            let mut ub_sec = name_raw!("kevin");
            let mut ub_prim = 0;
            let ub = db_idx64_upperbound( receiver.into(), receiver.into(), table.into(), &mut ub_sec, &mut ub_prim );
            check( ub_prim == 0 && ub_sec == name_raw!("kevin"), "" );
            check( ub < 0, "idx64_upperbound" );
        }
    }

    #[action]
    fn tia(code: Name, val: u64, index: u32, store: bool) {
        let scope = name_raw!("access");
        let table = scope;
        let mut itr = -1;
        let mut value: u64 = 0;
        let pk = scope;
        let receiver: Name = get_self();

        match index {
            1 => {
                itr = db_idx64_find_primary(code.into(), scope, table.into(), &mut value, pk);
            }
            _ => {
                itr = db_find_i64(code, scope, table.into(), pk);
            }
        }

        if store {
            let value_to_store = val;

            if itr < 0 {
                match index {
                    1 => {
                        db_idx64_store(scope, table.into(), receiver.into(), pk, &value_to_store);
                    }
                    _ => {
                        db_store_i64(scope, table.into(), receiver.into(), pk, &value_to_store.to_le_bytes(), 8);
                    }
                }
            } else {
                match index {
                    1 => {
                        db_idx64_update(itr, receiver.into(), &value_to_store);
                    }
                    _ => {
                        db_update_i64(itr, receiver.into(), &value_to_store.to_le_bytes(), 8);
                    }
                }
            }
        } else {
            check( itr >= 0, "test_invalid_access: could not find row" );

            match index {
                1 => {}
                _ => {
                    let bytes = value.to_le_bytes();
                    check( db_get_i64( itr, &bytes, 8 ) == 8, "test_invalid_access: value in primary table was incorrect size" );
                    value = u64::from_le_bytes(bytes);
                }
            }

            check( value == val, "test_invalid_access: value did not match" );
        }
    }
}

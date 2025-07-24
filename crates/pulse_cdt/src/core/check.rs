mod assert_impl {
    extern "C" {
        #[link_name = "pulse_assert"]
        pub fn pulse_assert(test: u32, msg: *const crate::c_char, msg_len: usize);
    }
}

#[inline]
pub fn check(pred: bool, msg: &str) {
    if !pred {
        let msg_ptr = msg.as_ptr() as *const i8;
        unsafe { assert_impl::pulse_assert(0, msg_ptr, msg.len()) }
    }
}

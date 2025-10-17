use crate::core::{BlockTimestamp, Microseconds, TimePoint};

mod system_impl {
    extern "C" {
        #[link_name = "current_time"]
        pub fn current_time() -> u64;
    }
}

#[inline]
pub fn current_time() -> u64 {
    unsafe { system_impl::current_time() }
}

#[inline]
pub fn current_time_point() -> TimePoint {
    TimePoint::new(Microseconds::new(current_time() as i64))
}

#[inline]
pub fn current_block_time() -> BlockTimestamp {
    BlockTimestamp::from(current_time_point())
}
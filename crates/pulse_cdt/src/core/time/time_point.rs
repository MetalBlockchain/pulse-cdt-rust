use core::ops::{Add, AddAssign, Sub, SubAssign};

use pulse_proc_macro::{NumBytes, Read, Write};

use crate::core::Microseconds;

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Read, Write, NumBytes,
)]
#[pulse(crate_path = "pulse_serialization")]
pub struct TimePoint {
    pub elapsed: Microseconds, // microseconds since UNIX epoch
}

impl TimePoint {
    #[inline]
    pub const fn new(elapsed: Microseconds) -> Self {
        Self { elapsed }
    }

    #[inline]
    pub const fn time_since_epoch(&self) -> Microseconds {
        self.elapsed
    }

    #[inline]
    pub const fn sec_since_epoch(&self) -> u32 {
        (self.elapsed.count() / 1_000_000) as u32
    }
}

/* ---- arithmetic/relations (match C++ semantics) ---- */

impl Add<Microseconds> for TimePoint {
    type Output = TimePoint;
    #[inline]
    fn add(self, rhs: Microseconds) -> Self::Output {
        TimePoint::new(self.elapsed + rhs)
    }
}

impl Add for TimePoint {
    type Output = TimePoint;
    #[inline]
    fn add(self, rhs: TimePoint) -> Self::Output {
        TimePoint::new(self.elapsed + rhs.elapsed)
    }
}

impl Sub<Microseconds> for TimePoint {
    type Output = TimePoint;
    #[inline]
    fn sub(self, rhs: Microseconds) -> Self::Output {
        TimePoint::new(self.elapsed - rhs)
    }
}

impl Sub for TimePoint {
    type Output = Microseconds;
    #[inline]
    fn sub(self, rhs: TimePoint) -> Self::Output {
        self.elapsed - rhs.elapsed
    }
}

impl AddAssign<Microseconds> for TimePoint {
    #[inline]
    fn add_assign(&mut self, rhs: Microseconds) {
        self.elapsed += rhs;
    }
}

impl SubAssign<Microseconds> for TimePoint {
    #[inline]
    fn sub_assign(&mut self, rhs: Microseconds) {
        self.elapsed -= rhs;
    }
}

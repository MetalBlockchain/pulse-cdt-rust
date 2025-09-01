use core::ops::{Add, AddAssign, Sub, SubAssign};

use pulse_proc_macro::{NumBytes, Read, Write};

use crate::core::{seconds, Microseconds, TimePoint};

/// Seconds since UNIX epoch (UTC), second precision.
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Read, Write, NumBytes,
)]
#[pulse(crate_path = "pulse_serialization")]
pub struct TimePointSec {
    pub utc_seconds: u32,
}

impl TimePointSec {
    #[inline]
    pub const fn new(seconds: u32) -> Self {
        Self {
            utc_seconds: seconds,
        }
    }

    #[inline]
    pub const fn maximum() -> Self {
        Self {
            utc_seconds: u32::MAX,
        }
    }

    #[inline]
    pub const fn min() -> Self {
        Self { utc_seconds: 0 }
    }

    #[inline]
    pub const fn sec_since_epoch(self) -> u32 {
        self.utc_seconds
    }
}

/* ----- conversions to/from TimePoint (microsecond precision) ----- */

impl From<TimePoint> for TimePointSec {
    #[inline]
    fn from(t: TimePoint) -> Self {
        // Truncate microseconds to whole seconds
        let secs = (t.elapsed.count() / 1_000_000) as i64;
        Self {
            utc_seconds: secs as u32,
        } // C++ semantics: wrap on cast if negative/large
    }
}

impl From<TimePointSec> for TimePoint {
    #[inline]
    fn from(t: TimePointSec) -> Self {
        TimePoint::new(seconds(t.utc_seconds as i64))
    }
}

/* ----- comparisons are via derives ----- */

/* ----- += / -= with u32, Microseconds, TimePointSec (wrap like C++) ----- */

impl AddAssign<u32> for TimePointSec {
    #[inline]
    fn add_assign(&mut self, rhs: u32) {
        self.utc_seconds = self.utc_seconds.wrapping_add(rhs);
    }
}
impl AddAssign<Microseconds> for TimePointSec {
    #[inline]
    fn add_assign(&mut self, rhs: Microseconds) {
        let secs = rhs.to_seconds() as u32;
        self.utc_seconds = self.utc_seconds.wrapping_add(secs);
    }
}
impl AddAssign<TimePointSec> for TimePointSec {
    #[inline]
    fn add_assign(&mut self, rhs: TimePointSec) {
        self.utc_seconds = self.utc_seconds.wrapping_add(rhs.utc_seconds);
    }
}

impl SubAssign<u32> for TimePointSec {
    #[inline]
    fn sub_assign(&mut self, rhs: u32) {
        self.utc_seconds = self.utc_seconds.wrapping_sub(rhs);
    }
}
impl SubAssign<Microseconds> for TimePointSec {
    #[inline]
    fn sub_assign(&mut self, rhs: Microseconds) {
        let secs = rhs.to_seconds() as u32;
        self.utc_seconds = self.utc_seconds.wrapping_sub(secs);
    }
}
impl SubAssign<TimePointSec> for TimePointSec {
    #[inline]
    fn sub_assign(&mut self, rhs: TimePointSec) {
        self.utc_seconds = self.utc_seconds.wrapping_sub(rhs.utc_seconds);
    }
}

/* ----- + / - with u32 returning TimePointSec (wrap like C++) ----- */

impl Add<u32> for TimePointSec {
    type Output = TimePointSec;
    #[inline]
    fn add(self, rhs: u32) -> Self::Output {
        TimePointSec::new(self.utc_seconds.wrapping_add(rhs))
    }
}
impl Sub<u32> for TimePointSec {
    type Output = TimePointSec;
    #[inline]
    fn sub(self, rhs: u32) -> Self::Output {
        TimePointSec::new(self.utc_seconds.wrapping_sub(rhs))
    }
}

/* ----- mixed ops with Microseconds / TimePoint ----- */

/// time_point_sec + microseconds -> time_point
impl Add<Microseconds> for TimePointSec {
    type Output = TimePoint;
    #[inline]
    fn add(self, rhs: Microseconds) -> Self::Output {
        TimePoint::from(self) + rhs
    }
}
/// time_point_sec - microseconds -> time_point
impl Sub<Microseconds> for TimePointSec {
    type Output = TimePoint;
    #[inline]
    fn sub(self, rhs: Microseconds) -> Self::Output {
        TimePoint::from(self) - rhs
    }
}
/// time_point_sec - time_point_sec -> microseconds
impl Sub for TimePointSec {
    type Output = Microseconds;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        TimePoint::from(self) - TimePoint::from(rhs)
    }
}

/// time_point - time_point_sec -> microseconds (for convenience)
impl Sub<TimePointSec> for TimePoint {
    type Output = Microseconds;
    #[inline]
    fn sub(self, rhs: TimePointSec) -> Self::Output {
        self - TimePoint::from(rhs)
    }
}

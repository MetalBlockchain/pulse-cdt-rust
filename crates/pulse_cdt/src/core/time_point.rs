use pulse_proc_macro::{NumBytes, Read, Write};

use crate::core::TimePointSec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Read, Write, NumBytes)]
#[pulse(crate_path = "pulse_serialization")]
pub struct TimePoint(i64);

impl TimePoint {
    #[inline]
    #[must_use]
    pub const fn from_micros(micros: i64) -> Self {
        Self(micros)
    }

    #[inline]
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self::from_micros(millis * 1_000)
    }

    #[inline]
    #[must_use]
    pub const fn as_micros(&self) -> i64 {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn as_millis(&self) -> i64 {
        self.0 / 1_000
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn as_secs(&self) -> i32 {
        (self.0 / 1_000_000) as i32
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub const fn as_time_point_sec(&self) -> TimePointSec {
        TimePointSec::from_secs(self.as_secs() as u32)
    }
}

impl From<i64> for TimePoint {
    #[inline]
    fn from(i: i64) -> Self {
        Self(i)
    }
}

impl From<TimePoint> for i64 {
    #[inline]
    fn from(t: TimePoint) -> Self {
        t.0
    }
}

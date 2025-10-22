use pulse_proc_macro::{NumBytes, Read, Write};

use crate::core::{milliseconds, TimePoint, TimePointSec};

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Read, Write, NumBytes,
)]
#[pulse(crate_path = "pulse_serialization")]
pub struct BlockTimestamp {
    pub slot: u32,
}

impl BlockTimestamp {
    pub const BLOCK_INTERVAL_MS: i32 = 500;
    pub const BLOCK_TIMESTAMP_EPOCH: i64 = 946_684_800_000; // 2000-01-01T00:00:00Z

    #[inline]
    pub const fn new(slot: u32) -> Self {
        Self { slot }
    }

    #[inline]
    pub const fn maximum() -> Self {
        Self { slot: 0xFFFF }
    }
    #[inline]
    pub const fn min() -> Self {
        Self { slot: 0 }
    }

    #[inline]
    pub fn next(self) -> Self {
        assert!(u32::MAX - self.slot >= 1, "block timestamp overflow");
        Self {
            slot: self.slot + 1,
        }
    }

    #[inline]
    pub fn to_time_point(self) -> TimePoint {
        self.into()
    }
}

impl From<BlockTimestamp> for TimePoint {
    #[inline]
    fn from(bt: BlockTimestamp) -> Self {
        let msec = (bt.slot as i64) * (BlockTimestamp::BLOCK_INTERVAL_MS as i64)
            + BlockTimestamp::BLOCK_TIMESTAMP_EPOCH;
        TimePoint::new(milliseconds(msec))
    }
}

impl From<TimePoint> for BlockTimestamp {
    #[inline]
    fn from(t: TimePoint) -> Self {
        let micro = t.time_since_epoch().count();
        let msec = micro / 1_000;
        let slot = ((msec - BlockTimestamp::BLOCK_TIMESTAMP_EPOCH)
            / (BlockTimestamp::BLOCK_INTERVAL_MS as i64)) as u32;
        BlockTimestamp { slot }
    }
}

impl From<TimePointSec> for BlockTimestamp {
    #[inline]
    fn from(t: TimePointSec) -> Self {
        let sec = t.sec_since_epoch() as i64;
        let slot = ((sec * 1_000 - BlockTimestamp::BLOCK_TIMESTAMP_EPOCH)
            / (BlockTimestamp::BLOCK_INTERVAL_MS as i64)) as u32;
        BlockTimestamp { slot }
    }
}

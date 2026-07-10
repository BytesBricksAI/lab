//! Half-open time intervals for sample batches.

use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};

use crate::error::{KernelError, Result};

pub(crate) mod timestamp_serde {
    use std::str::FromStr as _;

    use jiff::Timestamp;
    use serde::Deserialize as _;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(ts: &Timestamp, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(ts)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Timestamp, D::Error> {
        let value = String::deserialize(deserializer)?;
        Timestamp::from_str(&value).map_err(serde::de::Error::custom)
    }
}

/// A half-open time window `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeWindow {
    #[serde(with = "timestamp_serde")]
    start: Timestamp,
    #[serde(with = "timestamp_serde")]
    end: Timestamp,
}

impl TimeWindow {
    /// Creates a window, rejecting `start >= end`.
    pub fn new(start: Timestamp, end: Timestamp) -> Result<Self> {
        if start >= end {
            return Err(KernelError::InvalidTimeWindow);
        }
        Ok(Self { start, end })
    }

    /// Inclusive start timestamp.
    pub fn start(&self) -> Timestamp {
        self.start
    }

    /// Exclusive end timestamp.
    pub fn end(&self) -> Timestamp {
        self.end
    }

    /// Returns `true` if `ts` lies in `[start, end)`.
    pub fn contains(&self, ts: Timestamp) -> bool {
        self.start <= ts && ts < self.end
    }

    /// Returns `true` if this window overlaps `other` (intersection is non-empty).
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Duration from start to end.
    pub fn duration(&self) -> SignedDuration {
        self.end.duration_since(self.start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(seconds: i64) -> Timestamp {
        Timestamp::from_second(seconds).unwrap()
    }

    #[test]
    fn rejects_invalid_window() {
        let t = ts(100);
        assert_eq!(
            TimeWindow::new(t, t).unwrap_err(),
            KernelError::InvalidTimeWindow
        );
        assert_eq!(
            TimeWindow::new(ts(200), ts(100)).unwrap_err(),
            KernelError::InvalidTimeWindow,
        );
    }

    #[test]
    fn contains_half_open() {
        let window = TimeWindow::new(ts(100), ts(200)).unwrap();
        assert!(window.contains(ts(100)));
        assert!(window.contains(ts(150)));
        assert!(!window.contains(ts(200)));
        assert!(!window.contains(ts(99)));
    }

    #[test]
    fn overlaps() {
        let a = TimeWindow::new(ts(100), ts(200)).unwrap();
        let b = TimeWindow::new(ts(150), ts(250)).unwrap();
        let c = TimeWindow::new(ts(200), ts(300)).unwrap();

        assert!(a.overlaps(&b));
        assert!(!a.overlaps(&c));
    }
}

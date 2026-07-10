//! Scalar measurements and tagged sample batches.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::quality::Quality;
use crate::tag_id::TagId;
use crate::time_window::TimeWindow;

/// A single scalar sample with quality and timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    value: f64,
    quality: Quality,
    #[serde(with = "crate::time_window::timestamp_serde")]
    timestamp: Timestamp,
}

impl Measurement {
    /// Creates a measurement sample.
    pub fn new(value: f64, quality: Quality, timestamp: Timestamp) -> Self {
        Self {
            value,
            quality,
            timestamp,
        }
    }

    /// Sample value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Sample quality.
    pub fn quality(&self) -> Quality {
        self.quality
    }

    /// Sample timestamp.
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }
}

/// A time-ordered batch of measurements for one process tag.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementBatch {
    tag: TagId,
    samples: Vec<Measurement>,
}

impl MeasurementBatch {
    /// Creates a batch for `tag` with the given `samples`.
    pub fn new(tag: TagId, samples: Vec<Measurement>) -> Self {
        Self { tag, samples }
    }

    /// Tag identifier for this batch.
    pub fn tag(&self) -> &TagId {
        &self.tag
    }

    /// Samples in this batch.
    pub fn samples(&self) -> &[Measurement] {
        &self.samples
    }

    /// Number of samples in the batch.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns `true` if the batch contains no samples.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Returns the half-open window `[min_ts, max_ts)` across samples.
    ///
    /// Returns `None` when the batch is empty or when all samples share the same
    /// timestamp, because [`TimeWindow`] requires `start < end`.
    pub fn time_span(&self) -> Option<TimeWindow> {
        if self.samples.is_empty() {
            return None;
        }

        let mut min_ts = self.samples[0].timestamp();
        let mut max_ts = min_ts;

        for sample in &self.samples[1..] {
            let ts = sample.timestamp();
            if ts < min_ts {
                min_ts = ts;
            }
            if ts > max_ts {
                max_ts = ts;
            }
        }

        if min_ts >= max_ts {
            return None;
        }

        TimeWindow::new(min_ts, max_ts).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(seconds: i64) -> Timestamp {
        Timestamp::from_second(seconds).unwrap()
    }

    fn sample(value: f64, at: i64) -> Measurement {
        Measurement::new(value, Quality::Good, ts(at))
    }

    #[test]
    fn time_span_for_multiple_samples() {
        let tag = TagId::new("PT-1101A").unwrap();
        let batch = MeasurementBatch::new(
            tag,
            vec![sample(1.0, 100), sample(2.0, 150), sample(3.0, 200)],
        );

        let span = batch.time_span().unwrap();
        assert_eq!(span.start(), ts(100));
        assert_eq!(span.end(), ts(200));
    }

    #[test]
    fn time_span_none_for_empty_batch() {
        let tag = TagId::new("PT-1101A").unwrap();
        let batch = MeasurementBatch::new(tag, vec![]);
        assert!(batch.time_span().is_none());
    }

    #[test]
    fn time_span_none_for_single_timestamp() {
        let tag = TagId::new("PT-1101A").unwrap();
        let batch = MeasurementBatch::new(tag, vec![sample(1.0, 100), sample(2.0, 100)]);
        assert!(batch.time_span().is_none());
    }
}

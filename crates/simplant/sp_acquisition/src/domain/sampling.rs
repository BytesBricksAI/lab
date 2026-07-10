//! Sampling policy value object.

use serde::{Deserialize, Serialize};

/// Controls how often samples are polled and when a change is significant enough to record.
///
/// `deadband` is the minimum absolute change required before a new sample is worth
/// re-recording. Filtering logic is applied in infrastructure adapters, not here.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SamplingPolicy {
    period_ms: u64,
    deadband: Option<f64>,
}

impl SamplingPolicy {
    /// Creates a sampling policy with the given poll period and optional deadband.
    pub fn new(period_ms: u64, deadband: Option<f64>) -> Self {
        Self {
            period_ms,
            deadband,
        }
    }

    /// Poll period in milliseconds.
    pub fn period_ms(&self) -> u64 {
        self.period_ms
    }

    /// Minimum significant change for re-recording, if set.
    pub fn deadband(&self) -> Option<f64> {
        self.deadband
    }
}

impl Default for SamplingPolicy {
    fn default() -> Self {
        Self::new(1000, None)
    }
}

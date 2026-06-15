//! Acceptance criteria and measured outcomes.

use serde::{Deserialize, Serialize};

/// A single acceptance criterion: `metric` passes when measured value is ≤ `max_value`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    metric: String,
    max_value: f64,
}

impl AcceptanceCriterion {
    /// Creates a criterion for `metric` with upper bound `max_value`.
    pub fn new(metric: impl Into<String>, max_value: f64) -> Self {
        Self {
            metric: metric.into(),
            max_value,
        }
    }

    /// Metric name.
    pub fn metric(&self) -> &str {
        &self.metric
    }

    /// Maximum acceptable measured value.
    pub fn max_value(&self) -> f64 {
        self.max_value
    }
}

/// A measured outcome for a single metric after a stress run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasuredOutcome {
    metric: String,
    value: f64,
}

impl MeasuredOutcome {
    /// Creates an outcome for `metric` with measured `value`.
    pub fn new(metric: impl Into<String>, value: f64) -> Self {
        Self {
            metric: metric.into(),
            value,
        }
    }

    /// Metric name.
    pub fn metric(&self) -> &str {
        &self.metric
    }

    /// Measured value.
    pub fn value(&self) -> f64 {
        self.value
    }
}

//! Domain events for stress test lifecycle.

use serde::{Deserialize, Serialize};

/// Emitted when a stress test is successfully planned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StressTestPlanned {
    /// Stress test identifier.
    pub test: String,
    /// Number of load points in the profile.
    pub point_count: usize,
}

/// Emitted when a stress test evaluation completes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StressTestCompleted {
    /// Stress test identifier.
    pub test: String,
    /// `true` when all acceptance criteria were met.
    pub passed: bool,
}

/// Stress testing domain events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StressEvent {
    /// Test planned and ready for execution.
    Planned(StressTestPlanned),
    /// Test evaluated against measured outcomes.
    Completed(StressTestCompleted),
}

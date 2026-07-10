//! Control-plane domain events for acquisition sessions.

use serde::{Deserialize, Serialize};

/// Emitted when an acquisition session enters the running state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionStarted {
    /// Session identifier.
    pub session: String,

    /// Number of bound tags.
    pub tag_count: usize,
}

/// Emitted when an acquisition session stops normally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionStopped {
    /// Session identifier.
    pub session: String,

    /// Total measurement batches recorded during the session.
    pub batches_recorded: u64,
}

/// Emitted when the data source is lost unexpectedly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLost {
    /// Session identifier.
    pub session: String,

    /// Human-readable reason for source loss.
    pub reason: String,
}

/// Control-plane events for acquisition sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquisitionEvent {
    /// Session started.
    Started(AcquisitionStarted),

    /// Session stopped.
    Stopped(AcquisitionStopped),

    /// Data source lost.
    SourceLost(SourceLost),
}

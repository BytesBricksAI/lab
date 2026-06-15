//! Domain events for dataset lifecycle.

use serde::{Deserialize, Serialize};

/// Emitted when a new dataset specification version is published.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetPublished {
    /// Dataset identifier.
    pub dataset: String,
    /// Published version number.
    pub version: u32,
}

/// Domain events for the ML data loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatasetEvent {
    /// A dataset specification was published.
    Published(DatasetPublished),
}

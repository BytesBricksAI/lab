//! Feature and target column specifications.

use serde::{Deserialize, Serialize};
use sp_kernel::TagId;

use crate::domain::error::{DatasetError, Result};

/// A single feature or target column bound to a process tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureSpec {
    tag: TagId,
    name: String,
}

impl FeatureSpec {
    /// Creates a feature spec with a non-empty trimmed name.
    pub fn new(tag: TagId, name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(DatasetError::EmptyName("feature"));
        }
        Ok(Self { tag, name })
    }

    /// Process tag identifier for this column.
    pub fn tag(&self) -> &TagId {
        &self.tag
    }

    /// Human-readable column name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

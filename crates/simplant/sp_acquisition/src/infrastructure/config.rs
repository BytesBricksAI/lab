//! Acquisition profile TOML configuration.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sp_kernel::TagId;

use crate::domain::binding::TagBinding;
use crate::domain::error::{AcquisitionError, Result};
use crate::domain::sampling::SamplingPolicy;

/// Tag binding entry in an acquisition profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingConfig {
    /// Catalog tag identifier.
    pub tag: String,

    /// Physical address in the data source.
    pub address: String,
}

/// TOML acquisition profile for a replay or live session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcquisitionProfile {
    /// Session identifier.
    pub session_id: String,

    /// Path to the data file (CSV replay in F1).
    pub source_path: String,

    /// Tag bindings.
    pub bindings: Vec<BindingConfig>,

    /// Poll period in milliseconds.
    pub period_ms: u64,

    /// Optional deadband for significant-change filtering.
    pub deadband: Option<f64>,
}

impl AcquisitionProfile {
    /// Parses a profile from a TOML string.
    pub fn from_toml_str(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(|err| AcquisitionError::Config(err.to_string()))
    }

    /// Loads a profile from a file path.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(path.as_ref())
            .map_err(|err| AcquisitionError::Config(err.to_string()))?;
        Self::from_toml_str(&contents)
    }

    /// Builds validated tag bindings from this profile.
    pub fn to_bindings(&self) -> Result<Vec<TagBinding>> {
        self.bindings
            .iter()
            .map(|b| {
                let tag =
                    TagId::new(&b.tag).map_err(|err| AcquisitionError::Config(err.to_string()))?;
                TagBinding::new(tag, &b.address)
            })
            .collect()
    }

    /// Returns the sampling policy for this profile.
    pub fn policy(&self) -> SamplingPolicy {
        SamplingPolicy::new(self.period_ms, self.deadband)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = r#"
session_id = "replay-01"
source_path = "/data/replay.csv"
period_ms = 500
deadband = 0.1

[[bindings]]
tag = "PT-1101"
address = "pressure_col"

[[bindings]]
tag = "TT-1102"
address = "temp_col"
"#;

    #[test]
    fn from_toml_str_parses_profile() {
        let profile = AcquisitionProfile::from_toml_str(SAMPLE_TOML).unwrap();
        assert_eq!(profile.session_id, "replay-01");
        assert_eq!(profile.source_path, "/data/replay.csv");
        assert_eq!(profile.period_ms, 500);
        assert_eq!(profile.deadband, Some(0.1));
        assert_eq!(profile.bindings.len(), 2);
    }

    #[test]
    fn to_bindings_produces_valid_bindings() {
        let profile = AcquisitionProfile::from_toml_str(SAMPLE_TOML).unwrap();
        let bindings = profile.to_bindings().unwrap();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].tag().as_str(), "PT-1101");
        assert_eq!(bindings[0].address(), "pressure_col");
        assert_eq!(bindings[1].tag().as_str(), "TT-1102");
        assert_eq!(bindings[1].address(), "temp_col");
    }
}

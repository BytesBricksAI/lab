//! Validated identifier newtypes for process simulation.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::domain::error::{Result, SimulationError};

macro_rules! define_id {
    ($name:ident, $label:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            /// Creates an identifier from `raw`, trimming whitespace and rejecting empties.
            pub fn new(raw: impl Into<String>) -> Result<Self> {
                let trimmed = raw.into().trim().to_owned();
                if trimmed.is_empty() {
                    return Err(SimulationError::EmptyId($label));
                }
                Ok(Self(trimmed))
            }

            /// Returns the identifier as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

define_id!(FlowsheetId, "flowsheet id");
define_id!(UnitOpId, "unit operation id");
define_id!(StreamId, "stream id");
define_id!(ScenarioId, "scenario id");
define_id!(RunId, "run id");
define_id!(RecordingId, "recording id");

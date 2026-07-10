//! OPC UA-style data quality semantics.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Data quality indicator aligned with OPC UA semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quality {
    /// Value is known good.
    Good,

    /// Value is uncertain.
    Uncertain,

    /// Value is bad and should not be used for control decisions.
    Bad,
}

impl Quality {
    /// Returns `true` only when the sample is known good.
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Good)
    }
}

impl fmt::Display for Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Good => f.write_str("Good"),
            Self::Uncertain => f.write_str("Uncertain"),
            Self::Bad => f.write_str("Bad"),
        }
    }
}

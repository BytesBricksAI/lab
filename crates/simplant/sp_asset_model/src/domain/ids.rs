//! Validated identifier newtypes for the asset model.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::domain::error::{AssetError, Result};

macro_rules! define_id {
    ($name:ident, $label:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            /// Creates an identifier from `raw`, trimming whitespace and rejecting empties.
            pub fn new(raw: impl Into<String>) -> Result<Self> {
                let trimmed = raw.into().trim().to_owned();
                if trimmed.is_empty() {
                    return Err(AssetError::EmptyId($label));
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

        impl FromStr for $name {
            type Err = AssetError;

            fn from_str(s: &str) -> Result<Self> {
                Self::new(s)
            }
        }
    };
}

define_id!(FacilityId, "facility id");
define_id!(AreaId, "area id");
define_id!(UnitId, "unit id");
define_id!(EquipmentId, "equipment id");

//! Tag binding value object.

use serde::{Deserialize, Serialize};
use sp_kernel::TagId;

use crate::domain::error::{AcquisitionError, Result};

/// Maps a catalog tag to its physical address in a data source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagBinding {
    tag: TagId,
    address: String,
}

impl TagBinding {
    /// Creates a binding, rejecting empty addresses (after trim).
    pub fn new(tag: TagId, address: impl Into<String>) -> Result<Self> {
        let address = address.into();
        if address.trim().is_empty() {
            return Err(AcquisitionError::EmptyAddress);
        }
        Ok(Self { tag, address })
    }

    /// Catalog tag identifier.
    pub fn tag(&self) -> &TagId {
        &self.tag
    }

    /// Physical address in the data source (e.g. CSV column or OPC UA node id).
    pub fn address(&self) -> &str {
        &self.address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_empty_address() {
        let tag = TagId::new("PT-1101").unwrap();
        let err = TagBinding::new(tag, "   ").unwrap_err();
        assert_eq!(err, AcquisitionError::EmptyAddress);
    }
}

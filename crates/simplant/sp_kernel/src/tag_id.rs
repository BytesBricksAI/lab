//! ISA-style process tag identifiers.

use std::fmt;
use std::str::FromStr;

use crate::error::{KernelError, Result};

/// A validated process tag identifier (ISA-5.1 / hierarchical naming).
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct TagId(String);

impl TagId {
    /// Creates a tag identifier from `raw`, trimming whitespace and validating characters.
    pub fn new(raw: impl Into<String>) -> Result<Self> {
        let trimmed = raw.into().trim().to_owned();
        if trimmed.is_empty() {
            return Err(KernelError::EmptyTagId);
        }
        if !trimmed.chars().all(is_valid_tag_char) {
            return Err(KernelError::InvalidTagId(trimmed));
        }
        Ok(Self(trimmed))
    }

    /// Returns the tag identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for TagId {
    type Err = KernelError;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

fn is_valid_tag_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | ':')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_typical_tag_ids() {
        assert!(TagId::new("PT-1101A").is_ok());
        assert!(TagId::new("/site/topping/t-101/pt-1101a").is_ok());
    }

    #[test]
    fn rejects_empty_and_whitespace() {
        assert_eq!(TagId::new(""), Err(KernelError::EmptyTagId));
        assert_eq!(TagId::new("  "), Err(KernelError::EmptyTagId));
    }

    #[test]
    fn rejects_invalid_characters() {
        let err = TagId::new("PT 1101").unwrap_err();
        assert!(matches!(err, KernelError::InvalidTagId(_)));
    }
}

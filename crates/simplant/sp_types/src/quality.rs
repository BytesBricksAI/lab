//! Custom `simplant.components.Quality` component (OPC UA-style, encoded as `u8`).
//!
//! Serialized to the store as [`re_sdk_types::components::Text`] (`"Bad"` / `"Uncertain"` / `"Good"`)
//! because `re_byte_size::SizeBytes` is not accessible outside the Rerun crate graph, which prevents
//! implementing a custom `Loggable` component in downstream crates.

use re_sdk_types::components;

use crate::namespace::COMPONENT_QUALITY;

/// OPC UA-style quality code (`Bad = 0`, `Uncertain = 1`, `Good = 2`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quality(pub u8);

impl Quality {
    /// Returns the underlying `u8` encoding.
    #[inline]
    pub fn to_u8(self) -> u8 {
        self.0
    }

    /// Returns the conventional quality label for store serialization.
    #[inline]
    pub fn as_str(self) -> &'static str {
        match self.0 {
            0 => "Bad",
            1 => "Uncertain",
            _ => "Good",
        }
    }

    /// Converts this quality to a builtin Rerun [`components::Text`] batch element.
    #[inline]
    pub fn to_text(self) -> components::Text {
        components::Text::from(self.as_str())
    }

    /// Fully-qualified component type name used in descriptors.
    #[inline]
    pub fn component_type() -> re_sdk_types::ComponentType {
        COMPONENT_QUALITY.into()
    }
}

impl From<sp_kernel::Quality> for Quality {
    fn from(quality: sp_kernel::Quality) -> Self {
        Self(match quality {
            sp_kernel::Quality::Bad => 0,
            sp_kernel::Quality::Uncertain => 1,
            sp_kernel::Quality::Good => 2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_encoding() {
        assert_eq!(Quality::from(sp_kernel::Quality::Good).to_u8(), 2);
        assert_eq!(Quality::from(sp_kernel::Quality::Bad).to_u8(), 0);
        assert_eq!(Quality::from(sp_kernel::Quality::Uncertain).to_u8(), 1);
    }
}

//! Unit operations and simplified degrees-of-freedom requirements.

use serde::{Deserialize, Serialize};

use crate::domain::error::{Result, SimulationError};
use crate::domain::ids::UnitOpId;

/// Kind of process unit operation in a flowsheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitOpKind {
    /// Combines multiple inlet streams.
    Mixer,
    /// Splits one inlet into multiple outlets.
    Splitter,
    /// Adds heat to a stream.
    Heater,
    /// Removes heat from a stream.
    Cooler,
    /// Reduces pressure through a restriction.
    Valve,
    /// Raises pressure of a liquid stream.
    Pump,
    /// Vapor-liquid separator drum.
    FlashDrum,
    /// Connects unit operations with optional pressure drop.
    Pipe,
}

/// Returns the number of specifications required to make `kind` fully defined.
///
/// This is a simplified degrees-of-freedom model for F4. A rigorous per-balance
/// model will ship with the native engine in F6.
#[expect(clippy::match_same_arms)]
pub fn required_specs(kind: UnitOpKind) -> u32 {
    match kind {
        UnitOpKind::Mixer => 0,
        UnitOpKind::Splitter => 1,
        UnitOpKind::Heater => 1,
        UnitOpKind::Cooler => 1,
        UnitOpKind::Valve => 1,
        UnitOpKind::Pump => 1,
        UnitOpKind::FlashDrum => 2,
        UnitOpKind::Pipe => 1,
    }
}

/// A unit operation node in a flowsheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitOp {
    id: UnitOpId,
    kind: UnitOpKind,
    name: String,
}

impl UnitOp {
    /// Creates a unit operation with validated identifiers and name.
    pub fn new(id: UnitOpId, kind: UnitOpKind, name: impl Into<String>) -> Result<Self> {
        let trimmed = name.into().trim().to_owned();
        if trimmed.is_empty() {
            return Err(SimulationError::EmptyId("unit operation name"));
        }
        Ok(Self {
            id,
            kind,
            name: trimmed,
        })
    }

    /// Unit operation identifier.
    pub fn id(&self) -> &UnitOpId {
        &self.id
    }

    /// Unit operation kind.
    pub fn kind(&self) -> UnitOpKind {
        self.kind
    }

    /// Human-readable name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_specs_table() {
        assert_eq!(required_specs(UnitOpKind::Mixer), 0);
        assert_eq!(required_specs(UnitOpKind::Splitter), 1);
        assert_eq!(required_specs(UnitOpKind::Heater), 1);
        assert_eq!(required_specs(UnitOpKind::Cooler), 1);
        assert_eq!(required_specs(UnitOpKind::Valve), 1);
        assert_eq!(required_specs(UnitOpKind::Pump), 1);
        assert_eq!(required_specs(UnitOpKind::FlashDrum), 2);
        assert_eq!(required_specs(UnitOpKind::Pipe), 1);
    }
}

//! Chemical components and stream compositions.

use serde::{Deserialize, Serialize};

use crate::domain::error::{Result, SimulationError};

/// A pure chemical species referenced by name in a flowsheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChemicalComponent {
    name: String,
}

impl ChemicalComponent {
    /// Creates a component with a non-empty name.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let trimmed = name.into().trim().to_owned();
        if trimmed.is_empty() {
            return Err(SimulationError::EmptyId("component name"));
        }
        Ok(Self { name: trimmed })
    }

    /// Component name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Molar fraction vector for a material stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Composition(pub Vec<f64>);

impl Composition {
    /// Creates a composition from raw molar fractions.
    pub fn new(fractions: Vec<f64>) -> Self {
        Self(fractions)
    }

    /// Sum of all molar fractions.
    pub fn sum(&self) -> f64 {
        self.0.iter().sum()
    }

    /// Returns `true` when the fractions sum to one within `tol`.
    pub fn is_normalized(&self, tol: f64) -> bool {
        (self.sum() - 1.0).abs() <= tol
    }

    /// Number of component fractions.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` when there are no fractions.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Molar fractions as a slice.
    pub fn fractions(&self) -> &[f64] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_is_normalized() {
        assert!(Composition::new(vec![0.5, 0.5]).is_normalized(1e-6));
        assert!(!Composition::new(vec![0.5, 0.4]).is_normalized(1e-6));
    }
}

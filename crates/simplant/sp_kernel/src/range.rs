//! Engineering measurement ranges.

use serde::{Deserialize, Serialize};

use crate::error::{KernelError, Result};
use crate::unit::UnitOfMeasure;

/// An inclusive engineering range with an associated unit.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EngineeringRange {
    low: f64,
    high: f64,
    unit: UnitOfMeasure,
}

impl EngineeringRange {
    /// Creates a range, rejecting non-finite bounds or `low >= high`.
    pub fn new(low: f64, high: f64, unit: UnitOfMeasure) -> Result<Self> {
        if !low.is_finite() || !high.is_finite() || low >= high {
            return Err(KernelError::InvalidRange { low, high });
        }
        Ok(Self { low, high, unit })
    }

    /// Lower bound (inclusive).
    pub fn low(&self) -> f64 {
        self.low
    }

    /// Upper bound (inclusive).
    pub fn high(&self) -> f64 {
        self.high
    }

    /// Unit associated with the range bounds.
    pub fn unit(&self) -> UnitOfMeasure {
        self.unit
    }

    /// Span of the range (`high - low`).
    pub fn span(&self) -> f64 {
        self.high - self.low
    }

    /// Returns `true` if `value` lies within the inclusive bounds.
    pub fn contains(&self, value: f64) -> bool {
        self.low <= value && value <= self.high
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_range() {
        assert!(EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).is_ok());
    }

    #[test]
    fn invalid_range_when_low_not_less_than_high() {
        let err = EngineeringRange::new(100.0, 0.0, UnitOfMeasure::Bar).unwrap_err();
        assert!(matches!(
            err,
            KernelError::InvalidRange {
                low: 100.0,
                high: 0.0
            }
        ));
    }

    #[test]
    fn contains_is_inclusive() {
        let range = EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap();
        assert!(range.contains(0.0));
        assert!(range.contains(50.0));
        assert!(range.contains(100.0));
        assert!(!range.contains(-1.0));
        assert!(!range.contains(101.0));
    }
}

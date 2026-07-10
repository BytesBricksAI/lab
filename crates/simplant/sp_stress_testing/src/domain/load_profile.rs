//! Load profiles, design limits, and safety factors.

use serde::{Deserialize, Serialize};

use crate::domain::error::{Result, StressError};

/// A single load value for a named process variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadPoint {
    variable: String,
    value: f64,
}

impl LoadPoint {
    /// Creates a load point for `variable` at `value`.
    pub fn new(variable: impl Into<String>, value: f64) -> Self {
        Self {
            variable: variable.into(),
            value,
        }
    }

    /// Process variable name.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Load value.
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// Ordered collection of load points defining a stress load profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadProfile {
    points: Vec<LoadPoint>,
}

impl LoadProfile {
    /// Creates a load profile from `points`.
    pub fn new(points: Vec<LoadPoint>) -> Self {
        Self { points }
    }

    /// Load points in profile order.
    pub fn points(&self) -> &[LoadPoint] {
        &self.points
    }

    /// Returns `true` when the profile has no load points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Maximum allowable value for a process variable under design conditions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignLimit {
    variable: String,
    max_value: f64,
}

impl DesignLimit {
    /// Creates a design limit for `variable` with `max_value`.
    pub fn new(variable: impl Into<String>, max_value: f64) -> Self {
        Self {
            variable: variable.into(),
            max_value,
        }
    }

    /// Process variable name.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Design maximum value.
    pub fn max_value(&self) -> f64 {
        self.max_value
    }
}

/// Positive finite multiplier applied to design limits when checking load profiles.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SafetyFactor(f64);

impl SafetyFactor {
    /// Creates a safety factor; rejects non-positive or non-finite values.
    pub fn new(f: f64) -> Result<Self> {
        if f <= 0.0 || !f.is_finite() {
            return Err(StressError::InvalidSafetyFactor(f));
        }
        Ok(Self(f))
    }

    /// Numeric safety factor value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_factor_rejects_zero() {
        assert_eq!(
            SafetyFactor::new(0.0),
            Err(StressError::InvalidSafetyFactor(0.0))
        );
    }

    #[test]
    fn safety_factor_rejects_negative() {
        assert_eq!(
            SafetyFactor::new(-1.0),
            Err(StressError::InvalidSafetyFactor(-1.0))
        );
    }

    #[test]
    fn safety_factor_rejects_nan() {
        let nan = f64::NAN;
        let err = SafetyFactor::new(nan).unwrap_err();
        assert!(matches!(err, StressError::InvalidSafetyFactor(v) if v.is_nan()));
    }

    #[test]
    fn safety_factor_accepts_valid_value() {
        let factor = SafetyFactor::new(1.5);
        assert!(factor.is_ok());
        assert_eq!(factor.unwrap().value(), 1.5);
    }
}

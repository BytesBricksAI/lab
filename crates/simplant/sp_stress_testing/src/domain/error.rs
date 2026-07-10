//! Stress testing domain errors.

use thiserror::Error;

/// Errors raised by the stress testing domain.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum StressError {
    /// Stress test identifier must not be empty.
    #[error("stress test id must not be empty")]
    EmptyId,

    /// Load profile must contain at least one point.
    #[error("load profile must not be empty")]
    EmptyLoadProfile,

    /// At least one acceptance criterion is required.
    #[error("acceptance criteria must not be empty")]
    EmptyAcceptanceCriteria,

    /// Safety factor must be positive and finite.
    #[error("invalid safety factor: {0}")]
    InvalidSafetyFactor(f64),

    /// A load point references a variable with no matching design limit.
    #[error("load variable has no design limit: {0}")]
    UnmatchedLoadVariable(String),

    /// A load point exceeds the allowable load (design limit × safety factor).
    #[error("load for {variable} ({load}) exceeds allowable ({allowable})")]
    LoadExceedsAllowable {
        variable: String,
        load: f64,
        allowable: f64,
    },

    /// Invalid lifecycle state transition.
    #[error("invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },
}

/// Domain result type.
pub type Result<T> = core::result::Result<T, StressError>;

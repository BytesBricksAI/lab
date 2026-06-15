//! Domain error types for the shared kernel.

/// Errors produced by shared-kernel value-object validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum KernelError {
    /// Tag identifier is empty after trimming whitespace.
    #[error("tag identifier must not be empty")]
    EmptyTagId,

    /// Tag identifier contains characters outside the allowed set.
    #[error("invalid tag identifier: {0}")]
    InvalidTagId(String),

    /// Engineering range bounds are invalid (`low >= high` or non-finite).
    #[error("invalid engineering range: low ({low}) must be less than high ({high})")]
    InvalidRange { low: f64, high: f64 },

    /// Two units do not share the same physical dimension.
    #[error("incompatible units: expected {expected}, found {found}")]
    IncompatibleUnits { expected: String, found: String },

    /// Alarm limit ordering rules were violated.
    #[error("invalid alarm limits: {0}")]
    InvalidAlarmLimits(String),

    /// No alarm limits were provided.
    #[error("alarm limits must contain at least one limit")]
    EmptyAlarmLimits,

    /// Time window start is not strictly before end.
    #[error("invalid time window: start must be strictly before end")]
    InvalidTimeWindow,
}

/// Result type alias for shared-kernel operations.
pub type Result<T> = core::result::Result<T, KernelError>;

//! Acquisition domain errors.

use thiserror::Error;

/// Errors raised by the acquisition domain and application layers.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AcquisitionError {
    /// Binding references a tag that is not in the asset catalog.
    #[error("unknown tag: {0}")]
    UnknownTag(String),

    /// Two bindings target the same tag.
    #[error("duplicate binding for tag: {0}")]
    DuplicateBinding(String),

    /// Session has no tag bindings.
    #[error("session has no tag bindings")]
    EmptyBindings,

    /// Invalid session state transition.
    #[error("invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// Tag binding address is empty after trimming.
    #[error("tag binding address must not be empty")]
    EmptyAddress,

    /// Profile parse or load failure.
    #[error("acquisition config error: {0}")]
    Config(String),

    /// Failure while recording to the store (raised by a driven recorder adapter).
    #[error("recording error: {0}")]
    Recording(String),

    /// Failure while reading from a data source (raised by a driven source adapter).
    #[error("data source error: {0}")]
    Source(String),
}

/// Domain result type.
pub type Result<T> = core::result::Result<T, AcquisitionError>;

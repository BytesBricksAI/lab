//! Domain error types for the ML data loop.

/// Errors produced by dataset specification and split validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DatasetError {
    /// No features were provided for the dataset.
    #[error("dataset must contain at least one feature")]
    EmptyFeatures,

    /// A required name field is empty after trimming whitespace.
    #[error("{0} name must not be empty")]
    EmptyName(&'static str),

    /// A feature or target references a tag that is not in the asset catalog.
    #[error("unknown feature tag: {0}")]
    UnknownFeatureTag(String),

    /// Two temporal split windows overlap, causing data leakage.
    #[error("temporal split windows overlap: {a} and {b}")]
    WindowOverlap { a: &'static str, b: &'static str },

    /// Two features share the same name.
    #[error("duplicate feature name: {0}")]
    DuplicateFeatureName(String),

    /// Configuration or serialization error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Error propagated from the shared kernel.
    #[error(transparent)]
    Kernel(#[from] sp_kernel::KernelError),
}

/// Result type alias for ML data loop operations.
pub type Result<T> = core::result::Result<T, DatasetError>;

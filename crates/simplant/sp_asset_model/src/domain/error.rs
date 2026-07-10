//! Domain error types for the asset model.

/// Errors produced by asset-model aggregate validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AssetError {
    /// Identifier is empty after trimming whitespace.
    #[error("{0} must not be empty")]
    EmptyId(&'static str),

    /// An area with the same identifier already exists in the facility.
    #[error("duplicate area: {0}")]
    DuplicateArea(String),

    /// A process unit with the same identifier already exists in the facility.
    #[error("duplicate unit: {0}")]
    DuplicateUnit(String),

    /// Equipment with the same identifier already exists in the catalog.
    #[error("duplicate equipment: {0}")]
    DuplicateEquipment(String),

    /// A tag with the same identifier already exists in the catalog.
    #[error("duplicate tag: {0}")]
    DuplicateTag(String),

    /// Referenced area was not found in the facility.
    #[error("area not found: {0}")]
    AreaNotFound(String),

    /// Referenced process unit was not found in the facility.
    #[error("unit not found: {0}")]
    UnitNotFound(String),

    /// Referenced equipment was not found in the catalog.
    #[error("equipment not found: {0}")]
    EquipmentNotFound(String),

    /// Tag unit does not match the engineering range or alarm unit.
    #[error("unit mismatch: expected {expected}, found {found}")]
    UnitMismatch { expected: String, found: String },

    /// Alarm limits fall outside the tag engineering range.
    #[error("alarm limits are outside the engineering range")]
    AlarmsOutOfRange,

    /// Design specification violates dimensional or finiteness rules.
    #[error("invalid design spec: {0}")]
    InvalidDesignSpec(String),

    /// I/O error while reading or writing the catalog file.
    #[error("I/O error: {0}")]
    Io(String),

    /// TOML parse error.
    #[error("parse error: {0}")]
    Parse(String),

    /// Propagated shared-kernel validation error.
    #[error(transparent)]
    Kernel(#[from] sp_kernel::KernelError),
}

/// Result type alias for asset-model operations.
pub type Result<T> = core::result::Result<T, AssetError>;

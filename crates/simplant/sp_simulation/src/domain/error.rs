//! Domain error types for process simulation.

/// Errors produced by simulation aggregate validation.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SimulationError {
    /// Identifier is empty after trimming whitespace.
    #[error("{0} must not be empty")]
    EmptyId(&'static str),

    /// Flowsheet has no unit operations.
    #[error("flowsheet must contain at least one unit operation")]
    EmptyFlowsheet,

    /// Feed-stream composition does not sum to one within tolerance.
    #[error("composition for stream {stream} is not normalized (sum = {sum})")]
    CompositionNotNormalized { stream: String, sum: f64 },

    /// Feed-stream composition length does not match the component list.
    #[error(
        "composition for stream {stream} has arity mismatch (expected {expected}, found {found})"
    )]
    CompositionArityMismatch {
        stream: String,
        expected: usize,
        found: usize,
    },

    /// Stream references a unit operation that does not exist.
    #[error("stream {stream} references unknown unit operation {unit_op}")]
    DanglingStream { stream: String, unit_op: String },

    /// Flowsheet is not square: declared specifications do not match required degrees of freedom.
    #[error("degrees of freedom mismatch (required specs: {required}, declared specs: {declared})")]
    DegreesOfFreedomMismatch { required: i64, declared: i64 },

    /// Specification targets a unit operation that does not exist.
    #[error("specification references unknown unit operation {0}")]
    UnknownSpecUnitOp(String),

    /// Scenario references a flowsheet that is not approved.
    #[error("flowsheet not approved: {0}")]
    FlowsheetNotApproved(String),

    /// Scenario duration must be strictly positive.
    #[error("scenario duration must be greater than zero")]
    InvalidDuration,

    /// Engine does not support the capability required by the scenario.
    #[error("incompatible engine capability (required: {required}, available: {available})")]
    IncompatibleCapability { required: String, available: String },

    /// Run references a scenario that is not approved.
    #[error("scenario not approved: {0}")]
    RunNotApproved(String),

    /// Flowsheet must declare at least one chemical component.
    #[error("flowsheet must contain at least one component")]
    EmptyComponents,

    /// Configuration or serialization error.
    #[error("configuration error: {0}")]
    Config(String),
}

/// Result type alias for simulation operations.
pub type Result<T> = core::result::Result<T, SimulationError>;

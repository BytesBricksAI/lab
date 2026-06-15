//! Public API surface for `sp_simulation`.

pub use crate::application::ports::{EngineCapabilities, SimState, SimulatorPort};
pub use crate::domain::{
    BoundaryCondition, ChemicalComponent, Composition, EngineCapability, FlowsheetApproved,
    FlowsheetId, FlowsheetRevised, FlowsheetSpec, FlowsheetState, MaterialStream, RecordingId,
    Result, RunCompleted, RunFailed, RunId, RunStatus, Scenario, ScenarioApproved, ScenarioId,
    SimulationError, SimulationEvent, SimulationRun, Specification, StreamId, ThermoPackage,
    UnitOp, UnitOpId, UnitOpKind, required_specs,
};
pub use crate::infrastructure::toml_flowsheet::{
    flowsheet_from_str, load_flowsheet, save_flowsheet,
};

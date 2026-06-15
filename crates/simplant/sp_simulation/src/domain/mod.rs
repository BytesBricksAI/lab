//! Simulation domain model.

pub mod component;
pub mod error;
pub mod events;
pub mod flowsheet;
pub mod ids;
pub mod run;
pub mod scenario;
pub mod stream;
pub mod unit_op;

pub use component::{ChemicalComponent, Composition};
pub use error::{Result, SimulationError};
pub use events::{
    FlowsheetApproved, FlowsheetRevised, RunCompleted, RunFailed, ScenarioApproved, SimulationEvent,
};
pub use flowsheet::{FlowsheetSpec, FlowsheetState, Specification, ThermoPackage};
pub use ids::{FlowsheetId, RecordingId, RunId, ScenarioId, StreamId, UnitOpId};
pub use run::{RunStatus, SimulationRun};
pub use scenario::{BoundaryCondition, EngineCapability, Scenario};
pub use stream::MaterialStream;
pub use unit_op::{UnitOp, UnitOpKind, required_specs};

//! Domain events emitted by simulation aggregates.

use serde::{Deserialize, Serialize};

/// A flowsheet passed degrees-of-freedom analysis and was approved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowsheetApproved {
    pub flowsheet: String,
    pub version: u32,
}

/// A new draft revision of a flowsheet was created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowsheetRevised {
    pub flowsheet: String,
    pub version: u32,
}

/// A scenario was approved against an approved flowsheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioApproved {
    pub scenario: String,
}

/// A simulation run completed successfully.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCompleted {
    pub run: String,
}

/// A simulation run failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunFailed {
    pub run: String,
    pub reason: String,
}

/// Union of simulation domain events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationEvent {
    /// Flowsheet approved.
    FlowsheetApproved(FlowsheetApproved),
    /// Flowsheet revised.
    FlowsheetRevised(FlowsheetRevised),
    /// Scenario approved.
    ScenarioApproved(ScenarioApproved),
    /// Run completed.
    RunCompleted(RunCompleted),
    /// Run failed.
    RunFailed(RunFailed),
}

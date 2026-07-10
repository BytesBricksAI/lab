//! Simulation run aggregate.

use crate::application::ports::EngineCapabilities;
use crate::domain::error::{Result, SimulationError};
use crate::domain::events::{RunCompleted, RunFailed};
use crate::domain::ids::{RecordingId, RunId, ScenarioId};
use crate::domain::scenario::{EngineCapability, Scenario};

/// Lifecycle status of a simulation run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    /// Created and ready to execute.
    Created,

    /// Finished successfully.
    Completed,

    /// Finished with failure.
    Failed,
}

/// Aggregate root for a single simulation execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationRun {
    id: RunId,
    scenario: ScenarioId,
    recording: RecordingId,
    status: RunStatus,
    engine_capability: EngineCapability,
}

impl SimulationRun {
    /// Starts a run against an approved scenario when the engine supports the required capability.
    pub fn start(
        id: RunId,
        scenario: &Scenario,
        recording: RecordingId,
        engine: EngineCapabilities,
    ) -> Result<Self> {
        if !scenario.is_approved() {
            return Err(SimulationError::RunNotApproved(
                scenario.id().as_str().to_owned(),
            ));
        }

        let supported = match scenario.required_capability() {
            EngineCapability::SteadyState => engine.steady_state,
            EngineCapability::Dynamic => engine.dynamic,
        };

        if !supported {
            return Err(SimulationError::IncompatibleCapability {
                required: scenario.required_capability().as_str().to_owned(),
                available: engine_available_label(engine),
            });
        }

        Ok(Self {
            id,
            scenario: scenario.id().clone(),
            recording,
            status: RunStatus::Created,
            engine_capability: scenario.required_capability(),
        })
    }

    /// Marks the run as completed.
    pub fn complete(&mut self) -> RunCompleted {
        self.status = RunStatus::Completed;
        RunCompleted {
            run: self.id.as_str().to_owned(),
        }
    }

    /// Marks the run as failed with a reason.
    pub fn fail(&mut self, reason: impl Into<String>) -> RunFailed {
        self.status = RunStatus::Failed;
        RunFailed {
            run: self.id.as_str().to_owned(),
            reason: reason.into(),
        }
    }

    /// Run identifier.
    pub fn id(&self) -> &RunId {
        &self.id
    }

    /// Linked scenario identifier.
    pub fn scenario(&self) -> &ScenarioId {
        &self.scenario
    }

    /// Linked recording identifier (1:1 binding).
    pub fn recording(&self) -> &RecordingId {
        &self.recording
    }

    /// Current run status.
    pub fn status(&self) -> RunStatus {
        self.status
    }

    /// Engine capability used for this run.
    pub fn engine_capability(&self) -> EngineCapability {
        self.engine_capability
    }
}

fn engine_available_label(engine: EngineCapabilities) -> String {
    let mut caps = Vec::new();
    if engine.steady_state {
        caps.push("SteadyState");
    }
    if engine.dynamic {
        caps.push("Dynamic");
    }
    if caps.is_empty() {
        "none".to_owned()
    } else {
        caps.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::EngineCapabilities;
    use crate::domain::component::{ChemicalComponent, Composition};
    use crate::domain::flowsheet::{FlowsheetSpec, Specification, ThermoPackage};
    use crate::domain::ids::{FlowsheetId, ScenarioId, StreamId, UnitOpId};
    use crate::domain::scenario::Scenario;
    use crate::domain::stream::MaterialStream;
    use crate::domain::unit_op::{UnitOp, UnitOpKind};

    fn approved_scenario(capability: EngineCapability) -> Scenario {
        let mut fs = FlowsheetSpec::draft(
            FlowsheetId::new("FS-01").unwrap(),
            vec![
                ChemicalComponent::new("N2").unwrap(),
                ChemicalComponent::new("O2").unwrap(),
            ],
            vec![
                UnitOp::new(UnitOpId::new("H-01").unwrap(), UnitOpKind::Heater, "Heater").unwrap(),
            ],
            vec![MaterialStream::new(
                StreamId::new("S-IN").unwrap(),
                None,
                Some(UnitOpId::new("H-01").unwrap()),
                Composition::new(vec![0.79, 0.21]),
            )],
            vec![Specification::new(UnitOpId::new("H-01").unwrap(), "duty", 100.0).unwrap()],
            ThermoPackage::IdealGas,
        )
        .unwrap();
        fs.approve().unwrap();
        Scenario::approve(
            &ScenarioId::new("SC-01").unwrap(),
            &fs,
            vec![],
            60.0,
            capability,
        )
        .unwrap()
        .0
    }

    #[test]
    fn start_rejects_unapproved_scenario() {
        let scenario = Scenario::unapproved_for_test(
            ScenarioId::new("SC-01").unwrap(),
            FlowsheetId::new("FS-01").unwrap(),
            1,
            EngineCapability::SteadyState,
        );
        let err = SimulationRun::start(
            RunId::new("RUN-01").unwrap(),
            &scenario,
            RecordingId::new("REC-01").unwrap(),
            EngineCapabilities {
                steady_state: true,
                dynamic: false,
            },
        )
        .unwrap_err();
        assert!(matches!(err, SimulationError::RunNotApproved(_)));
    }

    #[test]
    fn start_rejects_incompatible_capability() {
        let scenario = approved_scenario(EngineCapability::Dynamic);
        let err = SimulationRun::start(
            RunId::new("RUN-01").unwrap(),
            &scenario,
            RecordingId::new("REC-01").unwrap(),
            EngineCapabilities {
                steady_state: true,
                dynamic: false,
            },
        )
        .unwrap_err();
        assert!(matches!(
            err,
            SimulationError::IncompatibleCapability {
                required,
                available
            } if required == "Dynamic" && available == "SteadyState"
        ));
    }

    #[test]
    fn start_succeeds_and_complete_fail_update_status() {
        let scenario = approved_scenario(EngineCapability::SteadyState);
        let mut run = SimulationRun::start(
            RunId::new("RUN-01").unwrap(),
            &scenario,
            RecordingId::new("REC-01").unwrap(),
            EngineCapabilities {
                steady_state: true,
                dynamic: false,
            },
        )
        .unwrap();
        assert_eq!(run.status(), RunStatus::Created);

        let completed = run.complete();
        assert_eq!(run.status(), RunStatus::Completed);
        assert_eq!(completed.run, "RUN-01");

        let mut run2 = SimulationRun::start(
            RunId::new("RUN-02").unwrap(),
            &scenario,
            RecordingId::new("REC-02").unwrap(),
            EngineCapabilities {
                steady_state: true,
                dynamic: true,
            },
        )
        .unwrap();
        let failed = run2.fail("solver divergence");
        assert_eq!(run2.status(), RunStatus::Failed);
        assert_eq!(failed.reason, "solver divergence");
    }
}

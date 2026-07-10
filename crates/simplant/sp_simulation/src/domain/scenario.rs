//! Scenario aggregate binding an approved flowsheet to run conditions.

use serde::{Deserialize, Serialize};

use crate::domain::error::{Result, SimulationError};
use crate::domain::events::ScenarioApproved;
use crate::domain::flowsheet::FlowsheetSpec;
use crate::domain::ids::{FlowsheetId, ScenarioId};

/// Engine capability required by a scenario.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineCapability {
    /// Steady-state solution.
    SteadyState,

    /// Dynamic (transient) simulation.
    Dynamic,
}

impl EngineCapability {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::SteadyState => "SteadyState",
            Self::Dynamic => "Dynamic",
        }
    }
}

/// A boundary condition applied during scenario execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundaryCondition {
    variable: String,
    value: f64,
}

impl BoundaryCondition {
    /// Creates a boundary condition with a non-empty variable name.
    pub fn new(variable: impl Into<String>, value: f64) -> Result<Self> {
        let trimmed = variable.into().trim().to_owned();
        if trimmed.is_empty() {
            return Err(SimulationError::EmptyId("boundary condition variable"));
        }
        Ok(Self {
            variable: trimmed,
            value,
        })
    }

    /// Variable name.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Fixed value.
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// Aggregate root for a simulation scenario.
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    id: ScenarioId,
    flowsheet: FlowsheetId,
    flowsheet_version: u32,
    boundary_conditions: Vec<BoundaryCondition>,
    duration_secs: f64,
    required_capability: EngineCapability,
    approved: bool,
}

impl Scenario {
    /// Creates and approves a scenario against an approved flowsheet.
    pub fn approve(
        id: &ScenarioId,
        flowsheet: &FlowsheetSpec,
        boundary_conditions: Vec<BoundaryCondition>,
        duration_secs: f64,
        required_capability: EngineCapability,
    ) -> Result<(Self, ScenarioApproved)> {
        if !flowsheet.is_approved() {
            return Err(SimulationError::FlowsheetNotApproved(
                flowsheet.id().as_str().to_owned(),
            ));
        }
        if duration_secs <= 0.0 {
            return Err(SimulationError::InvalidDuration);
        }

        let scenario = Self {
            id: id.clone(),
            flowsheet: flowsheet.id().clone(),
            flowsheet_version: flowsheet.version(),
            boundary_conditions,
            duration_secs,
            required_capability,
            approved: true,
        };

        Ok((
            scenario,
            ScenarioApproved {
                scenario: id.as_str().to_owned(),
            },
        ))
    }

    /// Scenario identifier.
    pub fn id(&self) -> &ScenarioId {
        &self.id
    }

    /// Linked flowsheet identifier.
    pub fn flowsheet(&self) -> &FlowsheetId {
        &self.flowsheet
    }

    /// Linked flowsheet version at approval time.
    pub fn flowsheet_version(&self) -> u32 {
        self.flowsheet_version
    }

    /// Boundary conditions for this scenario.
    pub fn boundary_conditions(&self) -> &[BoundaryCondition] {
        &self.boundary_conditions
    }

    /// Scenario duration in seconds.
    pub fn duration_secs(&self) -> f64 {
        self.duration_secs
    }

    /// Required engine capability.
    pub fn required_capability(&self) -> EngineCapability {
        self.required_capability
    }

    /// Returns `true` when the scenario is approved.
    pub fn is_approved(&self) -> bool {
        self.approved
    }

    #[cfg(test)]
    pub(crate) fn unapproved_for_test(
        id: ScenarioId,
        flowsheet: FlowsheetId,
        flowsheet_version: u32,
        required_capability: EngineCapability,
    ) -> Self {
        Self {
            id,
            flowsheet,
            flowsheet_version,
            boundary_conditions: vec![],
            duration_secs: 1.0,
            required_capability,
            approved: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::component::{ChemicalComponent, Composition};
    use crate::domain::flowsheet::{FlowsheetSpec, Specification, ThermoPackage};
    use crate::domain::ids::{FlowsheetId, StreamId, UnitOpId};
    use crate::domain::stream::MaterialStream;
    use crate::domain::unit_op::{UnitOp, UnitOpKind};

    fn approved_flowsheet() -> FlowsheetSpec {
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
        fs
    }

    fn draft_flowsheet() -> FlowsheetSpec {
        FlowsheetSpec::draft(
            FlowsheetId::new("FS-DRAFT").unwrap(),
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
        .unwrap()
    }

    #[test]
    fn approve_rejects_unapproved_flowsheet() {
        let draft = draft_flowsheet();
        let err = Scenario::approve(
            &ScenarioId::new("SC-01").unwrap(),
            &draft,
            vec![],
            60.0,
            EngineCapability::SteadyState,
        )
        .unwrap_err();
        assert!(matches!(err, SimulationError::FlowsheetNotApproved(_)));
    }

    #[test]
    fn approve_rejects_invalid_duration() {
        let fs = approved_flowsheet();
        let err = Scenario::approve(
            &ScenarioId::new("SC-01").unwrap(),
            &fs,
            vec![],
            0.0,
            EngineCapability::SteadyState,
        )
        .unwrap_err();
        assert!(matches!(err, SimulationError::InvalidDuration));
    }

    #[test]
    fn approve_succeeds_for_valid_input() {
        let fs = approved_flowsheet();
        let (scenario, event) = Scenario::approve(
            &ScenarioId::new("SC-01").unwrap(),
            &fs,
            vec![BoundaryCondition::new("feed_rate", 10.0).unwrap()],
            3600.0,
            EngineCapability::SteadyState,
        )
        .unwrap();
        assert!(scenario.is_approved());
        assert_eq!(scenario.flowsheet_version(), fs.version());
        assert_eq!(event.scenario, "SC-01");
    }
}

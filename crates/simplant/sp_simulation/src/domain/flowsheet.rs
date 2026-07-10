//! Flowsheet aggregate with structural validation and degrees-of-freedom analysis.

use serde::{Deserialize, Serialize};

use crate::domain::component::ChemicalComponent;
use crate::domain::error::{Result, SimulationError};
use crate::domain::events::{FlowsheetApproved, FlowsheetRevised};
use crate::domain::ids::{FlowsheetId, UnitOpId};
use crate::domain::stream::MaterialStream;
use crate::domain::unit_op::{UnitOp, required_specs};

const COMPOSITION_TOLERANCE: f64 = 1e-6;

/// Thermodynamic property package declared for a flowsheet (not implemented here).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermoPackage {
    /// Peng-Robinson cubic equation of state.
    PengRobinson,

    /// Soave-Redlich-Kwong cubic equation of state.
    Srk,

    /// Perturbed-chain statistical associating fluid theory.
    PcSaft,

    /// Ideal-gas reference model.
    IdealGas,
}

/// Fixes a single degree of freedom on a unit operation variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Specification {
    unit_op: UnitOpId,
    variable: String,
    value: f64,
}

impl Specification {
    /// Creates a specification binding `variable` on `unit_op` to `value`.
    pub fn new(unit_op: UnitOpId, variable: impl Into<String>, value: f64) -> Result<Self> {
        let trimmed = variable.into().trim().to_owned();
        if trimmed.is_empty() {
            return Err(SimulationError::EmptyId("specification variable"));
        }
        Ok(Self {
            unit_op,
            variable: trimmed,
            value,
        })
    }

    /// Target unit operation.
    pub fn unit_op(&self) -> &UnitOpId {
        &self.unit_op
    }

    /// Fixed variable name.
    pub fn variable(&self) -> &str {
        &self.variable
    }

    /// Fixed numeric value.
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// Approval lifecycle state of a flowsheet specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowsheetState {
    /// Under construction; structural edits allowed.
    Draft,

    /// Approved after degrees-of-freedom analysis passed.
    Approved,
}

/// Aggregate root for a versioned process flowsheet specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowsheetSpec {
    id: FlowsheetId,
    version: u32,
    components: Vec<ChemicalComponent>,
    unit_ops: Vec<UnitOp>,
    streams: Vec<MaterialStream>,
    specs: Vec<Specification>,
    thermo: ThermoPackage,
    state: FlowsheetState,
}

impl FlowsheetSpec {
    /// Creates a draft flowsheet after structural validation (no DOF check).
    pub fn draft(
        id: FlowsheetId,
        components: Vec<ChemicalComponent>,
        unit_ops: Vec<UnitOp>,
        streams: Vec<MaterialStream>,
        specs: Vec<Specification>,
        thermo: ThermoPackage,
    ) -> Result<Self> {
        validate_structure(&id, &components, &unit_ops, &streams, &specs)?;

        Ok(Self {
            id,
            version: 1,
            components,
            unit_ops,
            streams,
            specs,
            thermo,
            state: FlowsheetState::Draft,
        })
    }

    /// Remaining degrees of freedom: required specs minus declared specs.
    ///
    /// Zero means the flowsheet is square. Negative means over-specified.
    pub fn degrees_of_freedom(&self) -> i64 {
        let required: i64 = self
            .unit_ops
            .iter()
            .map(|op| i64::from(required_specs(op.kind())))
            .sum();
        let declared = i64::try_from(self.specs.len()).unwrap_or(i64::MAX);
        required - declared
    }

    /// Approves the flowsheet when degrees of freedom are exactly zero.
    pub fn approve(&mut self) -> Result<FlowsheetApproved> {
        let required: i64 = self
            .unit_ops
            .iter()
            .map(|op| i64::from(required_specs(op.kind())))
            .sum();
        let declared = i64::try_from(self.specs.len()).unwrap_or(i64::MAX);

        if self.degrees_of_freedom() != 0 {
            return Err(SimulationError::DegreesOfFreedomMismatch { required, declared });
        }

        self.state = FlowsheetState::Approved;
        Ok(FlowsheetApproved {
            flowsheet: self.id.as_str().to_owned(),
            version: self.version,
        })
    }

    /// Creates a new draft revision with an incremented version number.
    pub fn revise(
        &self,
        components: Vec<ChemicalComponent>,
        unit_ops: Vec<UnitOp>,
        streams: Vec<MaterialStream>,
        specs: Vec<Specification>,
        thermo: ThermoPackage,
    ) -> Result<(Self, FlowsheetRevised)> {
        validate_structure(&self.id, &components, &unit_ops, &streams, &specs)?;

        let version = self.version.saturating_add(1);
        let next = Self {
            id: self.id.clone(),
            version,
            components,
            unit_ops,
            streams,
            specs,
            thermo,
            state: FlowsheetState::Draft,
        };

        Ok((
            next,
            FlowsheetRevised {
                flowsheet: self.id.as_str().to_owned(),
                version,
            },
        ))
    }

    /// Flowsheet identifier.
    pub fn id(&self) -> &FlowsheetId {
        &self.id
    }

    /// Version number (incremented on each revision).
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Current lifecycle state.
    pub fn state(&self) -> FlowsheetState {
        self.state
    }

    /// Returns `true` when the flowsheet is approved.
    pub fn is_approved(&self) -> bool {
        self.state == FlowsheetState::Approved
    }

    /// Declared chemical components.
    pub fn components(&self) -> &[ChemicalComponent] {
        &self.components
    }

    /// Unit operations in the flowsheet.
    pub fn unit_ops(&self) -> &[UnitOp] {
        &self.unit_ops
    }

    /// Material streams in the flowsheet.
    pub fn streams(&self) -> &[MaterialStream] {
        &self.streams
    }

    /// Declared specifications fixing unit-operation variables.
    pub fn specs(&self) -> &[Specification] {
        &self.specs
    }

    /// Declared thermodynamic package.
    pub fn thermo(&self) -> ThermoPackage {
        self.thermo
    }

    /// Re-applies a persisted version number after structural validation (TOML load).
    pub(crate) fn with_version(mut self, version: u32) -> Self {
        self.version = version;
        self
    }
}

fn validate_structure(
    _id: &FlowsheetId,
    components: &[ChemicalComponent],
    unit_ops: &[UnitOp],
    streams: &[MaterialStream],
    specs: &[Specification],
) -> Result<()> {
    if components.is_empty() {
        return Err(SimulationError::EmptyComponents);
    }
    if unit_ops.is_empty() {
        return Err(SimulationError::EmptyFlowsheet);
    }

    let unit_op_ids: std::collections::HashSet<&str> =
        unit_ops.iter().map(|op| op.id().as_str()).collect();

    for stream in streams {
        if let Some(from) = stream.from()
            && !unit_op_ids.contains(from.as_str())
        {
            return Err(SimulationError::DanglingStream {
                stream: stream.id().as_str().to_owned(),
                unit_op: from.as_str().to_owned(),
            });
        }
        if let Some(to) = stream.to()
            && !unit_op_ids.contains(to.as_str())
        {
            return Err(SimulationError::DanglingStream {
                stream: stream.id().as_str().to_owned(),
                unit_op: to.as_str().to_owned(),
            });
        }

        if stream.is_feed() {
            let composition = stream.composition();
            if composition.len() != components.len() {
                return Err(SimulationError::CompositionArityMismatch {
                    stream: stream.id().as_str().to_owned(),
                    expected: components.len(),
                    found: composition.len(),
                });
            }
            if !composition.is_normalized(COMPOSITION_TOLERANCE) {
                return Err(SimulationError::CompositionNotNormalized {
                    stream: stream.id().as_str().to_owned(),
                    sum: composition.sum(),
                });
            }
        }
    }

    for spec in specs {
        if !unit_op_ids.contains(spec.unit_op().as_str()) {
            return Err(SimulationError::UnknownSpecUnitOp(
                spec.unit_op().as_str().to_owned(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::component::Composition;
    use crate::domain::ids::{FlowsheetId, StreamId, UnitOpId};
    use crate::domain::unit_op::UnitOpKind;

    fn component(name: &str) -> ChemicalComponent {
        ChemicalComponent::new(name).unwrap()
    }

    fn heater(id: &str) -> UnitOp {
        UnitOp::new(UnitOpId::new(id).unwrap(), UnitOpKind::Heater, id).unwrap()
    }

    fn feed_stream(id: &str, to: &str, fractions: Vec<f64>) -> MaterialStream {
        MaterialStream::new(
            StreamId::new(id).unwrap(),
            None,
            Some(UnitOpId::new(to).unwrap()),
            Composition::new(fractions),
        )
    }

    fn try_flowsheet(
        unit_ops: Vec<UnitOp>,
        streams: Vec<MaterialStream>,
        specs: Vec<Specification>,
    ) -> Result<FlowsheetSpec> {
        FlowsheetSpec::draft(
            FlowsheetId::new("FS-01").unwrap(),
            vec![component("N2"), component("O2")],
            unit_ops,
            streams,
            specs,
            ThermoPackage::IdealGas,
        )
    }

    fn base_flowsheet(
        unit_ops: Vec<UnitOp>,
        streams: Vec<MaterialStream>,
        specs: Vec<Specification>,
    ) -> FlowsheetSpec {
        try_flowsheet(unit_ops, streams, specs).unwrap()
    }

    #[test]
    fn draft_rejects_empty_components() {
        let err = FlowsheetSpec::draft(
            FlowsheetId::new("FS-01").unwrap(),
            vec![],
            vec![heater("H-01")],
            vec![feed_stream("S-IN", "H-01", vec![0.5, 0.5])],
            vec![],
            ThermoPackage::IdealGas,
        )
        .unwrap_err();
        assert!(matches!(err, SimulationError::EmptyComponents));
    }

    #[test]
    fn draft_rejects_empty_unit_ops() {
        let err = FlowsheetSpec::draft(
            FlowsheetId::new("FS-01").unwrap(),
            vec![component("N2")],
            vec![],
            vec![],
            vec![],
            ThermoPackage::IdealGas,
        )
        .unwrap_err();
        assert!(matches!(err, SimulationError::EmptyFlowsheet));
    }

    #[test]
    fn draft_rejects_non_normalized_feed() {
        let err = try_flowsheet(
            vec![heater("H-01")],
            vec![feed_stream("S-IN", "H-01", vec![0.5, 0.4])],
            vec![],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            SimulationError::CompositionNotNormalized { .. }
        ));
    }

    #[test]
    fn draft_rejects_feed_arity_mismatch() {
        let err = try_flowsheet(
            vec![heater("H-01")],
            vec![feed_stream("S-IN", "H-01", vec![1.0])],
            vec![],
        )
        .unwrap_err();
        assert!(matches!(
            err,
            SimulationError::CompositionArityMismatch { .. }
        ));
    }

    #[test]
    fn draft_rejects_dangling_stream() {
        let err = try_flowsheet(
            vec![heater("H-01")],
            vec![feed_stream("S-IN", "MISSING", vec![0.5, 0.5])],
            vec![],
        )
        .unwrap_err();
        assert!(matches!(err, SimulationError::DanglingStream { .. }));
    }

    #[test]
    fn draft_rejects_unknown_spec_unit_op() {
        let err = try_flowsheet(
            vec![heater("H-01")],
            vec![feed_stream("S-IN", "H-01", vec![0.5, 0.5])],
            vec![Specification::new(UnitOpId::new("MISSING").unwrap(), "duty", 100.0).unwrap()],
        )
        .unwrap_err();
        assert!(matches!(err, SimulationError::UnknownSpecUnitOp(_)));
    }

    #[test]
    fn degrees_of_freedom_heater_without_spec() {
        let mut fs = base_flowsheet(
            vec![heater("H-01")],
            vec![feed_stream("S-IN", "H-01", vec![0.5, 0.5])],
            vec![],
        );
        assert_eq!(fs.degrees_of_freedom(), 1);
        let err = fs.approve().unwrap_err();
        assert!(matches!(
            err,
            SimulationError::DegreesOfFreedomMismatch {
                required: 1,
                declared: 0
            }
        ));
    }

    #[test]
    fn degrees_of_freedom_heater_with_one_spec_approves() {
        let mut fs = base_flowsheet(
            vec![heater("H-01")],
            vec![feed_stream("S-IN", "H-01", vec![0.5, 0.5])],
            vec![Specification::new(UnitOpId::new("H-01").unwrap(), "duty", 250.0).unwrap()],
        );
        assert_eq!(fs.degrees_of_freedom(), 0);
        let event = fs.approve().unwrap();
        assert!(fs.is_approved());
        assert_eq!(event.version, 1);
    }

    #[test]
    fn degrees_of_freedom_over_specified_fails_approval() {
        let mut fs = base_flowsheet(
            vec![heater("H-01")],
            vec![feed_stream("S-IN", "H-01", vec![0.5, 0.5])],
            vec![
                Specification::new(UnitOpId::new("H-01").unwrap(), "duty", 250.0).unwrap(),
                Specification::new(UnitOpId::new("H-01").unwrap(), "outlet_temp", 120.0).unwrap(),
            ],
        );
        assert_eq!(fs.degrees_of_freedom(), -1);
        let err = fs.approve().unwrap_err();
        assert!(matches!(
            err,
            SimulationError::DegreesOfFreedomMismatch {
                required: 1,
                declared: 2
            }
        ));
    }
}

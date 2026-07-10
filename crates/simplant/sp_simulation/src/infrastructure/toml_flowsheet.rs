//! TOML file adapter for flowsheet specifications.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::domain::component::{ChemicalComponent, Composition};
use crate::domain::error::{Result, SimulationError};
use crate::domain::flowsheet::{FlowsheetSpec, FlowsheetState, Specification, ThermoPackage};
use crate::domain::ids::{FlowsheetId, StreamId, UnitOpId};
use crate::domain::stream::MaterialStream;
use crate::domain::unit_op::{UnitOp, UnitOpKind};

/// Loads a draft flowsheet from a TOML file, validating all invariants.
pub fn load_flowsheet(path: impl AsRef<Path>) -> Result<FlowsheetSpec> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .map_err(|err| SimulationError::Config(format!("{}: {err}", path.display())))?;
    flowsheet_from_str(&contents)
}

/// Parses a draft flowsheet from a TOML string, validating all invariants.
pub fn flowsheet_from_str(contents: &str) -> Result<FlowsheetSpec> {
    let dto: FlowsheetDto =
        toml::from_str(contents).map_err(|err| SimulationError::Config(err.to_string()))?;
    dto_to_flowsheet(dto)
}

/// Serializes a flowsheet to a TOML file.
pub fn save_flowsheet(path: impl AsRef<Path>, flowsheet: &FlowsheetSpec) -> Result<()> {
    let path = path.as_ref();
    let dto = flowsheet_to_dto(flowsheet);
    let contents =
        toml::to_string_pretty(&dto).map_err(|err| SimulationError::Config(err.to_string()))?;
    fs::write(path, contents)
        .map_err(|err| SimulationError::Config(format!("{}: {err}", path.display())))
}

#[derive(Debug, Serialize, Deserialize)]
struct FlowsheetDto {
    id: String,
    version: u32,
    thermo: ThermoPackage,
    state: FlowsheetState,
    #[serde(default)]
    component: Vec<ComponentDto>,
    #[serde(default)]
    unit_op: Vec<UnitOpDto>,
    #[serde(default)]
    stream: Vec<StreamDto>,
    #[serde(default)]
    spec: Vec<SpecDto>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComponentDto {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnitOpDto {
    id: String,
    kind: UnitOpKind,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StreamDto {
    id: String,
    from: Option<String>,
    to: Option<String>,
    #[serde(default)]
    composition: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpecDto {
    unit_op: String,
    variable: String,
    value: f64,
}

fn dto_to_flowsheet(dto: FlowsheetDto) -> Result<FlowsheetSpec> {
    let components: Vec<ChemicalComponent> = dto
        .component
        .into_iter()
        .map(|c| ChemicalComponent::new(c.name))
        .collect::<Result<Vec<_>>>()?;

    let unit_ops: Vec<UnitOp> = dto
        .unit_op
        .into_iter()
        .map(|u| UnitOp::new(UnitOpId::new(u.id)?, u.kind, u.name))
        .collect::<Result<Vec<_>>>()?;

    let streams: Vec<MaterialStream> = dto
        .stream
        .into_iter()
        .map(|s| {
            Ok(MaterialStream::new(
                StreamId::new(s.id)?,
                match s.from {
                    Some(id) => Some(UnitOpId::new(id)?),
                    None => None,
                },
                match s.to {
                    Some(id) => Some(UnitOpId::new(id)?),
                    None => None,
                },
                Composition::new(s.composition),
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    let specs: Vec<Specification> = dto
        .spec
        .into_iter()
        .map(|s| Specification::new(UnitOpId::new(s.unit_op)?, s.variable, s.value))
        .collect::<Result<Vec<_>>>()?;

    let mut flowsheet = FlowsheetSpec::draft(
        FlowsheetId::new(dto.id)?,
        components,
        unit_ops,
        streams,
        specs,
        dto.thermo,
    )?
    .with_version(dto.version);

    if dto.state == FlowsheetState::Approved {
        flowsheet.approve()?;
    }

    Ok(flowsheet)
}

fn flowsheet_to_dto(flowsheet: &FlowsheetSpec) -> FlowsheetDto {
    FlowsheetDto {
        id: flowsheet.id().as_str().to_owned(),
        version: flowsheet.version(),
        thermo: flowsheet.thermo(),
        state: flowsheet.state(),
        component: flowsheet
            .components()
            .iter()
            .map(|c| ComponentDto {
                name: c.name().to_owned(),
            })
            .collect(),
        unit_op: flowsheet
            .unit_ops()
            .iter()
            .map(|u| UnitOpDto {
                id: u.id().as_str().to_owned(),
                kind: u.kind(),
                name: u.name().to_owned(),
            })
            .collect(),
        stream: flowsheet
            .streams()
            .iter()
            .map(|s| StreamDto {
                id: s.id().as_str().to_owned(),
                from: s.from().map(|id| id.as_str().to_owned()),
                to: s.to().map(|id| id.as_str().to_owned()),
                composition: s.composition().fractions().to_vec(),
            })
            .collect(),
        spec: flowsheet
            .specs()
            .iter()
            .map(|s| SpecDto {
                unit_op: s.unit_op().as_str().to_owned(),
                variable: s.variable().to_owned(),
                value: s.value(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ids::{FlowsheetId, StreamId, UnitOpId};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[expect(clippy::disallowed_methods)]
    fn unique_temp_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("sp_simulation_{name}_{nanos}.toml"))
    }

    fn sample_draft_flowsheet() -> FlowsheetSpec {
        FlowsheetSpec::draft(
            FlowsheetId::new("FS-TOML").unwrap(),
            vec![
                ChemicalComponent::new("Methane").unwrap(),
                ChemicalComponent::new("Ethane").unwrap(),
            ],
            vec![
                UnitOp::new(
                    UnitOpId::new("H-100").unwrap(),
                    UnitOpKind::Heater,
                    "Feed Heater",
                )
                .unwrap(),
            ],
            vec![MaterialStream::new(
                StreamId::new("S-FEED").unwrap(),
                None,
                Some(UnitOpId::new("H-100").unwrap()),
                Composition::new(vec![0.8, 0.2]),
            )],
            vec![Specification::new(UnitOpId::new("H-100").unwrap(), "duty", 500.0).unwrap()],
            ThermoPackage::PengRobinson,
        )
        .unwrap()
    }

    fn approved_heater_toml(state: FlowsheetState, include_spec: bool) -> String {
        let spec_section = if include_spec {
            r#"
[[spec]]
unit_op = "H-100"
variable = "duty"
value = 500.0
"#
        } else {
            ""
        };
        let state = match state {
            FlowsheetState::Draft => "Draft",
            FlowsheetState::Approved => "Approved",
        };
        format!(
            r#"id = "FS-APPROVED-TEST"
version = 1
thermo = "PengRobinson"
state = "{state}"

[[component]]
name = "Methane"

[[component]]
name = "Ethane"

[[unit_op]]
id = "H-100"
kind = "Heater"
name = "Feed Heater"

[[stream]]
id = "S-FEED"
to = "H-100"
composition = [0.8, 0.2]
{spec_section}"#
        )
    }

    #[test]
    fn load_rejects_approved_flowsheet_with_nonzero_dof() {
        let path = unique_temp_path("approved_nonzero_dof");
        let toml = approved_heater_toml(FlowsheetState::Approved, false);
        fs::write(&path, toml).unwrap();

        let err = load_flowsheet(&path).unwrap_err();
        assert!(matches!(
            err,
            SimulationError::DegreesOfFreedomMismatch {
                required: 1,
                declared: 0
            }
        ));

        assert!(fs::remove_file(path).is_ok());
    }

    #[test]
    fn load_accepts_approved_flowsheet_with_zero_dof() {
        let path = unique_temp_path("approved_zero_dof");
        let toml = approved_heater_toml(FlowsheetState::Approved, true);
        fs::write(&path, toml).unwrap();

        let loaded = load_flowsheet(&path).unwrap();
        assert!(loaded.is_approved());
        assert_eq!(loaded.degrees_of_freedom(), 0);

        assert!(fs::remove_file(path).is_ok());
    }

    #[test]
    fn toml_round_trip_preserves_draft_flowsheet() {
        let path = unique_temp_path("round_trip");
        let original = sample_draft_flowsheet();
        save_flowsheet(&path, &original).unwrap();
        let loaded = load_flowsheet(&path).unwrap();

        assert_eq!(loaded.id().as_str(), original.id().as_str());
        assert_eq!(loaded.version(), original.version());
        assert_eq!(loaded.state(), original.state());
        assert_eq!(loaded.components().len(), original.components().len());
        assert_eq!(loaded.unit_ops().len(), original.unit_ops().len());
        assert_eq!(loaded.streams().len(), original.streams().len());
        assert_eq!(loaded.specs().len(), original.specs().len());
        assert_eq!(loaded.degrees_of_freedom(), original.degrees_of_freedom());

        assert!(fs::remove_file(path).is_ok());
    }
}

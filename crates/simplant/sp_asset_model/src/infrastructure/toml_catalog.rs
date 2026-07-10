//! TOML file adapter for the asset catalog.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sp_kernel::{AlarmLimits, EngineeringRange, TagId, UnitOfMeasure};

use crate::application::ports::AssetCatalogPort;
use crate::domain::catalog::AssetCatalog;
use crate::domain::equipment::{DesignBound, DesignSpec, Equipment, EquipmentKind};
use crate::domain::error::{AssetError, Result};
use crate::domain::facility::Facility;
use crate::domain::ids::{AreaId, EquipmentId, FacilityId, UnitId};
use crate::domain::tag::{Tag, TagSpec};

/// TOML-backed repository for the asset catalog.
pub struct TomlCatalogRepository {
    path: PathBuf,
}

impl TomlCatalogRepository {
    /// Creates a repository targeting the given file path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl AssetCatalogPort for TomlCatalogRepository {
    fn load_catalog(&self) -> Result<AssetCatalog> {
        let contents = fs::read_to_string(&self.path)
            .map_err(|err| AssetError::Io(format!("{}: {err}", self.path.display())))?;
        let dto: CatalogDto =
            toml::from_str(&contents).map_err(|err| AssetError::Parse(err.to_string()))?;
        dto_to_catalog(dto)
    }

    fn save_catalog(&self, catalog: &AssetCatalog) -> Result<()> {
        let dto = catalog_to_dto(catalog);
        let contents =
            toml::to_string_pretty(&dto).map_err(|err| AssetError::Parse(err.to_string()))?;
        fs::write(&self.path, contents)
            .map_err(|err| AssetError::Io(format!("{}: {err}", self.path.display())))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CatalogDto {
    facility: FacilityDto,
    #[serde(default)]
    area: Vec<AreaDto>,
    #[serde(default)]
    unit: Vec<UnitDto>,
    #[serde(default)]
    equipment: Vec<EquipmentDto>,
    #[serde(default)]
    tag: Vec<TagDto>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FacilityDto {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AreaDto {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnitDto {
    id: String,
    area: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EquipmentDto {
    id: String,
    unit: String,
    name: String,
    kind: EquipmentKind,
    max_pressure: Option<f64>,
    max_pressure_unit: Option<UnitOfMeasure>,
    max_temperature: Option<f64>,
    max_temperature_unit: Option<UnitOfMeasure>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TagDto {
    id: String,
    equipment: String,
    description: String,
    unit: UnitOfMeasure,
    range_low: f64,
    range_high: f64,
    alarm_ll: Option<f64>,
    alarm_l: Option<f64>,
    alarm_h: Option<f64>,
    alarm_hh: Option<f64>,
}

fn design_bound_from_dto(
    value: Option<f64>,
    unit: Option<UnitOfMeasure>,
    field: &str,
) -> Result<Option<DesignBound>> {
    match (value, unit) {
        (None, None) => Ok(None),
        (Some(v), Some(u)) => DesignBound::new(v, u).map(Some),
        _ => Err(AssetError::InvalidDesignSpec(format!(
            "{field}: value and unit must both be present or both absent"
        ))),
    }
}

fn dto_to_catalog(dto: CatalogDto) -> Result<AssetCatalog> {
    let (mut facility, _) = Facility::define(FacilityId::new(dto.facility.id)?, dto.facility.name);

    for area in dto.area {
        facility.add_area(AreaId::new(area.id)?, area.name)?;
    }

    for unit in dto.unit {
        facility.add_unit(&AreaId::new(unit.area)?, UnitId::new(unit.id)?, unit.name)?;
    }

    let mut equipment = Vec::new();
    for eq in dto.equipment {
        let max_pressure =
            design_bound_from_dto(eq.max_pressure, eq.max_pressure_unit, "max_pressure")?;
        let max_temperature = design_bound_from_dto(
            eq.max_temperature,
            eq.max_temperature_unit,
            "max_temperature",
        )?;
        let design = DesignSpec::new(max_pressure, max_temperature)?;
        let (eq_item, _) = Equipment::commission(
            EquipmentId::new(eq.id)?,
            UnitId::new(eq.unit)?,
            eq.name,
            eq.kind,
            design,
        )?;
        equipment.push(eq_item);
    }

    let mut tags = Vec::new();
    for tag in dto.tag {
        let range = EngineeringRange::new(tag.range_low, tag.range_high, tag.unit)?;
        let alarms = build_alarms(
            tag.alarm_ll,
            tag.alarm_l,
            tag.alarm_h,
            tag.alarm_hh,
            tag.unit,
        )?;
        let spec = TagSpec {
            id: TagId::new(tag.id)?,
            equipment: EquipmentId::new(tag.equipment)?,
            description: tag.description,
            unit: tag.unit,
            range,
            alarms,
        };
        let (tag_item, _) = Tag::define(spec)?;
        tags.push(tag_item);
    }

    AssetCatalog::assemble(facility, equipment, tags)
}

fn build_alarms(
    low_low: Option<f64>,
    low: Option<f64>,
    high: Option<f64>,
    high_high: Option<f64>,
    unit: UnitOfMeasure,
) -> Result<Option<AlarmLimits>> {
    if low_low.is_none() && low.is_none() && high.is_none() && high_high.is_none() {
        return Ok(None);
    }
    AlarmLimits::new(low_low, low, high, high_high, unit)
        .map(Some)
        .map_err(AssetError::from)
}

fn catalog_to_dto(catalog: &AssetCatalog) -> CatalogDto {
    let facility = catalog.facility();
    CatalogDto {
        facility: FacilityDto {
            id: facility.id().as_str().to_owned(),
            name: facility.name().to_owned(),
        },
        area: facility
            .areas()
            .iter()
            .map(|a| AreaDto {
                id: a.id().as_str().to_owned(),
                name: a.name().to_owned(),
            })
            .collect(),
        unit: facility
            .areas()
            .iter()
            .flat_map(|a| {
                a.units().iter().map(|u| UnitDto {
                    id: u.id().as_str().to_owned(),
                    area: a.id().as_str().to_owned(),
                    name: u.name().to_owned(),
                })
            })
            .collect(),
        equipment: catalog
            .equipment()
            .iter()
            .map(|eq| {
                let design = eq.design();
                let (max_pressure, max_pressure_unit) = design
                    .max_pressure()
                    .map(|b| (Some(b.value()), Some(b.unit())))
                    .unwrap_or((None, None));
                let (max_temperature, max_temperature_unit) = design
                    .max_temperature()
                    .map(|b| (Some(b.value()), Some(b.unit())))
                    .unwrap_or((None, None));
                EquipmentDto {
                    id: eq.id().as_str().to_owned(),
                    unit: eq.unit().as_str().to_owned(),
                    name: eq.name().to_owned(),
                    kind: eq.kind(),
                    max_pressure,
                    max_pressure_unit,
                    max_temperature,
                    max_temperature_unit,
                }
            })
            .collect(),
        tag: catalog
            .tags()
            .iter()
            .map(|tag| TagDto {
                id: tag.id().as_str().to_owned(),
                equipment: tag.equipment().as_str().to_owned(),
                description: tag.description().to_owned(),
                unit: tag.unit(),
                range_low: tag.range().low(),
                range_high: tag.range().high(),
                alarm_ll: tag.alarms().and_then(|a| a.low_low()),
                alarm_l: tag.alarms().and_then(|a| a.low()),
                alarm_h: tag.alarms().and_then(|a| a.high()),
                alarm_hh: tag.alarms().and_then(|a| a.high_high()),
            })
            .collect(),
    }
}

#[cfg(test)]
#[expect(clippy::disallowed_methods)]
mod tests {
    use super::*;
    use crate::domain::equipment::{DesignBound, DesignSpec, Equipment, EquipmentKind};
    use crate::domain::facility::Facility;
    use crate::domain::ids::{AreaId, EquipmentId, FacilityId, UnitId};
    use crate::domain::tag::{Tag, TagSpec};
    use sp_kernel::{AlarmLimits, EngineeringRange, TagId, UnitOfMeasure};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("sp_asset_model_{name}_{nanos}.toml"))
    }

    fn sample_catalog() -> AssetCatalog {
        let (mut facility, _) = Facility::define(FacilityId::new("FAC-01").unwrap(), "Refinery");
        facility
            .add_area(AreaId::new("AREA-A").unwrap(), "Crude")
            .unwrap();
        facility
            .add_unit(
                &AreaId::new("AREA-A").unwrap(),
                UnitId::new("UNIT-100").unwrap(),
                "CDU",
            )
            .unwrap();
        let (equipment, _) = Equipment::commission(
            EquipmentId::new("EQ-101").unwrap(),
            UnitId::new("UNIT-100").unwrap(),
            "Separator",
            EquipmentKind::Vessel,
            DesignSpec::new(
                Some(DesignBound::new(10.0, UnitOfMeasure::Bar).unwrap()),
                Some(DesignBound::new(200.0, UnitOfMeasure::DegreeCelsius).unwrap()),
            )
            .unwrap(),
        )
        .unwrap();
        let spec = TagSpec {
            id: TagId::new("PT-1101").unwrap(),
            equipment: EquipmentId::new("EQ-101").unwrap(),
            description: "Column pressure".to_owned(),
            unit: UnitOfMeasure::Bar,
            range: EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap(),
            alarms: Some(
                AlarmLimits::new(
                    Some(10.0),
                    Some(20.0),
                    Some(80.0),
                    Some(90.0),
                    UnitOfMeasure::Bar,
                )
                .unwrap(),
            ),
        };
        let (tag, _) = Tag::define(spec).unwrap();
        AssetCatalog::assemble(facility, vec![equipment], vec![tag]).unwrap()
    }

    fn catalogs_equivalent(a: &AssetCatalog, b: &AssetCatalog) -> bool {
        a.facility().id() == b.facility().id()
            && a.facility().name() == b.facility().name()
            && a.facility().areas().len() == b.facility().areas().len()
            && a.equipment().len() == b.equipment().len()
            && a.tags().len() == b.tags().len()
            && a.equipment()
                .iter()
                .zip(b.equipment())
                .all(|(left, right)| {
                    left.id() == right.id()
                        && left.unit() == right.unit()
                        && left.name() == right.name()
                        && left.kind() == right.kind()
                })
            && a.tags().iter().zip(b.tags()).all(|(left, right)| {
                left.id() == right.id()
                    && left.equipment() == right.equipment()
                    && left.unit() == right.unit()
                    && left.range().low() == right.range().low()
                    && left.range().high() == right.range().high()
            })
    }

    #[test]
    fn toml_round_trip_preserves_catalog() {
        let path = unique_temp_path("round_trip");
        let repo = TomlCatalogRepository::new(&path);
        let original = sample_catalog();
        repo.save_catalog(&original).unwrap();
        let loaded = repo.load_catalog().unwrap();
        assert!(catalogs_equivalent(&original, &loaded));
        assert!(fs::remove_file(path).is_ok());
    }

    #[test]
    fn load_rejects_disordered_alarms() {
        let path = unique_temp_path("bad_alarms");
        let toml = r#"
[facility]
id = "FAC-01"
name = "Refinery"

[[area]]
id = "AREA-A"
name = "Crude"

[[unit]]
id = "UNIT-100"
area = "AREA-A"
name = "CDU"

[[equipment]]
id = "EQ-101"
unit = "UNIT-100"
name = "Separator"
kind = "Vessel"

[[tag]]
id = "PT-1101"
equipment = "EQ-101"
description = "Column pressure"
unit = "Bar"
range_low = 0.0
range_high = 100.0
alarm_ll = 30.0
alarm_l = 10.0
alarm_h = 80.0
alarm_hh = 90.0
"#;
        fs::write(&path, toml).unwrap();
        let repo = TomlCatalogRepository::new(&path);
        let err = repo.load_catalog().unwrap_err();
        assert!(
            matches!(err, AssetError::Kernel(_)),
            "expected kernel alarm validation error, got {err}"
        );
        assert!(fs::remove_file(path).is_ok());
    }
}

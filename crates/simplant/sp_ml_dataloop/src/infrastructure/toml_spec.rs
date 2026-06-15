//! TOML file adapter for dataset specifications.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sp_asset_model::AssetCatalog;
use sp_kernel::{TagId, TimeWindow};

use crate::domain::dataset_spec::DatasetSpec;
use crate::domain::error::{DatasetError, Result};
use crate::domain::feature::FeatureSpec;
use crate::domain::split::DataSplit;

/// Loads a dataset specification from a TOML file.
pub fn load_dataset_spec(path: impl AsRef<Path>, catalog: &AssetCatalog) -> Result<DatasetSpec> {
    let contents = fs::read_to_string(path.as_ref())
        .map_err(|e| DatasetError::Config(format!("{}: {e}", path.as_ref().display())))?;
    dataset_spec_from_str(&contents, catalog)
}

/// Parses a dataset specification from a TOML string.
pub fn dataset_spec_from_str(s: &str, catalog: &AssetCatalog) -> Result<DatasetSpec> {
    let dto: DatasetSpecDto = toml::from_str(s).map_err(|e| DatasetError::Config(e.to_string()))?;
    dto_to_spec(dto, catalog)
}

/// Saves a dataset specification to a TOML file.
pub fn save_dataset_spec(path: impl AsRef<Path>, spec: &DatasetSpec) -> Result<()> {
    let dto = spec_to_dto(spec);
    let contents = toml::to_string_pretty(&dto).map_err(|e| DatasetError::Config(e.to_string()))?;
    fs::write(path.as_ref(), contents)
        .map_err(|e| DatasetError::Config(format!("{}: {e}", path.as_ref().display())))
}

#[derive(Debug, Serialize, Deserialize)]
struct DatasetSpecDto {
    id: String,
    #[serde(default = "default_version")]
    version: u32,
    features: Vec<FeatureDto>,
    #[serde(default)]
    targets: Vec<FeatureDto>,
    split: SplitDto,
}

fn default_version() -> u32 {
    1
}

#[derive(Debug, Serialize, Deserialize)]
struct FeatureDto {
    tag: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SplitDto {
    train_start: i64,
    train_end: i64,
    val_start: Option<i64>,
    val_end: Option<i64>,
    test_start: i64,
    test_end: i64,
}

fn timestamp_from_epoch_secs(secs: i64) -> Result<jiff::Timestamp> {
    jiff::Timestamp::from_second(secs).map_err(|e| DatasetError::Config(e.to_string()))
}

fn time_window_from_epochs(start_secs: i64, end_secs: i64) -> Result<TimeWindow> {
    let start = timestamp_from_epoch_secs(start_secs)?;
    let end = timestamp_from_epoch_secs(end_secs)?;
    TimeWindow::new(start, end).map_err(DatasetError::from)
}

fn epoch_secs_from_timestamp(ts: jiff::Timestamp) -> i64 {
    ts.as_second()
}

fn split_from_dto(dto: &SplitDto) -> Result<DataSplit> {
    let train = time_window_from_epochs(dto.train_start, dto.train_end)?;
    let val = match (dto.val_start, dto.val_end) {
        (Some(start), Some(end)) => Some(time_window_from_epochs(start, end)?),
        (None, None) => None,
        _ => {
            return Err(DatasetError::Config(
                "val_start and val_end must both be set or both absent".to_owned(),
            ));
        }
    };
    let test = time_window_from_epochs(dto.test_start, dto.test_end)?;
    DataSplit::new(train, val, test)
}

fn feature_from_dto(dto: FeatureDto) -> Result<FeatureSpec> {
    let tag = TagId::new(dto.tag)?;
    FeatureSpec::new(tag, dto.name)
}

fn dto_to_spec(dto: DatasetSpecDto, catalog: &AssetCatalog) -> Result<DatasetSpec> {
    let features = dto
        .features
        .into_iter()
        .map(feature_from_dto)
        .collect::<Result<Vec<_>>>()?;
    let targets = dto
        .targets
        .into_iter()
        .map(feature_from_dto)
        .collect::<Result<Vec<_>>>()?;
    let split = split_from_dto(&dto.split)?;

    let (spec, _) = DatasetSpec::define(dto.id, features, targets, split, catalog)?;

    if dto.version != spec.version() {
        return Err(DatasetError::Config(format!(
            "dataset spec version mismatch: file has {}, expected {}",
            dto.version,
            spec.version()
        )));
    }

    Ok(spec)
}

fn spec_to_dto(spec: &DatasetSpec) -> DatasetSpecDto {
    let split = spec.split();
    let val = split.val().map(|window| {
        (
            epoch_secs_from_timestamp(window.start()),
            epoch_secs_from_timestamp(window.end()),
        )
    });

    DatasetSpecDto {
        id: spec.id().to_owned(),
        version: spec.version(),
        features: spec
            .features()
            .iter()
            .map(|feature| FeatureDto {
                tag: feature.tag().as_str().to_owned(),
                name: feature.name().to_owned(),
            })
            .collect(),
        targets: spec
            .targets()
            .iter()
            .map(|target| FeatureDto {
                tag: target.tag().as_str().to_owned(),
                name: target.name().to_owned(),
            })
            .collect(),
        split: SplitDto {
            train_start: epoch_secs_from_timestamp(split.train().start()),
            train_end: epoch_secs_from_timestamp(split.train().end()),
            val_start: val.as_ref().map(|(start, _)| *start),
            val_end: val.map(|(_, end)| end),
            test_start: epoch_secs_from_timestamp(split.test().start()),
            test_end: epoch_secs_from_timestamp(split.test().end()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_asset_model::{
        AreaId, DesignSpec, Equipment, EquipmentId, EquipmentKind, Facility, FacilityId, Tag,
        TagSpec, UnitId,
    };
    use sp_kernel::{EngineeringRange, UnitOfMeasure};

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
            DesignSpec::new(None, None).unwrap(),
        )
        .unwrap();

        let spec = TagSpec {
            id: TagId::new("PT-1101").unwrap(),
            equipment: EquipmentId::new("EQ-101").unwrap(),
            description: "Pressure".to_owned(),
            unit: UnitOfMeasure::Bar,
            range: EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap(),
            alarms: None,
        };
        let (tag, _) = Tag::define(spec).unwrap();

        AssetCatalog::assemble(facility, vec![equipment], vec![tag]).unwrap()
    }

    #[test]
    fn load_rejects_overlapping_windows() {
        let catalog = sample_catalog();
        let toml = r#"
id = "ds-1"

[[features]]
tag = "PT-1101"
name = "pressure"

[split]
train_start = 100
train_end = 250
test_start = 200
test_end = 300
"#;
        let err = dataset_spec_from_str(toml, &catalog).unwrap_err();
        assert_eq!(
            err,
            DatasetError::WindowOverlap {
                a: "train",
                b: "test"
            }
        );
    }
}

//! Reproducible dataset manifest for export alongside training data.

use serde::{Deserialize, Serialize};

use crate::domain::dataset_spec::DatasetSpec;
use crate::domain::error::{DatasetError, Result};
use crate::domain::split::DataSplit;

/// Schema version for serialized manifests.
pub const SCHEMA_VERSION: u32 = 1;

/// Reproducible manifest written alongside exported dataset files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetManifest {
    dataset_id: String,
    version: u32,
    schema_version: u32,
    feature_names: Vec<String>,
    target_names: Vec<String>,
    split: DataSplit,
}

impl DatasetManifest {
    /// Builds a manifest from a validated dataset specification.
    pub fn from_spec(spec: &DatasetSpec) -> Self {
        Self {
            dataset_id: spec.id().to_owned(),
            version: spec.version(),
            schema_version: SCHEMA_VERSION,
            feature_names: spec
                .features()
                .iter()
                .map(|f| f.name().to_owned())
                .collect(),
            target_names: spec.targets().iter().map(|t| t.name().to_owned()).collect(),
            split: spec.split().clone(),
        }
    }

    /// Dataset identifier.
    pub fn dataset_id(&self) -> &str {
        &self.dataset_id
    }

    /// Dataset specification version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Manifest schema version.
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Feature column names.
    pub fn feature_names(&self) -> &[String] {
        &self.feature_names
    }

    /// Target column names.
    pub fn target_names(&self) -> &[String] {
        &self.target_names
    }

    /// Temporal split definition.
    pub fn split(&self) -> &DataSplit {
        &self.split
    }

    /// Serializes the manifest to a TOML string.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| DatasetError::Config(e.to_string()))
    }

    /// Deserializes a manifest from a TOML string.
    pub fn from_toml_str(s: &str) -> Result<Self> {
        toml::from_str(s).map_err(|e| DatasetError::Config(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dataset_spec::DatasetSpec;
    use crate::domain::feature::FeatureSpec;
    use crate::domain::split::DataSplit;
    use jiff::Timestamp;
    use sp_asset_model::{
        AreaId, AssetCatalog, DesignSpec, Equipment, EquipmentId, EquipmentKind, Facility,
        FacilityId, Tag, TagSpec, UnitId,
    };
    use sp_kernel::{EngineeringRange, TagId, TimeWindow, UnitOfMeasure};

    fn ts(seconds: i64) -> Timestamp {
        Timestamp::from_second(seconds).unwrap()
    }

    fn window(start: i64, end: i64) -> TimeWindow {
        TimeWindow::new(ts(start), ts(end)).unwrap()
    }

    fn sample_spec() -> DatasetSpec {
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

        let tag_spec = TagSpec {
            id: TagId::new("PT-1101").unwrap(),
            equipment: EquipmentId::new("EQ-101").unwrap(),
            description: "Pressure".to_owned(),
            unit: UnitOfMeasure::Bar,
            range: EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap(),
            alarms: None,
        };
        let (tag, _) = Tag::define(tag_spec).unwrap();
        let catalog = AssetCatalog::assemble(facility, vec![equipment], vec![tag]).unwrap();

        let split = DataSplit::new(window(100, 200), None, window(200, 300)).unwrap();
        let feature = FeatureSpec::new(TagId::new("PT-1101").unwrap(), "pressure").unwrap();
        DatasetSpec::define("ds-1", vec![feature], vec![], split, &catalog)
            .unwrap()
            .0
    }

    #[test]
    fn from_spec_produces_feature_names() {
        let spec = sample_spec();
        let manifest = DatasetManifest::from_spec(&spec);
        assert_eq!(manifest.feature_names(), &["pressure".to_owned()]);
        assert_eq!(manifest.dataset_id(), "ds-1");
        assert_eq!(manifest.schema_version(), SCHEMA_VERSION);
    }

    #[test]
    fn manifest_toml_round_trip() {
        let spec = sample_spec();
        let manifest = DatasetManifest::from_spec(&spec);
        let toml = manifest.to_toml().unwrap();
        let restored = DatasetManifest::from_toml_str(&toml).unwrap();
        assert_eq!(manifest, restored);
    }
}

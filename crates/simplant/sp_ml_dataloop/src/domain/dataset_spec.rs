//! Dataset specification aggregate.

use sp_asset_model::AssetCatalog;

use crate::domain::error::{DatasetError, Result};
use crate::domain::events::DatasetPublished;
use crate::domain::feature::FeatureSpec;
use crate::domain::split::DataSplit;

/// Versioned, validated dataset specification for ML training.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetSpec {
    id: String,
    version: u32,
    features: Vec<FeatureSpec>,
    targets: Vec<FeatureSpec>,
    split: DataSplit,
}

impl DatasetSpec {
    /// Defines a new dataset specification at version 1.
    pub fn define(
        id: impl Into<String>,
        features: Vec<FeatureSpec>,
        targets: Vec<FeatureSpec>,
        split: DataSplit,
        catalog: &AssetCatalog,
    ) -> Result<(Self, DatasetPublished)> {
        let id = id.into();
        validate_id(&id)?;
        validate_features(&features)?;
        validate_tags(&features, &targets, catalog)?;

        let spec = Self {
            id: id.clone(),
            version: 1,
            features,
            targets,
            split,
        };

        let event = DatasetPublished {
            dataset: id,
            version: 1,
        };

        Ok((spec, event))
    }

    /// Creates a new version with updated features, targets, and split.
    pub fn revise(
        self,
        features: Vec<FeatureSpec>,
        targets: Vec<FeatureSpec>,
        split: DataSplit,
        catalog: &AssetCatalog,
    ) -> Result<(Self, DatasetPublished)> {
        validate_features(&features)?;
        validate_tags(&features, &targets, catalog)?;

        let version = self.version + 1;
        let spec = Self {
            id: self.id.clone(),
            version,
            features,
            targets,
            split,
        };

        let event = DatasetPublished {
            dataset: self.id,
            version,
        };

        Ok((spec, event))
    }

    /// Dataset identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Current specification version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Input feature columns.
    pub fn features(&self) -> &[FeatureSpec] {
        &self.features
    }

    /// Target columns.
    pub fn targets(&self) -> &[FeatureSpec] {
        &self.targets
    }

    /// Temporal train/validation/test split.
    pub fn split(&self) -> &DataSplit {
        &self.split
    }
}

fn validate_id(id: &str) -> Result<()> {
    if id.trim().is_empty() {
        return Err(DatasetError::EmptyName("dataset"));
    }
    Ok(())
}

fn validate_features(features: &[FeatureSpec]) -> Result<()> {
    if features.is_empty() {
        return Err(DatasetError::EmptyFeatures);
    }

    let mut seen = std::collections::HashSet::new();
    for feature in features {
        if !seen.insert(feature.name()) {
            return Err(DatasetError::DuplicateFeatureName(
                feature.name().to_owned(),
            ));
        }
    }

    Ok(())
}

fn validate_tags(
    features: &[FeatureSpec],
    targets: &[FeatureSpec],
    catalog: &AssetCatalog,
) -> Result<()> {
    for feature in features {
        if catalog.tag(feature.tag()).is_none() {
            return Err(DatasetError::UnknownFeatureTag(
                feature.tag().as_str().to_owned(),
            ));
        }
    }

    for target in targets {
        if catalog.tag(target.tag()).is_none() {
            return Err(DatasetError::UnknownFeatureTag(
                target.tag().as_str().to_owned(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn sample_split() -> DataSplit {
        DataSplit::new(window(100, 200), None, window(200, 300)).unwrap()
    }

    fn sample_feature() -> FeatureSpec {
        FeatureSpec::new(TagId::new("PT-1101").unwrap(), "pressure").unwrap()
    }

    #[test]
    fn define_rejects_empty_features() {
        let catalog = sample_catalog();
        let err =
            DatasetSpec::define("ds-1", vec![], vec![], sample_split(), &catalog).unwrap_err();
        assert_eq!(err, DatasetError::EmptyFeatures);
    }

    #[test]
    fn define_rejects_duplicate_feature_names() {
        let catalog = sample_catalog();
        let features = vec![
            FeatureSpec::new(TagId::new("PT-1101").unwrap(), "pressure").unwrap(),
            FeatureSpec::new(TagId::new("PT-1101").unwrap(), "pressure").unwrap(),
        ];
        let err =
            DatasetSpec::define("ds-1", features, vec![], sample_split(), &catalog).unwrap_err();
        assert!(matches!(err, DatasetError::DuplicateFeatureName(_)));
    }

    #[test]
    fn define_rejects_unknown_feature_tag() {
        let catalog = sample_catalog();
        let features = vec![FeatureSpec::new(TagId::new("TT-9999").unwrap(), "temp").unwrap()];
        let err =
            DatasetSpec::define("ds-1", features, vec![], sample_split(), &catalog).unwrap_err();
        assert!(matches!(err, DatasetError::UnknownFeatureTag(_)));
    }

    #[test]
    fn define_happy_path_version_one() {
        let catalog = sample_catalog();
        let (spec, event) = DatasetSpec::define(
            "ds-1",
            vec![sample_feature()],
            vec![],
            sample_split(),
            &catalog,
        )
        .unwrap();
        assert_eq!(spec.version(), 1);
        assert_eq!(event.version, 1);
        assert_eq!(event.dataset, "ds-1");
    }

    #[test]
    fn revise_increments_version() {
        let catalog = sample_catalog();
        let (spec, _) = DatasetSpec::define(
            "ds-1",
            vec![sample_feature()],
            vec![],
            sample_split(),
            &catalog,
        )
        .unwrap();
        let (revised, event) = spec
            .revise(vec![sample_feature()], vec![], sample_split(), &catalog)
            .unwrap();
        assert_eq!(revised.version(), 2);
        assert_eq!(event.version, 2);
    }
}

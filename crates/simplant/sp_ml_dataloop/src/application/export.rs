//! Use cases for exporting versioned datasets.

use sp_kernel::TagId;

use crate::application::ports::{DataframeQueryPort, DatasetSinkPort};
use crate::domain::dataset_spec::DatasetSpec;
use crate::domain::error::Result;
use crate::domain::manifest::DatasetManifest;

/// Orchestrates a dataset export: queries the training window for all feature+target tags and
/// writes the result through the sink, together with a reproducible manifest.
///
/// F3 exports the training split; per-split export (train/val/test) is a trivial extension.
pub fn export_dataset(
    spec: &DatasetSpec,
    query: &dyn DataframeQueryPort,
    sink: &dyn DatasetSinkPort,
) -> Result<DatasetManifest> {
    let manifest = DatasetManifest::from_spec(spec);
    let tags = collect_unique_tags(spec);
    let train_window = spec.split().train();
    let result = query.query(&train_window, &tags)?;
    sink.write(&manifest, &result)?;
    Ok(manifest)
}

fn collect_unique_tags(spec: &DatasetSpec) -> Vec<TagId> {
    let mut tags = Vec::new();
    for feature in spec.features() {
        push_unique(&mut tags, feature.tag());
    }
    for target in spec.targets() {
        push_unique(&mut tags, target.tag());
    }
    tags
}

fn push_unique(tags: &mut Vec<TagId>, tag: &TagId) {
    if !tags.iter().any(|existing| existing == tag) {
        tags.push(tag.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::{QueryResult, TagSeries};
    use crate::domain::feature::FeatureSpec;
    use crate::domain::split::DataSplit;
    use crate::infrastructure::csv_sink::CsvDatasetSink;
    use jiff::Timestamp;
    use sp_asset_model::{
        AreaId, AssetCatalog, DesignSpec, Equipment, EquipmentId, EquipmentKind, Facility,
        FacilityId, Tag, TagSpec, UnitId,
    };
    use sp_kernel::{EngineeringRange, Measurement, Quality, TagId, TimeWindow, UnitOfMeasure};

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

        let tag_spec = TagSpec {
            id: TagId::new("PT-1101").unwrap(),
            equipment: EquipmentId::new("EQ-101").unwrap(),
            description: "Pressure".to_owned(),
            unit: UnitOfMeasure::Bar,
            range: EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).unwrap(),
            alarms: None,
        };
        let (tag, _) = Tag::define(tag_spec).unwrap();
        AssetCatalog::assemble(facility, vec![equipment], vec![tag]).unwrap()
    }

    fn sample_spec() -> DatasetSpec {
        let catalog = sample_catalog();
        let split = DataSplit::new(window(100, 200), None, window(200, 300)).unwrap();
        let feature = FeatureSpec::new(TagId::new("PT-1101").unwrap(), "pressure").unwrap();
        DatasetSpec::define("ds-export", vec![feature], vec![], split, &catalog)
            .unwrap()
            .0
    }

    struct FakeQueryPort;

    impl DataframeQueryPort for FakeQueryPort {
        fn query(&self, _window: &TimeWindow, _tags: &[TagId]) -> Result<QueryResult> {
            Ok(QueryResult {
                series: vec![TagSeries {
                    tag: TagId::new("PT-1101").unwrap(),
                    measurements: vec![
                        Measurement::new(10.0, Quality::Good, ts(150)),
                        Measurement::new(11.0, Quality::Good, ts(160)),
                    ],
                }],
            })
        }
    }

    #[expect(clippy::disallowed_methods)]
    fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn export_dataset_writes_csv_and_returns_manifest() {
        let spec = sample_spec();
        let dir = unique_temp_dir("sp_ml_dataloop-export");
        let sink = CsvDatasetSink::new(&dir);
        let query = FakeQueryPort;

        let manifest = export_dataset(&spec, &query, &sink).unwrap();

        assert_eq!(manifest.dataset_id(), spec.id());
        assert_eq!(manifest.version(), spec.version());

        let csv_path = dir.join("ds-export_v1.csv");
        let manifest_path = dir.join("ds-export_v1.manifest.toml");
        assert!(csv_path.is_file());
        assert!(manifest_path.is_file());

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        let mut lines = contents.lines();
        assert_eq!(lines.next(), Some("timestamp,tag,value,quality"));
        assert_eq!(lines.filter(|line| !line.is_empty()).count(), 2);

        std::fs::remove_dir_all(&dir).ok();
    }
}

//! CSV file sink for exported dataset materializations.
//!
//! Writes two files per export under the configured directory:
//!
//! - `{dataset_id}_v{version}.csv` — long-format measurements with columns
//!   `timestamp,tag,value,quality`. Timestamps are RFC 3339 strings; quality uses
//!   the kernel `Display` representation (`Good`, `Uncertain`, `Bad`).
//! - `{dataset_id}_v{version}.manifest.toml` — serialized [`DatasetManifest`].

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use csv::Writer;

use crate::application::ports::{DatasetSinkPort, QueryResult};
use crate::domain::error::{DatasetError, Result};
use crate::domain::manifest::DatasetManifest;

/// Persists dataset exports as CSV plus a sidecar TOML manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvDatasetSink {
    dir: PathBuf,
}

impl CsvDatasetSink {
    /// Creates a sink that writes files under `dir`.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn csv_path(&self, manifest: &DatasetManifest) -> PathBuf {
        self.dir.join(format!(
            "{}_v{}.csv",
            manifest.dataset_id(),
            manifest.version()
        ))
    }

    fn manifest_path(&self, manifest: &DatasetManifest) -> PathBuf {
        self.dir.join(format!(
            "{}_v{}.manifest.toml",
            manifest.dataset_id(),
            manifest.version()
        ))
    }

    fn write_csv(path: &Path, data: &QueryResult) -> Result<()> {
        let file = File::create(path).map_err(|err| config_error(path, err))?;
        let mut writer = Writer::from_writer(file);
        writer
            .write_record(["timestamp", "tag", "value", "quality"])
            .map_err(|err| config_error(path, err))?;

        for series in &data.series {
            for measurement in &series.measurements {
                writer
                    .write_record([
                        measurement.timestamp().to_string(),
                        series.tag.as_str().to_owned(),
                        measurement.value().to_string(),
                        measurement.quality().to_string(),
                    ])
                    .map_err(|err| config_error(path, err))?;
            }
        }

        writer.flush().map_err(|err| config_error(path, err))?;
        Ok(())
    }

    fn write_manifest(path: &Path, manifest: &DatasetManifest) -> Result<()> {
        let contents = manifest.to_toml()?;
        fs::write(path, contents).map_err(|err| config_error(path, err))
    }
}

impl DatasetSinkPort for CsvDatasetSink {
    fn write(&self, manifest: &DatasetManifest, data: &QueryResult) -> Result<()> {
        fs::create_dir_all(&self.dir).map_err(|err| config_error(&self.dir, err))?;

        let csv_path = self.csv_path(manifest);
        Self::write_csv(&csv_path, data)?;

        let manifest_path = self.manifest_path(manifest);
        Self::write_manifest(&manifest_path, manifest)?;

        Ok(())
    }
}

fn config_error(path: impl AsRef<Path>, err: impl std::error::Error) -> DatasetError {
    DatasetError::Config(format!("{}: {err}", path.as_ref().display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::TagSeries;
    use jiff::Timestamp;
    use sp_kernel::{Measurement, Quality, TagId};

    fn ts(seconds: i64) -> Timestamp {
        Timestamp::from_second(seconds).unwrap()
    }

    fn sample_manifest() -> DatasetManifest {
        DatasetManifest::from_spec(
            &crate::domain::dataset_spec::DatasetSpec::define(
                "export-test",
                vec![
                    crate::domain::feature::FeatureSpec::new(
                        TagId::new("PT-1101").unwrap(),
                        "pressure",
                    )
                    .unwrap(),
                ],
                vec![],
                crate::domain::split::DataSplit::new(
                    sp_kernel::TimeWindow::new(ts(100), ts(200)).unwrap(),
                    None,
                    sp_kernel::TimeWindow::new(ts(200), ts(300)).unwrap(),
                )
                .unwrap(),
                &sample_catalog(),
            )
            .unwrap()
            .0,
        )
    }

    fn sample_catalog() -> sp_asset_model::AssetCatalog {
        use sp_asset_model::{
            AreaId, AssetCatalog, DesignSpec, Equipment, EquipmentId, EquipmentKind, Facility,
            FacilityId, Tag, TagSpec, UnitId,
        };
        use sp_kernel::{EngineeringRange, UnitOfMeasure};

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

    fn sample_query_result() -> QueryResult {
        QueryResult {
            series: vec![TagSeries {
                tag: TagId::new("PT-1101").unwrap(),
                measurements: vec![
                    Measurement::new(1.0, Quality::Good, ts(110)),
                    Measurement::new(2.0, Quality::Uncertain, ts(120)),
                    Measurement::new(3.0, Quality::Bad, ts(130)),
                ],
            }],
        }
    }

    #[expect(clippy::disallowed_methods)]
    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn writes_csv_and_manifest_with_expected_row_count() {
        let dir = unique_temp_dir("sp_ml_dataloop-csv-sink");
        let sink = CsvDatasetSink::new(&dir);
        let manifest = sample_manifest();
        let data = sample_query_result();

        sink.write(&manifest, &data).unwrap();

        let csv_path = dir.join("export-test_v1.csv");
        let manifest_path = dir.join("export-test_v1.manifest.toml");
        assert!(csv_path.is_file());
        assert!(manifest_path.is_file());

        let contents = fs::read_to_string(&csv_path).unwrap();
        let mut lines = contents.lines();
        assert_eq!(lines.next(), Some("timestamp,tag,value,quality"));
        assert_eq!(lines.filter(|line| !line.is_empty()).count(), 3);

        fs::remove_dir_all(&dir).ok();
    }
}

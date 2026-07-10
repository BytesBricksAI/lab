//! Parquet file sink for exported dataset materializations.
//!
//! Writes two files per export under the configured directory:
//!
//! - `{dataset_id}_v{version}.parquet` — long-format measurements with columns
//!   `timestamp_nanos`, `tag`, `value`, `quality`. Timestamps are nanoseconds since
//!   the Unix epoch; quality uses the kernel `Display` representation (`Good`,
//!   `Uncertain`, `Bad`).
//! - `{dataset_id}_v{version}.manifest.toml` — serialized [`DatasetManifest`].

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow::array::{Float64Array, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;

use crate::application::ports::{DatasetSinkPort, QueryResult};
use crate::domain::error::{DatasetError, Result};
use crate::domain::manifest::DatasetManifest;

/// Persists dataset exports as Parquet plus a sidecar TOML manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParquetDatasetSink {
    dir: PathBuf,
}

impl ParquetDatasetSink {
    /// Creates a sink that writes files under `dir`.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn parquet_path(&self, manifest: &DatasetManifest) -> PathBuf {
        self.dir.join(format!(
            "{}_v{}.parquet",
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

    fn write_parquet(path: &Path, data: &QueryResult) -> Result<()> {
        let mut timestamp_nanos = Vec::new();
        let mut tags = Vec::new();
        let mut values = Vec::new();
        let mut qualities = Vec::new();

        for series in &data.series {
            for measurement in &series.measurements {
                let nanos = measurement.timestamp().as_nanosecond();
                let nanos_i64 = i64::try_from(nanos).map_err(|err| {
                    DatasetError::Config(format!("timestamp out of range: {nanos} ({err})"))
                })?;
                timestamp_nanos.push(nanos_i64);
                tags.push(series.tag.as_str().to_owned());
                values.push(measurement.value());
                qualities.push(measurement.quality().to_string());
            }
        }

        let schema = Arc::new(Schema::new_with_metadata(
            vec![
                Field::new("timestamp_nanos", DataType::Int64, false),
                Field::new("tag", DataType::Utf8, false),
                Field::new("value", DataType::Float64, false),
                Field::new("quality", DataType::Utf8, false),
            ],
            Default::default(),
        ));

        let batch = RecordBatch::try_new_with_options(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(timestamp_nanos)),
                Arc::new(StringArray::from(tags)),
                Arc::new(Float64Array::from(values)),
                Arc::new(StringArray::from(qualities)),
            ],
            &Default::default(),
        )
        .map_err(|err| config_error(path, err))?;

        let file = File::create(path).map_err(|err| config_error(path, err))?;
        let mut writer =
            ArrowWriter::try_new(file, schema, None).map_err(|err| config_error(path, err))?;
        writer
            .write(&batch)
            .map_err(|err| config_error(path, err))?;
        writer.close().map_err(|err| config_error(path, err))?;

        Ok(())
    }

    fn write_manifest(path: &Path, manifest: &DatasetManifest) -> Result<()> {
        let contents = manifest.to_toml()?;
        fs::write(path, contents).map_err(|err| config_error(path, err))
    }
}

impl DatasetSinkPort for ParquetDatasetSink {
    fn write(&self, manifest: &DatasetManifest, data: &QueryResult) -> Result<()> {
        fs::create_dir_all(&self.dir).map_err(|err| config_error(&self.dir, err))?;

        let parquet_path = self.parquet_path(manifest);
        Self::write_parquet(&parquet_path, data)?;

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
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
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
    fn writes_parquet_and_manifest_with_expected_row_count() {
        let dir = unique_temp_dir("sp_ml_dataloop-parquet-sink");
        let sink = ParquetDatasetSink::new(&dir);
        let manifest = sample_manifest();
        let data = sample_query_result();

        sink.write(&manifest, &data).unwrap();

        let parquet_path = dir.join("export-test_v1.parquet");
        let manifest_path = dir.join("export-test_v1.manifest.toml");
        assert!(parquet_path.is_file());
        assert!(manifest_path.is_file());

        let file = File::open(&parquet_path).unwrap();
        let builder = ParquetRecordBatchReaderBuilder::try_new(file).unwrap();
        let mut reader = builder.build().unwrap();
        let batch = reader.next().unwrap().unwrap();
        assert_eq!(batch.num_rows(), 3);
        assert!(reader.next().is_none());

        fs::remove_dir_all(&dir).ok();
    }
}

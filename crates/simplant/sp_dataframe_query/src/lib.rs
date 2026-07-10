//! `SimPlant` Lab dataframe query adapter.
//!
//! Implements [`sp_ml_dataloop::DataframeQueryPort`] by querying `.rrd` recordings through
//! [`re_dataframe::QueryEngine`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use arrow::array::{
    Array, Float64Array, Int64Array, ListArray, StringArray, TimestampNanosecondArray,
};
use arrow::datatypes::Schema;
use jiff::Timestamp;
use re_chunk_store::{ChunkStoreConfig, QueryExpression, ViewContentsSelector};
use re_dataframe::{
    AbsoluteTimeRange, EntityPath, QueryEngine, SparseFillStrategy, StorageEngine, TimeInt,
    TimelineName,
};
use sp_kernel::{Measurement, Quality, TagId, TimeWindow};
use sp_ml_dataloop::{DataframeQueryPort, DatasetError, QueryResult, Result, TagSeries};
use sp_recording::{PLANT_TIME, tag_entity_path};
use sp_types::namespace::{ARCHETYPE_PROCESS_VARIABLE, COMPONENT_QUALITY};

const METADATA_INDEX_NAME: &str = "rerun:index_name";
const METADATA_ENTITY_PATH: &str = "rerun:entity_path";
const METADATA_COMPONENT: &str = "rerun:component";
const METADATA_COMPONENT_TYPE: &str = "rerun:component_type";
const SCALARS_COMPONENT: &str = "Scalars:scalars";

/// Queries process-variable samples from a `.rrd` file via [`re_dataframe`].
pub struct RrdDataframeQuery {
    path: PathBuf,
    engines: Vec<QueryEngine<StorageEngine>>,
}

impl RrdDataframeQuery {
    /// Opens a recording file and prepares query engines for its recording stores.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let engines_map = QueryEngine::from_rrd_filepath(&ChunkStoreConfig::DEFAULT, &path)
            .map_err(|err| config_error(&path, &err))?;

        let engines = engines_map
            .into_iter()
            .filter(|(store_id, _)| store_id.is_recording())
            .map(|(_, engine)| engine)
            .collect::<Vec<_>>();

        if engines.is_empty() {
            return Err(DatasetError::Config(format!(
                "{}: no recording store found",
                path.display()
            )));
        }

        Ok(Self { path, engines })
    }

    fn query_engine(&self) -> Result<&QueryEngine<StorageEngine>> {
        self.engines.first().ok_or_else(|| {
            DatasetError::Config(format!(
                "{}: no recording store available",
                self.path.display()
            ))
        })
    }
}

impl DataframeQueryPort for RrdDataframeQuery {
    fn query(&self, window: &TimeWindow, tags: &[TagId]) -> Result<QueryResult> {
        if tags.is_empty() {
            return Ok(QueryResult::default());
        }

        let engine = self.query_engine()?;
        let timeline = TimelineName::new(PLANT_TIME);
        let time_range = window_to_time_range(window)?;

        let view_contents = tags
            .iter()
            .map(|tag| (EntityPath::from(tag_entity_path(tag).as_str()), None))
            .collect::<ViewContentsSelector>();

        let query = QueryExpression {
            filtered_index: Some(timeline),
            view_contents: Some(view_contents),
            filtered_index_range: Some(time_range),
            sparse_fill_strategy: SparseFillStrategy::None,
            ..Default::default()
        };

        let mut measurements_by_tag = tags
            .iter()
            .map(|tag| (tag.clone(), Vec::new()))
            .collect::<HashMap<_, _>>();

        let mut query_handle = engine.query(query);
        let schema = query_handle.schema().clone();
        let column_map = ColumnMap::from_schema(&schema);

        for batch in query_handle.batch_iter() {
            let plant_time_idx = column_map.plant_time;

            for row in 0..batch.num_rows() {
                let Some(nanos) = plant_time_idx
                    .map(|idx| batch.column(idx))
                    .and_then(|array| read_plant_time_nanos(array.as_ref(), row))
                else {
                    continue;
                };
                let Ok(timestamp) = nanos_to_timestamp(nanos) else {
                    continue;
                };
                if !window.contains(timestamp) {
                    continue;
                }

                for (entity_path, columns) in &column_map.entities {
                    let Some(tag) = tag_from_entity_path(entity_path) else {
                        continue;
                    };
                    if !measurements_by_tag.contains_key(&tag) {
                        continue;
                    }

                    let value = columns
                        .value
                        .map(|idx| batch.column(idx))
                        .and_then(|array| list_first_f64(array.as_ref(), row));
                    let quality_text = columns
                        .quality
                        .map(|idx| batch.column(idx))
                        .and_then(|array| list_first_utf8(array.as_ref(), row));

                    let (Some(value), Some(quality_text)) = (value, quality_text) else {
                        continue;
                    };

                    measurements_by_tag
                        .get_mut(&tag)
                        .expect("tag initialized above")
                        .push(Measurement::new(
                            value,
                            quality_from_text(quality_text),
                            timestamp,
                        ));
                }
            }
        }

        let mut series = Vec::with_capacity(tags.len());
        for tag in tags {
            let mut measurements = measurements_by_tag.remove(tag).unwrap_or_default();
            measurements.sort_by_key(|sample| sample.timestamp());
            series.push(TagSeries {
                tag: tag.clone(),
                measurements,
            });
        }

        Ok(QueryResult { series })
    }
}

#[derive(Debug, Default)]
struct EntityColumns {
    value: Option<usize>,
    quality: Option<usize>,
}

#[derive(Debug, Default)]
struct ColumnMap {
    plant_time: Option<usize>,
    entities: HashMap<String, EntityColumns>,
}

impl ColumnMap {
    fn from_schema(schema: &Schema) -> Self {
        let mut map = Self::default();

        for (idx, field) in schema.fields().iter().enumerate() {
            let metadata = field.metadata();

            if metadata
                .get(METADATA_INDEX_NAME)
                .is_some_and(|name| name == PLANT_TIME)
            {
                map.plant_time = Some(idx);
                continue;
            }

            let Some(entity_path) = metadata.get(METADATA_ENTITY_PATH) else {
                continue;
            };

            let component = metadata.get(METADATA_COMPONENT).map(String::as_str);
            let component_type = metadata.get(METADATA_COMPONENT_TYPE).map(String::as_str);

            let entry = map.entities.entry(entity_path.clone()).or_default();

            if component.is_some_and(|name| name == SCALARS_COMPONENT) {
                entry.value = Some(idx);
            } else if component_type.is_some_and(|name| name == COMPONENT_QUALITY)
                || component
                    .is_some_and(|name| name == format!("{ARCHETYPE_PROCESS_VARIABLE}:quality"))
            {
                entry.quality = Some(idx);
            }
        }

        map
    }
}

fn window_to_time_range(window: &TimeWindow) -> Result<AbsoluteTimeRange> {
    let start_nanos = timestamp_to_nanos(window.start())?;
    let end_nanos = timestamp_to_nanos(window.end())?;

    let max_nanos = if end_nanos > i64::MIN {
        end_nanos.saturating_sub(1)
    } else {
        i64::MIN
    };

    if max_nanos < start_nanos {
        return Ok(AbsoluteTimeRange::EMPTY);
    }

    Ok(AbsoluteTimeRange::new(
        TimeInt::new_temporal(start_nanos),
        TimeInt::new_temporal(max_nanos),
    ))
}

fn timestamp_to_nanos(timestamp: Timestamp) -> Result<i64> {
    let nanos = timestamp.as_nanosecond();
    i64::try_from(nanos)
        .map_err(|err| DatasetError::Config(format!("timestamp out of range: {nanos} ({err})")))
}

fn nanos_to_timestamp(nanos: i64) -> Result<Timestamp> {
    Timestamp::from_nanosecond(i128::from(nanos))
        .map_err(|err| DatasetError::Config(err.to_string()))
}

fn tag_from_entity_path(entity_path: &str) -> Option<TagId> {
    let stripped = entity_path.trim_start_matches('/');
    let tag_id = stripped.strip_prefix("tags/")?;
    TagId::new(tag_id).ok()
}

fn quality_from_text(text: &str) -> Quality {
    match text {
        "Bad" => Quality::Bad,
        "Uncertain" => Quality::Uncertain,
        _ => Quality::Good,
    }
}

fn read_plant_time_nanos(array: &dyn Array, row: usize) -> Option<i64> {
    if let Some(values) = array.as_any().downcast_ref::<TimestampNanosecondArray>() {
        if values.is_null(row) {
            None
        } else {
            Some(values.value(row))
        }
    } else if let Some(values) = array.as_any().downcast_ref::<Int64Array>() {
        read_i64(values, row)
    } else {
        None
    }
}

fn read_i64(array: &Int64Array, row: usize) -> Option<i64> {
    if array.is_null(row) {
        None
    } else {
        Some(array.value(row))
    }
}

fn list_first_f64(array: &dyn Array, row: usize) -> Option<f64> {
    let list = array.as_any().downcast_ref::<ListArray>()?;
    if list.is_null(row) {
        return None;
    }

    let offsets = list.value_offsets();
    let start = offsets[row] as usize;
    let end = offsets[row + 1] as usize;
    if start >= end {
        return None;
    }

    let values = list.values();
    let floats = values.as_any().downcast_ref::<Float64Array>()?;
    if floats.is_null(start) {
        None
    } else {
        Some(floats.value(start))
    }
}

fn list_first_utf8(array: &dyn Array, row: usize) -> Option<&str> {
    let list = array.as_any().downcast_ref::<ListArray>()?;
    if list.is_null(row) {
        return None;
    }

    let offsets = list.value_offsets();
    let start = offsets[row] as usize;
    let end = offsets[row + 1] as usize;
    if start >= end {
        return None;
    }

    let values = list.values();
    let strings = values.as_any().downcast_ref::<StringArray>()?;
    if strings.is_null(start) {
        None
    } else {
        Some(strings.value(start))
    }
}

fn config_error(path: &Path, err: impl std::fmt::Display) -> DatasetError {
    DatasetError::Config(format!("{}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr as _;

    use sp_acquisition::RecorderPort as _;
    use sp_kernel::MeasurementBatch;

    use super::*;

    fn ts(text: &str) -> Timestamp {
        Timestamp::from_str(text).expect("timestamp")
    }

    #[expect(clippy::disallowed_methods)]
    fn unique_rrd_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "sp_dataframe_query-{name}-{}-{}.rrd",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ))
    }

    fn write_fixture_rrd(path: &Path) {
        let tag_a = TagId::new("PT-1101").expect("tag a");
        let tag_b = TagId::new("FT-2201").expect("tag b");

        let ts1 = ts("2026-01-01T00:00:00Z");
        let ts2 = ts("2026-01-01T00:00:01Z");
        let ts3 = ts("2026-01-01T00:00:02Z");

        {
            let recorder = sp_recording::RerunRecorder::to_file("sp_dataframe_query_test", path)
                .expect("create recorder");

            recorder
                .record_batch(&MeasurementBatch::new(
                    tag_a.clone(),
                    vec![
                        Measurement::new(10.0, Quality::Good, ts1),
                        Measurement::new(11.5, Quality::Uncertain, ts2),
                    ],
                ))
                .expect("record tag a");

            recorder
                .record_batch(&MeasurementBatch::new(
                    tag_b.clone(),
                    vec![Measurement::new(42.0, Quality::Bad, ts3)],
                ))
                .expect("record tag b");

            recorder.flush();
        }
    }

    #[test]
    fn query_returns_measurements_for_requested_tags() {
        let path = unique_rrd_path("integration");
        write_fixture_rrd(&path);

        let query = RrdDataframeQuery::open(&path).expect("open rrd");
        let tag_a = TagId::new("PT-1101").expect("tag a");
        let tag_b = TagId::new("FT-2201").expect("tag b");
        let window = TimeWindow::new(ts("2026-01-01T00:00:00Z"), ts("2026-01-01T00:00:03Z"))
            .expect("window");

        let result = query
            .query(&window, &[tag_a.clone(), tag_b.clone()])
            .expect("query");

        assert_eq!(result.series.len(), 2);

        let series_a = result
            .series
            .iter()
            .find(|series| series.tag == tag_a)
            .expect("tag a series");
        assert_eq!(
            series_a.measurements,
            vec![
                Measurement::new(10.0, Quality::Good, ts("2026-01-01T00:00:00Z")),
                Measurement::new(11.5, Quality::Uncertain, ts("2026-01-01T00:00:01Z")),
            ]
        );

        let series_b = result
            .series
            .iter()
            .find(|series| series.tag == tag_b)
            .expect("tag b series");
        assert_eq!(
            series_b.measurements,
            vec![Measurement::new(
                42.0,
                Quality::Bad,
                ts("2026-01-01T00:00:02Z")
            )]
        );

        fs::remove_file(path).ok();
    }

    #[test]
    fn query_returns_empty_series_for_tag_without_data_in_window() {
        let path = unique_rrd_path("missing-tag");
        write_fixture_rrd(&path);

        let query = RrdDataframeQuery::open(&path).expect("open rrd");
        let present = TagId::new("PT-1101").expect("present tag");
        let missing = TagId::new("TT-9999").expect("missing tag");
        let window = TimeWindow::new(ts("2026-01-01T00:00:00Z"), ts("2026-01-01T00:00:03Z"))
            .expect("window");

        let result = query
            .query(&window, &[present, missing.clone()])
            .expect("query");

        assert_eq!(result.series.len(), 2);

        let missing_series = result
            .series
            .iter()
            .find(|series| series.tag == missing)
            .expect("missing tag series");
        assert!(missing_series.measurements.is_empty());

        fs::remove_file(path).ok();
    }

    #[test]
    fn query_returns_empty_measurements_when_window_excludes_all_samples() {
        let path = unique_rrd_path("empty-window");
        write_fixture_rrd(&path);

        let query = RrdDataframeQuery::open(&path).expect("open rrd");
        let tag = TagId::new("PT-1101").expect("tag");
        let window = TimeWindow::new(ts("2026-01-01T00:01:00Z"), ts("2026-01-01T00:02:00Z"))
            .expect("window");

        let result = query
            .query(&window, std::slice::from_ref(&tag))
            .expect("query");

        assert_eq!(result.series.len(), 1);
        assert_eq!(result.series[0].tag, tag);
        assert!(result.series[0].measurements.is_empty());

        fs::remove_file(path).ok();
    }
}

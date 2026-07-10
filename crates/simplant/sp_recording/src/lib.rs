//! `SimPlant` Lab recording adapter.
//!
//! Implements [`sp_acquisition::RecorderPort`] over [`re_sdk::RecordingStream`].
//! This crate is the anti-corruption boundary between domain types and the Rerun store.

mod entity_path;
mod recorder;

pub use entity_path::{EVENTS_PATH, PLANT_TIME, tag_entity_path};
pub use recorder::RerunRecorder;

#[cfg(test)]
mod tests {
    use sp_acquisition::{AcquisitionEvent, AcquisitionStarted, RecorderPort as _};
    use sp_asset_model::{EquipmentId, Tag, TagSpec};
    use sp_kernel::{
        AlarmLimits, EngineeringRange, Measurement, MeasurementBatch, Quality, TagId, UnitOfMeasure,
    };
    use std::fs;
    use std::path::PathBuf;

    use super::RerunRecorder;

    #[test]
    #[expect(clippy::disallowed_methods)]
    fn records_metadata_batch_and_event_to_rrd() {
        let path: PathBuf =
            std::env::temp_dir().join(format!("sp_recording_test_{}.rrd", std::process::id()));

        let recorder = RerunRecorder::to_file("simplant_test", &path).expect("create recorder");

        let spec = TagSpec {
            id: TagId::new("PT-1101").expect("tag id"),
            equipment: EquipmentId::new("EQ-101").expect("equipment id"),
            description: "Column pressure".to_owned(),
            unit: UnitOfMeasure::Bar,
            range: EngineeringRange::new(0.0, 100.0, UnitOfMeasure::Bar).expect("range"),
            alarms: Some(
                AlarmLimits::new(None, Some(10.0), Some(90.0), None, UnitOfMeasure::Bar)
                    .expect("alarms"),
            ),
        };
        let (tag, _) = Tag::define(spec).expect("define tag");
        recorder
            .record_tag_metadata(&tag)
            .expect("record tag metadata");

        use std::str::FromStr as _;

        let ts1 = jiff::Timestamp::from_str("2026-01-01T00:00:00Z").expect("ts1");
        let ts2 = jiff::Timestamp::from_str("2026-01-01T00:00:01Z").expect("ts2");
        let batch = MeasurementBatch::new(
            TagId::new("PT-1101").expect("tag id"),
            vec![
                Measurement::new(42.0, Quality::Good, ts1),
                Measurement::new(43.5, Quality::Good, ts2),
            ],
        );
        recorder.record_batch(&batch).expect("record batch");

        let event = AcquisitionEvent::Started(AcquisitionStarted {
            session: "test-session".to_owned(),
            tag_count: 1,
        });
        recorder.record_event(&event).expect("record event");

        recorder.flush();

        let metadata = fs::metadata(&path).expect("rrd file metadata");
        assert!(metadata.len() > 0, "rrd file should be non-empty");

        fs::remove_file(&path).ok();
    }

    /// Control-plane events have no plant-side timestamp, so they must not inherit
    /// the `plant_time` left "sticky" on the stream by the preceding sample batch.
    /// Otherwise a dataframe view indexed by `plant_time` shows phantom event rows.
    #[test]
    fn events_are_not_anchored_to_plant_time() {
        use re_sdk::RecordingStreamBuilder;
        use re_sdk::log::{Chunk, LogMsg};
        use std::str::FromStr as _;

        let (stream, storage) = RecordingStreamBuilder::new("rerun_example_simplant_test")
            .memory()
            .expect("memory stream");
        let recorder = RerunRecorder::new(stream);

        // Logging a sample batch sets `plant_time` and leaves it sticky on the stream.
        let ts = jiff::Timestamp::from_str("2026-01-01T00:29:00Z").expect("ts");
        let batch = MeasurementBatch::new(
            TagId::new("LT-101").expect("tag id"),
            vec![Measurement::new(70.0, Quality::Good, ts)],
        );
        recorder.record_batch(&batch).expect("record batch");

        // A control-plane event logged afterwards must not be anchored to plant_time.
        let event = AcquisitionEvent::Started(AcquisitionStarted {
            session: "test-session".to_owned(),
            tag_count: 1,
        });
        recorder.record_event(&event).expect("record event");

        recorder.flush();

        let mut event_has_plant_time: Option<bool> = None;
        let mut tag_has_plant_time = false;
        for msg in &storage.take() {
            let LogMsg::ArrowMsg(_, arrow_msg) = msg else {
                continue;
            };
            let chunk = Chunk::from_arrow_msg(arrow_msg).expect("decode chunk");
            let entity = chunk.entity_path().to_string();
            let entity = entity.trim_start_matches('/');
            let on_plant_time = chunk
                .timelines()
                .keys()
                .any(|name| name.as_str() == super::PLANT_TIME);

            if entity == super::EVENTS_PATH {
                event_has_plant_time = Some(on_plant_time);
            } else if entity.starts_with("tags/") && on_plant_time {
                tag_has_plant_time = true;
            }
        }

        assert!(
            tag_has_plant_time,
            "sanity: tag samples must live on the plant_time timeline"
        );
        assert_eq!(
            event_has_plant_time,
            Some(false),
            "control-plane events must not be anchored to plant_time"
        );
    }
}

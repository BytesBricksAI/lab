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
}

//! `SimPlant` Lab CSV replay adapter.
//!
//! Implements [`sp_acquisition::DataSourcePort`] by replaying historian CSV exports.
//! This crate has no `re_*` dependencies.

mod csv_source;

pub use csv_source::CsvReplaySource;

#[cfg(test)]
mod tests {
    use sp_acquisition::{AcquisitionError, DataSourcePort as _, SamplingPolicy, TagBinding};
    use sp_kernel::TagId;
    use std::fs;
    use std::io::Write as _;
    use std::path::PathBuf;

    use super::CsvReplaySource;

    fn write_test_csv(path: &PathBuf, contents: &str) {
        let mut file = fs::File::create(path).expect("create csv");
        file.write_all(contents.as_bytes()).expect("write csv");
    }

    #[test]
    #[expect(clippy::disallowed_methods)]
    fn replays_csv_into_measurement_batch() {
        let path: PathBuf = std::env::temp_dir().join(format!(
            "sp_acquisition_replay_test_{}.csv",
            std::process::id()
        ));

        write_test_csv(
            &path,
            "timestamp,pt_col\n\
             2026-01-01T00:00:00Z,10.5\n\
             2026-01-01T00:00:01Z,11.0\n\
             2026-01-01T00:00:02Z,11.5\n",
        );

        let source = CsvReplaySource::new(&path);
        let tag = TagId::new("PT-1101").expect("tag id");
        let binding = TagBinding::new(tag, "pt_col").expect("binding");

        let mut stream = source
            .subscribe(&[binding], &SamplingPolicy::default())
            .expect("subscribe");

        let batch = stream.next_batch().expect("next batch").expect("one batch");
        assert_eq!(batch.tag().as_str(), "PT-1101");
        assert_eq!(batch.samples().len(), 3);
        assert!((batch.samples()[0].value() - 10.5).abs() < f64::EPSILON);
        assert!((batch.samples()[1].value() - 11.0).abs() < f64::EPSILON);
        assert!((batch.samples()[2].value() - 11.5).abs() < f64::EPSILON);
        assert_eq!(
            batch.samples()[0].timestamp().to_string(),
            "2026-01-01T00:00:00Z"
        );
        assert_eq!(
            batch.samples()[1].timestamp().to_string(),
            "2026-01-01T00:00:01Z"
        );
        assert_eq!(
            batch.samples()[2].timestamp().to_string(),
            "2026-01-01T00:00:02Z"
        );
        assert!(stream.next_batch().expect("exhausted").is_none());

        fs::remove_file(&path).ok();
    }

    #[test]
    #[expect(clippy::disallowed_methods)]
    fn missing_column_returns_source_error() {
        let path: PathBuf = std::env::temp_dir().join(format!(
            "sp_acquisition_replay_missing_col_{}.csv",
            std::process::id()
        ));

        write_test_csv(&path, "timestamp,pt_col\n2026-01-01T00:00:00Z,1.0\n");

        let source = CsvReplaySource::new(&path);
        let tag = TagId::new("PT-1101").expect("tag id");
        let binding = TagBinding::new(tag, "missing_col").expect("binding");

        let Err(err) = source.subscribe(&[binding], &SamplingPolicy::default()) else {
            panic!("expected missing column error");
        };

        assert_eq!(
            err,
            AcquisitionError::Source("missing column: missing_col".to_owned())
        );

        fs::remove_file(&path).ok();
    }
}

//! [`RerunRecorder`]: [`RecorderPort`] implementation backed by [`re_sdk::RecordingStream`].

use re_sdk::RecordingStream;
use re_sdk_types::archetypes::TextLog;
use sp_acquisition::{AcquisitionError, AcquisitionEvent, RecorderPort, Result};
use sp_asset_model::Tag;
use sp_kernel::MeasurementBatch;
use sp_types::{ProcessVariableSample, Quality, TagMetadata};

use crate::entity_path::{EVENTS_PATH, PLANT_TIME, tag_entity_path};

/// Records domain data to a Rerun `.rrd` file via [`RecordingStream`].
pub struct RerunRecorder {
    rec: RecordingStream,
}

impl RerunRecorder {
    /// Wraps an existing recording stream.
    pub fn new(rec: RecordingStream) -> Self {
        Self { rec }
    }

    /// Creates a recorder that writes to `path` under the given `app_id`.
    pub fn to_file(app_id: &str, path: impl AsRef<std::path::Path>) -> Result<Self> {
        re_sdk::RecordingStreamBuilder::new(app_id)
            .save(path.as_ref().to_path_buf())
            .map(Self::new)
            .map_err(|err| AcquisitionError::Recording(err.to_string()))
    }

    /// Flushes pending data to the underlying sink.
    pub fn flush(&self) {
        self.rec.flush_blocking().ok();
    }
}

impl RecorderPort for RerunRecorder {
    fn record_batch(&self, batch: &MeasurementBatch) -> Result<()> {
        let entity_path = tag_entity_path(batch.tag());

        for sample in batch.samples() {
            let nanos = sample.timestamp().as_nanosecond();
            let nanos_i64 = i64::try_from(nanos).map_err(|err| {
                AcquisitionError::Recording(format!("timestamp out of range: {nanos} ({err})"))
            })?;

            self.rec
                .set_timestamp_nanos_since_epoch(PLANT_TIME, nanos_i64);

            let archetype = ProcessVariableSample {
                value: sample.value(),
                quality: Quality::from(sample.quality()),
            };

            self.rec
                .log(entity_path.as_str(), &archetype)
                .map_err(|err| AcquisitionError::Recording(err.to_string()))?;
        }

        Ok(())
    }

    fn record_tag_metadata(&self, tag: &Tag) -> Result<()> {
        let mut metadata = TagMetadata::new(tag.unit(), tag.range().low(), tag.range().high());

        if let Some(alarms) = tag.alarms() {
            if let Some(low) = alarms.low() {
                metadata = metadata.with_alarm_low(low);
            }
            if let Some(high) = alarms.high() {
                metadata = metadata.with_alarm_high(high);
            }
        }

        self.rec
            .log_static(tag_entity_path(tag.id()).as_str(), &metadata)
            .map_err(|err| AcquisitionError::Recording(err.to_string()))
    }

    fn record_event(&self, event: &AcquisitionEvent) -> Result<()> {
        self.rec
            .log(EVENTS_PATH, &TextLog::new(format!("{event:?}")))
            .map_err(|err| AcquisitionError::Recording(err.to_string()))
    }
}

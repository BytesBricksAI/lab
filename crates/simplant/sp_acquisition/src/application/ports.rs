use sp_asset_model::Tag;
use sp_kernel::MeasurementBatch;

use crate::domain::binding::TagBinding;
use crate::domain::error::Result;
use crate::domain::events::AcquisitionEvent;
use crate::domain::sampling::SamplingPolicy;

/// Driven port: a read-only source of process measurements.
///
/// DELIBERATELY has no write capability (OT safety: no write-back to PLCs/DCS).
pub trait DataSourcePort {
    /// Subscribe to the given bindings; returns a pull-based measurement source.
    fn subscribe(
        &self,
        bindings: &[TagBinding],
        policy: &SamplingPolicy,
    ) -> Result<Box<dyn MeasurementSource>>;
}

/// A pull-based stream of measurement batches.
pub trait MeasurementSource {
    /// Returns the next batch, `Ok(None)` when the source is exhausted (replay),
    /// or `Err` on source loss.
    fn next_batch(&mut self) -> Result<Option<MeasurementBatch>>;
}

/// Driven port: persists domain data to the recording store.
pub trait RecorderPort {
    /// Record a batch of measurements (data plane).
    fn record_batch(&self, batch: &MeasurementBatch) -> Result<()>;

    /// Record a tag's static metadata (unit, range, alarms).
    fn record_tag_metadata(&self, tag: &Tag) -> Result<()>;

    /// Record a control-plane domain event.
    fn record_event(&self, event: &AcquisitionEvent) -> Result<()>;
}

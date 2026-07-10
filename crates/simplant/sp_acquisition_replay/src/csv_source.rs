//! CSV historian replay source.

use std::path::PathBuf;
use std::str::FromStr as _;

use jiff::Timestamp;
use sp_acquisition::{
    AcquisitionError, DataSourcePort, MeasurementSource, Result, SamplingPolicy, TagBinding,
};
use sp_kernel::{Measurement, MeasurementBatch, Quality};

/// Replays measurements from a historian CSV export.
pub struct CsvReplaySource {
    path: PathBuf,
}

impl CsvReplaySource {
    /// Creates a replay source for the CSV at `path`.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

struct VecBatchSource {
    batches: Vec<MeasurementBatch>,
    index: usize,
}

impl MeasurementSource for VecBatchSource {
    fn next_batch(&mut self) -> Result<Option<MeasurementBatch>> {
        if self.index >= self.batches.len() {
            return Ok(None);
        }

        let batch = self.batches[self.index].clone();
        self.index += 1;
        Ok(Some(batch))
    }
}

impl DataSourcePort for CsvReplaySource {
    fn subscribe(
        &self,
        bindings: &[TagBinding],
        _policy: &SamplingPolicy,
    ) -> Result<Box<dyn MeasurementSource>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(&self.path)
            .map_err(|err| AcquisitionError::Source(err.to_string()))?;

        let headers = reader
            .headers()
            .map_err(|err| AcquisitionError::Source(err.to_string()))?
            .clone();

        let timestamp_idx = headers
            .iter()
            .position(|header| header == "timestamp")
            .ok_or_else(|| AcquisitionError::Source("missing column: timestamp".to_owned()))?;

        let column_indices: Vec<usize> = bindings
            .iter()
            .map(|binding| {
                let address = binding.address();
                headers
                    .iter()
                    .position(|header| header == address)
                    .ok_or_else(|| AcquisitionError::Source(format!("missing column: {address}")))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut samples_by_binding: Vec<Vec<Measurement>> = vec![Vec::new(); bindings.len()];

        for record in reader.records() {
            let record = record.map_err(|err| AcquisitionError::Source(err.to_string()))?;

            let timestamp_str = record
                .get(timestamp_idx)
                .ok_or_else(|| AcquisitionError::Source("missing timestamp cell".to_owned()))?;
            let timestamp = parse_timestamp(timestamp_str)?;

            for (binding_idx, column_idx) in column_indices.iter().enumerate() {
                let cell = record.get(*column_idx).unwrap_or("");
                if cell.trim().is_empty() {
                    continue;
                }

                let value = cell
                    .parse::<f64>()
                    .map_err(|err| AcquisitionError::Source(err.to_string()))?;

                samples_by_binding[binding_idx].push(Measurement::new(
                    value,
                    Quality::Good,
                    timestamp,
                ));
            }
        }

        let batches = bindings
            .iter()
            .zip(samples_by_binding)
            .map(|(binding, samples)| MeasurementBatch::new(binding.tag().clone(), samples))
            .collect();

        Ok(Box::new(VecBatchSource { batches, index: 0 }))
    }
}

fn parse_timestamp(raw: &str) -> Result<Timestamp> {
    let trimmed = raw.trim();

    if let Ok(timestamp) = Timestamp::from_str(trimmed) {
        return Ok(timestamp);
    }

    let seconds = trimmed
        .parse::<f64>()
        .map_err(|err| AcquisitionError::Source(err.to_string()))?;

    if !seconds.is_finite() {
        return Err(AcquisitionError::Source(format!(
            "invalid timestamp: {trimmed}"
        )));
    }

    let nanos = (seconds * 1_000_000_000.0).round() as i128;
    Timestamp::from_nanosecond(nanos).map_err(|err| AcquisitionError::Source(err.to_string()))
}

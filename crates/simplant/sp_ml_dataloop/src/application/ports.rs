//! Driven ports for querying and persisting dataset materializations.

use sp_kernel::{Measurement, TagId, TimeWindow};

use crate::domain::error::Result;
use crate::domain::manifest::DatasetManifest;

/// One tag's measurements returned by a query.
#[derive(Debug, Clone, PartialEq)]
pub struct TagSeries {
    /// Process tag identifier.
    pub tag: TagId,
    /// Measurements in the requested window.
    pub measurements: Vec<Measurement>,
}

/// Result of a dataframe query: one series per requested tag.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct QueryResult {
    /// One time series per requested tag.
    pub series: Vec<TagSeries>,
}

/// Driven port: query measurements from the store for a window and a set of tags.
pub trait DataframeQueryPort {
    /// Returns measurements for each tag within the given time window.
    fn query(&self, window: &TimeWindow, tags: &[TagId]) -> Result<QueryResult>;
}

/// Driven port: persist a dataset (manifest + data) to a versioned sink (e.g. Parquet).
pub trait DatasetSinkPort {
    /// Writes the manifest and queried data to the sink.
    fn write(&self, manifest: &DatasetManifest, data: &QueryResult) -> Result<()>;
}

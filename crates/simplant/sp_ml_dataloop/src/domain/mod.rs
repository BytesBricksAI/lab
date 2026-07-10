//! Domain model for versioned ML datasets and leakage-free splits.

pub mod dataset_spec;
pub mod error;
pub mod events;
pub mod feature;
pub mod manifest;
pub mod split;

pub use dataset_spec::DatasetSpec;
pub use error::{DatasetError, Result};
pub use events::{DatasetEvent, DatasetPublished};
pub use feature::FeatureSpec;
pub use manifest::{DatasetManifest, SCHEMA_VERSION};
pub use split::DataSplit;

//! Public API surface for `sp_ml_dataloop`.

pub use crate::application::export::export_dataset;
pub use crate::application::ports::{DataframeQueryPort, DatasetSinkPort, QueryResult, TagSeries};
pub use crate::domain::{
    DataSplit, DatasetError, DatasetEvent, DatasetManifest, DatasetPublished, DatasetSpec,
    FeatureSpec, Result, SCHEMA_VERSION,
};
pub use crate::infrastructure::csv_sink::CsvDatasetSink;
pub use crate::infrastructure::parquet_sink::ParquetDatasetSink;
pub use crate::infrastructure::toml_spec::{
    dataset_spec_from_str, load_dataset_spec, save_dataset_spec,
};

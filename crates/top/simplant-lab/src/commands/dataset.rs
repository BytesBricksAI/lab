use std::path::{Path, PathBuf};

use anyhow::Context as _;
use sp_asset_model::{AssetCatalog, AssetCatalogPort as _, TomlCatalogRepository};
use sp_dataframe_query::RrdDataframeQuery;
use sp_ml_dataloop::{ParquetDatasetSink, export_dataset, load_dataset_spec};

/// Export a dataset (Parquet + manifest) from an `.rrd` recording.
#[derive(Debug, Clone, clap::Parser)]
pub struct DatasetCommand {
    /// Path to the dataset specification TOML file.
    #[arg(long = "spec")]
    spec: PathBuf,

    /// Path to the TOML asset catalog.
    #[arg(long = "catalog")]
    catalog: PathBuf,

    /// Path to the input `.rrd` recording.
    #[arg(long = "rrd")]
    rrd: PathBuf,

    /// Output directory for the Parquet file and manifest.
    #[arg(long = "out")]
    out: PathBuf,
}

impl DatasetCommand {
    pub fn run(self) -> anyhow::Result<()> {
        let (parquet_path, manifest_path) =
            run_dataset_export(&self.spec, &self.catalog, &self.rrd, &self.out)?;

        println!("Parquet file:  {}", absolute_path(&parquet_path).display());
        println!("Manifest file: {}", absolute_path(&manifest_path).display());

        Ok(())
    }
}

/// Export a dataset from an `.rrd` recording using a TOML spec and asset catalog.
pub(crate) fn run_dataset_export(
    spec_path: &Path,
    catalog_path: &Path,
    rrd_path: &Path,
    output_dir: &Path,
) -> anyhow::Result<(PathBuf, PathBuf)> {
    let catalog = load_catalog(catalog_path)?;

    let spec = map_err(load_dataset_spec(spec_path, &catalog))
        .with_context(|| format!("loading dataset spec from {}", spec_path.display()))?;

    let query = map_err(RrdDataframeQuery::open(rrd_path))
        .with_context(|| format!("opening recording {}", rrd_path.display()))?;

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("creating output directory {}", output_dir.display()))?;

    let sink = ParquetDatasetSink::new(output_dir);
    let manifest = map_err(export_dataset(&spec, &query, &sink))?;

    Ok(export_paths(output_dir, &manifest))
}

fn load_catalog(path: &Path) -> anyhow::Result<AssetCatalog> {
    let catalog = TomlCatalogRepository::new(path)
        .load_catalog()
        .map_err(|e| anyhow::anyhow!(e.to_string()))
        .with_context(|| format!("loading catalog from {}", path.display()))?;

    catalog
        .validate()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(catalog)
}

fn export_paths(
    output_dir: &Path,
    manifest: &sp_ml_dataloop::DatasetManifest,
) -> (PathBuf, PathBuf) {
    let stem = format!("{}_v{}", manifest.dataset_id(), manifest.version());
    (
        output_dir.join(format!("{stem}.parquet")),
        output_dir.join(format!("{stem}.manifest.toml")),
    )
}

fn absolute_path(path: &Path) -> PathBuf {
    match std::fs::canonicalize(path) {
        Ok(path) => path,
        Err(_) => std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf()),
    }
}

fn map_err<T>(result: sp_ml_dataloop::Result<T>) -> anyhow::Result<T> {
    result.map_err(|e| anyhow::anyhow!(e.to_string()))
}

use std::path::PathBuf;

use anyhow::Context as _;
use sp_acquisition::{AcquisitionSession, SamplingPolicy, TagBinding, run_session};
use sp_acquisition_replay::CsvReplaySource;
use sp_asset_model::{AssetCatalog, AssetCatalogPort as _, TomlCatalogRepository};
use sp_recording::RerunRecorder;

/// Replay historiador CSV data into an `.rrd` recording.
#[derive(Debug, Clone, clap::Parser)]
pub struct AcquireCommand {
    /// Path to the TOML asset catalog.
    #[arg(long = "catalog")]
    catalog: PathBuf,

    /// Path to the historiador CSV file.
    #[arg(long = "csv")]
    csv: PathBuf,

    /// Output `.rrd` recording path.
    #[arg(long = "output")]
    output: PathBuf,
}

impl AcquireCommand {
    pub fn run(self) -> anyhow::Result<()> {
        let batches_recorded = run_acquire(&self.catalog, &self.csv, &self.output)?;
        println!("Batches recorded: {batches_recorded}");
        Ok(())
    }
}

/// Replay CSV historiador data into an `.rrd` file.
pub(crate) fn run_acquire(
    catalog_path: &std::path::Path,
    csv_path: &std::path::Path,
    output_path: &std::path::Path,
) -> anyhow::Result<u64> {
    let catalog = load_catalog(catalog_path)?;

    let bindings: Vec<TagBinding> = catalog
        .tags()
        .iter()
        .map(|tag| {
            TagBinding::new(tag.id().clone(), tag.id().as_str())
                .map_err(|err| anyhow::anyhow!(err.to_string()))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mut session = AcquisitionSession::create(
        "simplant-lab-acquire",
        bindings,
        SamplingPolicy::default(),
        &catalog,
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating output directory {}", parent.display()))?;
        }
    }

    let source = CsvReplaySource::new(csv_path);
    let recorder = RerunRecorder::to_file("simplant_lab_acquire", output_path)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    let batches_recorded = run_session(&mut session, &catalog, &source, &recorder)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    recorder.flush();

    Ok(batches_recorded)
}

fn load_catalog(path: &std::path::Path) -> anyhow::Result<AssetCatalog> {
    let catalog = TomlCatalogRepository::new(path)
        .load_catalog()
        .map_err(|err| anyhow::anyhow!(err.to_string()))
        .with_context(|| format!("loading catalog from {}", path.display()))?;

    catalog
        .validate()
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    Ok(catalog)
}

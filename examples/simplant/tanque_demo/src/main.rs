//! End-to-end F1 pipeline demo: TOML catalog → CSV replay → `.rrd` recording.

use std::path::PathBuf;

use anyhow::Context as _;
use sp_acquisition::{AcquisitionSession, SamplingPolicy, TagBinding, run_session};
use sp_acquisition_replay::CsvReplaySource;
use sp_asset_model::{AssetCatalogPort as _, TomlCatalogRepository};
use sp_recording::RerunRecorder;

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn resolve_output_path() -> PathBuf {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tanque_demo.rrd"))
}

fn main() -> anyhow::Result<()> {
    let catalog_path = manifest_path("config/catalogo.toml");
    let csv_path = manifest_path("data/tanque.csv");
    let output_path = resolve_output_path();

    let catalog = TomlCatalogRepository::new(&catalog_path)
        .load_catalog()
        .map_err(|e| anyhow::anyhow!(e.to_string()))
        .with_context(|| format!("loading catalog from {}", catalog_path.display()))?;

    catalog
        .validate()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let bindings: Vec<TagBinding> = catalog
        .tags()
        .iter()
        .map(|tag| {
            TagBinding::new(tag.id().clone(), tag.id().as_str())
                .map_err(|e| anyhow::anyhow!(e.to_string()))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let tag_count = bindings.len();

    let mut session =
        AcquisitionSession::create("tanque-demo", bindings, SamplingPolicy::default(), &catalog)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let source = CsvReplaySource::new(&csv_path);
    let recorder = RerunRecorder::to_file("simplant_lab_tanque_demo", &output_path)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let batches_recorded = run_session(&mut session, &catalog, &source, &recorder)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    recorder.flush();

    let absolute_rrd = match std::fs::canonicalize(&output_path) {
        Ok(path) => path,
        Err(_) => std::env::current_dir()?.join(&output_path),
    };

    println!("Tags recorded:      {tag_count}");
    println!("Batches recorded:   {batches_recorded}");
    println!("Output file:        {}", absolute_rrd.display());
    println!(
        "Open it with:  pixi run simplant-lab {}",
        absolute_rrd.display()
    );

    Ok(())
}

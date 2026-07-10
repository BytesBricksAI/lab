//! End-to-end F3 pipeline demo: catalog → CSV replay → `.rrd` → dataframe query → Parquet export.

use std::path::{Path, PathBuf};
use std::str::FromStr as _;

use anyhow::Context as _;
use jiff::Timestamp;
use sp_acquisition::{AcquisitionSession, SamplingPolicy, TagBinding, run_session};
use sp_acquisition_replay::CsvReplaySource;
use sp_asset_model::{AssetCatalog, AssetCatalogPort as _, TomlCatalogRepository};
use sp_dataframe_query::RrdDataframeQuery;
use sp_kernel::{TagId, TimeWindow};
use sp_ml_dataloop::{DataSplit, DatasetSpec, FeatureSpec, ParquetDatasetSink, export_dataset};
use sp_recording::RerunRecorder;

const DATASET_ID: &str = "tanque-dataset";

fn manifest_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn resolve_output_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DATASET_EXPORT_DEMO_OUT") {
        return PathBuf::from(dir);
    }

    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/dataset_export_demo"))
}

fn parse_ts(value: &str) -> anyhow::Result<Timestamp> {
    Timestamp::from_str(value).with_context(|| format!("parsing timestamp `{value}`"))
}

fn map_err<T>(result: sp_ml_dataloop::Result<T>) -> anyhow::Result<T> {
    result.map_err(|err| anyhow::anyhow!(err.to_string()))
}

fn load_catalog(path: &Path) -> anyhow::Result<AssetCatalog> {
    let catalog = TomlCatalogRepository::new(path)
        .load_catalog()
        .map_err(|err| anyhow::anyhow!(err.to_string()))
        .with_context(|| format!("loading catalog from {}", path.display()))?;

    catalog
        .validate()
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    Ok(catalog)
}

fn record_rrd(
    catalog: &AssetCatalog,
    csv_path: &Path,
    rrd_path: &Path,
) -> anyhow::Result<(usize, u64)> {
    let bindings: Vec<TagBinding> = catalog
        .tags()
        .iter()
        .map(|tag| {
            TagBinding::new(tag.id().clone(), tag.id().as_str())
                .map_err(|err| anyhow::anyhow!(err.to_string()))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let tag_count = bindings.len();

    let mut session = AcquisitionSession::create(
        "dataset-export-demo",
        bindings,
        SamplingPolicy::default(),
        catalog,
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    if let Some(parent) = rrd_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating output directory {}", parent.display()))?;
    }

    let source = CsvReplaySource::new(csv_path);
    let recorder = RerunRecorder::to_file("simplant_lab_dataset_export_demo", rrd_path)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    let batches_recorded = run_session(&mut session, catalog, &source, &recorder)
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    recorder.flush();

    Ok((tag_count, batches_recorded))
}

fn build_dataset_spec(catalog: &AssetCatalog) -> anyhow::Result<DatasetSpec> {
    let train = TimeWindow::new(
        parse_ts("2026-01-01T00:00:00Z")?,
        parse_ts("2026-01-01T00:30:00Z")?,
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    let test = TimeWindow::new(
        parse_ts("2026-01-01T01:00:00Z")?,
        parse_ts("2026-01-01T02:00:00Z")?,
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

    let split = map_err(DataSplit::new(train, None, test))?;

    let features = vec![
        map_err(FeatureSpec::new(TagId::new("PT-101")?, "pressure"))?,
        map_err(FeatureSpec::new(TagId::new("LT-101")?, "level"))?,
    ];

    let targets = vec![map_err(FeatureSpec::new(TagId::new("FT-101")?, "flow"))?];

    let (spec, _) = map_err(DatasetSpec::define(
        DATASET_ID, features, targets, split, catalog,
    ))?;

    Ok(spec)
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

fn print_manifest_summary(manifest: &sp_ml_dataloop::DatasetManifest) -> anyhow::Result<()> {
    let train = manifest.split().train();
    println!("Manifest summary:");
    println!("  dataset_id:      {}", manifest.dataset_id());
    println!("  version:         {}", manifest.version());
    println!("  schema_version:  {}", manifest.schema_version());
    println!("  features:        {}", manifest.feature_names().join(", "));
    println!(
        "  targets:         {}",
        if manifest.target_names().is_empty() {
            "(none)".to_owned()
        } else {
            manifest.target_names().join(", ")
        }
    );
    println!("  train window:    {} .. {}", train.start(), train.end());
    println!(
        "  test window:     {} .. {}",
        manifest.split().test().start(),
        manifest.split().test().end()
    );

    let toml = map_err(manifest.to_toml())?;
    let restored = map_err(sp_ml_dataloop::DatasetManifest::from_toml_str(&toml))?;
    anyhow::ensure!(restored == *manifest, "manifest TOML round-trip mismatch");

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let catalog_path = manifest_path("config/catalogo.toml");
    let csv_path = manifest_path("config/tanque.csv");
    let output_dir = resolve_output_dir();
    let rrd_path = output_dir.join("recording.rrd");

    let catalog = load_catalog(&catalog_path)?;

    let (tag_count, batches_recorded) = record_rrd(&catalog, &csv_path, &rrd_path)?;

    let query = map_err(RrdDataframeQuery::open(&rrd_path))
        .with_context(|| format!("opening recording {}", rrd_path.display()))?;

    let spec = build_dataset_spec(&catalog)?;
    let sink = ParquetDatasetSink::new(&output_dir);
    let manifest = map_err(export_dataset(&spec, &query, &sink))?;

    let (parquet_path, manifest_path) = export_paths(&output_dir, &manifest);

    println!("Tags recorded:      {tag_count}");
    println!("Batches recorded:   {batches_recorded}");
    println!("Recording file:     {}", absolute_path(&rrd_path).display());
    println!(
        "Parquet file:       {}",
        absolute_path(&parquet_path).display()
    );
    println!(
        "Manifest file:      {}",
        absolute_path(&manifest_path).display()
    );
    print_manifest_summary(&manifest)?;

    Ok(())
}

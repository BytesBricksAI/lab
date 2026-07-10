# dataset_export_demo

End-to-end SimPlant Lab F3 pipeline demo: replay historian CSV into a `.rrd`, query it through `DataframeQueryPort`, and export a versioned ML dataset as Parquet plus a TOML manifest.

## What it demonstrates

1. **Acquisition + recording** — same flow as `tanque_demo`: TOML catalog, CSV replay, `RerunRecorder` to `.rrd`.
2. **Dataframe query** — `RrdDataframeQuery::open` reads the recording and serves `DataframeQueryPort`.
3. **ML data loop** — `DatasetSpec` + `export_dataset` + `ParquetDatasetSink` materialize training data and a reproducible manifest.

## Run

From the repository root:

```bash
cargo run -p dataset_export_demo
```

Writes under `target/dataset_export_demo/`:

- `recording.rrd`
- `tanque-dataset_v1.parquet`
- `tanque-dataset_v1.manifest.toml`

To choose the output directory:

```bash
cargo run -p dataset_export_demo -- /path/to/output
```

Or via environment variable:

```bash
DATASET_EXPORT_DEMO_OUT=/tmp/dataset_export_demo cargo run -p dataset_export_demo
```

## Layout

```
dataset_export_demo/
├── config/
│   ├── catalogo.toml   # plant catalog (PT-101, LT-101, FT-101)
│   └── tanque.csv      # 30-minute historian replay
└── src/main.rs         # replay → record → query → export
```

# sp_ml_dataloop

SimPlant Lab ML data loop: versioned, reproducible dataset specifications with leakage-free temporal train/validation/test splits.

This crate is the pure-domain core of the `ml-dataloop` capability. It has no `re_*` dependencies; export adapters over `re_dataframe` / `re_parquet` will live in separate crates.

## Features

- **Anti-leakage splits** — `DataSplit` rejects overlapping `TimeWindow`s so future data cannot leak into training windows.
- **Dataset specs** — `DatasetSpec` aggregate with catalog-validated feature/target tags and immutable versioning.
- **Reproducible manifests** — `DatasetManifest` serializes to TOML for diffable, versioned exports alongside dataset files.
- **Hexagonal ports** — `DataframeQueryPort` and `DatasetSinkPort` for future infrastructure adapters.

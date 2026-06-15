# sp_acquisition_replay

SimPlant Lab replay adapter: implements [`DataSourcePort`] from `sp_acquisition` by reading a historian CSV export.

## CSV format

| Column | Description |
|--------|-------------|
| `timestamp` | Required. RFC 3339 (`2026-01-01T00:00:00Z`) or epoch seconds (`1735689600`). |
| `<address>` | One column per bound tag; header must match the binding `address`. Cell values are `f64` samples. |

Example:

```csv
timestamp,pt_col
2026-01-01T00:00:00Z,10.5
2026-01-01T00:00:01Z,11.0
2026-01-01T00:00:02Z,11.5
```

Empty cells are skipped. In F1 every sample is assigned [`Quality::Good`]; real quality codes will arrive with industrial adapters in F2.

## Usage

```rust
use sp_acquisition_replay::CsvReplaySource;
use sp_acquisition::{DataSourcePort, MeasurementSource};

let source = CsvReplaySource::new("historian_export.csv");
let mut stream = source.subscribe(&bindings, &SamplingPolicy::default())?;
while let Some(batch) = stream.next_batch()? {
    // ...
}
```

[`DataSourcePort`]: https://docs.rs/sp_acquisition/latest/sp_acquisition/trait.DataSourcePort.html
[`Quality::Good`]: https://docs.rs/sp_kernel/latest/sp_kernel/enum.Quality.html

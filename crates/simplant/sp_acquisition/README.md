# sp_acquisition

SimPlant Lab acquisition capability: domain (`AcquisitionSession`) and ports (`DataSourcePort`, `RecorderPort`).

## Acquisition profile (TOML)

```toml
session_id = "replay-01"
source_path = "/data/replay.csv"
period_ms = 1000
deadband = 0.5

[[bindings]]
tag = "PT-1101"
address = "PT-1101"

[[bindings]]
tag = "TT-1102"
address = "temperature_col"
```

`source_path` points at the replay data file (CSV in F1). `address` is the physical address in the data source (column name, OPC UA node id, etc.).

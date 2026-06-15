# tanque_demo

End-to-end SimPlant Lab F1 pipeline demo without hardware.

The example loads a plant asset catalog from TOML, validates domain invariants, replays historian CSV samples for a storage tank and transfer pump, and records everything to a Rerun `.rrd` file. Open the recording in SimPlant Lab to inspect process variable trends with engineering units, quality, and alarm limits.

## What it demonstrates

1. **Asset model** — facility hierarchy (tank `T-101`, pump `P-1101A`) and tags (`PT-101`, `LT-101`, `FT-101`) with ranges and alarms.
2. **Acquisition** — tag bindings, session lifecycle, CSV replay as a `DataSourcePort`.
3. **Recording** — `RerunRecorder` writes tag metadata and time-series samples to `.rrd`.

## Run

From the repository root:

```bash
cargo run -p tanque_demo
```

Writes `tanque_demo.rrd` in the current working directory.

To choose the output path:

```bash
cargo run -p tanque_demo -- /path/to/output.rrd
```

## View the recording

```bash
pixi run simplant-lab /path/to/tanque_demo.rrd
```

The pressure tag `PT-101` crosses the high alarm (8 bar) around minutes 14–16 of the replay, so alarm bands should be visible in the viewer.

## Layout

```
tanque_demo/
├── config/catalogo.toml   # plant catalog
├── data/tanque.csv        # 30-minute historian replay
└── src/main.rs            # orchestration
```

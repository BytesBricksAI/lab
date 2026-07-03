# sp_pid_viewer

SimPlant Lab P&ID viewer: embedded [Equinor engineering-symbols](https://github.com/equinor/engineering-symbols)
icons (MIT, vendored in `assets/symbols/`) plus an interactive egui canvas
(`PidCanvas`) with pan, zoom, hover and click.

## Modules, by reason of change

- `symbols` — embedded SVG catalogue; changes when Equinor releases symbols.
- `mapping` — `EquipmentKind` → symbol id; changes with the asset model / DEXPI.
- `visualizer` — egui canvas; changes with UX and upstream egui.

SVG icons keep full resolution at any zoom level, and embedding them via
`include_bytes!` keeps the viewer fully offline: no network, no filesystem.

The host app must have egui image loaders with SVG support installed
(the SimPlant Lab viewer already does, via `re_ui`).

```bash
cargo test -p sp_pid_viewer
```

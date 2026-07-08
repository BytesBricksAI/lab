# sp_pid_viewer

SimPlant Lab P&ID viewer: embedded [Equinor engineering-symbols](https://github.com/equinor/engineering-symbols)
icons (MIT, vendored in `assets/symbols/`) plus an interactive egui canvas
(`PidCanvas`) with pan, zoom, hover and click.

## Modules, by reason of change

- `symbols` — embedded SVG catalogue; changes when Equinor releases symbols.
- `mapping` — `EquipmentKind` → symbol id; changes with the asset model / DEXPI.
- `visualizer` — egui canvas; changes with UX and upstream egui.
- `view` — `PidView`, the viewer `ViewClass`; changes with the `re_viewer_context` / `re_view` view-class contract.

SVG icons keep full resolution at any zoom level, and embedding them via
`include_bytes!` keeps the viewer fully offline: no network, no filesystem.

The host app must have egui image loaders with SVG support installed
(the SimPlant Lab viewer already does, via `re_ui`).

```bash
cargo test -p sp_pid_viewer
```

## End-to-end demo

`PidView` is registered as a native view (identifier `"SimPlantPid"`) in the
SimPlant Lab viewer binary. See `examples/simplant/pid_canvas_demo` for a
recording that logs `PidSymbol` entities plus process data and ships a
blueprint laying out the P&ID view next to the linked-tag trends.

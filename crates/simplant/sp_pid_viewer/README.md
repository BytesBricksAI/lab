# sp_pid_viewer

SimPlant Lab P&ID viewer: embedded [Equinor engineering-symbols](https://github.com/equinor/engineering-symbols)
icons (MIT, vendored in `assets/symbols/`) plus an interactive egui canvas
(`PidCanvas`) with pan, zoom, hover and click.

## Modules, by reason of change

- `symbols` — embedded SVG catalogue + drawing metadata; changes when Equinor releases symbols.
- `mapping` — `EquipmentKind` → symbol id; changes with the asset model / DEXPI.
- `visualizer` — egui canvas; changes with UX and upstream egui.
- `view` — `PidView`, the viewer `ViewClass`; changes with the `re_viewer_context` / `re_view` view-class contract.

SVG icons keep full resolution at any zoom level, and embedding them via
`include_bytes!` keeps the viewer fully offline: no network, no filesystem.

## Connectors and anchors

Every Equinor SVG declares its connection points
(`annotation-connector-<index>-<degrees>` circles) and `symbols` parses them
into [`SymbolMeta`]: viewBox extents plus [`Connector`] positions.
`symbols::glyph_rect` (the aspect-preserving rect a symbol is drawn in) and
`symbols::connector_point` (connector → diagram coordinates) are the single
source of truth for that geometry: the canvas paints through them and the
Python bindings (`simplant_lab.pid.Symbol.anchor`) compute pipe anchors
through them, so drawn glyphs and anchored pipes meet exactly — no gaps.

Symbols also carry a `SymbolKind`: instruments (`IM*`, `LZ*`) draw their ISA
tag *inside* the bubble, equipment tags go underneath. `PlacedPipe` has a
`PipeKind`: process lines are solid, instrument signal lines are dashed
(logged from Python as `PidPipe(points, kind="signal")`).

The host app must have egui image loaders with SVG support installed
(the SimPlant Lab viewer already does, via `re_ui`).

```bash
cargo test -p sp_pid_viewer
```

## End-to-end demo

`PidView` is registered as a native view (identifier `"SimPlantPid"`) in the
SimPlant Lab viewer binary. See `examples/simplant/pid_canvas_demo` for a
recording that logs `PidSymbol` and `PidPipe` entities plus process data and ships a
blueprint laying out the P&ID view next to the linked-tag trends.

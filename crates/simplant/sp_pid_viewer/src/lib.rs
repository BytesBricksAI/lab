//! SimPlant Lab P&ID viewer.
//!
//! Draws piping & instrumentation diagrams from embedded Equinor
//! `engineering-symbols` icons (MIT, vendored in `assets/symbols/`) on an
//! interactive egui canvas with pan, zoom, hover and click.
//!
//! Internal dependency direction (each module has one reason to change):
//!
//! ```text
//! visualizer (egui, UX)  →  mapping (domain table)  →  symbols (catalogue)
//! ```
//!
//! `symbols` and `mapping` are egui-free and unit-testable in isolation;
//! `visualizer` is the only module that renders.

pub mod mapping;
pub mod symbols;
pub mod visualizer;

pub use mapping::{symbol_for, symbol_id_for};
pub use symbols::Symbol;
pub use visualizer::{PidCanvas, PidCanvasResponse, PlacedSymbol};

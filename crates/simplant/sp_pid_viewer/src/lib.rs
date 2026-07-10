//! SimPlant Lab P&ID viewer.
//!
//! Draws piping & instrumentation diagrams from embedded Equinor
//! `engineering-symbols` icons (MIT, vendored in `assets/symbols/`) on an
//! interactive egui canvas with pan, zoom, hover and click.
//!
//! Internal dependency direction (each module has one reason to change):
//!
//! ```text
//! view (ViewClass)  →  visualizer (egui, UX)  →  mapping (domain table)  →  symbols (catalogue)
//! ```
//!
//! `symbols` and `mapping` are egui-free and unit-testable in isolation;
//! `visualizer` renders the canvas widget; `view` wires it into the SimPlant
//! Lab viewer as a native view (`PidView`, identifier `"SimPlantPid"`),
//! queried from logged `sp_types::PidSymbol` entities.

pub mod mapping;
pub mod symbols;
pub mod view;
pub mod visualizer;

pub use mapping::{symbol_for, symbol_id_for};
pub use symbols::Symbol;
pub use view::PidView;
pub use visualizer::{PidCanvas, PidCanvasResponse, PlacedSymbol};

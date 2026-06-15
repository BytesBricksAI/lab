//! `SimPlant` Lab process simulation core.
//!
//! Versionable [`FlowsheetSpec`] aggregates with degrees-of-freedom analysis,
//! plus [`Scenario`] and [`SimulationRun`] lifecycle types. Pure domain — no
//! `re_*` dependencies.

mod api;
mod application;
mod domain;
mod infrastructure;

pub use api::*;

//! `SimPlant` Lab stress testing core.
//!
//! Pure-domain aggregates for planning load profiles bounded by design limits times a
//! safety factor, and for evaluating measured outcomes against acceptance criteria.
//! No `re_*` dependencies, no async, and no I/O in this crate.

pub mod api;
pub mod domain;

pub use api::*;

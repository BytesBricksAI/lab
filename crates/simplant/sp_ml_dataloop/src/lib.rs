//! `SimPlant` Lab ML data loop capability.
//!
//! Versioned, reproducible dataset specifications with leakage-free temporal splits
//! for industrial ML training. This crate has no `re_*` dependencies; export adapters
//! over the Rerun data stack will live in separate crates.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

pub use api::*;

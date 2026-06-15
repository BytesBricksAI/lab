//! `SimPlant` Lab store types: `simplant.*` components and archetypes.
//!
//! This crate is the anti-corruption layer between `SimPlant` domain concepts and the Rerun
//! columnar store. It defines manual components and archetypes under the `simplant.*` namespace
//! without modifying Rerun's codegen or `.fbs` definitions.

mod api;
pub mod namespace;
mod process_variable;
mod quality;
mod tag_metadata;

pub use api::*;

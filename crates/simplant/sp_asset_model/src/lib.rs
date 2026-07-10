//! `SimPlant` Lab asset model: plant hierarchy and process tags as DDD aggregates.
//!
//! This crate models facilities, equipment, and process tags with enforced invariants.
//! Aggregates can only be constructed through validating constructors; the TOML adapter
//! deserializes via DTOs and rebuilds domain objects through those same constructors.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

pub use api::*;

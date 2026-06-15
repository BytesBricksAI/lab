//! `SimPlant` Lab acquisition capability.
//!
//! Domain model and hexagonal ports for plant data acquisition: session lifecycle,
//! tag bindings, sampling policy, and driven ports for data sources and recorders.
//! This crate has no `re_*` dependencies and performs no async I/O in the core.

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

pub use api::*;

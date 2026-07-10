//! Driven ports for simulation engines.

use crate::domain::error::Result;
use crate::domain::scenario::Scenario;

/// What a simulation engine can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineCapabilities {
    pub steady_state: bool,
    pub dynamic: bool,
}

/// A single simulation state snapshot (neutral representation; no re_* types).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SimState {
    pub values: Vec<(String, f64)>,
}

/// Driven port: a process simulation engine. `step` is also the natural signature of an RL env step.
///
/// Adapters: native engine (`sp_sim_engine`, F6) and DWSIM sidecar over gRPC
/// (`sp_simulation_dwsim`, F4 — external, `GPLv3`).
pub trait SimulatorPort {
    fn capabilities(&self) -> EngineCapabilities;
    fn initialize(&mut self, scenario: &Scenario) -> Result<()>;
    fn step(&mut self, dt_secs: f64) -> Result<SimState>;
    fn finalize(&mut self) -> Result<()>;
}

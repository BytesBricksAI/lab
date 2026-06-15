//! Native minimal simulation engine for `SimPlant Lab` (plan §4.11, Piece 3).
//!
//! This crate provides [`FirstOrderEngine`], a first-order dynamics model that implements
//! [`SimulatorPort`] from `sp_simulation`. Each scenario boundary variable is treated as a
//! setpoint; the simulated value converges toward it with time constant `tau_secs` using
//! explicit Euler integration:
//!
//! ```text
//! x(t + dt) = x(t) + (setpoint - x(t)) * (dt / tau)
//! ```
//!
//! The model is ubiquitous in process engineering (tanks, heat exchangers, control loops)
//! and demonstrates the `SimulatorPort` contract end-to-end without depending on DWSIM.
//! Steady-state modular sequential solvers and DAE integration via diffsol will arrive in
//! the full F6 engine.

mod first_order;

pub use first_order::FirstOrderEngine;

# sp_sim_engine

SimPlant Lab native simulation engine (minimal): first-order dynamics implementing [`SimulatorPort`] from `sp_simulation`.

## Model

Each scenario boundary variable is treated as a **setpoint**. The simulated value starts at zero and converges toward the setpoint with time constant `tau_secs` using explicit Euler integration:

```text
x(t + dt) = x(t) + (setpoint - x(t)) * (dt / tau)
```

This first-order lag is ubiquitous in process engineering (tanks, heat exchangers, control loops) and provides a deterministic, dependency-free demonstration of the `SimulatorPort` contract.

## Capabilities

[`FirstOrderEngine`] reports:

| Capability   | Supported |
|--------------|-----------|
| Steady-state | yes       |
| Dynamic      | yes       |

Each [`step`](sp_simulation::SimulatorPort::step) returns a [`SimState`] containing all boundary variables plus `sim_time`.

## Roadmap (F6)

This crate is the foundation of the native engine (plan §4.11, Piece 3). The full F6 engine will add:

- Steady-state sequential modular solution
- DAE integration via [diffsol](https://github.com/crates/diffsol)

[`SimulatorPort`]: https://docs.rs/sp_simulation/latest/sp_simulation/trait.SimulatorPort.html
[`FirstOrderEngine`]: crate::FirstOrderEngine

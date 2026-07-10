# sp_stress_testing

SimPlant Lab stress testing core: pure-domain aggregates for planning load profiles against design limits and evaluating acceptance criteria.

## Scope (F4.b)

This crate defines:

- **Load profiles** — ordered load points per process variable.
- **Design limits and safety factors** — the load profile must not exceed `design_limit × safety_factor` for each variable.
- **Acceptance criteria** — measured outcomes are compared after a stress run; each metric passes when `measured ≤ max_value`.
- **`StressTest` aggregate** — `plan` validates invariants; `evaluate` checks outcomes and transitions to `Completed`.

## Out of scope (later phases)

Execution of the load profile against equipment and piping (via `sp_simulation::SimulatorPort`) and production of `MeasuredOutcome` values is **not** implemented here. A future runner will drive the simulator, collect metrics, and call `StressTest::evaluate`.

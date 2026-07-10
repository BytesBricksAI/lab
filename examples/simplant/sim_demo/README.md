# sim_demo

End-to-end SimPlant Lab native simulation demo: build an approved flowsheet and scenario,
run [`FirstOrderEngine`](../../../crates/simplant/sp_sim_engine) step-by-step, and record the
trajectory to a `.rrd` file on the `sim_time` timeline.

No DWSIM dependency — this exercises the `SimulatorPort` contract and SimPlant Lab recording in one
binary.

## Run

From the repository root:

```bash
cargo run -p sim_demo
```

Optional output path (defaults to `sim_demo.rrd` in the current directory):

```bash
cargo run -p sim_demo -- /tmp/my_simulation.rrd
```

## Viewer

Open the recording with:

```bash
pixi run simplant-lab <path-to.rrd>
```

You should see time-series plots under `sim/outlet_temp` and `sim/outlet_pressure`. Both curves
follow first-order dynamics (time constant τ = 20 s) and converge toward their scenario setpoints
(180 °C and 12 bar) over the 120 s simulation, indexed on the `sim_time` timeline.

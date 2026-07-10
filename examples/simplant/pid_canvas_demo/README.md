# pid_canvas_demo

Logs a toy P&ID (tank → pump → valve) as `simplant.archetypes.PidSymbol`
entities plus five minutes of simulated process data (`Scalars` on the
`plant_time` timeline), and ships a blueprint that lays out the SimPlant Lab
viewer with the **P&ID view** (`sp_pid_viewer`) on the left half and a trend
per linked process variable stacked on the right half.

```bash
cargo run -p pid_canvas_demo               # writes pid_canvas_demo.rrd
cargo run -p simplant-lab-cli -- pid_canvas_demo.rrd
```

In the viewer: drag to pan, scroll to zoom, double-click the background to
re-fit — and click an equipment to select it.

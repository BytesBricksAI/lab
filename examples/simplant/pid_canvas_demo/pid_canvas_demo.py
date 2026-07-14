#!/usr/bin/env python3
"""
Logs a small but honest P&ID plus five minutes of simulated process data.

Tank → pump → shutdown valve, with ISA-5.1 instrument bubbles. Process lines
are anchored to the connection points the Equinor `engineering-symbols` SVGs
declare — exposed through `simplant_lab.pid.Symbol.anchor` — so pipes meet
the equipment glyphs with no gap. Instruments are IM005A bubbles with their
ISA tag inside; the valve position transmitter is wired with a dashed signal
line (`kind="signal"`).

Ships a blueprint that lays out the SimPlant Lab viewer with the **P&ID view**
on the left half and a trend per instrument stacked on the right half.

```bash
python examples/simplant/pid_canvas_demo/pid_canvas_demo.py
cargo run -p simplant-lab-cli -- pid_canvas_demo.rrd
```
"""

from __future__ import annotations

import math
import sys
import time
from pathlib import Path

import simplant_lab as sl
import simplant_lab.blueprint as rrb

PLANT_TIME = "plant_time"

# Equinor connector indices, as declared inside each vendored SVG
# (inspect them with `sl.pid.find_symbol(...).connectors()`):
#   PT002A tank    1 = right nozzle · 2 = bottom nozzle · 3 = left · 4 = top
#   PP007A pump    1 = discharge stub (east) · 2 = suction at the impeller eye
#   ND0001 valve   1 = right end · 2 = left end
#   IM005A bubble  1 = east · 2 = south · 3 = west · 4 = north
TANK_RIGHT, TANK_BOTTOM = 1, 2
PUMP_DISCHARGE, PUMP_SUCTION = 1, 2
VALVE_OUT, VALVE_IN = 1, 2
BUBBLE_SOUTH, BUBBLE_WEST = 2, 3


def _build_blueprint() -> rrb.Blueprint:
    return rrb.Blueprint(
        rrb.Horizontal(
            sl.PidView(origin="/pid", name="P&ID"),
            rrb.Vertical(
                rrb.TimeSeriesView(name="LT-101 level", origin="/tags/TK-101"),
                rrb.TimeSeriesView(name="PT-101 pressure", origin="/tags/P-101"),
                rrb.TimeSeriesView(name="ZT-101 opening", origin="/tags/XV-101"),
            ),
            column_shares=[0.5, 0.5],
        ),
        rrb.TimePanel(timeline=PLANT_TIME),
    )


def main() -> None:
    output = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("pid_canvas_demo.rrd")
    blueprint = _build_blueprint()

    sl.init("rerun_example_pid_canvas_demo", default_blueprint=blueprint)
    sl.save(str(output))

    rec = sl.get_global_data_recording()
    if rec is None:
        raise RuntimeError("No active recording after init/save")

    tank = sl.pid.find_symbol("PT002A")
    pump = sl.pid.find_symbol("PP007A")
    valve = sl.pid.find_symbol("ND0001")
    bubble = sl.pid.find_symbol("IM005A")
    if tank is None or pump is None or valve is None or bubble is None:
        raise RuntimeError("Missing vendored Equinor symbols")

    # Aspect-true half sizes: the drawn glyph then fills exactly the box the
    # anchors are computed against, so nothing is letterboxed or distorted.
    tank_pos, tank_half = (0.0, 0.0), tank.aspect_half_size(height=180.0)
    pump_pos, pump_half = (240.0, 160.0), pump.aspect_half_size(width=96.0)
    valve_half = valve.aspect_half_size(width=72.0)
    bubble_half = bubble.aspect_half_size(width=36.0)

    # Anchors, in diagram coordinates.
    tank_bottom = tank.anchor(TANK_BOTTOM, tank_pos, tank_half)
    tank_right = tank.anchor(TANK_RIGHT, tank_pos, tank_half)
    pump_suction = pump.anchor(PUMP_SUCTION, pump_pos, pump_half)
    pump_discharge = pump.anchor(PUMP_DISCHARGE, pump_pos, pump_half)

    # The shutdown valve sits inline with the pump discharge.
    valve_pos = (430.0, pump_discharge[1])
    valve_in = valve.anchor(VALVE_IN, valve_pos, valve_half)
    valve_out = valve.anchor(VALVE_OUT, valve_pos, valve_half)

    # Instrument bubbles: LT beside the tank, PT tapping the discharge line,
    # ZT above the valve it reports on.
    lt_pos, pt_pos, zt_pos = (110.0, 0.0), (340.0, 70.0), (430.0, 70.0)

    # --- Equipment -----------------------------------------------------------
    rec.log(
        "pid/TK-101",
        sl.PidSymbol(position=tank_pos, symbol_id="PT002A", label="TK-101", half_size=tank_half),
        static=True,
    )
    rec.log(
        "pid/P-101",
        sl.PidSymbol(position=pump_pos, symbol_id="PP007A", label="P-101", half_size=pump_half),
        static=True,
    )
    rec.log(
        "pid/XV-101",
        sl.PidSymbol(position=valve_pos, symbol_id="ND0001", label="XV-101", half_size=valve_half),
        static=True,
    )

    # --- Process lines (anchored connector-to-connector) --------------------
    rec.log(
        "pid/pipes/TK-101-P-101",
        sl.PidPipe([tank_bottom, (tank_bottom[0], pump_suction[1]), pump_suction]),
        static=True,
    )
    rec.log(
        "pid/pipes/P-101-XV-101",
        sl.PidPipe([pump_discharge, valve_in]),
        static=True,
    )
    rec.log(
        "pid/pipes/XV-101-out",
        sl.PidPipe([valve_out, (540.0, valve_out[1])]),
        static=True,
    )

    # --- Instruments (ISA-5.1 bubbles, tag inside) ---------------------------
    rec.log(
        "pid/LT-101",
        sl.PidSymbol(
            position=lt_pos,
            symbol_id="IM005A",
            label="LT-101",
            half_size=bubble_half,
            linked_tag="tags/TK-101/level",
        ),
        static=True,
    )
    rec.log(
        "pid/PT-101",
        sl.PidSymbol(
            position=pt_pos,
            symbol_id="IM005A",
            label="PT-101",
            half_size=bubble_half,
            linked_tag="tags/P-101/pressure",
        ),
        static=True,
    )
    rec.log(
        "pid/ZT-101",
        sl.PidSymbol(
            position=zt_pos,
            symbol_id="IM005A",
            label="ZT-101",
            half_size=bubble_half,
            linked_tag="tags/XV-101/opening",
        ),
        static=True,
    )

    # Instrument process connections (solid) …
    rec.log(
        "pid/leads/LT-101",
        sl.PidPipe([tank_right, bubble.anchor(BUBBLE_WEST, lt_pos, bubble_half)]),
        static=True,
    )
    rec.log(
        "pid/leads/PT-101",
        sl.PidPipe(
            [(pt_pos[0], pump_discharge[1]), bubble.anchor(BUBBLE_SOUTH, pt_pos, bubble_half)],
        ),
        static=True,
    )
    # … and the valve position transmitter as a dashed ISA signal line, down
    # to the top edge of the valve glyph (ND0001 declares no top connector).
    valve_top = (valve_pos[0], valve_pos[1] - valve_half[1])
    rec.log(
        "pid/signals/ZT-101",
        sl.PidPipe(
            [valve_top, bubble.anchor(BUBBLE_SOUTH, zt_pos, bubble_half)],
            kind="signal",
        ),
        static=True,
    )

    # --- Five minutes of simulated process data ------------------------------
    t0 = int(time.time())
    for step in range(1000):
        x = float(step)
        rec.set_time(PLANT_TIME, timestamp=t0 + step)
        level = 74.0 + 8.0 * math.sin(x * 0.02)
        pressure = 12.0 + math.sin(x * 0.08) + 0.3 * math.sin(x * 0.9)
        opening = 50.0 + 35.0 * math.cos(x * 0.05)
        rec.log("tags/TK-101/level", sl.Scalars(level))
        rec.log("tags/P-101/pressure", sl.Scalars(pressure))
        rec.log("tags/XV-101/opening", sl.Scalars(opening))

    rec.flush()
    print(f"Wrote {output}")
    print(f"Open it with: cargo run -p simplant-lab-cli -- {output}")


if __name__ == "__main__":
    main()

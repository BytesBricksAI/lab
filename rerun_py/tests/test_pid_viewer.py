"""Behavior tests for P&ID viewer Python bindings (pyo3-bindings-pid-viewer)."""

from __future__ import annotations

import math
import time
from typing import TYPE_CHECKING

import pytest

import simplant_lab as sl
import simplant_lab.blueprint as rrb
from simplant_lab.experimental import RrdReader

if TYPE_CHECKING:
    from pathlib import Path

    from simplant_lab.catalog import Schema

PLANT_TIME = "plant_time"

_PID_SYMBOL_FIELDS = ("position", "symbol_id", "label", "half_size", "linked_tag")


def _pid_field(field_name: str) -> str:
    return sl.types.field(sl.types.ARCHETYPE_PID_SYMBOL, field_name)


def _pid_pipe_field(field_name: str) -> str:
    return sl.types.field(sl.types.ARCHETYPE_PID_PIPE, field_name)


def _build_pid_canvas_blueprint() -> rrb.Blueprint:
    return rrb.Blueprint(
        rrb.Horizontal(
            sl.PidView(origin="/pid", name="P&ID"),
            rrb.Vertical(
                rrb.TimeSeriesView(name="TK-101 level", origin="/tags/TK-101"),
                rrb.TimeSeriesView(name="P-101 pressure", origin="/tags/P-101"),
                rrb.TimeSeriesView(name="XV-101 opening", origin="/tags/XV-101"),
            ),
            column_shares=[0.5, 0.5],
        ),
        rrb.TimePanel(timeline=PLANT_TIME),
    )


def _log_pid_canvas_demo(rec: sl.RecordingStream, *, t0: int) -> None:
    rec.log(
        "pid/TK-101",
        sl.PidSymbol(
            position=[0.0, 0.0],
            symbol_id="PT002A",
            label="TK-101",
            half_size=[60.0, 80.0],
            linked_tag="tags/TK-101/level",
        ),
        static=True,
    )
    rec.log(
        "pid/P-101",
        sl.PidSymbol(
            position=[260.0, 60.0],
            symbol_id="PP007A",
            label="P-101",
            half_size=[48.0, 48.0],
            linked_tag="tags/P-101/pressure",
        ),
        static=True,
    )
    rec.log(
        "pid/XV-101",
        sl.PidSymbol(
            position=[460.0, 60.0],
            symbol_id="ND0001",
            label="XV-101",
            half_size=[36.0, 36.0],
            linked_tag="tags/XV-101/opening",
        ),
        static=True,
    )
    rec.log(
        "pid/pipes/TK-101-P-101",
        sl.PidPipe([[60.0, 60.0], [212.0, 60.0]]),
        static=True,
    )
    rec.log(
        "pid/pipes/P-101-XV-101",
        sl.PidPipe([[308.0, 60.0], [424.0, 60.0]]),
        static=True,
    )
    rec.log(
        "pid/ZT-101",
        sl.PidSymbol(
            position=[460.0, 0.0],
            symbol_id="IM005A",
            label="ZT-101",
            half_size=[18.0, 18.0],
            linked_tag="tags/XV-101/opening",
        ),
        static=True,
    )
    rec.log(
        "pid/signals/ZT-101",
        sl.PidPipe([[460.0, 18.0], [460.0, 24.0]], kind="signal"),
        static=True,
    )

    for step in range(300):
        x = float(step)
        rec.set_time(PLANT_TIME, timestamp=t0 + step)
        level = 74.0 + 8.0 * math.sin(x * 0.02)
        pressure = 12.0 + math.sin(x * 0.08) + 0.3 * math.sin(x * 0.9)
        opening = 50.0 + 35.0 * math.cos(x * 0.05)
        rec.log("tags/TK-101/level", sl.Scalars(level))
        rec.log("tags/P-101/pressure", sl.Scalars(pressure))
        rec.log("tags/XV-101/opening", sl.Scalars(opening))


def _schema_component_fields(schema: Schema, *, entity_path: str) -> set[str]:
    fields: set[str] = set()
    for col in schema.columns_for(entity_path=entity_path):
        if col.archetype is None:
            continue
        if col.component.startswith("simplant."):
            fields.add(col.component)
            continue
        field_name = col.component.split(":", 1)[-1]
        fields.add(sl.types.field(col.archetype, field_name))
    return fields


# ---------------------------------------------------------------------------
# PidSymbol component batches
# ---------------------------------------------------------------------------


def test_pid_symbol_full_as_component_batches() -> None:
    symbol = sl.PidSymbol(
        position=[10.0, 20.0],
        symbol_id="PP007A",
        label="P-101",
        half_size=[48.0, 48.0],
        linked_tag="tags/P-101/pressure",
    )
    batches = symbol.as_component_batches()

    assert len(batches) == 5
    components = {batch.component_descriptor().component for batch in batches}
    assert components == {_pid_field(name) for name in _PID_SYMBOL_FIELDS}

    for batch in batches:
        descriptor = batch.component_descriptor()
        assert descriptor.archetype == sl.types.ARCHETYPE_PID_SYMBOL

    assert sl.types.ARCHETYPE_PID_SYMBOL == "simplant.archetypes.PidSymbol"
    assert _pid_field("position") == "simplant.archetypes.PidSymbol:position"


def test_pid_symbol_required_only_as_component_batches() -> None:
    symbol = sl.PidSymbol(position=[0.0, 0.0], symbol_id="PT002A")
    assert len(symbol.as_component_batches()) == 2


def test_pid_symbol_rejects_invalid_position_shape() -> None:
    with pytest.raises(ValueError):
        sl.PidSymbol(position=[1.0, 2.0, 3.0], symbol_id="PP007A")


# ---------------------------------------------------------------------------
# PidPipe component batches
# ---------------------------------------------------------------------------


def test_pid_pipe_as_component_batches() -> None:
    pipe = sl.PidPipe([[0.0, 0.0], [10.0, 0.0], [10.0, 5.0]])
    batches = pipe.as_component_batches()

    assert len(batches) == 1
    descriptor = batches[0].component_descriptor()
    assert descriptor.component == _pid_pipe_field("points")
    assert descriptor.component == "simplant.archetypes.PidPipe:points"
    assert descriptor.archetype == sl.types.ARCHETYPE_PID_PIPE
    assert sl.types.ARCHETYPE_PID_PIPE == "simplant.archetypes.PidPipe"


def test_pid_pipe_rejects_invalid_points() -> None:
    with pytest.raises(ValueError):
        sl.PidPipe([[0.0, 0.0]])
    with pytest.raises(ValueError):
        sl.PidPipe([[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]])


def test_pid_pipe_kind_adds_a_batch() -> None:
    pipe = sl.PidPipe([[0.0, 0.0], [10.0, 0.0]], kind="signal")
    batches = pipe.as_component_batches()

    assert len(batches) == 2
    components = {batch.component_descriptor().component for batch in batches}
    assert _pid_pipe_field("kind") in components
    assert _pid_pipe_field("kind") == "simplant.archetypes.PidPipe:kind"


def test_pid_pipe_rejects_unknown_kind() -> None:
    with pytest.raises(ValueError):
        sl.PidPipe([[0.0, 0.0], [1.0, 1.0]], kind="hydraulic")


# ---------------------------------------------------------------------------
# PidView + pid bindings
# ---------------------------------------------------------------------------


def test_pid_view_class_identifier() -> None:
    view = sl.PidView(origin="/pid")
    assert view.class_identifier == sl.pid.VIEW_CLASS_IDENTIFIER == "SimPlantPid"


def test_pid_bindings_symbol_catalog_and_mapping() -> None:
    assert sl.pid.symbol_id_for(sl.asset_model.EquipmentKind.Pump) == "PP007A"
    assert sl.pid.symbol_id_for(sl.asset_model.EquipmentKind.Tank) == "PT002A"
    assert sl.pid.symbol_id_for(sl.asset_model.EquipmentKind.Valve) == "ND0001"
    assert sl.pid.symbol_id_for(sl.asset_model.EquipmentKind.HeatExchanger) is None

    symbol = sl.pid.find_symbol("PP007A")
    assert symbol is not None
    assert symbol.svg().startswith(b"<svg")
    assert sl.pid.find_symbol("NOPE") is None

    symbol_ids = sl.pid.symbol_ids()
    assert symbol_ids == sorted(symbol_ids)
    assert len(symbol_ids) >= 50
    assert "PP007A" in symbol_ids
    assert "PT002A" in symbol_ids
    assert "ND0001" in symbol_ids


def test_pid_symbol_connector_metadata() -> None:
    tank = sl.pid.find_symbol("PT002A")
    assert tank is not None
    assert tank.kind() == "equipment"
    assert tank.view_box() == (96.0, 216.0)

    connectors = tank.connectors()
    assert {connector.index for connector in connectors} == {1, 2, 3, 4}
    right = next(connector for connector in connectors if connector.index == 1)
    assert right.direction_deg == 90
    assert right.pos == (87.5, 108.0)

    bubble = sl.pid.find_symbol("IM005A")
    assert bubble is not None
    assert bubble.kind() == "instrument"


def test_pid_symbol_anchor_maps_into_diagram_coordinates() -> None:
    tank = sl.pid.find_symbol("PT002A")
    assert tank is not None

    # Right-side nozzle (87.5, 108) of the 96 x 216 viewBox on an 80 x 180
    # glyph centered at the origin: x scales, y lands dead center.
    x, y = tank.anchor(1, [0.0, 0.0], [40.0, 90.0])
    assert x == pytest.approx(87.5 / 96.0 * 80.0 - 40.0, abs=1e-3)
    assert y == pytest.approx(0.0, abs=1e-3)

    with pytest.raises(ValueError):
        tank.anchor(9, [0.0, 0.0], [40.0, 90.0])


def test_pid_symbol_aspect_half_size() -> None:
    valve = sl.pid.find_symbol("ND0001")
    assert valve is not None
    assert valve.aspect_half_size(width=72.0) == pytest.approx((36.0, 12.0))

    tank = sl.pid.find_symbol("PT002A")
    assert tank is not None
    assert tank.aspect_half_size(height=180.0) == pytest.approx((40.0, 90.0))

    with pytest.raises(ValueError):
        tank.aspect_half_size()
    with pytest.raises(ValueError):
        tank.aspect_half_size(width=1.0, height=2.0)


# ---------------------------------------------------------------------------
# pid_canvas_demo E2E
# ---------------------------------------------------------------------------


def test_pid_canvas_demo_e2e(tmp_path: Path) -> None:
    output_rrd = tmp_path / "pid_canvas_demo.rrd"
    blueprint = _build_pid_canvas_blueprint()
    t0 = int(time.time())

    with sl.RecordingStream("rerun_example_pid_canvas_demo") as rec:
        rec.save(str(output_rrd), default_blueprint=blueprint)
        _log_pid_canvas_demo(rec, t0=t0)

    assert output_rrd.is_file()
    assert output_rrd.stat().st_size > 0

    schema = RrdReader(str(output_rrd)).store().schema()
    tk101_fields = _schema_component_fields(schema, entity_path="/pid/TK-101")
    assert _pid_field("position") in tk101_fields
    assert _pid_field("symbol_id") in tk101_fields

    pipe_entity = "/pid/pipes/TK-101-P-101"
    pipe_fields = _schema_component_fields(schema, entity_path=pipe_entity)
    assert _pid_pipe_field("points") in pipe_fields

    signal_entity = "/pid/signals/ZT-101"
    signal_fields = _schema_component_fields(schema, entity_path=signal_entity)
    assert _pid_pipe_field("points") in signal_fields
    assert _pid_pipe_field("kind") in signal_fields

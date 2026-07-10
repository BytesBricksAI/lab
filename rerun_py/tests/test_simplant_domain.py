"""Behavior tests for SimPlant domain Python bindings (pyo3-bindings-sp-domain Phase 7)."""

from __future__ import annotations

from pathlib import Path

import pytest

import simplant_lab as sl

LAB_ROOT = Path(__file__).resolve().parents[2]
TANQUE_DEMO = LAB_ROOT / "examples" / "simplant" / "tanque_demo"


# ---------------------------------------------------------------------------
# 7.2 — submodule imports + one central type each
# ---------------------------------------------------------------------------


def test_submodules_import_and_construct_central_types() -> None:
    tag = sl.kernel.TagId("FT-101")
    assert tag.as_str() == "FT-101"

    facility = sl.asset_model.Facility.define(sl.asset_model.FacilityId("FAC-1"), "Demo")
    assert facility.name() == "Demo"

    binding = sl.acquisition.TagBinding(tag, "holding:40001")
    assert binding.tag().as_str() == "FT-101"

    component = sl.simulation.ChemicalComponent("Methane")
    assert component.name() == "Methane"

    feature = sl.ml_dataloop.FeatureSpec(tag, "level")
    assert feature.name() == "level"

    load = sl.stress_testing.LoadPoint("outlet_temp", 180.0)
    assert load.variable() == "outlet_temp"

    assert isinstance(sl.recording.PLANT_TIME, str)
    entity_path = sl.recording.tag_entity_path(tag)
    assert entity_path
    assert entity_path == "tags/FT-101"

    sample = sl.types.ProcessVariableSample(42.0, sl.kernel.Quality.Good)
    assert sample.value() == 42.0


# ---------------------------------------------------------------------------
# 7.3 — tanque_demo E2E smoke
# ---------------------------------------------------------------------------


def test_tanque_demo_e2e_smoke(tmp_path: Path) -> None:
    catalog_path = TANQUE_DEMO / "config" / "catalogo.toml"
    csv_path = TANQUE_DEMO / "data" / "tanque.csv"
    output_rrd = tmp_path / "tanque_demo.rrd"

    repo = sl.asset_model.TomlCatalogRepository(str(catalog_path))
    catalog = repo.load_catalog()
    catalog.validate()

    bindings = [sl.acquisition.TagBinding(tag.id(), tag.id().as_str()) for tag in catalog.tags()]
    session = sl.acquisition.AcquisitionSession.create(
        "tanque-demo",
        bindings,
        sl.acquisition.SamplingPolicy(1000, deadband=None),
        catalog,
    )
    source = sl.acquisition.replay.CsvReplaySource(str(csv_path))
    recorder = sl.recording.RerunRecorder.to_file(
        "simplant_lab_tanque_demo",
        str(output_rrd),
    )

    batches = sl.acquisition.run_session(
        session,
        catalog=catalog,
        source=source,
        recorder=recorder,
    )
    recorder.flush()

    assert batches > 0
    assert output_rrd.is_file() and output_rrd.stat().st_size > 0

    query = sl.ml_dataloop.dataframe_query.RrdDataframeQuery.open(str(output_rrd))
    assert query is not None


# ---------------------------------------------------------------------------
# 7.4 — sim_demo E2E smoke
# ---------------------------------------------------------------------------


def _build_sim_demo_flowsheet(*, with_specs: bool) -> sl.simulation.FlowsheetSpec:
    h100 = sl.simulation.UnitOpId("H-100")
    specs = [sl.simulation.Specification(h100, "outlet_temp", 0.0)] if with_specs else []
    return sl.simulation.FlowsheetSpec.draft(
        sl.simulation.FlowsheetId("FS-SIM-DEMO"),
        components=[sl.simulation.ChemicalComponent("Methane")],
        unit_ops=[sl.simulation.UnitOp(h100, sl.simulation.UnitOpKind.Heater, "Heater")],
        streams=[
            sl.simulation.MaterialStream(
                sl.simulation.StreamId("S1"),
                sl.simulation.Composition([1.0]),
                to=h100,
            )
        ],
        specs=specs,
        thermo=sl.simulation.ThermoPackage.PengRobinson,
    )


def test_sim_demo_e2e_smoke() -> None:
    flowsheet = _build_sim_demo_flowsheet(with_specs=True)
    assert flowsheet.degrees_of_freedom() == 0
    flowsheet.approve()
    assert flowsheet.is_approved()

    scenario = sl.simulation.Scenario.approve(
        sl.simulation.ScenarioId("SC-1"),
        flowsheet,
        boundary_conditions=[
            sl.simulation.BoundaryCondition("outlet_temp", 180.0),
            sl.simulation.BoundaryCondition("outlet_pressure", 12.0),
        ],
        duration_secs=120.0,
        required_capability=sl.simulation.EngineCapability.Dynamic,
    )
    assert scenario.is_approved()
    assert scenario.duration_secs() == 120.0

    engine = sl.simulation.engine.FirstOrderEngine(20.0)
    engine.initialize(scenario)

    dt = 2.0
    num_steps = int(scenario.duration_secs() / dt)
    final_state: list[tuple[str, float]] = []
    for _ in range(num_steps):
        final_state = engine.step(dt)

    assert engine.current_time() == pytest.approx(scenario.duration_secs())
    assert final_state
    values = dict(final_state)
    assert "outlet_temp" in values
    assert values["outlet_temp"] == pytest.approx(180.0, rel=0.01)
    assert "outlet_pressure" in values
    assert values["outlet_pressure"] == pytest.approx(12.0, rel=0.01)


# ---------------------------------------------------------------------------
# 7.5 — error paths raise Python exceptions
# ---------------------------------------------------------------------------


def test_approve_rejects_nonzero_degrees_of_freedom() -> None:
    flowsheet = _build_sim_demo_flowsheet(with_specs=False)
    assert flowsheet.degrees_of_freedom() != 0
    with pytest.raises(ValueError):
        flowsheet.approve()


def test_safety_factor_rejects_zero() -> None:
    with pytest.raises(ValueError):
        sl.stress_testing.SafetyFactor(0.0)


def test_data_split_rejects_overlapping_windows() -> None:
    train = sl.kernel.TimeWindow(100.0, 250.0)
    test = sl.kernel.TimeWindow(200.0, 300.0)
    with pytest.raises(ValueError):
        sl.ml_dataloop.DataSplit(train, test)


def test_parse_modbus_address_rejects_bogus() -> None:
    with pytest.raises(ValueError):
        sl.acquisition.modbus.parse_modbus_address("bogus")

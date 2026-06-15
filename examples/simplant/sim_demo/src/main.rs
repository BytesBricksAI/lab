//! End-to-end native simulation demo: flowsheet → scenario → FirstOrderEngine → `.rrd`.

use std::path::PathBuf;

use anyhow::Context as _;
use re_sdk::RecordingStreamBuilder;
use sp_sim_engine::FirstOrderEngine;
use sp_simulation::{
    BoundaryCondition, ChemicalComponent, Composition, EngineCapability, FlowsheetId,
    FlowsheetSpec, MaterialStream, Scenario, ScenarioId, Specification, StreamId, SimulatorPort,
    ThermoPackage, UnitOp, UnitOpId, UnitOpKind,
};

fn domain_err<E: std::fmt::Display>(err: E) -> anyhow::Error {
    anyhow::anyhow!(err.to_string())
}

fn resolve_output_path() -> PathBuf {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("sim_demo.rrd"))
}

fn main() -> anyhow::Result<()> {
    let h100 = UnitOpId::new("H-100").map_err(domain_err)?;

    let mut flowsheet = FlowsheetSpec::draft(
        FlowsheetId::new("FS-SIM-DEMO").map_err(domain_err)?,
        vec![ChemicalComponent::new("Methane").map_err(domain_err)?],
        vec![UnitOp::new(h100.clone(), UnitOpKind::Heater, "Heater").map_err(domain_err)?],
        vec![MaterialStream::new(
            StreamId::new("S1").map_err(domain_err)?,
            None,
            Some(h100.clone()),
            Composition::new(vec![1.0]),
        )],
        vec![Specification::new(h100, "outlet_temp", 0.0).map_err(domain_err)?],
        ThermoPackage::PengRobinson,
    )
    .map_err(domain_err)?;

    if flowsheet.degrees_of_freedom() != 0 {
        anyhow::bail!(
            "expected degrees of freedom = 0, got {}",
            flowsheet.degrees_of_freedom()
        );
    }
    flowsheet.approve().map_err(domain_err)?;

    let (scenario, _) = Scenario::approve(
        &ScenarioId::new("SC-1").map_err(domain_err)?,
        &flowsheet,
        vec![
            BoundaryCondition::new("outlet_temp", 180.0).map_err(domain_err)?,
            BoundaryCondition::new("outlet_pressure", 12.0).map_err(domain_err)?,
        ],
        120.0,
        EngineCapability::Dynamic,
    )
    .map_err(domain_err)?;

    let mut engine = FirstOrderEngine::new(20.0);
    engine.initialize(&scenario).map_err(domain_err)?;

    let output_path = resolve_output_path();
    let rec = RecordingStreamBuilder::new("simplant_lab_sim_demo")
        .save(&output_path)
        .map_err(domain_err)
        .with_context(|| format!("creating recording at {}", output_path.display()))?;

    let dt = 2.0;
    let num_steps = (scenario.duration_secs() / dt) as usize;
    let mut final_values = Vec::new();

    for _ in 0..num_steps {
        let state = engine.step(dt).map_err(domain_err)?;
        rec.set_duration_secs("sim_time", engine.current_time());

        for (var, value) in &state.values {
            if var == "sim_time" {
                continue;
            }
            rec.log(
                format!("sim/{var}"),
                &re_sdk_types::archetypes::Scalars::single(*value),
            )
            .map_err(domain_err)?;
        }

        final_values = state
            .values
            .into_iter()
            .filter(|(var, _)| var != "sim_time")
            .collect();
    }

    rec.flush_blocking().map_err(domain_err)?;

    let absolute_rrd = match std::fs::canonicalize(&output_path) {
        Ok(path) => path,
        Err(_) => std::env::current_dir()?.join(&output_path),
    };

    println!("Simulation steps:  {num_steps}");
    println!("Final values:");
    for (var, value) in &final_values {
        println!("  {var}: {value:.4}");
    }
    println!("Output file:        {}", absolute_rrd.display());
    println!(
        "Open it with:  pixi run simplant-lab {}",
        absolute_rrd.display()
    );

    Ok(())
}

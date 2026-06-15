//! First-order dynamics engine implementing [`SimulatorPort`].

use sp_simulation::{EngineCapabilities, Result, Scenario, SimState, SimulatorPort};

/// Default time constant (seconds) when `tau_secs` is invalid in [`FirstOrderEngine::new`].
const DEFAULT_TAU_SECS: f64 = 10.0;

/// First-order lag simulator: each boundary variable tracks its setpoint with time constant `tau_secs`.
#[derive(Debug, Clone, PartialEq)]
pub struct FirstOrderEngine {
    tau_secs: f64,
    state: Vec<(String, f64)>,
    setpoints: Vec<(String, f64)>,
    time_secs: f64,
}

impl FirstOrderEngine {
    /// Creates an engine with time constant `tau_secs`.
    ///
    /// If `tau_secs` is non-positive or non-finite, [`DEFAULT_TAU_SECS`] is used instead so the
    /// engine never panics on bad input.
    pub fn new(tau_secs: f64) -> Self {
        let tau_secs = if tau_secs.is_finite() && tau_secs > 0.0 {
            tau_secs
        } else {
            DEFAULT_TAU_SECS
        };
        Self {
            tau_secs,
            state: Vec::new(),
            setpoints: Vec::new(),
            time_secs: 0.0,
        }
    }

    /// Elapsed simulation time in seconds.
    pub fn current_time(&self) -> f64 {
        self.time_secs
    }

    /// Current value of a state variable, if present.
    pub fn value_of(&self, variable: &str) -> Option<f64> {
        self.state
            .iter()
            .find(|(name, _)| name == variable)
            .map(|(_, value)| *value)
    }
}

impl Default for FirstOrderEngine {
    fn default() -> Self {
        Self::new(DEFAULT_TAU_SECS)
    }
}

impl SimulatorPort for FirstOrderEngine {
    fn capabilities(&self) -> EngineCapabilities {
        EngineCapabilities {
            steady_state: true,
            dynamic: true,
        }
    }

    fn initialize(&mut self, scenario: &Scenario) -> Result<()> {
        self.setpoints = scenario
            .boundary_conditions()
            .iter()
            .map(|bc| (bc.variable().to_owned(), bc.value()))
            .collect();
        self.state = self
            .setpoints
            .iter()
            .map(|(var, _)| (var.clone(), 0.0))
            .collect();
        self.time_secs = 0.0;
        Ok(())
    }

    fn step(&mut self, dt_secs: f64) -> Result<SimState> {
        if dt_secs > 0.0 {
            for (var, setpoint) in &self.setpoints {
                for (name, value) in &mut self.state {
                    if name == var {
                        *value += (setpoint - *value) * (dt_secs / self.tau_secs);
                        break;
                    }
                }
            }
            self.time_secs += dt_secs;
        }

        let mut values = self.state.clone();
        values.push(("sim_time".to_owned(), self.time_secs));
        Ok(SimState { values })
    }

    fn finalize(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_simulation::{
        BoundaryCondition, ChemicalComponent, Composition, EngineCapability, FlowsheetId,
        FlowsheetSpec, MaterialStream, Scenario, ScenarioId, Specification, StreamId,
        ThermoPackage, UnitOp, UnitOpId, UnitOpKind,
    };

    fn approved_scenario(
        boundary: Vec<BoundaryCondition>,
        duration_secs: f64,
    ) -> (FlowsheetSpec, Scenario) {
        let mut flowsheet = FlowsheetSpec::draft(
            FlowsheetId::new("FS-ENGINE-01").unwrap(),
            vec![ChemicalComponent::new("N2").unwrap()],
            vec![
                UnitOp::new(UnitOpId::new("H-01").unwrap(), UnitOpKind::Heater, "Heater").unwrap(),
            ],
            vec![MaterialStream::new(
                StreamId::new("S-IN").unwrap(),
                None,
                Some(UnitOpId::new("H-01").unwrap()),
                Composition::new(vec![1.0]),
            )],
            vec![Specification::new(UnitOpId::new("H-01").unwrap(), "duty", 100.0).unwrap()],
            ThermoPackage::IdealGas,
        )
        .unwrap();
        flowsheet.approve().unwrap();

        let (scenario, _) = Scenario::approve(
            &ScenarioId::new("SC-ENGINE-01").unwrap(),
            &flowsheet,
            boundary,
            duration_secs,
            EngineCapability::Dynamic,
        )
        .unwrap();

        (flowsheet, scenario)
    }

    fn scenario_with_outlet_temp(setpoint: f64) -> Scenario {
        let (_, scenario) = approved_scenario(
            vec![BoundaryCondition::new("outlet_temp", setpoint).unwrap()],
            60.0,
        );
        scenario
    }

    #[test]
    fn capabilities_reports_steady_state_and_dynamic() {
        let engine = FirstOrderEngine::new(10.0);
        let caps = engine.capabilities();
        assert!(caps.steady_state);
        assert!(caps.dynamic);
    }

    #[test]
    fn first_order_step_exact() {
        let scenario = scenario_with_outlet_temp(100.0);
        let mut engine = FirstOrderEngine::new(10.0);
        engine.initialize(&scenario).unwrap();

        let state = engine.step(1.0).unwrap();
        let outlet = state
            .values
            .iter()
            .find(|(name, _)| name == "outlet_temp")
            .map(|(_, v)| *v)
            .unwrap();
        assert!((outlet - 10.0).abs() < 1e-9);

        let sim_time = state
            .values
            .iter()
            .find(|(name, _)| name == "sim_time")
            .map(|(_, v)| *v)
            .unwrap();
        assert!((sim_time - 1.0).abs() < 1e-9);
    }

    #[test]
    fn converges_toward_setpoint() {
        let scenario = scenario_with_outlet_temp(100.0);
        let mut engine = FirstOrderEngine::new(10.0);
        engine.initialize(&scenario).unwrap();

        let mut last_outlet = 0.0;
        for _ in 0..200 {
            let state = engine.step(1.0).unwrap();
            last_outlet = state
                .values
                .iter()
                .find(|(name, _)| name == "outlet_temp")
                .map(|(_, v)| *v)
                .unwrap();
        }
        assert!((last_outlet - 100.0).abs() < 1e-3);
    }

    #[test]
    fn deterministic_for_identical_inputs() {
        let scenario = scenario_with_outlet_temp(100.0);

        let mut engine_a = FirstOrderEngine::new(10.0);
        engine_a.initialize(&scenario).unwrap();
        let mut engine_b = FirstOrderEngine::new(10.0);
        engine_b.initialize(&scenario).unwrap();

        for _ in 0..50 {
            let state_a = engine_a.step(1.0).unwrap();
            let state_b = engine_b.step(1.0).unwrap();
            assert_eq!(state_a, state_b);
        }
    }

    #[test]
    fn value_of_and_current_time_track_state() {
        let scenario = scenario_with_outlet_temp(100.0);
        let mut engine = FirstOrderEngine::new(10.0);
        engine.initialize(&scenario).unwrap();

        assert_eq!(engine.value_of("outlet_temp"), Some(0.0));
        assert_eq!(engine.current_time(), 0.0);

        engine.step(2.0).unwrap();
        let expected = 100.0 * (2.0 / 10.0);
        assert!((engine.value_of("outlet_temp").unwrap() - expected).abs() < 1e-9);
        assert!((engine.current_time() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn zero_dt_does_not_change_state() {
        let scenario = scenario_with_outlet_temp(100.0);
        let mut engine = FirstOrderEngine::new(10.0);
        engine.initialize(&scenario).unwrap();

        engine.step(1.0).unwrap();
        let before = engine.value_of("outlet_temp").unwrap();
        let time_before = engine.current_time();

        let state = engine.step(0.0).unwrap();
        assert!((engine.value_of("outlet_temp").unwrap() - before).abs() < 1e-12);
        assert!((engine.current_time() - time_before).abs() < 1e-12);

        let outlet_in_state = state
            .values
            .iter()
            .find(|(name, _)| name == "outlet_temp")
            .map(|(_, v)| *v)
            .unwrap();
        assert!((outlet_in_state - before).abs() < 1e-12);
    }

    #[test]
    fn invalid_tau_uses_default() {
        let engine = FirstOrderEngine::new(-1.0);
        let scenario = scenario_with_outlet_temp(100.0);
        let mut engine = engine;
        engine.initialize(&scenario).unwrap();
        let state = engine.step(1.0).unwrap();
        let outlet = state
            .values
            .iter()
            .find(|(name, _)| name == "outlet_temp")
            .map(|(_, v)| *v)
            .unwrap();
        assert!((outlet - 10.0).abs() < 1e-9);
    }
}

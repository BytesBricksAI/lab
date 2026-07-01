use pyo3::prelude::*;
use sp_stress_testing::{
    AcceptanceCriterion, DesignLimit, LoadPoint, LoadProfile, MeasuredOutcome, SafetyFactor,
    StressTest, StressTestState as DomainStressTestState,
};

use crate::map_err;

#[pyclass(name = "LoadPoint", module = "simplant_lab.stress_testing")]
#[derive(Clone)]
pub struct PyLoadPoint(pub LoadPoint);

#[pymethods]
impl PyLoadPoint {
    #[new]
    fn new(variable: String, value: f64) -> Self {
        Self(LoadPoint::new(variable, value))
    }

    fn variable(&self) -> &str {
        self.0.variable()
    }

    fn value(&self) -> f64 {
        self.0.value()
    }
}

#[pyclass(name = "LoadProfile", module = "simplant_lab.stress_testing")]
#[derive(Clone)]
pub struct PyLoadProfile(pub LoadProfile);

#[pymethods]
impl PyLoadProfile {
    #[new]
    fn new(points: Vec<PyLoadPoint>) -> Self {
        Self(LoadProfile::new(points.into_iter().map(|p| p.0).collect()))
    }

    fn points(&self) -> Vec<PyLoadPoint> {
        self.0.points().iter().cloned().map(PyLoadPoint).collect()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[pyclass(name = "DesignLimit", module = "simplant_lab.stress_testing")]
#[derive(Clone)]
pub struct PyDesignLimit(pub DesignLimit);

#[pymethods]
impl PyDesignLimit {
    #[new]
    fn new(variable: String, max_value: f64) -> Self {
        Self(DesignLimit::new(variable, max_value))
    }

    fn variable(&self) -> &str {
        self.0.variable()
    }

    fn max_value(&self) -> f64 {
        self.0.max_value()
    }
}

#[pyclass(name = "SafetyFactor", module = "simplant_lab.stress_testing")]
#[derive(Clone, Copy)]
pub struct PySafetyFactor(pub SafetyFactor);

#[pymethods]
impl PySafetyFactor {
    #[new]
    fn new(value: f64) -> PyResult<Self> {
        SafetyFactor::new(value)
            .map(PySafetyFactor)
            .map_err(map_err)
    }

    fn value(&self) -> f64 {
        self.0.value()
    }
}

#[pyclass(name = "AcceptanceCriterion", module = "simplant_lab.stress_testing")]
#[derive(Clone)]
pub struct PyAcceptanceCriterion(pub AcceptanceCriterion);

#[pymethods]
impl PyAcceptanceCriterion {
    #[new]
    fn new(metric: String, max_value: f64) -> Self {
        Self(AcceptanceCriterion::new(metric, max_value))
    }

    fn metric(&self) -> &str {
        self.0.metric()
    }

    fn max_value(&self) -> f64 {
        self.0.max_value()
    }
}

#[pyclass(name = "MeasuredOutcome", module = "simplant_lab.stress_testing")]
#[derive(Clone)]
pub struct PyMeasuredOutcome(pub MeasuredOutcome);

#[pymethods]
impl PyMeasuredOutcome {
    #[new]
    fn new(metric: String, value: f64) -> Self {
        Self(MeasuredOutcome::new(metric, value))
    }

    fn metric(&self) -> &str {
        self.0.metric()
    }

    fn value(&self) -> f64 {
        self.0.value()
    }
}

#[pyclass(eq, eq_int, module = "simplant_lab.stress_testing")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StressTestState {
    Planned,
    Completed,
}

fn py_stress_test_state_from_domain(state: DomainStressTestState) -> StressTestState {
    match state {
        DomainStressTestState::Planned => StressTestState::Planned,
        DomainStressTestState::Completed => StressTestState::Completed,
    }
}

#[pyclass(name = "StressTest", module = "simplant_lab.stress_testing")]
pub struct PyStressTest(pub StressTest);

#[pymethods]
impl PyStressTest {
    #[staticmethod]
    fn plan(
        id: String,
        load_profile: PyLoadProfile,
        safety_factor: PySafetyFactor,
        design_limits: Vec<PyDesignLimit>,
        acceptance_criteria: Vec<PyAcceptanceCriterion>,
    ) -> PyResult<Self> {
        StressTest::plan(
            id,
            load_profile.0,
            safety_factor.0,
            design_limits.into_iter().map(|l| l.0).collect(),
            acceptance_criteria.into_iter().map(|c| c.0).collect(),
        )
        .map(|(test, _)| PyStressTest(test))
        .map_err(map_err)
    }

    fn id(&self) -> &str {
        self.0.id()
    }

    fn state(&self) -> StressTestState {
        py_stress_test_state_from_domain(self.0.state())
    }

    fn load_profile(&self) -> PyLoadProfile {
        PyLoadProfile(self.0.load_profile().clone())
    }

    fn safety_factor(&self) -> PySafetyFactor {
        PySafetyFactor(self.0.safety_factor())
    }

    fn design_limits(&self) -> Vec<PyDesignLimit> {
        self.0
            .design_limits()
            .iter()
            .cloned()
            .map(PyDesignLimit)
            .collect()
    }

    fn acceptance_criteria(&self) -> Vec<PyAcceptanceCriterion> {
        self.0
            .acceptance_criteria()
            .iter()
            .cloned()
            .map(PyAcceptanceCriterion)
            .collect()
    }

    fn evaluate(&mut self, outcomes: Vec<PyMeasuredOutcome>) -> PyResult<bool> {
        self.0
            .evaluate(&outcomes.iter().map(|o| o.0.clone()).collect::<Vec<_>>())
            .map(|event| event.passed)
            .map_err(map_err)
    }
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let stress_testing = PyModule::new(py, "stress_testing")?;
    stress_testing.add_class::<PyLoadPoint>()?;
    stress_testing.add_class::<PyLoadProfile>()?;
    stress_testing.add_class::<PyDesignLimit>()?;
    stress_testing.add_class::<PySafetyFactor>()?;
    stress_testing.add_class::<PyAcceptanceCriterion>()?;
    stress_testing.add_class::<PyMeasuredOutcome>()?;
    stress_testing.add_class::<StressTestState>()?;
    stress_testing.add_class::<PyStressTest>()?;

    crate::attach_simplant_submodule(py, parent, "stress_testing", &stress_testing)
}

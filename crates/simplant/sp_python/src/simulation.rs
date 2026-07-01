use pyo3::prelude::*;
use sp_simulation::{
    BoundaryCondition, ChemicalComponent, Composition, EngineCapability as DomainEngineCapability,
    FlowsheetId, FlowsheetSpec, FlowsheetState as DomainFlowsheetState, MaterialStream, Scenario,
    ScenarioId, Specification, StreamId, ThermoPackage as DomainThermoPackage, UnitOp, UnitOpId,
    UnitOpKind as DomainUnitOpKind,
};

use crate::map_err;

macro_rules! define_py_id {
    ($py_name:ident, $domain:ty, $py_class:literal) => {
        #[pyclass(name = $py_class, module = "simplant_lab.simulation")]
        #[derive(Clone)]
        pub struct $py_name(pub $domain);

        #[pymethods]
        impl $py_name {
            #[new]
            fn new(raw: String) -> PyResult<Self> {
                <$domain>::new(raw).map($py_name).map_err(map_err)
            }

            fn as_str(&self) -> &str {
                self.0.as_str()
            }

            fn __str__(&self) -> &str {
                self.as_str()
            }
        }
    };
}

define_py_id!(PyFlowsheetId, FlowsheetId, "FlowsheetId");
define_py_id!(PyUnitOpId, UnitOpId, "UnitOpId");
define_py_id!(PyStreamId, StreamId, "StreamId");
define_py_id!(PyScenarioId, ScenarioId, "ScenarioId");

#[pyclass(eq, eq_int, module = "simplant_lab.simulation")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnitOpKind {
    Mixer,
    Splitter,
    Heater,
    Cooler,
    Valve,
    Pump,
    FlashDrum,
    Pipe,
}

fn unit_op_kind_from_py(kind: UnitOpKind) -> DomainUnitOpKind {
    match kind {
        UnitOpKind::Mixer => DomainUnitOpKind::Mixer,
        UnitOpKind::Splitter => DomainUnitOpKind::Splitter,
        UnitOpKind::Heater => DomainUnitOpKind::Heater,
        UnitOpKind::Cooler => DomainUnitOpKind::Cooler,
        UnitOpKind::Valve => DomainUnitOpKind::Valve,
        UnitOpKind::Pump => DomainUnitOpKind::Pump,
        UnitOpKind::FlashDrum => DomainUnitOpKind::FlashDrum,
        UnitOpKind::Pipe => DomainUnitOpKind::Pipe,
    }
}

fn py_unit_op_kind_from_domain(kind: DomainUnitOpKind) -> UnitOpKind {
    match kind {
        DomainUnitOpKind::Mixer => UnitOpKind::Mixer,
        DomainUnitOpKind::Splitter => UnitOpKind::Splitter,
        DomainUnitOpKind::Heater => UnitOpKind::Heater,
        DomainUnitOpKind::Cooler => UnitOpKind::Cooler,
        DomainUnitOpKind::Valve => UnitOpKind::Valve,
        DomainUnitOpKind::Pump => UnitOpKind::Pump,
        DomainUnitOpKind::FlashDrum => UnitOpKind::FlashDrum,
        DomainUnitOpKind::Pipe => UnitOpKind::Pipe,
    }
}

#[pyclass(eq, eq_int, module = "simplant_lab.simulation")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThermoPackage {
    PengRobinson,
    Srk,
    PcSaft,
    IdealGas,
}

fn thermo_from_py(thermo: ThermoPackage) -> DomainThermoPackage {
    match thermo {
        ThermoPackage::PengRobinson => DomainThermoPackage::PengRobinson,
        ThermoPackage::Srk => DomainThermoPackage::Srk,
        ThermoPackage::PcSaft => DomainThermoPackage::PcSaft,
        ThermoPackage::IdealGas => DomainThermoPackage::IdealGas,
    }
}

fn py_thermo_from_domain(thermo: DomainThermoPackage) -> ThermoPackage {
    match thermo {
        DomainThermoPackage::PengRobinson => ThermoPackage::PengRobinson,
        DomainThermoPackage::Srk => ThermoPackage::Srk,
        DomainThermoPackage::PcSaft => ThermoPackage::PcSaft,
        DomainThermoPackage::IdealGas => ThermoPackage::IdealGas,
    }
}

#[pyclass(eq, eq_int, module = "simplant_lab.simulation")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EngineCapability {
    SteadyState,
    Dynamic,
}

fn engine_capability_from_py(cap: EngineCapability) -> DomainEngineCapability {
    match cap {
        EngineCapability::SteadyState => DomainEngineCapability::SteadyState,
        EngineCapability::Dynamic => DomainEngineCapability::Dynamic,
    }
}

fn py_engine_capability_from_domain(cap: DomainEngineCapability) -> EngineCapability {
    match cap {
        DomainEngineCapability::SteadyState => EngineCapability::SteadyState,
        DomainEngineCapability::Dynamic => EngineCapability::Dynamic,
    }
}

#[pyclass(eq, eq_int, module = "simplant_lab.simulation")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FlowsheetState {
    Draft,
    Approved,
}

fn py_flowsheet_state_from_domain(state: DomainFlowsheetState) -> FlowsheetState {
    match state {
        DomainFlowsheetState::Draft => FlowsheetState::Draft,
        DomainFlowsheetState::Approved => FlowsheetState::Approved,
    }
}

#[pyclass(name = "ChemicalComponent", module = "simplant_lab.simulation")]
#[derive(Clone)]
pub struct PyChemicalComponent(pub ChemicalComponent);

#[pymethods]
impl PyChemicalComponent {
    #[new]
    fn new(name: String) -> PyResult<Self> {
        ChemicalComponent::new(name)
            .map(PyChemicalComponent)
            .map_err(map_err)
    }

    fn name(&self) -> &str {
        self.0.name()
    }
}

#[pyclass(name = "Composition", module = "simplant_lab.simulation")]
#[derive(Clone)]
pub struct PyComposition(pub Composition);

#[pymethods]
impl PyComposition {
    #[new]
    fn new(fractions: Vec<f64>) -> Self {
        Self(Composition::new(fractions))
    }

    fn fractions(&self) -> Vec<f64> {
        self.0.fractions().to_vec()
    }

    fn sum(&self) -> f64 {
        self.0.sum()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[pyclass(name = "UnitOp", module = "simplant_lab.simulation")]
#[derive(Clone)]
pub struct PyUnitOp(pub UnitOp);

#[pymethods]
impl PyUnitOp {
    #[new]
    fn new(id: PyUnitOpId, kind: UnitOpKind, name: String) -> PyResult<Self> {
        UnitOp::new(id.0, unit_op_kind_from_py(kind), name)
            .map(PyUnitOp)
            .map_err(map_err)
    }

    fn id(&self) -> PyUnitOpId {
        PyUnitOpId(self.0.id().clone())
    }

    fn kind(&self) -> UnitOpKind {
        py_unit_op_kind_from_domain(self.0.kind())
    }

    fn name(&self) -> &str {
        self.0.name()
    }
}

#[pyclass(name = "MaterialStream", module = "simplant_lab.simulation")]
#[derive(Clone)]
pub struct PyMaterialStream(pub MaterialStream);

#[pymethods]
impl PyMaterialStream {
    #[new]
    #[pyo3(signature = (id, composition, from=None, to=None))]
    fn new(
        id: PyStreamId,
        composition: PyComposition,
        from: Option<PyUnitOpId>,
        to: Option<PyUnitOpId>,
    ) -> Self {
        Self(MaterialStream::new(
            id.0,
            from.map(|f| f.0),
            to.map(|t| t.0),
            composition.0,
        ))
    }

    fn id(&self) -> PyStreamId {
        PyStreamId(self.0.id().clone())
    }

    #[getter]
    #[pyo3(name = "from")]
    fn from_(&self) -> Option<PyUnitOpId> {
        self.0.from().cloned().map(PyUnitOpId)
    }

    fn to(&self) -> Option<PyUnitOpId> {
        self.0.to().cloned().map(PyUnitOpId)
    }

    fn composition(&self) -> PyComposition {
        PyComposition(self.0.composition().clone())
    }

    fn is_feed(&self) -> bool {
        self.0.is_feed()
    }

    fn is_product(&self) -> bool {
        self.0.is_product()
    }
}

#[pyclass(name = "Specification", module = "simplant_lab.simulation")]
#[derive(Clone)]
pub struct PySpecification(pub Specification);

#[pymethods]
impl PySpecification {
    #[new]
    fn new(unit_op: PyUnitOpId, variable: String, value: f64) -> PyResult<Self> {
        Specification::new(unit_op.0, variable, value)
            .map(PySpecification)
            .map_err(map_err)
    }

    fn unit_op(&self) -> PyUnitOpId {
        PyUnitOpId(self.0.unit_op().clone())
    }

    fn variable(&self) -> &str {
        self.0.variable()
    }

    fn value(&self) -> f64 {
        self.0.value()
    }
}

#[pyclass(name = "BoundaryCondition", module = "simplant_lab.simulation")]
#[derive(Clone)]
pub struct PyBoundaryCondition(pub BoundaryCondition);

#[pymethods]
impl PyBoundaryCondition {
    #[new]
    fn new(variable: String, value: f64) -> PyResult<Self> {
        BoundaryCondition::new(variable, value)
            .map(PyBoundaryCondition)
            .map_err(map_err)
    }

    fn variable(&self) -> &str {
        self.0.variable()
    }

    fn value(&self) -> f64 {
        self.0.value()
    }
}

#[pyclass(name = "FlowsheetSpec", module = "simplant_lab.simulation")]
pub struct PyFlowsheetSpec(pub FlowsheetSpec);

#[pymethods]
impl PyFlowsheetSpec {
    #[staticmethod]
    #[allow(clippy::too_many_arguments)]
    fn draft(
        id: PyFlowsheetId,
        components: Vec<PyChemicalComponent>,
        unit_ops: Vec<PyUnitOp>,
        streams: Vec<PyMaterialStream>,
        specs: Vec<PySpecification>,
        thermo: ThermoPackage,
    ) -> PyResult<Self> {
        FlowsheetSpec::draft(
            id.0,
            components.into_iter().map(|c| c.0).collect(),
            unit_ops.into_iter().map(|op| op.0).collect(),
            streams.into_iter().map(|s| s.0).collect(),
            specs.into_iter().map(|s| s.0).collect(),
            thermo_from_py(thermo),
        )
        .map(PyFlowsheetSpec)
        .map_err(map_err)
    }

    fn id(&self) -> PyFlowsheetId {
        PyFlowsheetId(self.0.id().clone())
    }

    fn version(&self) -> u32 {
        self.0.version()
    }

    fn state(&self) -> FlowsheetState {
        py_flowsheet_state_from_domain(self.0.state())
    }

    fn is_approved(&self) -> bool {
        self.0.is_approved()
    }

    fn degrees_of_freedom(&self) -> i64 {
        self.0.degrees_of_freedom()
    }

    fn thermo(&self) -> ThermoPackage {
        py_thermo_from_domain(self.0.thermo())
    }

    fn approve(&mut self) -> PyResult<()> {
        self.0.approve().map(|_| ()).map_err(map_err)
    }
}

#[pyclass(name = "Scenario", module = "simplant_lab.simulation")]
pub struct PyScenario(pub Scenario);

#[pymethods]
impl PyScenario {
    #[staticmethod]
    fn approve(
        id: PyScenarioId,
        flowsheet: &PyFlowsheetSpec,
        boundary_conditions: Vec<PyBoundaryCondition>,
        duration_secs: f64,
        required_capability: EngineCapability,
    ) -> PyResult<Self> {
        Scenario::approve(
            &id.0,
            &flowsheet.0,
            boundary_conditions.into_iter().map(|bc| bc.0).collect(),
            duration_secs,
            engine_capability_from_py(required_capability),
        )
        .map(|(scenario, _)| PyScenario(scenario))
        .map_err(map_err)
    }

    fn id(&self) -> PyScenarioId {
        PyScenarioId(self.0.id().clone())
    }

    fn flowsheet(&self) -> PyFlowsheetId {
        PyFlowsheetId(self.0.flowsheet().clone())
    }

    fn flowsheet_version(&self) -> u32 {
        self.0.flowsheet_version()
    }

    fn boundary_conditions(&self) -> Vec<PyBoundaryCondition> {
        self.0
            .boundary_conditions()
            .iter()
            .cloned()
            .map(PyBoundaryCondition)
            .collect()
    }

    fn duration_secs(&self) -> f64 {
        self.0.duration_secs()
    }

    fn required_capability(&self) -> EngineCapability {
        py_engine_capability_from_domain(self.0.required_capability())
    }

    fn is_approved(&self) -> bool {
        self.0.is_approved()
    }
}

pub mod engine {
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    use sp_sim_engine::{
        FirstOrderEngine, StreamState, mix as domain_mix, pipe as domain_pipe, pump as domain_pump,
        split as domain_split, valve as domain_valve,
    };
    use sp_simulation::SimulatorPort as _;

    use crate::map_err;

    #[pyclass(name = "StreamState", module = "simplant_lab.simulation.engine")]
    #[derive(Clone)]
    pub struct PyStreamState(pub StreamState);

    #[pymethods]
    impl PyStreamState {
        #[new]
        fn new(
            mass_flow: f64,
            pressure: f64,
            temperature: f64,
            composition: Vec<f64>,
        ) -> PyResult<Self> {
            StreamState::new(mass_flow, pressure, temperature, composition)
                .map(PyStreamState)
                .map_err(map_err)
        }

        fn mass_flow(&self) -> f64 {
            self.0.mass_flow()
        }

        fn pressure(&self) -> f64 {
            self.0.pressure()
        }

        fn temperature(&self) -> f64 {
            self.0.temperature()
        }

        fn composition(&self) -> Vec<f64> {
            self.0.composition().to_vec()
        }
    }

    #[pyclass(name = "FirstOrderEngine", module = "simplant_lab.simulation.engine")]
    pub struct PyFirstOrderEngine {
        inner: FirstOrderEngine,
        initialized: bool,
    }

    #[pymethods]
    impl PyFirstOrderEngine {
        #[new]
        fn new(tau_secs: f64) -> Self {
            Self {
                inner: FirstOrderEngine::new(tau_secs),
                initialized: false,
            }
        }

        fn initialize(&mut self, scenario: &super::PyScenario) -> PyResult<()> {
            self.inner.initialize(&scenario.0).map_err(map_err)?;
            self.initialized = true;
            Ok(())
        }

        fn step(&mut self, dt_secs: f64) -> PyResult<Vec<(String, f64)>> {
            if !self.initialized {
                return Err(PyValueError::new_err(
                    "engine not initialized; call initialize(scenario) first",
                ));
            }
            self.inner
                .step(dt_secs)
                .map(|state| state.values)
                .map_err(map_err)
        }

        fn current_time(&self) -> f64 {
            self.inner.current_time()
        }

        fn value_of(&self, variable: &str) -> Option<f64> {
            self.inner.value_of(variable)
        }
    }

    #[pyfunction]
    #[pyo3(name = "mix")]
    fn py_mix(inlets: Vec<PyStreamState>) -> PyResult<PyStreamState> {
        let inlets: Vec<StreamState> = inlets.into_iter().map(|s| s.0).collect();
        domain_mix(&inlets).map(PyStreamState).map_err(map_err)
    }

    #[pyfunction]
    #[pyo3(name = "split")]
    fn py_split(inlet: &PyStreamState, fractions: Vec<f64>) -> PyResult<Vec<PyStreamState>> {
        domain_split(&inlet.0, &fractions)
            .map(|outlets| outlets.into_iter().map(PyStreamState).collect())
            .map_err(map_err)
    }

    #[pyfunction]
    #[pyo3(name = "valve")]
    fn py_valve(inlet: &PyStreamState, outlet_pressure: f64) -> PyResult<PyStreamState> {
        domain_valve(&inlet.0, outlet_pressure)
            .map(PyStreamState)
            .map_err(map_err)
    }

    #[pyfunction]
    #[pyo3(name = "pump")]
    fn py_pump(
        inlet: &PyStreamState,
        outlet_pressure: f64,
        density: f64,
        efficiency: f64,
    ) -> PyResult<(PyStreamState, f64)> {
        domain_pump(&inlet.0, outlet_pressure, density, efficiency)
            .map(|(outlet, work)| (PyStreamState(outlet), work))
            .map_err(map_err)
    }

    #[pyfunction]
    #[pyo3(name = "pipe")]
    fn py_pipe(inlet: &PyStreamState, delta_p: f64) -> PyResult<PyStreamState> {
        domain_pipe(&inlet.0, delta_p)
            .map(PyStreamState)
            .map_err(map_err)
    }

    pub fn register(parent: &Bound<'_, PyModule>) -> PyResult<()> {
        let engine = PyModule::new(parent.py(), "engine")?;
        engine.add_class::<PyStreamState>()?;
        engine.add_class::<PyFirstOrderEngine>()?;
        engine.add_function(wrap_pyfunction!(py_mix, &engine)?)?;
        engine.add_function(wrap_pyfunction!(py_split, &engine)?)?;
        engine.add_function(wrap_pyfunction!(py_valve, &engine)?)?;
        engine.add_function(wrap_pyfunction!(py_pump, &engine)?)?;
        engine.add_function(wrap_pyfunction!(py_pipe, &engine)?)?;
        parent.add_submodule(&engine)?;
        Ok(())
    }
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let simulation = PyModule::new(py, "simulation")?;
    simulation.add_class::<PyFlowsheetId>()?;
    simulation.add_class::<PyUnitOpId>()?;
    simulation.add_class::<PyStreamId>()?;
    simulation.add_class::<PyScenarioId>()?;
    simulation.add_class::<UnitOpKind>()?;
    simulation.add_class::<ThermoPackage>()?;
    simulation.add_class::<EngineCapability>()?;
    simulation.add_class::<FlowsheetState>()?;
    simulation.add_class::<PyChemicalComponent>()?;
    simulation.add_class::<PyComposition>()?;
    simulation.add_class::<PyUnitOp>()?;
    simulation.add_class::<PyMaterialStream>()?;
    simulation.add_class::<PySpecification>()?;
    simulation.add_class::<PyBoundaryCondition>()?;
    simulation.add_class::<PyFlowsheetSpec>()?;
    simulation.add_class::<PyScenario>()?;

    engine::register(&simulation)?;

    crate::attach_simplant_submodule(py, parent, "simulation", &simulation)
}

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use sp_acquisition::{
    AcquisitionSession, DataSourcePort, SamplingPolicy, SessionState as DomainSessionState,
    TagBinding, run_session as domain_run_session,
};
use sp_acquisition_modbus::ModbusTcpSource;
use sp_acquisition_replay::CsvReplaySource;

use crate::asset_model::PyAssetCatalog;
use crate::kernel::PyTagId;
use crate::map_err;
use crate::recording::PyRerunRecorder;

#[pyclass(name = "TagBinding", module = "simplant_lab.acquisition")]
#[derive(Clone)]
pub struct PyTagBinding(pub TagBinding);

#[pymethods]
impl PyTagBinding {
    #[new]
    fn new(tag: PyTagId, address: String) -> PyResult<Self> {
        TagBinding::new(tag.0, address)
            .map(PyTagBinding)
            .map_err(map_err)
    }

    fn tag(&self) -> PyTagId {
        PyTagId(self.0.tag().clone())
    }

    fn address(&self) -> &str {
        self.0.address()
    }
}

#[pyclass(name = "SamplingPolicy", module = "simplant_lab.acquisition")]
#[derive(Clone, Copy)]
pub struct PySamplingPolicy(pub SamplingPolicy);

#[pymethods]
impl PySamplingPolicy {
    #[new]
    #[pyo3(signature = (period_ms, deadband=None))]
    fn new(period_ms: u64, deadband: Option<f64>) -> Self {
        Self(SamplingPolicy::new(period_ms, deadband))
    }

    fn period_ms(&self) -> u64 {
        self.0.period_ms()
    }

    fn deadband(&self) -> Option<f64> {
        self.0.deadband()
    }
}

#[pyclass(eq, eq_int, module = "simplant_lab.acquisition")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Created,
    Running,
    Stopped,
}

fn py_session_state_from_domain(state: DomainSessionState) -> SessionState {
    match state {
        DomainSessionState::Created => SessionState::Created,
        DomainSessionState::Running => SessionState::Running,
        DomainSessionState::Stopped => SessionState::Stopped,
    }
}

#[pyclass(name = "AcquisitionSession", module = "simplant_lab.acquisition")]
pub struct PyAcquisitionSession(pub AcquisitionSession);

#[pymethods]
impl PyAcquisitionSession {
    #[staticmethod]
    fn create(
        id: String,
        bindings: Vec<PyTagBinding>,
        policy: PySamplingPolicy,
        catalog: &PyAssetCatalog,
    ) -> PyResult<Self> {
        AcquisitionSession::create(
            id,
            bindings.into_iter().map(|binding| binding.0).collect(),
            policy.0,
            &catalog.0,
        )
        .map(PyAcquisitionSession)
        .map_err(map_err)
    }

    fn start(&mut self) -> PyResult<()> {
        self.0.start().map(|_| ()).map_err(map_err)
    }

    fn stop(&mut self, batches_recorded: u64) -> PyResult<()> {
        self.0.stop(batches_recorded).map(|_| ()).map_err(map_err)
    }

    fn id(&self) -> &str {
        self.0.id()
    }

    fn bindings(&self) -> Vec<PyTagBinding> {
        self.0
            .bindings()
            .iter()
            .cloned()
            .map(PyTagBinding)
            .collect()
    }

    fn policy(&self) -> PySamplingPolicy {
        PySamplingPolicy(self.0.policy())
    }

    fn state(&self) -> SessionState {
        py_session_state_from_domain(self.0.state())
    }
}

/// Wrapper over concrete data-source adapters; dereferences to [`DataSourcePort`].
enum OwnedDataSource {
    Replay(CsvReplaySource),
    Modbus(ModbusTcpSource),
}

impl OwnedDataSource {
    fn as_port(&self) -> &dyn DataSourcePort {
        match self {
            Self::Replay(source) => source,
            Self::Modbus(source) => source,
        }
    }
}

fn resolve_data_source(source: &Bound<'_, PyAny>) -> PyResult<OwnedDataSource> {
    if let Ok(replay) = source.extract::<PyRef<'_, PyCsvReplaySource>>() {
        return Ok(OwnedDataSource::Replay(CsvReplaySource::new(
            replay.path.clone(),
        )));
    }
    if let Ok(modbus) = source.extract::<PyRef<'_, PyModbusTcpSource>>() {
        return Ok(OwnedDataSource::Modbus(ModbusTcpSource::new(modbus.addr)));
    }
    Err(PyTypeError::new_err(
        "source must be CsvReplaySource or ModbusTcpSource",
    ))
}

#[pyfunction]
#[pyo3(name = "run_session")]
fn py_run_session(
    py: Python<'_>,
    session: &mut PyAcquisitionSession,
    catalog: &PyAssetCatalog,
    source: &Bound<'_, PyAny>,
    recorder: &PyRerunRecorder,
) -> PyResult<u64> {
    let data_source = resolve_data_source(source)?;
    py.detach(|| {
        domain_run_session(
            &mut session.0,
            &catalog.0,
            data_source.as_port(),
            &recorder.0,
        )
        .map_err(map_err)
    })
}

pub mod replay {
    use std::path::PathBuf;

    use pyo3::prelude::*;

    #[pyclass(name = "CsvReplaySource", module = "simplant_lab.acquisition.replay")]
    pub struct PyCsvReplaySource {
        pub(crate) path: PathBuf,
    }

    #[pymethods]
    impl PyCsvReplaySource {
        #[new]
        fn new(path: String) -> Self {
            Self {
                path: PathBuf::from(path),
            }
        }
    }

    pub fn register(parent: &Bound<'_, PyModule>) -> PyResult<()> {
        let replay = PyModule::new(parent.py(), "replay")?;
        replay.add_class::<PyCsvReplaySource>()?;
        parent.add_submodule(&replay)?;
        Ok(())
    }
}

pub mod modbus {
    use std::net::SocketAddr;
    use std::str::FromStr as _;

    use pyo3::prelude::*;
    use sp_acquisition::AcquisitionError;
    use sp_acquisition_modbus::{
        ModbusPoint, RegisterKind as DomainRegisterKind, map_register, parse_modbus_address,
    };

    use crate::map_err;

    #[pyclass(eq, eq_int, module = "simplant_lab.acquisition.modbus")]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub enum RegisterKind {
        Holding,
        Input,
    }

    fn py_register_kind_from_domain(kind: DomainRegisterKind) -> RegisterKind {
        match kind {
            DomainRegisterKind::Holding => RegisterKind::Holding,
            DomainRegisterKind::Input => RegisterKind::Input,
        }
    }

    #[pyclass(name = "ModbusPoint", module = "simplant_lab.acquisition.modbus")]
    #[derive(Clone)]
    pub struct PyModbusPoint(pub ModbusPoint);

    #[pymethods]
    impl PyModbusPoint {
        fn kind(&self) -> RegisterKind {
            py_register_kind_from_domain(self.0.kind())
        }

        fn register(&self) -> u16 {
            self.0.register()
        }

        fn scale(&self) -> f64 {
            self.0.scale()
        }

        fn offset(&self) -> f64 {
            self.0.offset()
        }
    }

    #[pyclass(name = "ModbusTcpSource", module = "simplant_lab.acquisition.modbus")]
    pub struct PyModbusTcpSource {
        pub(crate) addr: SocketAddr,
    }

    #[pymethods]
    impl PyModbusTcpSource {
        #[new]
        fn new(host_port: String) -> PyResult<Self> {
            let addr = SocketAddr::from_str(&host_port).map_err(|err| {
                map_err(AcquisitionError::Source(format!(
                    "invalid socket address: {err}"
                )))
            })?;
            Ok(Self { addr })
        }
    }

    #[pyfunction]
    #[pyo3(name = "parse_modbus_address")]
    fn py_parse_modbus_address(s: &str) -> PyResult<PyModbusPoint> {
        parse_modbus_address(s).map(PyModbusPoint).map_err(map_err)
    }

    #[pyfunction]
    #[pyo3(name = "map_register")]
    fn py_map_register(raw: u16, point: &PyModbusPoint) -> f64 {
        map_register(raw, &point.0)
    }

    pub fn register(parent: &Bound<'_, PyModule>) -> PyResult<()> {
        let modbus = PyModule::new(parent.py(), "modbus")?;
        modbus.add_class::<RegisterKind>()?;
        modbus.add_class::<PyModbusPoint>()?;
        modbus.add_class::<PyModbusTcpSource>()?;
        modbus.add_function(wrap_pyfunction!(py_parse_modbus_address, &modbus)?)?;
        modbus.add_function(wrap_pyfunction!(py_map_register, &modbus)?)?;
        parent.add_submodule(&modbus)?;
        Ok(())
    }
}

pub use modbus::PyModbusTcpSource;
pub use replay::PyCsvReplaySource;

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let acquisition = PyModule::new(py, "acquisition")?;
    acquisition.add_class::<PyTagBinding>()?;
    acquisition.add_class::<PySamplingPolicy>()?;
    acquisition.add_class::<SessionState>()?;
    acquisition.add_class::<PyAcquisitionSession>()?;
    acquisition.add_function(wrap_pyfunction!(py_run_session, &acquisition)?)?;

    replay::register(&acquisition)?;
    modbus::register(&acquisition)?;

    crate::attach_simplant_submodule(py, parent, "acquisition", &acquisition)
}

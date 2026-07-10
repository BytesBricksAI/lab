use std::sync::OnceLock;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use re_sdk::RecordingStream;
use sp_recording::{EVENTS_PATH, PLANT_TIME, RerunRecorder, tag_entity_path};

use crate::kernel::PyTagId;
use crate::map_err;

type RerunRecorderFactory = Box<dyn Fn(&Bound<'_, PyAny>) -> PyResult<RerunRecorder> + Send + Sync>;

static RECORDER_FACTORY: OnceLock<RerunRecorderFactory> = OnceLock::new();

/// Registers a factory that clones a native [`RecordingStream`] from a Python object.
pub fn register_recording_stream_extractor(
    extractor: Box<dyn Fn(&Bound<'_, PyAny>) -> PyResult<RecordingStream> + Send + Sync>,
) {
    if RECORDER_FACTORY
        .set(Box::new(move |obj| extractor(obj).map(RerunRecorder::new)))
        .is_err()
    {
        eprintln!("sp_python::recording: recorder factory already registered");
    }
}

fn recorder_from_py(stream: &Bound<'_, PyAny>) -> PyResult<RerunRecorder> {
    let factory = RECORDER_FACTORY
        .get()
        .ok_or_else(|| PyRuntimeError::new_err("RerunRecorder stream factory not registered"))?;
    factory(stream)
}

#[pyclass(name = "RerunRecorder", module = "simplant_lab.recording")]
pub struct PyRerunRecorder(pub RerunRecorder);

#[pymethods]
impl PyRerunRecorder {
    #[new]
    fn new(stream: &Bound<'_, PyAny>) -> PyResult<Self> {
        recorder_from_py(stream).map(PyRerunRecorder)
    }

    #[staticmethod]
    fn to_file(py: Python<'_>, app_id: String, path: String) -> PyResult<Self> {
        py.detach(|| {
            RerunRecorder::to_file(&app_id, path)
                .map(PyRerunRecorder)
                .map_err(map_err)
        })
    }

    fn flush(&self) {
        self.0.flush();
    }
}

#[pyfunction]
#[pyo3(name = "tag_entity_path")]
fn py_tag_entity_path(tag: &PyTagId) -> String {
    tag_entity_path(&tag.0)
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let recording = PyModule::new(py, "recording")?;
    recording.add_class::<PyRerunRecorder>()?;
    recording.add("PLANT_TIME", PLANT_TIME)?;
    recording.add("EVENTS_PATH", EVENTS_PATH)?;
    recording.add_function(wrap_pyfunction!(py_tag_entity_path, &recording)?)?;

    crate::attach_simplant_submodule(py, parent, "recording", &recording)
}

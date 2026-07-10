mod acquisition;
mod asset_model;
mod error;
mod kernel;
mod ml_dataloop;
mod recording;
mod simulation;
mod stress_testing;
mod types;

pub use error::map_err;
pub use recording::register_recording_stream_extractor;

use pyo3::prelude::*;

pub(crate) fn attach_simplant_submodule(
    py: Python<'_>,
    parent: &Bound<'_, PyModule>,
    name: &str,
    submodule: &Bound<'_, PyModule>,
) -> PyResult<()> {
    parent.add_submodule(submodule)?;
    if let Ok(simplant_lab) = py.import("simplant_lab") {
        simplant_lab.setattr(name, submodule)?;
    }
    Ok(())
}

/// Register all SimPlant domain submodules on `parent`.
pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    kernel::register(py, parent)?;
    asset_model::register(py, parent)?;
    acquisition::register(py, parent)?;
    simulation::register(py, parent)?;
    ml_dataloop::register(py, parent)?;
    stress_testing::register(py, parent)?;
    recording::register(py, parent)?;
    types::register(py, parent)?;
    Ok(())
}

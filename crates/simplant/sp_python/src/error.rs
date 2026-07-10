use pyo3::PyErr;
use pyo3::exceptions::PyValueError;

/// Map domain invariant violations to Python `ValueError`.
pub fn map_err<E: std::error::Error>(e: E) -> PyErr {
    PyValueError::new_err(e.to_string())
}

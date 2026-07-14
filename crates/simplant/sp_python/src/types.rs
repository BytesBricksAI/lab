use pyo3::prelude::*;
use sp_types::{
    ARCHETYPE_PID_PIPE, ARCHETYPE_PID_SYMBOL, ARCHETYPE_PROCESS_VARIABLE, ARCHETYPE_TAG_METADATA,
    COMPONENT_QUALITY, ProcessVariableSample, TagMetadata, field as sp_field,
};

use crate::kernel::{Quality, UnitOfMeasure, kernel_quality_from_py, kernel_unit_from_py};

#[pyclass(name = "ProcessVariableSample", module = "simplant_lab.types")]
pub struct PyProcessVariableSample(pub ProcessVariableSample);

#[pymethods]
impl PyProcessVariableSample {
    #[new]
    fn new(value: f64, quality: Quality) -> Self {
        Self(ProcessVariableSample {
            value,
            quality: sp_types::Quality::from(kernel_quality_from_py(quality)),
        })
    }

    fn value(&self) -> f64 {
        self.0.value
    }

    fn quality(&self) -> Quality {
        match self.0.quality.to_u8() {
            0 => Quality::Bad,
            1 => Quality::Uncertain,
            _ => Quality::Good,
        }
    }
}

#[pyclass(name = "TagMetadata", module = "simplant_lab.types")]
pub struct PyTagMetadata(pub TagMetadata);

#[pymethods]
impl PyTagMetadata {
    #[new]
    fn new(unit: UnitOfMeasure, range_low: f64, range_high: f64) -> Self {
        Self(TagMetadata::new(
            kernel_unit_from_py(unit),
            range_low,
            range_high,
        ))
    }

    fn unit_symbol(&self) -> &str {
        self.0.unit_symbol.as_str()
    }

    fn range_low(&self) -> f64 {
        self.0.range_low
    }

    fn range_high(&self) -> f64 {
        self.0.range_high
    }

    fn alarm_low(&self) -> Option<f64> {
        self.0.alarm_low
    }

    fn alarm_high(&self) -> Option<f64> {
        self.0.alarm_high
    }
}

#[pyfunction]
#[pyo3(name = "field")]
fn py_field(archetype: &str, field_name: &str) -> String {
    sp_field(archetype, field_name)
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let types = PyModule::new(py, "types")?;
    types.add_class::<PyProcessVariableSample>()?;
    types.add_class::<PyTagMetadata>()?;
    types.add("Quality", py.get_type::<Quality>())?;
    types.add("ARCHETYPE_PID_PIPE", ARCHETYPE_PID_PIPE)?;
    types.add("ARCHETYPE_PID_SYMBOL", ARCHETYPE_PID_SYMBOL)?;
    types.add("ARCHETYPE_PROCESS_VARIABLE", ARCHETYPE_PROCESS_VARIABLE)?;
    types.add("ARCHETYPE_TAG_METADATA", ARCHETYPE_TAG_METADATA)?;
    types.add("COMPONENT_QUALITY", COMPONENT_QUALITY)?;
    types.add_function(wrap_pyfunction!(py_field, &types)?)?;

    crate::attach_simplant_submodule(py, parent, "types", &types)
}

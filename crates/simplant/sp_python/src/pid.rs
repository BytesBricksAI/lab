use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use sp_pid_viewer::symbols;

use crate::asset_model::{PyEquipmentKind, equipment_kind_to_domain};

#[pyclass(name = "Connector", module = "simplant_lab.pid")]
#[derive(Clone, Copy)]
pub struct PyConnector {
    index: u8,
    direction_deg: u16,
    pos: (f32, f32),
}

#[pymethods]
impl PyConnector {
    /// 1-based connector index within the symbol.
    #[getter]
    fn index(&self) -> u8 {
        self.index
    }

    /// Compass direction a line approaches from, in degrees (0 = top, 90 = right).
    #[getter]
    fn direction_deg(&self) -> u16 {
        self.direction_deg
    }

    /// Position in the symbol's viewBox coordinates (y down).
    #[getter]
    fn pos(&self) -> (f32, f32) {
        self.pos
    }

    fn __repr__(&self) -> String {
        format!(
            "Connector(index={}, direction_deg={}, pos=({}, {}))",
            self.index, self.direction_deg, self.pos.0, self.pos.1
        )
    }
}

#[pyclass(name = "Symbol", module = "simplant_lab.pid")]
pub struct PySymbol(&'static sp_pid_viewer::Symbol);

impl PySymbol {
    fn meta(&self) -> PyResult<&'static sp_pid_viewer::SymbolMeta> {
        self.0.meta().ok_or_else(|| {
            PyValueError::new_err(format!(
                "symbol {:?} has no parseable SVG metadata",
                self.0.id
            ))
        })
    }
}

#[pymethods]
impl PySymbol {
    fn id(&self) -> &'static str {
        self.0.id
    }

    /// `"equipment"` or `"instrument"` (ISA bubble: tag drawn inside).
    fn kind(&self) -> &'static str {
        match self.0.kind {
            sp_pid_viewer::SymbolKind::Equipment => "equipment",
            sp_pid_viewer::SymbolKind::Instrument => "instrument",
        }
    }

    fn svg(&self) -> &'static [u8] {
        self.0.svg
    }

    /// viewBox width/height of the vendored SVG.
    fn view_box(&self) -> PyResult<(f32, f32)> {
        Ok(self.meta()?.view_box.into())
    }

    /// Connection points declared by the Equinor SVG (may be empty).
    fn connectors(&self) -> PyResult<Vec<PyConnector>> {
        Ok(self
            .meta()?
            .connectors
            .iter()
            .map(|connector| PyConnector {
                index: connector.index,
                direction_deg: connector.direction_deg,
                pos: connector.pos.into(),
            })
            .collect())
    }

    /// Diagram position of connector `index` for a symbol drawn at
    /// `position` with `half_size` — the exact point the viewer maps it to,
    /// so pipes anchored here meet the glyph with no gap.
    fn anchor(&self, index: u8, position: [f32; 2], half_size: [f32; 2]) -> PyResult<(f32, f32)> {
        symbols::connector_point(position, half_size, self.meta()?, index)
            .map(Into::into)
            .ok_or_else(|| {
                PyValueError::new_err(format!(
                    "symbol {:?} has no connector with index {index}",
                    self.0.id
                ))
            })
    }

    /// Half-extents matching the symbol's native aspect ratio, from a full
    /// width *or* a full height in diagram units (exactly one).
    #[pyo3(signature = (*, width=None, height=None))]
    fn aspect_half_size(&self, width: Option<f32>, height: Option<f32>) -> PyResult<(f32, f32)> {
        let view_box = self.meta()?.view_box;
        let half_size = match (width, height) {
            (Some(width), None) => symbols::half_size_for_width(view_box, width),
            (None, Some(height)) => symbols::half_size_for_height(view_box, height),
            _ => {
                return Err(PyValueError::new_err(
                    "pass exactly one of width= or height=",
                ));
            }
        };
        Ok(half_size.into())
    }
}

#[pyfunction]
fn symbol_id_for(kind: PyEquipmentKind) -> Option<&'static str> {
    sp_pid_viewer::symbol_id_for(equipment_kind_to_domain(kind))
}

#[pyfunction]
fn symbol_ids() -> Vec<&'static str> {
    symbols::SYMBOLS.iter().map(|symbol| symbol.id).collect()
}

#[pyfunction]
fn find_symbol(symbol_id: &str) -> Option<PySymbol> {
    symbols::find(symbol_id).map(PySymbol)
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let pid = PyModule::new(py, "pid")?;
    pid.add_class::<PySymbol>()?;
    pid.add_class::<PyConnector>()?;
    pid.add(
        "VIEW_CLASS_IDENTIFIER",
        sp_pid_viewer::VIEW_CLASS_IDENTIFIER,
    )?;
    pid.add_function(wrap_pyfunction!(symbol_id_for, &pid)?)?;
    pid.add_function(wrap_pyfunction!(symbol_ids, &pid)?)?;
    pid.add_function(wrap_pyfunction!(find_symbol, &pid)?)?;

    crate::attach_simplant_submodule(py, parent, "pid", &pid)
}

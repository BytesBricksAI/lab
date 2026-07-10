use pyo3::prelude::*;
use sp_asset_model::application::ports::AssetCatalogPort;
use sp_asset_model::{
    Area, AreaId, AssetCatalog, Equipment, EquipmentId, EquipmentKind, Facility, FacilityId,
    ProcessUnit, TomlCatalogRepository, UnitId,
};
use sp_kernel::UnitOfMeasure as KernelUnitOfMeasure;

use crate::kernel::{PyAlarmLimits, PyEngineeringRange, PyTagId, UnitOfMeasure};
use crate::map_err;

fn py_unit_from_kernel(unit: KernelUnitOfMeasure) -> UnitOfMeasure {
    match unit {
        KernelUnitOfMeasure::Kilopascal => UnitOfMeasure::Kilopascal,
        KernelUnitOfMeasure::Bar => UnitOfMeasure::Bar,
        KernelUnitOfMeasure::Psi => UnitOfMeasure::Psi,
        KernelUnitOfMeasure::Megapascal => UnitOfMeasure::Megapascal,
        KernelUnitOfMeasure::DegreeCelsius => UnitOfMeasure::DegreeCelsius,
        KernelUnitOfMeasure::Kelvin => UnitOfMeasure::Kelvin,
        KernelUnitOfMeasure::CubicMeterPerHour => UnitOfMeasure::CubicMeterPerHour,
        KernelUnitOfMeasure::BarrelPerDay => UnitOfMeasure::BarrelPerDay,
        KernelUnitOfMeasure::KilogramPerHour => UnitOfMeasure::KilogramPerHour,
        KernelUnitOfMeasure::Meter => UnitOfMeasure::Meter,
        KernelUnitOfMeasure::Percent => UnitOfMeasure::Percent,
        KernelUnitOfMeasure::Ratio => UnitOfMeasure::Ratio,
    }
}

macro_rules! define_py_id {
    ($py_name:ident, $domain:ty, $py_class:literal) => {
        #[pyclass(name = $py_class, module = "simplant_lab.asset_model")]
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

define_py_id!(PyFacilityId, FacilityId, "FacilityId");
define_py_id!(PyAreaId, AreaId, "AreaId");
define_py_id!(PyUnitId, UnitId, "UnitId");
define_py_id!(PyEquipmentId, EquipmentId, "EquipmentId");

#[pyclass(
    name = "EquipmentKind",
    eq,
    eq_int,
    module = "simplant_lab.asset_model"
)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyEquipmentKind {
    Vessel,
    Tank,
    Pump,
    HeatExchanger,
    Valve,
    Pipe,
    Other,
}

fn py_equipment_kind_from_domain(kind: EquipmentKind) -> PyEquipmentKind {
    match kind {
        EquipmentKind::Vessel => PyEquipmentKind::Vessel,
        EquipmentKind::Tank => PyEquipmentKind::Tank,
        EquipmentKind::Pump => PyEquipmentKind::Pump,
        EquipmentKind::HeatExchanger => PyEquipmentKind::HeatExchanger,
        EquipmentKind::Valve => PyEquipmentKind::Valve,
        EquipmentKind::Pipe => PyEquipmentKind::Pipe,
        EquipmentKind::Other => PyEquipmentKind::Other,
    }
}

#[pyclass(name = "ProcessUnit", module = "simplant_lab.asset_model")]
#[derive(Clone)]
pub struct PyProcessUnit(pub ProcessUnit);

#[pymethods]
impl PyProcessUnit {
    fn id(&self) -> PyUnitId {
        PyUnitId(self.0.id().clone())
    }

    fn name(&self) -> &str {
        self.0.name()
    }
}

#[pyclass(name = "Area", module = "simplant_lab.asset_model")]
#[derive(Clone)]
pub struct PyArea(pub Area);

#[pymethods]
impl PyArea {
    fn id(&self) -> PyAreaId {
        PyAreaId(self.0.id().clone())
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn units(&self) -> Vec<PyProcessUnit> {
        self.0.units().iter().cloned().map(PyProcessUnit).collect()
    }
}

#[pyclass(name = "Facility", module = "simplant_lab.asset_model")]
pub struct PyFacility(pub Facility);

#[pymethods]
impl PyFacility {
    #[staticmethod]
    fn define(id: PyFacilityId, name: String) -> Self {
        let (facility, _) = Facility::define(id.0, name);
        Self(facility)
    }

    fn id(&self) -> PyFacilityId {
        PyFacilityId(self.0.id().clone())
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn areas(&self) -> Vec<PyArea> {
        self.0.areas().iter().cloned().map(PyArea).collect()
    }

    fn has_area(&self, area: PyAreaId) -> bool {
        self.0.has_area(&area.0)
    }

    fn has_unit(&self, unit: PyUnitId) -> bool {
        self.0.has_unit(&unit.0)
    }

    fn add_area(&mut self, id: PyAreaId, name: String) -> PyResult<()> {
        self.0.add_area(id.0, name).map(|_| ()).map_err(map_err)
    }

    fn add_unit(&mut self, area: PyAreaId, unit: PyUnitId, name: String) -> PyResult<()> {
        self.0
            .add_unit(&area.0, unit.0, name)
            .map(|_| ())
            .map_err(map_err)
    }
}

#[pyclass(name = "Equipment", module = "simplant_lab.asset_model")]
#[derive(Clone)]
pub struct PyEquipment(pub Equipment);

#[pymethods]
impl PyEquipment {
    fn id(&self) -> PyEquipmentId {
        PyEquipmentId(self.0.id().clone())
    }

    fn unit(&self) -> PyUnitId {
        PyUnitId(self.0.unit().clone())
    }

    fn name(&self) -> &str {
        self.0.name()
    }

    fn kind(&self) -> PyEquipmentKind {
        py_equipment_kind_from_domain(self.0.kind())
    }
}

#[pyclass(name = "Tag", module = "simplant_lab.asset_model")]
#[derive(Clone)]
pub struct PyTag(pub sp_asset_model::Tag);

#[pymethods]
impl PyTag {
    fn id(&self) -> PyTagId {
        PyTagId(self.0.id().clone())
    }

    fn equipment(&self) -> PyEquipmentId {
        PyEquipmentId(self.0.equipment().clone())
    }

    fn description(&self) -> &str {
        self.0.description()
    }

    fn unit(&self) -> UnitOfMeasure {
        py_unit_from_kernel(self.0.unit())
    }

    fn range(&self) -> PyEngineeringRange {
        PyEngineeringRange(self.0.range())
    }

    fn alarms(&self) -> Option<PyAlarmLimits> {
        self.0.alarms().map(PyAlarmLimits)
    }
}

#[pyclass(name = "AssetCatalog", module = "simplant_lab.asset_model")]
pub struct PyAssetCatalog(pub AssetCatalog);

#[pymethods]
impl PyAssetCatalog {
    fn facility(&self) -> PyFacility {
        PyFacility(self.0.facility().clone())
    }

    fn equipment(&self) -> Vec<PyEquipment> {
        self.0
            .equipment()
            .iter()
            .cloned()
            .map(PyEquipment)
            .collect()
    }

    fn tags(&self) -> Vec<PyTag> {
        self.0.tags().iter().cloned().map(PyTag).collect()
    }

    #[pyo3(signature = (id))]
    fn tag(&self, id: PyTagId) -> Option<PyTag> {
        self.0.tag(&id.0).cloned().map(PyTag)
    }

    #[pyo3(signature = (id))]
    fn equipment_by_id(&self, id: PyEquipmentId) -> Option<PyEquipment> {
        self.0.equipment_by_id(&id.0).cloned().map(PyEquipment)
    }

    fn validate(&self) -> PyResult<()> {
        self.0.validate().map_err(map_err)
    }
}

#[pyclass(name = "TomlCatalogRepository", module = "simplant_lab.asset_model")]
pub struct PyTomlCatalogRepository(pub TomlCatalogRepository);

#[pymethods]
impl PyTomlCatalogRepository {
    #[new]
    fn new(path: String) -> Self {
        Self(TomlCatalogRepository::new(path))
    }

    fn load_catalog(&self, py: Python<'_>) -> PyResult<PyAssetCatalog> {
        py.detach(|| self.0.load_catalog().map(PyAssetCatalog).map_err(map_err))
    }
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let asset_model = PyModule::new(py, "asset_model")?;
    asset_model.add_class::<PyFacilityId>()?;
    asset_model.add_class::<PyAreaId>()?;
    asset_model.add_class::<PyUnitId>()?;
    asset_model.add_class::<PyEquipmentId>()?;
    asset_model.add_class::<PyProcessUnit>()?;
    asset_model.add_class::<PyArea>()?;
    asset_model.add_class::<PyFacility>()?;
    asset_model.add_class::<PyEquipment>()?;
    asset_model.add_class::<PyTag>()?;
    asset_model.add_class::<PyAssetCatalog>()?;
    asset_model.add_class::<PyTomlCatalogRepository>()?;
    asset_model.add_class::<PyEquipmentKind>()?;

    crate::attach_simplant_submodule(py, parent, "asset_model", &asset_model)
}

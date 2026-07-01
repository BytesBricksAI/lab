use jiff::Timestamp;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use sp_kernel::{
    AlarmLimits, Dimension as KernelDimension, EngineeringRange, Measurement, MeasurementBatch,
    Quality as KernelQuality, TagId, TimeWindow, UnitOfMeasure as KernelUnitOfMeasure,
};

use crate::map_err;

fn timestamp_from_epoch_secs(epoch_secs: f64) -> PyResult<Timestamp> {
    if !epoch_secs.is_finite() {
        return Err(PyValueError::new_err("timestamp must be finite"));
    }
    let nanos = (epoch_secs * 1_000_000_000.0).round() as i128;
    Timestamp::from_nanosecond(nanos).map_err(map_err)
}

fn timestamp_to_epoch_secs(ts: Timestamp) -> f64 {
    ts.as_second() as f64 + ts.subsec_nanosecond() as f64 / 1_000_000_000.0
}

fn timestamp_to_nanos(ts: Timestamp) -> PyResult<i64> {
    ts.as_nanosecond()
        .try_into()
        .map_err(|_| PyValueError::new_err("timestamp nanoseconds out of i64 range"))
}

fn timestamp_to_iso(ts: Timestamp) -> String {
    ts.to_string()
}

fn signed_duration_to_secs(duration: jiff::SignedDuration) -> f64 {
    duration.as_nanos() as f64 / 1_000_000_000.0
}

#[pyclass(eq, eq_int, module = "simplant_lab.kernel")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Good,
    Uncertain,
    Bad,
}

#[pymethods]
impl Quality {
    fn is_usable(&self) -> bool {
        kernel_quality_from_py(*self).is_usable()
    }
}

pub(crate) fn kernel_quality_from_py(quality: Quality) -> KernelQuality {
    match quality {
        Quality::Good => KernelQuality::Good,
        Quality::Uncertain => KernelQuality::Uncertain,
        Quality::Bad => KernelQuality::Bad,
    }
}

fn py_quality_from_kernel(quality: KernelQuality) -> Quality {
    match quality {
        KernelQuality::Good => Quality::Good,
        KernelQuality::Uncertain => Quality::Uncertain,
        KernelQuality::Bad => Quality::Bad,
    }
}

#[pyclass(eq, eq_int, module = "simplant_lab.kernel")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    Pressure,
    Temperature,
    VolumetricFlow,
    MassFlow,
    Length,
    Dimensionless,
}

fn py_dimension_from_kernel(dimension: KernelDimension) -> Dimension {
    match dimension {
        KernelDimension::Pressure => Dimension::Pressure,
        KernelDimension::Temperature => Dimension::Temperature,
        KernelDimension::VolumetricFlow => Dimension::VolumetricFlow,
        KernelDimension::MassFlow => Dimension::MassFlow,
        KernelDimension::Length => Dimension::Length,
        KernelDimension::Dimensionless => Dimension::Dimensionless,
    }
}

#[pyclass(eq, eq_int, module = "simplant_lab.kernel")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UnitOfMeasure {
    Kilopascal,
    Bar,
    Psi,
    Megapascal,
    DegreeCelsius,
    Kelvin,
    CubicMeterPerHour,
    BarrelPerDay,
    KilogramPerHour,
    Meter,
    Percent,
    Ratio,
}

#[pymethods]
impl UnitOfMeasure {
    fn dimension(&self) -> Dimension {
        py_dimension_from_kernel(kernel_unit_from_py(*self).dimension())
    }

    fn symbol(&self) -> &'static str {
        kernel_unit_from_py(*self).symbol()
    }

    fn to_base(&self, value: f64) -> f64 {
        kernel_unit_from_py(*self).to_base(value)
    }

    fn from_base(&self, base_value: f64) -> f64 {
        kernel_unit_from_py(*self).from_base(base_value)
    }

    fn same_dimension(&self, other: UnitOfMeasure) -> bool {
        kernel_unit_from_py(*self).same_dimension(&kernel_unit_from_py(other))
    }
}

pub(crate) fn kernel_unit_from_py(unit: UnitOfMeasure) -> KernelUnitOfMeasure {
    match unit {
        UnitOfMeasure::Kilopascal => KernelUnitOfMeasure::Kilopascal,
        UnitOfMeasure::Bar => KernelUnitOfMeasure::Bar,
        UnitOfMeasure::Psi => KernelUnitOfMeasure::Psi,
        UnitOfMeasure::Megapascal => KernelUnitOfMeasure::Megapascal,
        UnitOfMeasure::DegreeCelsius => KernelUnitOfMeasure::DegreeCelsius,
        UnitOfMeasure::Kelvin => KernelUnitOfMeasure::Kelvin,
        UnitOfMeasure::CubicMeterPerHour => KernelUnitOfMeasure::CubicMeterPerHour,
        UnitOfMeasure::BarrelPerDay => KernelUnitOfMeasure::BarrelPerDay,
        UnitOfMeasure::KilogramPerHour => KernelUnitOfMeasure::KilogramPerHour,
        UnitOfMeasure::Meter => KernelUnitOfMeasure::Meter,
        UnitOfMeasure::Percent => KernelUnitOfMeasure::Percent,
        UnitOfMeasure::Ratio => KernelUnitOfMeasure::Ratio,
    }
}

#[pyclass(name = "TagId", module = "simplant_lab.kernel")]
#[derive(Clone)]
pub struct PyTagId(pub TagId);

#[pymethods]
impl PyTagId {
    #[new]
    fn new(raw: String) -> PyResult<Self> {
        TagId::new(raw).map(PyTagId).map_err(map_err)
    }

    fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn __str__(&self) -> &str {
        self.as_str()
    }
}

#[pyclass(name = "Measurement", module = "simplant_lab.kernel")]
#[derive(Clone, Copy)]
pub struct PyMeasurement(pub Measurement);

#[pymethods]
impl PyMeasurement {
    #[new]
    fn new(value: f64, quality: Quality, timestamp: f64) -> PyResult<Self> {
        let timestamp = timestamp_from_epoch_secs(timestamp)?;
        Ok(Self(Measurement::new(
            value,
            kernel_quality_from_py(quality),
            timestamp,
        )))
    }

    fn value(&self) -> f64 {
        self.0.value()
    }

    fn quality(&self) -> Quality {
        py_quality_from_kernel(self.0.quality())
    }

    fn timestamp(&self) -> f64 {
        timestamp_to_epoch_secs(self.0.timestamp())
    }

    fn timestamp_nanos(&self) -> PyResult<i64> {
        timestamp_to_nanos(self.0.timestamp())
    }

    fn timestamp_iso(&self) -> String {
        timestamp_to_iso(self.0.timestamp())
    }

    fn __str__(&self) -> String {
        let quality = match self.quality() {
            Quality::Good => "Good",
            Quality::Uncertain => "Uncertain",
            Quality::Bad => "Bad",
        };
        format!(
            "Measurement(value={}, quality={}, timestamp={})",
            self.value(),
            quality,
            self.timestamp_iso()
        )
    }
}

#[pyclass(name = "MeasurementBatch", module = "simplant_lab.kernel")]
#[derive(Clone)]
pub struct PyMeasurementBatch(pub MeasurementBatch);

#[pymethods]
impl PyMeasurementBatch {
    #[new]
    fn new(tag: PyTagId, samples: Vec<PyMeasurement>) -> Self {
        Self(MeasurementBatch::new(
            tag.0,
            samples.into_iter().map(|sample| sample.0).collect(),
        ))
    }

    fn tag(&self) -> PyTagId {
        PyTagId(self.0.tag().clone())
    }

    fn samples(&self) -> Vec<PyMeasurement> {
        self.0
            .samples()
            .iter()
            .copied()
            .map(PyMeasurement)
            .collect()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn time_span(&self) -> Option<PyTimeWindow> {
        self.0.time_span().map(PyTimeWindow)
    }
}

#[pyclass(name = "TimeWindow", module = "simplant_lab.kernel")]
#[derive(Clone, Copy)]
pub struct PyTimeWindow(pub TimeWindow);

#[pymethods]
impl PyTimeWindow {
    #[new]
    fn new(start: f64, end: f64) -> PyResult<Self> {
        let start = timestamp_from_epoch_secs(start)?;
        let end = timestamp_from_epoch_secs(end)?;
        TimeWindow::new(start, end)
            .map(PyTimeWindow)
            .map_err(map_err)
    }

    fn start(&self) -> f64 {
        timestamp_to_epoch_secs(self.0.start())
    }

    fn end(&self) -> f64 {
        timestamp_to_epoch_secs(self.0.end())
    }

    fn start_nanos(&self) -> PyResult<i64> {
        timestamp_to_nanos(self.0.start())
    }

    fn end_nanos(&self) -> PyResult<i64> {
        timestamp_to_nanos(self.0.end())
    }

    fn start_iso(&self) -> String {
        timestamp_to_iso(self.0.start())
    }

    fn end_iso(&self) -> String {
        timestamp_to_iso(self.0.end())
    }

    fn contains(&self, ts: f64) -> PyResult<bool> {
        let ts = timestamp_from_epoch_secs(ts)?;
        Ok(self.0.contains(ts))
    }

    fn overlaps(&self, other: PyTimeWindow) -> bool {
        self.0.overlaps(&other.0)
    }

    fn duration(&self) -> f64 {
        signed_duration_to_secs(self.0.duration())
    }

    fn __str__(&self) -> String {
        format!(
            "TimeWindow(start={}, end={})",
            self.start_iso(),
            self.end_iso()
        )
    }
}

#[pyclass(name = "EngineeringRange", module = "simplant_lab.kernel")]
#[derive(Clone, Copy)]
pub struct PyEngineeringRange(pub EngineeringRange);

#[pymethods]
impl PyEngineeringRange {
    #[new]
    fn new(low: f64, high: f64, unit: UnitOfMeasure) -> PyResult<Self> {
        EngineeringRange::new(low, high, kernel_unit_from_py(unit))
            .map(PyEngineeringRange)
            .map_err(map_err)
    }

    fn low(&self) -> f64 {
        self.0.low()
    }

    fn high(&self) -> f64 {
        self.0.high()
    }

    fn unit(&self) -> UnitOfMeasure {
        match self.0.unit() {
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

    fn span(&self) -> f64 {
        self.0.span()
    }

    fn contains(&self, value: f64) -> bool {
        self.0.contains(value)
    }
}

#[pyclass(name = "AlarmLimits", module = "simplant_lab.kernel")]
#[derive(Clone, Copy)]
pub struct PyAlarmLimits(pub AlarmLimits);

#[pymethods]
impl PyAlarmLimits {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (low_low=None, low=None, high=None, high_high=None, *, unit))]
    fn new(
        low_low: Option<f64>,
        low: Option<f64>,
        high: Option<f64>,
        high_high: Option<f64>,
        unit: UnitOfMeasure,
    ) -> PyResult<Self> {
        AlarmLimits::new(low_low, low, high, high_high, kernel_unit_from_py(unit))
            .map(PyAlarmLimits)
            .map_err(map_err)
    }

    fn low_low(&self) -> Option<f64> {
        self.0.low_low()
    }

    fn low(&self) -> Option<f64> {
        self.0.low()
    }

    fn high(&self) -> Option<f64> {
        self.0.high()
    }

    fn high_high(&self) -> Option<f64> {
        self.0.high_high()
    }

    fn unit(&self) -> UnitOfMeasure {
        match self.0.unit() {
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
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let kernel = PyModule::new(py, "kernel")?;
    kernel.add_class::<PyTagId>()?;
    kernel.add_class::<Quality>()?;
    kernel.add_class::<PyMeasurement>()?;
    kernel.add_class::<PyMeasurementBatch>()?;
    kernel.add_class::<PyTimeWindow>()?;
    kernel.add_class::<Dimension>()?;
    kernel.add_class::<UnitOfMeasure>()?;
    kernel.add_class::<PyEngineeringRange>()?;
    kernel.add_class::<PyAlarmLimits>()?;

    crate::attach_simplant_submodule(py, parent, "kernel", &kernel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::exceptions::PyValueError;
    use pyo3::types::PyModule;

    fn register_kernel(py: Python<'_>) -> Bound<'_, PyModule> {
        let module = PyModule::new(py, "kernel").expect("kernel module");
        module.add_class::<PyTagId>().expect("TagId");
        module.add_class::<Quality>().expect("Quality");
        module.add_class::<PyMeasurement>().expect("Measurement");
        module.add_class::<PyTimeWindow>().expect("TimeWindow");
        module
    }

    #[test]
    fn tag_id_round_trip_and_rejects_empty() {
        Python::initialize();
        Python::attach(|py| {
            let kernel = register_kernel(py);

            let tag = PyTagId::new("FT-101".to_owned()).expect("valid TagId");
            assert_eq!(tag.as_str(), "FT-101");
            assert_eq!(tag.__str__(), "FT-101");

            let py_tag = kernel.getattr("TagId")?.call1(("FT-101",))?;
            assert_eq!(
                py_tag.call_method0("as_str")?.extract::<String>()?,
                "FT-101"
            );

            assert!(PyTagId::new(String::new()).is_err());

            let err = kernel
                .getattr("TagId")?
                .call1(("",))
                .expect_err("empty TagId must raise");
            assert!(err.is_instance_of::<PyValueError>(py));

            Ok::<(), PyErr>(())
        })
        .expect("Python::attach");
    }

    #[test]
    fn measurement_round_trip() {
        Python::initialize();
        Python::attach(|py| {
            let kernel = register_kernel(py);
            let epoch_secs = 1_704_067_200.0;

            let measurement =
                PyMeasurement::new(42.0, Quality::Good, epoch_secs).expect("Measurement");
            assert_eq!(measurement.value(), 42.0);
            assert_eq!(measurement.quality(), Quality::Good);
            assert!(
                (measurement.timestamp() - epoch_secs).abs() < 1e-6,
                "timestamp round-trip: got {}, expected {epoch_secs}",
                measurement.timestamp()
            );

            let py_measurement =
                kernel
                    .getattr("Measurement")?
                    .call1((42.0, Quality::Good, epoch_secs))?;
            assert_eq!(
                py_measurement.call_method0("value")?.extract::<f64>()?,
                42.0
            );
            assert_eq!(
                py_measurement.call_method0("timestamp")?.extract::<f64>()?,
                measurement.timestamp()
            );

            Ok::<(), PyErr>(())
        })
        .expect("Python::attach");
    }

    #[test]
    fn time_window_rejects_inverted_range() {
        Python::initialize();
        Python::attach(|py| {
            let kernel = register_kernel(py);

            let valid = PyTimeWindow::new(100.0, 200.0).expect("valid TimeWindow");
            assert!(valid.contains(150.0).expect("contains"));
            assert!(!valid.contains(200.0).expect("contains"));

            assert!(PyTimeWindow::new(200.0, 100.0).is_err());

            let err = kernel
                .getattr("TimeWindow")?
                .call1((200.0, 100.0))
                .expect_err("inverted TimeWindow must raise");
            assert!(err.is_instance_of::<PyValueError>(py));

            Ok::<(), PyErr>(())
        })
        .expect("Python::attach");
    }
}

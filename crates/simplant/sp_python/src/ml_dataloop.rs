use pyo3::prelude::*;
use sp_ml_dataloop::{DataSplit, DatasetSpec, FeatureSpec};

use crate::asset_model::PyAssetCatalog;
use crate::kernel::{PyTagId, PyTimeWindow};
use crate::map_err;

#[pyclass(name = "FeatureSpec", module = "simplant_lab.ml_dataloop")]
#[derive(Clone)]
pub struct PyFeatureSpec(pub FeatureSpec);

#[pymethods]
impl PyFeatureSpec {
    #[new]
    fn new(tag: PyTagId, name: String) -> PyResult<Self> {
        FeatureSpec::new(tag.0, name)
            .map(PyFeatureSpec)
            .map_err(map_err)
    }

    fn tag(&self) -> PyTagId {
        PyTagId(self.0.tag().clone())
    }

    fn name(&self) -> &str {
        self.0.name()
    }
}

#[pyclass(name = "DataSplit", module = "simplant_lab.ml_dataloop")]
#[derive(Clone)]
pub struct PyDataSplit(pub DataSplit);

#[pymethods]
impl PyDataSplit {
    #[new]
    #[pyo3(signature = (train, test, val=None))]
    fn new(train: PyTimeWindow, test: PyTimeWindow, val: Option<PyTimeWindow>) -> PyResult<Self> {
        DataSplit::new(train.0, val.map(|v| v.0), test.0)
            .map(PyDataSplit)
            .map_err(map_err)
    }

    fn train(&self) -> PyTimeWindow {
        PyTimeWindow(self.0.train())
    }

    fn val(&self) -> Option<PyTimeWindow> {
        self.0.val().map(PyTimeWindow)
    }

    fn test(&self) -> PyTimeWindow {
        PyTimeWindow(self.0.test())
    }

    fn windows(&self) -> Vec<(String, PyTimeWindow)> {
        self.0
            .windows()
            .into_iter()
            .map(|(name, window)| (name.to_owned(), PyTimeWindow(window)))
            .collect()
    }
}

#[pyclass(name = "DatasetSpec", module = "simplant_lab.ml_dataloop")]
pub struct PyDatasetSpec(pub DatasetSpec);

#[pymethods]
impl PyDatasetSpec {
    #[staticmethod]
    fn define(
        id: String,
        features: Vec<PyFeatureSpec>,
        targets: Vec<PyFeatureSpec>,
        split: PyDataSplit,
        catalog: &PyAssetCatalog,
    ) -> PyResult<Self> {
        DatasetSpec::define(
            id,
            features.into_iter().map(|f| f.0).collect(),
            targets.into_iter().map(|t| t.0).collect(),
            split.0,
            &catalog.0,
        )
        .map(|(spec, _)| PyDatasetSpec(spec))
        .map_err(map_err)
    }

    fn id(&self) -> &str {
        self.0.id()
    }

    fn version(&self) -> u32 {
        self.0.version()
    }

    fn features(&self) -> Vec<PyFeatureSpec> {
        self.0
            .features()
            .iter()
            .cloned()
            .map(PyFeatureSpec)
            .collect()
    }

    fn targets(&self) -> Vec<PyFeatureSpec> {
        self.0
            .targets()
            .iter()
            .cloned()
            .map(PyFeatureSpec)
            .collect()
    }

    fn split(&self) -> PyDataSplit {
        PyDataSplit(self.0.split().clone())
    }
}

pub mod dataframe_query {
    use pyo3::prelude::*;
    use sp_dataframe_query::RrdDataframeQuery;
    use sp_kernel::TagId;
    use sp_ml_dataloop::{DataframeQueryPort as _, QueryResult, TagSeries};

    use crate::kernel::{PyMeasurement, PyTagId, PyTimeWindow};
    use crate::map_err;

    #[pyclass(
        name = "TagSeries",
        module = "simplant_lab.ml_dataloop.dataframe_query"
    )]
    #[derive(Clone)]
    pub struct PyTagSeries(pub TagSeries);

    #[pymethods]
    impl PyTagSeries {
        fn tag(&self) -> PyTagId {
            PyTagId(self.0.tag.clone())
        }

        fn measurements(&self) -> Vec<PyMeasurement> {
            self.0
                .measurements
                .iter()
                .copied()
                .map(PyMeasurement)
                .collect()
        }
    }

    #[pyclass(
        name = "QueryResult",
        module = "simplant_lab.ml_dataloop.dataframe_query"
    )]
    #[derive(Clone)]
    pub struct PyQueryResult(pub QueryResult);

    #[pymethods]
    impl PyQueryResult {
        fn series(&self) -> Vec<PyTagSeries> {
            self.0.series.iter().cloned().map(PyTagSeries).collect()
        }
    }

    #[pyclass(
        name = "RrdDataframeQuery",
        module = "simplant_lab.ml_dataloop.dataframe_query"
    )]
    pub struct PyRrdDataframeQuery(pub RrdDataframeQuery);

    #[pymethods]
    impl PyRrdDataframeQuery {
        #[staticmethod]
        fn open(py: Python<'_>, path: String) -> PyResult<Self> {
            py.detach(|| {
                RrdDataframeQuery::open(path)
                    .map(PyRrdDataframeQuery)
                    .map_err(map_err)
            })
        }

        fn query(
            &self,
            py: Python<'_>,
            window: PyTimeWindow,
            tags: Vec<PyTagId>,
        ) -> PyResult<PyQueryResult> {
            let tag_ids: Vec<TagId> = tags.into_iter().map(|t| t.0).collect();
            py.detach(|| {
                self.0
                    .query(&window.0, &tag_ids)
                    .map(PyQueryResult)
                    .map_err(map_err)
            })
        }
    }

    pub fn register(parent: &Bound<'_, PyModule>) -> PyResult<()> {
        let dataframe_query = PyModule::new(parent.py(), "dataframe_query")?;
        dataframe_query.add_class::<PyTagSeries>()?;
        dataframe_query.add_class::<PyQueryResult>()?;
        dataframe_query.add_class::<PyRrdDataframeQuery>()?;
        parent.add_submodule(&dataframe_query)?;
        Ok(())
    }
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let ml_dataloop = PyModule::new(py, "ml_dataloop")?;
    ml_dataloop.add_class::<PyFeatureSpec>()?;
    ml_dataloop.add_class::<PyDataSplit>()?;
    ml_dataloop.add_class::<PyDatasetSpec>()?;

    dataframe_query::register(&ml_dataloop)?;

    crate::attach_simplant_submodule(py, parent, "ml_dataloop", &ml_dataloop)
}

use pyo3::prelude::*;

#[pyclass]
pub struct Metadata(liquers_core::metadata::Metadata);

#[pymethods]
impl Metadata {
    #[new]
    pub fn new() -> Self {
        Metadata(liquers_core::metadata::Metadata::new())
    }
    #[staticmethod]
    pub fn from_json(json: &str) -> PyResult<Self> {
        match liquers_core::metadata::Metadata::from_json(json) {
            Ok(m) => Ok(Metadata(m)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(
                e.to_string(),
            )),
        }
    }
    pub fn to_json(&self) -> PyResult<String> {
        match self.0.to_json() {
            Ok(s) => Ok(s),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(
                e.to_string(),
            )),
        }
    }
}

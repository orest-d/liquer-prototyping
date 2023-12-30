use pyo3::prelude::*;

#[pyclass]
pub struct Store(Box<dyn liquers_core::store::Store + Send>);

#[pyfunction]
pub fn local_filesystem_store(path: &str, prefix: &str) -> PyResult<Store> {
    let key = liquers_core::parse::parse_key(prefix)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string()))?;
    Ok(Store(Box::new(liquers_core::store::FileStore::new(
        path, &key,
    ))))
}

#[pymethods]
impl Store {
    /// Get store name
    pub fn store_name(&self) -> String {
        self.0.store_name()
    }

    /// Key prefix common to all keys in this store.
    pub fn key_prefix(&self) -> crate::parse::Key {
        crate::parse::Key(self.0.key_prefix().to_owned())
    }

}

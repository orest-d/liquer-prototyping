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

    /// Get data and metadata
    fn get(&self, key: &crate::parse::Key) -> PyResult<(Vec<u8>, crate::metadata::Metadata)> {
        match self.0.get(&key.0) {
            Ok((data, metadata)) => Ok((data, crate::metadata::Metadata(metadata))),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string())),
        }
    }

    /// Get data as bytes
    fn get_bytes(&self, key: &crate::parse::Key) -> PyResult<Vec<u8>> {
        match self.0.get_bytes(&key.0) {
            Ok(data) => Ok(data),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string())),
        }
    }

    /// Get metadata
    fn get_metadata(&self, key: &crate::parse::Key) -> PyResult<crate::metadata::Metadata> {
        match self.0.get_metadata(&key.0) {
            Ok(metadata) => Ok(crate::metadata::Metadata(metadata)),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string())),
        }
    }

    /// Store data and metadata.
    fn set(&mut self, key: &crate::parse::Key, data: &[u8], metadata: &crate::metadata::Metadata) -> PyResult<()> {
        match self.0.set(&key.0, data, &metadata.0) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string())),
        }
    }

    /// Store metadata.
    fn set_metadata(&mut self, key: &crate::parse::Key, metadata: &crate::metadata::Metadata) -> PyResult<()> {
        match self.0.set_metadata(&key.0, &metadata.0) {
            Ok(_) => Ok(()),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string())),
        }
    }


}

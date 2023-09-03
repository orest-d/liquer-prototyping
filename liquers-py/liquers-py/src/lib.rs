use pyo3::prelude::*;

use liquers_core::parse::parse_key;

#[pyclass]
struct Key(liquers_core::query::Key);

#[pymethods]
impl Key {
    #[new]
    fn new(key: &str) -> Self {
        Key(parse_key(key).unwrap())
    }

    /// Return the last element of the key if present, None otherwise.
    /// This is typically interpreted as a filename in a Store object.
    pub fn filename(&self) -> Option<String> {
        self.0.filename().map(|s| s.to_string())
    }

    fn to_string(&self) -> String {
        self.0.to_string()
    }

}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn liquers_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<Key>()?;
    Ok(())
}
use pyo3::prelude::*;
pub mod parse;
pub mod store;
pub mod cache;
pub mod metadata;
use crate::parse::*;


/// A Python module implemented in Rust.
#[pymodule]
fn liquers_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Position>()?;
    m.add_class::<ActionParameter>()?;
    m.add_class::<ResourceName>()?;
    m.add_class::<ActionRequest>()?;
    m.add_class::<SegmentHeader>()?;
    m.add_class::<TransformQuerySegment>()?;
    m.add_class::<Key>()?;
    m.add_class::<ResourceQuerySegment>()?;
    m.add_class::<Query>()?;
    m.add_function(wrap_pyfunction!(crate::parse::parse, m)?)?;
    m.add_function(wrap_pyfunction!(crate::parse::parse_key, m)?)?;

    m.add_class::<crate::metadata::Metadata>()?;

    m.add_class::<crate::store::Store>()?;
    m.add_function(wrap_pyfunction!(crate::store::local_filesystem_store, m)?)?;

    m.add_class::<crate::cache::Cache>()?;
    m.add_function(wrap_pyfunction!(crate::cache::memory_cache, m)?)?;
   
    Ok(())
}

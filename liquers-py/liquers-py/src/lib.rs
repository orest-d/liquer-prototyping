use pyo3::prelude::*;


#[pyclass]
struct Position(liquers_core::query::Position);

#[pymethods]
impl Position {
    #[new]
    fn new(offset: usize, line: u32, column: usize) -> Self {
        Position(liquers_core::query::Position {
            offset,
            line,
            column,
        })
    }

    #[staticmethod]
    fn unknown() -> Self {
        Position(liquers_core::query::Position::unknown())
    }

    #[getter]
    fn offset(&self) -> PyResult<usize> {
        Ok(self.0.offset)
    }

    #[getter]
    fn line(&self) -> u32 {
        self.0.line
    }

    #[getter]
    fn column(&self) -> usize {
        self.0.column
    }

    fn __repr__(&self) -> String {
        format!(
            "Position(offset={}, line={}, column={})",
            self.0.offset, self.0.line, self.0.column
        )
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

#[pyclass]
struct ActionParameter(liquers_core::query::ActionParameter);

#[pymethods]
impl ActionParameter {
    #[new]
    fn new(parameter: String, position: &Position) -> Self {
        ActionParameter(
            liquers_core::query::ActionParameter::new_string(parameter)
                .with_position(position.0.clone()),
        )
    }

    #[getter]
    fn position(&self) -> Position {
        Position(self.0.position().clone())
    }

    fn encode(&self) -> String {
        self.0.encode()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }
}

#[pyclass]
struct ResourceName(liquers_core::query::ResourceName);

#[pymethods]
impl ResourceName {
    #[new]
    fn new(name: String, position: &Position) -> Self {
        ResourceName(liquers_core::query::ResourceName::new(name).with_position(position.0.clone()))
    }

    #[getter]
    fn position(&self) -> Position {
        Position(self.0.position.clone())
    }

    fn encode(&self) -> &str {
        self.0.encode()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.encode().to_string()
    }
}

#[pyclass]
struct ActionRequest(liquers_core::query::ActionRequest);

#[pymethods]
impl ActionRequest {
    #[new]
    fn new(name: &str) -> Self {
        ActionRequest(liquers_core::query::ActionRequest::new(name.to_owned()))
    }

    #[staticmethod]
    fn from_arguments(name: &str) -> Self {
        ActionRequest(liquers_core::query::ActionRequest::new(name.to_owned()))
    }

    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    fn encode(&self) -> String {
        self.0.encode()
    }

    fn to_list(&self) -> Vec<String> {
        let mut result = vec![self.0.name.to_string()];
        for parameter in &self.0.parameters {
            match parameter {
                liquers_core::query::ActionParameter::String(s, _) => result.push(s.to_string()),
                liquers_core::query::ActionParameter::Link(q, _) => result.push(q.encode()),
            }
        }
        result
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.encode()
    }
}

#[pyclass]
struct SegmentHeader(liquers_core::query::SegmentHeader);

#[pymethods]
impl SegmentHeader {
    #[new]
    fn new() -> Self {
        SegmentHeader(liquers_core::query::SegmentHeader::new())
    }
    #[getter]
    fn name(&self) -> String {
        self.0.name.to_string()
    }

    #[getter]
    fn position(&self) -> Position {
        Position(self.0.position.clone())
    }

    #[getter]
    fn level(&self) -> usize {
        self.0.level
    }
    /*
        #[getter]
        fn parameters(&self) -> Vec<ActionParameter> {
            self.0.parameters.iter().map(|p| HeaderParameter(p.clone())).collect()
        }
    */

    #[getter]
    fn resource(&self) -> bool {
        self.0.resource
    }

    fn is_trivial(&self) -> bool {
        self.0.is_trivial()
    }

    fn encode(&self) -> String {
        self.0.encode()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.encode()
    }
}

#[pyclass]
struct TransformQuerySegment(liquers_core::query::TransformQuerySegment);

#[pymethods]
impl TransformQuerySegment {
    #[new]
    fn new() -> Self {
        TransformQuerySegment(liquers_core::query::TransformQuerySegment::default())
    }

    #[getter]
    fn header(&self) -> Option<SegmentHeader> {
        match &self.0.header {
            Some(h) => Some(SegmentHeader(h.clone())),
            None => None,
        }
    }

    #[getter]
    fn query(&self) -> Vec<ActionRequest> {
        self.0
            .query
            .iter()
            .map(|q| ActionRequest(q.clone()))
            .collect()
    }

    #[getter]
    fn filename(&self) -> Option<String> {
        self.0.filename.as_ref().map(|s| s.to_string())
    }

    fn predecessor(&self) -> (Option<TransformQuerySegment>, Option<TransformQuerySegment>) {
        let (p, r) = self.0.predecessor();
        (
            p.map(|s| TransformQuerySegment(s)),
            r.map(|s| TransformQuerySegment(s)),
        )
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_filename(&self) -> bool {
        self.0.is_filename()
    }

    fn is_action_request(&self) -> bool {
        self.0.is_action_request()
    }

    fn action(&self) -> Option<ActionRequest> {
        self.0.action().map(|a| ActionRequest(a))
    }

    fn encode(&self) -> String {
        self.0.encode()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.encode()
    }
}

#[pyclass]
struct Key(liquers_core::query::Key);

#[pymethods]
impl Key {
    #[new]
    fn new(key: &str) -> Self {
        Key(liquers_core::parse::parse_key(key).unwrap())
    }

    fn encode(&self) -> String {
        self.0.encode()
    }

    fn to_absolute(&self, cwd_key:&Key) -> Key {
        Key(self.0.to_absolute(&cwd_key.0))
    }

    fn parent(&self) -> Key {
        Key(self.0.parent())
    }   

    fn __len__(&self) -> usize {
        self.0.len()
    }

    fn __getitem__(&self, index: isize) -> ResourceName {
        ResourceName(self.0 .0.get(index as usize).unwrap().clone())
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.encode()
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

#[pyclass]
struct ResourceQuerySegment(liquers_core::query::ResourceQuerySegment);

#[pymethods]
impl ResourceQuerySegment {
    #[new]
    fn new() -> Self {
        ResourceQuerySegment(liquers_core::query::ResourceQuerySegment::default())
    }

    #[getter]
    fn header(&self) -> Option<SegmentHeader> {
        match &self.0.header {
            Some(h) => Some(SegmentHeader(h.clone())),
            None => None,
        }
    }

    fn segment_name(&self) -> String {
        match self.0.header {
            Some(ref h) => h.name.to_string(),
            None => "".to_string(),
        }
    }

    fn path(&self) -> String {
        self.0.path()
    }

    #[getter]
    fn key(&self) -> Key {
        Key(self.0.key.clone())
    }

    fn encode(&self) -> String {
        self.0.encode()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.encode()
    }
}

#[pyclass]
struct QuerySegment(liquers_core::query::QuerySegment);

#[pymethods]
impl QuerySegment {
    #[new]
    fn new() -> Self {
        QuerySegment(liquers_core::query::QuerySegment::default())
    }

    #[getter]
    fn filename(&self) -> Option<String> {
        self.0.filename().map(|s| s.to_string())
    }

    #[getter]
    fn header(&self) -> Option<SegmentHeader> {
        match &self.0 {
            liquers_core::query::QuerySegment::Transform(t) => {
                t.header.as_ref().map(|h| SegmentHeader(h.clone()))
            }
            liquers_core::query::QuerySegment::Resource(r) => {
                r.header.as_ref().map(|h| SegmentHeader(h.clone()))
            }
        }
    }

    fn is_resource_query_segment(&self) -> bool {
        self.0.is_resource_query_segment()
    }

    fn is_transform_query_segment(&self) -> bool {
        self.0.is_transform_query_segment()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_filename(&self) -> bool {
        self.0.is_filename()
    }

    fn is_action_request(&self) -> bool {
        self.0.is_action_request()
    }

    fn encode(&self) -> String {
        self.0.encode()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.encode()
    }
}

#[pyclass]
struct Query(liquers_core::query::Query);

#[pymethods]
impl Query {
    #[new]
    fn new() -> Self {
        Query(liquers_core::query::Query::default())
    }

    #[getter]
    fn absolute(&self) -> bool {
        self.0.absolute
    }

    #[getter]
    fn segments(&self) -> Vec<QuerySegment> {
        self.0
            .segments
            .iter()
            .map(|s| QuerySegment(s.clone()))
            .collect()
    }

    fn filename(&self) -> Option<String> {
        self.0.filename().map(|s| s.to_string())
    }

    fn without_filename(&self) -> Query {
        Query(self.0.clone().without_filename())
    }

    fn extension(&self) -> Option<String> {
        self.0.extension()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn is_transform_query(&self) -> bool {
        self.0.is_transform_query()
    }

    fn transform_query(&self) -> Option<TransformQuerySegment> {
        self.0.transform_query().map(|s| TransformQuerySegment(s))
    }

    fn is_resource_query(&self) -> bool {
        self.0.is_resource_query()
    }

    fn resource_query(&self) -> Option<ResourceQuerySegment> {
        self.0.resource_query().map(|s| ResourceQuerySegment(s))
    }

    fn is_action_request(&self) -> bool {
        self.0.is_action_request()
    }

    fn action(&self) -> Option<ActionRequest> {
        if self.0.is_action_request() {
            self.0.action().map(|a| ActionRequest(a))
        } else {
            None
        }
    }

    fn predecessor(&self) -> (Option<Query>, Option<QuerySegment>) {
        let (p, r) = self.0.predecessor();
        (p.map(|s| Query(s)), r.map(|s| QuerySegment(s)))
    }

    fn all_predecessors(&self) -> Vec<(Option<Query>, Option<QuerySegment>)> {
        self.0
            .all_predecessors()
            .into_iter()
            .map(|(p, r)| (p.map(|s| Query(s)), r.map(|s| QuerySegment(s))))
            .collect()
    }

    //#[args(n = 30)]
    fn short(&self, n: usize) -> String {
        self.0.short(n)
    }

    fn encode(&self) -> String {
        self.0.encode()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }

    fn __str__(&self) -> String {
        self.0.encode()
    }
}

#[pyfunction]
fn parse(query: &str) -> PyResult<Query> {
    match liquers_core::parse::parse_query(query) {
        Ok(q) => Ok(Query(q)),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(
            e.to_string(),
        )),
    }
}

#[pyfunction]
fn parse_key(key: &str) -> PyResult<Key> {
    match liquers_core::parse::parse_key(key) {
        Ok(k) => Ok(Key(k)),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyException, _>(
            e.to_string(),
        )),
    }
}

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
    m.add_function(wrap_pyfunction!(parse, m)?)?;
    m.add_function(wrap_pyfunction!(parse_key, m)?)?;
    Ok(())
}

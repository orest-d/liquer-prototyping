use std::sync::Arc;

use crate::{error::Error, metadata::Metadata, value::ValueInterface};

#[derive(Debug)]
pub struct State<V: ValueInterface> {
    pub data: Option<Arc<V>>,
    pub metadata: Arc<Metadata>,
}

impl<V: ValueInterface> State<V> {
    pub fn new() -> State<V> {
        State {
            data: None,
            metadata: Arc::new(Metadata::new()),
        }
    }
    pub fn from_query(query: &str) -> Result<Self, Error> {
        Ok(Self::new().with_metadata(Metadata::from_query(query)?))
    }
    pub fn with_metadata(&self, metadata: Metadata) -> Self {
        State {
            data: self.data.clone(),
            metadata: Arc::new(metadata),
        }
    }
    pub fn with_data(&self, value: V) -> Self {
        State {
            data: Some(Arc::new(value)),
            metadata: Arc::new((*self.metadata).clone()),
        }
    }
    pub fn cache_key(&self) -> String {
        self.metadata.cache_key()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_none()
    }
}

impl<V: ValueInterface> Default for State<V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<V: ValueInterface> Clone for State<V> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            metadata: self.metadata.clone(),
        }
    }
}
/*
impl<V: ValueInterface> ToOwned for State<V> {
    type Owned = State<V>;

    fn to_owned(&self) -> Self::Owned {
        State{data:self.data.clone(), metadata:self.metadata.clone()}
    }
}
*/

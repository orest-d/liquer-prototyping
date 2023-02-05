use crate::error::Error;
use crate::state::State;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::metadata::Metadata;
use crate::value::ValueInterface;

/// Definition of Cache interface
/// Cache is meant to temporarily store results of queries as values.
/// Primary use of Cache is accelerating the evaluation of queries and making short-lived results available via web API.
pub trait Cache<V>
where
    V: ValueInterface,
{
    /// Clean the cache
    /// Empties all the data in the cache
    fn clear(&mut self);
    /// Get a state associated with the key
    fn get(&self, key: &str) -> Option<State<V>>;
    /// Get metadata associated with the key
    fn get_metadata(&self, key: &str) -> Option<Arc<Metadata>>;
    /// Set a state associated with the key
    fn set(&mut self, state: State<V>) -> Result<(), Error>;
    /// Set metadata associated with the key
    fn set_metadata(&mut self, metadata: &Metadata) -> Result<(), Error>;
    /// Remove a state associated with the key
    fn remove(&mut self, key: &str) -> Result<(), Error>;
    /// Check whether cache contains the key
    fn contains(&self, key: &str) -> bool;
    /// List of cached keys
    fn keys(&self) -> Vec<String>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NoCache<V>(PhantomData<V>);

impl<V> Cache<V> for NoCache<V>
where
    V: ValueInterface,
{
    fn clear(&mut self) {}

    fn get(&self, key: &str) -> Option<State<V>> {
        None
    }

    fn get_metadata(&self, key: &str) -> Option<Arc<Metadata>> {
        None
    }

    fn set(&mut self, state: State<V>) -> Result<(), Error> {
        Err(Error::CacheNotSupported)
    }

    fn set_metadata(&mut self, metadata: &Metadata) -> Result<(), Error> {
        Err(Error::CacheNotSupported)
    }

    fn remove(&mut self, key: &str) -> Result<(), Error> {
        Err(Error::CacheNotSupported)
    }

    fn contains(&self, key: &str) -> bool {
        false
    }

    fn keys(&self) -> Vec<String> {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct MemoryCache<V: ValueInterface>(HashMap<String, State<V>>);

impl<V: ValueInterface> MemoryCache<V> {
    pub fn new() -> Self {
        MemoryCache(HashMap::new())
    }
}

impl<V> Cache<V> for MemoryCache<V>
where
    V: ValueInterface,
{
    fn clear(&mut self) {
        self.0.clear();
    }

    fn get(&self, key: &str) -> Option<State<V>> {
        self.0.get(key).map(|x| x.to_owned())
    }

    fn get_metadata(&self, key: &str) -> Option<Arc<Metadata>> {
        self.0.get(key).map(|x| x.metadata.clone())
    }

    fn set(&mut self, state: State<V>) -> Result<(), Error> {
        self.0.insert(state.cache_key(), state);
        Ok(())
    }

    fn set_metadata(&mut self, metadata: &Metadata) -> Result<(), Error> {
        let key = metadata.cache_key();
        if let Some(state) = self.get(&key) {
            self.0.insert(key, state.with_metadata(metadata.clone()));
        } else {
            self.0
                .insert(key, State::new().with_metadata(metadata.clone()));
        }
        Ok(())
    }

    fn remove(&mut self, key: &str) -> Result<(), Error> {
        self.0.remove(key);
        Ok(())
    }

    fn contains(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    fn keys(&self) -> Vec<String> {
        self.0.keys().map(|x| x.to_owned()).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Mutex, thread, time::Duration};

    use super::*;
    use crate::value::*;

    #[test]
    fn test_no_cache() {
        let cache: NoCache<Value> = NoCache(PhantomData);
        assert!(cache.get("key").is_none());
        assert_eq!(cache.contains("key"), false);
    }
    #[test]
    fn test_memory_cache() -> Result<(), Error> {
        let mut cache = MemoryCache::<Value>::new();
        assert!(cache.get("key").is_none());
        assert_eq!(cache.contains("key"), false);
        cache.set(State::from_query("key")?)?;
        assert_eq!(cache.contains("key"), true);
        assert_eq!(cache.get("key").unwrap().cache_key(), "key");
        Ok(())
    }
    #[test]
    fn test_memory_cache_threaded() -> Result<(), Error> {
        let mut cache = MemoryCache::<Value>::new();
        assert!(cache.get("key").is_none());
        let cache = Arc::new(Mutex::new(cache));
        let c1 = cache.clone();
        let t1 = thread::spawn(move || {
            if let Ok(mut cache) = c1.lock() {
                let state = State::from_query("key").unwrap();
                cache.set(state).unwrap();
                assert!(cache.get("key").unwrap().is_empty());
                println!("T1 CACHED {:?}", cache.get("key"));
            }
        });
        let c2 = cache.clone();
        let t2 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            if let Ok(mut cache) = c2.lock() {
                let state = State::from_query("key").unwrap().with_data(Value::I32(123));
                cache.set(state).unwrap();
                println!("T2 CACHED {:?}", cache.get("key"));
            }
        });
        t1.join().unwrap();
        if let Ok(cache) = cache.lock() {
            assert!(cache.contains("key"));
            println!("Jointed t1 CACHED {:?}", cache.get("key"));
            assert!(cache.get("key").unwrap().is_empty());
        } else {
            assert!(false);
        }
        t2.join().unwrap();
        if let Ok(cache) = cache.lock() {
            assert!(cache.contains("key"));
            println!("Jointed t2 CACHED {:?}", cache.get("key"));
            assert!(!cache.get("key").unwrap().is_empty());
            assert_eq!(*cache.get("key").unwrap().data, Value::I32(123));
        } else {
            assert!(false);
        }

        Ok(())
    }
}

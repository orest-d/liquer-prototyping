use liquers_core::{error::Error, metadata::{Metadata, MetadataRecord}, query::Key, store::AsyncStore};
use async_trait::async_trait;

pub struct SimpleUrlStore{
    url_prefix: String,
}

impl SimpleUrlStore {
    pub fn new(url_prefix: String) -> Self {     
        SimpleUrlStore { url_prefix }
    }
    fn key_to_url(&self, key: &Key) -> String {
        format!("{}/{}", self.url_prefix, key)
    }
}

#[async_trait(?Send)]
impl AsyncStore for SimpleUrlStore {
    async fn async_get(&self, key: &Key) -> Result<(Vec<u8>, Metadata), Error> {
        let url = self.key_to_url(key);
        let resp = reqwest::get(url).await.map_err(|e| Error::general_error(format!("Fetch error")).with_key(key))?;
        //let bytes = resp.bytes().await.map_err(|e|Error::general_error(format!("Error getting data ")).with_key(key))?;
        let mut metadata = MetadataRecord::new();
        metadata.with_key(key.clone());

        Ok((vec![], Metadata::MetadataRecord(metadata)))
    }
}

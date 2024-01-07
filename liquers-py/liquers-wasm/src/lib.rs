use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[derive(Serialize, Deserialize)]
pub struct Query (pub liquers_core::query::Query);

#[wasm_bindgen]
impl Query {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self (liquers_core::query::Query::new())
    }

    pub fn encode(&self) -> String {
        self.0.encode()
    }   
}

#[wasm_bindgen]
pub fn parse_query(query: &str) -> Result<JsValue, JsValue> {
    match liquers_core::parse::parse_query(query) {
        Ok(q) => {
            serde_wasm_bindgen::to_value(&q).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

#[wasm_bindgen]
pub fn parse_key(key: &str) -> Result<JsValue, JsValue> {
    match liquers_core::parse::parse_key(key) {
        Ok(k) => {
            serde_wasm_bindgen::to_value(&k).map_err(|e| JsValue::from_str(&e.to_string()))
        }
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

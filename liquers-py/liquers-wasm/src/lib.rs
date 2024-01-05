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

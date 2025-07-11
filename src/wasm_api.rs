// src/wasm_api.rs
use wasm_bindgen::prelude::*;
use crate::models::VectorEngine;

#[wasm_bindgen]
pub struct WasmVectorEngine {
    engine: VectorEngine,
}

#[wasm_bindgen]
impl WasmVectorEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(dimension: usize) -> Self {
        Self { engine: VectorEngine::new(dimension) }
    }

    pub fn load_from_bytes(bytes: &[u8], dimension: usize) -> Result<WasmVectorEngine, JsValue> {
        let engine = VectorEngine::load_from_bytes(bytes, dimension)
            .map_err(|e| serde_wasm_bindgen::to_value(&e).unwrap())?;
        Ok(Self { engine })
    }

    pub fn save_to_bytes(&self) -> Result<Vec<u8>, JsValue> {
        self.engine.save_to_bytes().map_err(|e| serde_wasm_bindgen::to_value(&e).unwrap())
    }

    pub fn add_document(&mut self, id: u64, vector: &[f32]) -> Result<(), JsValue> {
        self.engine.add_document(id, vector.to_vec()).map_err(|e| serde_wasm_bindgen::to_value(&e).unwrap())
    }
    
    pub fn update_document(&mut self, id: u64, vector: &[f32]) -> Result<(), JsValue> {
        self.engine.update_document(&id, vector.to_vec()).map_err(|e| serde_wasm_bindgen::to_value(&e).unwrap())
    }

    pub fn delete_document(&mut self, id: u64) -> Result<(), JsValue> {
        self.engine.delete_document(&id).map_err(|e| serde_wasm_bindgen::to_value(&e).unwrap())
    }

    pub fn search(&mut self, query_vector: &[f32], top_k: usize) -> Result<JsValue, JsValue> {
        let results = self.engine.search(&query_vector.to_vec(), top_k)
            .map_err(|e| serde_wasm_bindgen::to_value(&e).unwrap())?;
        Ok(serde_wasm_bindgen::to_value(&results).unwrap())
    }

    pub fn document_count(&self) -> usize { self.engine.document_count() }
    pub fn dimension(&self) -> usize { self.engine.dimension() }
}
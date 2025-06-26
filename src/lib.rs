use wasm_bindgen::prelude::*;

// TypeScript에서 호출할 간단한 함수를 정의합니다.
#[wasm_bindgen]
pub fn greet() -> String {
    "Hello, Jin!".to_string()
}
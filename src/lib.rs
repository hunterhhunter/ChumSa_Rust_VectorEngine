use wasm_bindgen::prelude::*;

// 모듈을 공개적으로 선언
pub mod models;
pub mod utils;
pub mod wasm_api;

// TypeScript에서 호출할 간단한 함수를 정의합니다.
#[wasm_bindgen]
pub fn greet() -> String {
    "Hello, Jin!".to_string()
}


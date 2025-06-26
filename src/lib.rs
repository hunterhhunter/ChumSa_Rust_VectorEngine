use wasm_bindgen::prelude::*;

// data_structures 모듈을 공개적으로 선언
pub mod structures;

// 편의를 위해 Document 구조체를 최상위 경로로 다시 내보냄
pub use structures::Document;


// TypeScript에서 호출할 간단한 함수를 정의합니다.
#[wasm_bindgen]
pub fn greet() -> String {
    "Hello, Jin!".to_string()
}


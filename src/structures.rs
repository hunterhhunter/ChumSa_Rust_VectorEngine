// tonic_build가 OUT_DIR 환경 변수에 지정된 경로에 생성한 Rust 코드를
// 컴파일 시점에 그대로 이곳에 포함시킵니다.
// `engine.rs`는 .proto 파일의 package 이름에서 따온 것입니다.
include!(concat!(env!("OUT_DIR"), "/engine.rs"));

// 2. 위에서 생성한 Document 구조체에 impl 선언
impl Document {
    pub fn new(id: u64, vector: Vec<f32>) -> Self {
        Self {
            id,
            vector,
        }
    }
}
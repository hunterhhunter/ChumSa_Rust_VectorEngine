include!(concat!(env!("OUT_DIR"), "/engine.rs"));

// prost에서 생성한 Document 구조체에 impl 선언
impl Document {
    pub fn new(id: u64, vector: Vec<f32>) -> Self {
        Self {
            id,
            vector,
        }
    }
}
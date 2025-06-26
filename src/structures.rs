use serde::{Serialize, Deserialize};
use serde_json::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Document {
    // TypeScript와 연결하기 위한 고유 식별자
    pub id: usize,

    // 유사도 계산을 위한 벡터
    pub vector: Vec<f32>,
}

impl Document {
    pub fn new(id: usize, vector:Vec<f32>) -> Document {
        Document { id, vector }
    }
}

#[cfg(test)]
mod tests {

}
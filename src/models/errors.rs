use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum VectorEngineError {
    DimensionMismatch(String),
    SerializationError(String),
    DeserializationError(String),
    ItemNotFound(String),
}

// prost의 EncodeError를 받으면 우리 SerializationError로 변환하는 방법
impl From<prost::EncodeError> for VectorEngineError {
    fn from(err: prost::EncodeError) -> Self {
        VectorEngineError::SerializationError(err.to_string())
    }
}

// prost의 DecodeError를 받으면 우리 DeserializationError로 변환하는 방법
impl From<prost::DecodeError> for VectorEngineError {
    fn from(err: prost::DecodeError) -> Self {
        VectorEngineError::DeserializationError(err.to_string())
    }
}
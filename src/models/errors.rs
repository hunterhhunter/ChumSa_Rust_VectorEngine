#[derive(Debug, PartialEq)]
pub enum VectorEngineError {
    DimensionMismatch(String),
    SerializationError(String),
    DeserializationError(String),
    ItemNotFound(String),
}
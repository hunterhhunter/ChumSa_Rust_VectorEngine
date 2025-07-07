pub mod document;
pub mod engine;
pub mod search_cache;
pub mod errors;

pub use document::Document;
pub use engine::VectorEngine;
pub use search_cache::{SearchCache, CacheStats};
pub use errors::VectorEngineError;
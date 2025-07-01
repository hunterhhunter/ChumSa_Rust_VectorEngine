use std::{collections::HashMap, vec};
use crate::components::{
    SearchCache, VectorIndex, create_search_cache, create_vector_index
};

pub struct VectorEngine<'a> {
    index: VectorIndex<'a>,
    query_cache: SearchCache,
    dimension: usize,
    documents: HashMap<u64, Vec<f32>>,
}

impl<'a> VectorEngine<'a> {
    /// 지정된  차원의 비어 있는 새로운 VectorEngine을 생성
    pub fn new(dimension: usize) -> VectorEngine<'static> {
        VectorEngine {
            dimension,
            index: create_vector_index(),
            query_cache: create_search_cache(100),
            documents: HashMap::new()
        }
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn add_document(&mut self, id: u64, vector: Vec<f32>) -> Result<(), String> {
        // 차원 일치 검사
        if self.dimension != vector.len() {
            return Err(format!(
                "입력된 벡터의 차원({})이 엔진의 차원({})과 일치하지 않습니다.",
                vector.len(),
                self.dimension,
            ));
        }
        // 하나는 복제본(clone)을 사용합니다.
        // hnsw_rs는 ID로 usize 타입을 사용하므로 변환해줍니다.
        self.index.insert((&vector.clone(), id as usize));

        // documents 해시에 추가
        self.documents.insert(id, vector);

        Ok(())
    }
}
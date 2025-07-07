use std::{collections::HashMap, vec};
use prost::Message;

use crate::{components::{create_vector_index, VectorIndex}, 
            models::{document::{EngineState, Document}, 
            CacheStats, SearchCache}, 
            utils::hash_vector,
            models::errors::VectorEngineError
};

pub struct VectorEngine<'a> {
    index: VectorIndex<'a>,
    query_cache: SearchCache<'a, u64, Vec<(u64, f32)>>,
    dimension: usize,
    documents: HashMap<u64, Vec<f32>>,
}

impl<'a> VectorEngine<'a> {
    /// 지정된  차원의 비어 있는 새로운 VectorEngine을 생성
    pub fn new(dimension: usize) -> VectorEngine<'static> {
        VectorEngine {
            dimension,
            index: create_vector_index(),
            query_cache: SearchCache::new(100),
            documents: HashMap::new()
        }
    }

    /// 테스트 목적으로 캐시에 저장된 항목의 수를 반환합니다.
    pub fn query_cache_len(&self) -> usize {
        self.query_cache.len()
    }

    /// 테스트 목적으로 캐시의 히트/미스 통계를 반환합니다.
    pub fn query_cache_stats(&self) -> &CacheStats {
        self.query_cache.stats()
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }
    
    /// 엔진에 저장된 documents 해시맵의 불변 참조를 반환합니다.
    pub fn documents(&self) -> &HashMap<u64, Vec<f32>> {
        &self.documents
    }

    pub fn add_document(&mut self, id: u64, vector: Vec<f32>) -> Result<(), VectorEngineError> {
        // 차원 일치 검사
        if self.dimension != vector.len() {
            let error_message = format!(
                "입력된의 차원({})이 엔진의 차원({})과 일치하지 않습니다.",
                vector.len(),
                self.dimension
            );
            // 생성된 String을 에러에 담아 반환
            return Err(VectorEngineError::DimensionMismatch(error_message));
        }
        // 하나는 복제본(clone)을 사용합니다.
        // hnsw_rs는 ID로 usize 타입을 사용하므로 변환해줍니다.
        self.index.insert((&vector.clone(), id as usize));

        // documents 해시에 추가
        self.documents.insert(id, vector);

        self.query_cache.clear();

        Ok(())
    }

    pub fn search(&mut self, query_vector: &Vec<f32>, top_k: usize, ef_search: usize) -> Result<Vec<(u64, f32)>, VectorEngineError> {
        // 1. 차원 검사
        if self.dimension != query_vector.len() {
            let error_message = format!(
                "쿼리 벡터의 차원({})이 엔진의 차원({})과 일치하지 않습니다.",
                query_vector.len(),
                self.dimension
            );
            // 생성된 String을 에러에 담아 반환
            return Err(VectorEngineError::DimensionMismatch(error_message));
        }

        // 2. 캐시 키 생성
        let hash_id: u64 = hash_vector(query_vector);

        // 3. 캐시 검색 (Hit)
        if let Some(cached_results) = self.query_cache.get(&hash_id) {
            return Ok(cached_results.clone());
        }
        // Cache Miss 로직
        // 4. HNSW에서 검색 수행
        let search_result_neighbors = self.index.search(query_vector, top_k+1, ef_search);

        // 5. 검색 결과를 (u64, f32) 튜플 형태로 변환
        let mut results: Vec<(u64, f32)> = search_result_neighbors
            .into_iter()
            .map(|neighbor| (neighbor.d_id as u64, neighbor.distance))
            .collect();

        // 5-1. 유사도를 기준으로 높은 순으로 정렬
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // 5-2. 결과 개수를 top_k개로 잘라냄
        results.truncate(top_k);

        // 6. 캐시에 새로운 검색 결과 저장
        self.query_cache.put(hash_id, results.clone());

        // 7. 최종 결과 반환
        Ok(results)
    }

    pub fn delete_document(&mut self, id: &u64) -> Result<(), VectorEngineError> {
        // 1. documents 해시맵에 입력 id의 문서가 없으면 에러 반환
        if self.documents.get(id) == None {
            let error_msg = format!(
                "입력한 id {}에 맞는 문서가 존재하지 않습니다.",
                id
            );
            return Err(VectorEngineError::ItemNotFound(error_msg));
        }

        // 2. document 삭제
        self.documents.remove(id).unwrap();

        // 3. HNSW 인덱스 재구성
        self.rebuild_index()?;
        
        // 4. 캐시 일관성을 위한 쿼리 캐시 초기화
        self.query_cache.clear();
        
        Ok(())
    }

    pub fn update_document(&mut self, id: &u64, new_vector: Vec<f32>) -> Result<(), VectorEngineError> {
        // 1. 차원 검사
        if self.dimension != new_vector.len() {
            let error_message = format!(
                "쿼리 벡터의 차원({})이 엔진의 차원({})과 일치하지 않습니다.",
                new_vector.len(),
                self.dimension
            );
            // 생성된 String을 에러에 담아 반환
            return Err(VectorEngineError::DimensionMismatch(error_message));
        }

        // 2. 해당 ID의 문서가 존재하는지 확인
        if let Some(vector_in_map) = self.documents.get_mut(&id) {
            *vector_in_map = new_vector;
        } else {
            let error_msg = format!(
                "입력한 id {}에 맞는 문서가 존재하지 않아 업데이트 할 수 없습니다.",
                id
            );
            return Err(VectorEngineError::ItemNotFound(error_msg));
        }

        // 3. 변경된 내용을 HNSW 인덱스에 반영하기 위해 전체를 재구성
        self.rebuild_index()?;

        // 4. 쿼리 캐시 제거
        self.query_cache.clear();

        Ok(())
    }

    /// 현재 documents의 내용을 바탕으로 HNSW 인덱스를 재생성
    fn rebuild_index(&mut self) -> Result<(), VectorEngineError> {
        // 1. 동일한 빈 인덱스 생성
        let mut new_index= create_vector_index();

        // 2. documents 순회
        for (&id, vector)in self.documents.iter() {
            // 3. 각 문서를 삽입
            new_index.insert((&vector.clone(), id as usize));
        }

        // 4. 기존 인덱스를 교체
        self.index = new_index;

        Ok(())
    }

    pub fn save_to_bytes(&self) -> Result<Vec<u8>, VectorEngineError> {
        let documents_to_save: Vec<Document> = self.documents
            .iter()
            .map(|(&id, vector)| {
                Document {
                    id,
                    vector: vector.clone(),
                }
            })
            .collect();
    
        let current_engine_state = EngineState {
            format_version: 1,
            documents: documents_to_save,
        };
    
        let mut buf: Vec<u8> = Vec::new();
        // prost::EncodeError를 VectorEngineError::SerializationError로 변환
        current_engine_state.encode(&mut buf)
            .map_err(|e| VectorEngineError::SerializationError(e.to_string()))?;
        
        Ok(buf)
    }

    pub fn load_from_bytes(bytes: &[u8], dimension: usize) -> Result<VectorEngine<'static>, VectorEngineError> {
        // prost::DecodeError를 VectorEngineError::DeserializationError로 변환
        let state = EngineState::decode(bytes)
            .map_err(|e| VectorEngineError::DeserializationError(e.to_string()))?;
    
        let mut engine = VectorEngine::new(dimension);
    
        for doc in state.documents {
            // add_document가 반환하는 VectorEngineError가 '?'를 통해 자동으로 전파됨
            engine.add_document(doc.id, doc.vector)?;
        }
        
        Ok(engine)
    }
}
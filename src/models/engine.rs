use crate::{
    models::errors::VectorEngineError,
    models::point::MyPoint,
    models::{
        CacheStats, SearchCache,
        document::{Document, EngineState},
    },
    utils::hash_vector,
};
use instant_distance::{Builder, HnswMap, Search};
use prost::Message;
use rand::seq::index;
use serde::de::value;
use std::collections::HashMap;

pub struct VectorEngine {
    index: HnswMap<MyPoint, u64>,
    query_cache: SearchCache<'static, u64, Vec<(u64, f32)>>,
    dimension: usize,
    documents: HashMap<u64, Vec<f32>>,
}

impl VectorEngine {
    /// 지정된  차원의 비어 있는 새로운 VectorEngine을 생성
    pub fn new(dimension: usize) -> Self {
        let points: Vec<MyPoint> = Vec::new();
        let values: Vec<u64> = Vec::new();
        let index = Builder::default().build(points, values);
        VectorEngine {
            dimension,
            index: index,
            query_cache: SearchCache::new(100),
            documents: HashMap::new(),
        }
    }

    /// 현재 documents의 내용을 바탕으로 HNSW 인덱스를 재생성
    fn rebuild_index(&mut self) -> Result<(), VectorEngineError> {
        // 1. 동일한 빈 인덱스 생성
        let points = self
            .documents
            .values()
            .map(|v| MyPoint(v.clone()))
            .collect();
        let values: Vec<u64> = self.documents.keys().copied().collect();
        // 4. 기존 인덱스를 교체
        self.index = Builder::default().build(points, values);

        Ok(())
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
        // documents 해시에 추가
        self.documents.insert(id, vector);

        self.rebuild_index()?; // 인덱스 리빌딩

        self.query_cache.clear();

        Ok(())
    }

    pub fn update_document(
        &mut self,
        id: &u64,
        new_vector: Vec<f32>,
    ) -> Result<(), VectorEngineError> {
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

    pub fn delete_document(&mut self, id: &u64) -> Result<(), VectorEngineError> {
        if self.documents.remove(id).is_none() {
            return Err(VectorEngineError::ItemNotFound(format!("ID {} not found", id)));
        }

        // 3. HNSW 인덱스 재구성
        self.rebuild_index()?;

        // 4. 캐시 일관성을 위한 쿼리 캐시 초기화
        self.query_cache.clear();

        Ok(())
    }

    pub fn save_to_bytes(&self) -> Result<Vec<u8>, VectorEngineError> {
        let documents_to_save: Vec<Document> = self
            .documents
            .iter()
            .map(|(&id, vector)| Document {
                id,
                vector: vector.clone(),
            })
            .collect();

        let current_engine_state = EngineState {
            format_version: 1,
            documents: documents_to_save,
        };

        let mut buf: Vec<u8> = Vec::new();
        // prost::EncodeError를 VectorEngineError::SerializationError로 변환
        current_engine_state
            .encode(&mut buf)
            .map_err(|e| VectorEngineError::SerializationError(e.to_string()))?;

        Ok(buf)
    }

    pub fn search(
        &mut self,
        query_vector: &Vec<f32>,
        top_k: usize,
    ) -> Result<Vec<(u64, f32)>, VectorEngineError> {
        // 1. 차원 검사
        if self.dimension != query_vector.len() {
            let error_message = format!(
                "쿼리 벡터의 차원({})이 엔진의 차원({})과 일치하지 않습니다.",
                query_vector.len(),
                self.dimension
            );
            return Err(VectorEngineError::DimensionMismatch(error_message));
        }

        // 2. 캐시 키 생성
        let hash_id: u64 = hash_vector(query_vector);

        // 3. 캐시 검색 (Hit)
        if let Some(cached_results) = self.query_cache.get(&hash_id) {
            return Ok(cached_results.clone());
        }
        // Cache Miss 로직

        let query_point = MyPoint(query_vector.clone());
        let mut search = Search::default();

        // 5. 검색 결과를 (u64, f32) 튜플 형태로 변환
        let mut results: Vec<(u64, f32)> = self.index.search(&query_point, &mut search)
            .take(top_k)
            .map(|results| (*results.value as u64, results.distance))
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

    pub fn load_from_bytes(
        bytes: &[u8],
        dimension: usize,
    ) -> Result<Self, VectorEngineError> {
        let state = EngineState::decode(bytes)?;
        let documents: HashMap<u64, Vec<f32>> = state.documents.into_iter()
            .map(|doc| (doc.id, doc.vector))
            .collect();

        let mut engine = Self::new(dimension);
        engine.documents = documents;
        engine.rebuild_index()?; // 모든 문서를 채운 뒤, 마지막에 한 번만 재구성
        Ok(engine)
    }
}

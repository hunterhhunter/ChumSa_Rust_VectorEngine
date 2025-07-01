use std::{collections::HashMap};
use crate::{components::{
    create_vector_index, VectorIndex}, 
    utils::hash_vector, 
    models::{SearchCache, CacheStats}
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

        // 캐시 초기화하여 캐시 데이터 일관성 유지
        self.query_cache.clear();

        Ok(())
    }

    pub fn search(&mut self, query_vector: &Vec<f32>, top_k: usize) -> Result<Vec<(u64, f32)>, String> {
        // 1. 차원 검사
        if self.dimension != query_vector.len() {
            return Err(format!(
                "쿼리 벡터의 차원({})이 엔진의 차원({})과 일치하지 않습니다.",
                query_vector.len(),
                self.dimension
            ));
        }

        // 2. 캐시 키 생성
        let hash_id: u64 = hash_vector(query_vector);

        // 3. 캐시 검색 (Hit)
        if let Some(cached_results) = self.query_cache.get(&hash_id) {
            return Ok(cached_results.clone());
        }

        // Cache Miss 로직
        // 4. HNSW에서 검색 수행
        let search_result_neighbors = self.index.search(query_vector, top_k, top_k*3);

        // 5. 검색 결과를 (u64, f32) 튜플 형태로 변환
        let mut results: Vec<(u64, f32)> = search_result_neighbors
            .into_iter()
            .map(|neighbor| (neighbor.d_id as u64, neighbor.distance))
            .collect();

        // 5-1. 유사도를 기준으로 높은 순으로 정렬
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // 6. 캐시에 새로운 검색 결과 저장
        self.query_cache.put(hash_id, results.clone());

        // 7. 최종 결과 반환
        Ok(results)
    }
}
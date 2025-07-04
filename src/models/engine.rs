use std::{collections::HashMap};
use prost::Message;

use crate::{components::{
    create_vector_index, VectorIndex}, models::{document::{EngineState, Document}, CacheStats, SearchCache}, utils::hash_vector
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
    pub fn get_query_cache_len(&self) -> usize {
        self.query_cache.len()
    }

    /// 테스트 목적으로 캐시의 히트/미스 통계를 반환합니다.
    pub fn get_query_cache_stats(&self) -> &CacheStats {
        self.query_cache.stats()
    }

    pub fn get_document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn get_dimension(&self) -> usize {
        self.dimension
    }
    
    /// 엔진에 저장된 documents 해시맵의 불변 참조를 반환합니다.
    pub fn  get_documents(&self) -> &HashMap<u64, Vec<f32>> {
        &self.documents
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
        let search_result_neighbors = self.index.search(query_vector, top_k+1, top_k*3);

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

    pub fn delete_document(&mut self, id: &u64) -> Result<(), String> {
        // 1. documents 해시맵에 입력 id의 문서가 없으면 에러 반환
        if self.documents.get(id) == None {
            return Err(format!(
                "저장된 문서들에 해당 id({})의 문서가 없습니다.",
                id
            ))
        }

        // 2. document 삭제
        self.documents.remove(id).unwrap();

        // 3. HNSW 인덱스 재구성
        self.rebuild_index().unwrap();
        
        // 4. 캐시 일관성을 위한 쿼리 캐시 초기화
        self.query_cache.clear();
        
        Ok(())
    }

    pub fn update_document(&mut self, id: &u64, new_vector: Vec<f32>) -> Result<(), String> {
        // 1. 차원 검사
        if self.dimension != new_vector.len() {
            return Err(format!(
                "입력된 벡터의 차원({})이 엔진의 차원({})과 일치하지 않습니다.",
                new_vector.len(),
                self.dimension
            ));
        }

        // 2. 해당 ID의 문서가 존재하는지 확인
        if let Some(vector_in_map) = self.documents.get_mut(&id) {
            *vector_in_map = new_vector;
        } else {
            return Err(format!("ID {}에 해당하는 문서를 찾을 수 없어 업데이트할 수 없습니다.", id));
        }

        // 3. 변경된 내용을 HNSW 인덱스에 반영하기 위해 전체를 재구성
        self.rebuild_index().unwrap();

        // 4. 쿼리 캐시 제거
        self.query_cache.clear();

        Ok(())
    }

    /// 현재 documents의 내용을 바탕으로 HNSW 인덱스를 재생성
    pub fn rebuild_index(&mut self) -> Result<(), String> {
        // 1. 동일한 빈 인덱스 생성
        let new_index= create_vector_index();

        // 2. documents 순회
        for (&id, vector)in self.documents.iter() {
            // 3. 각 문서를 삽입
            new_index.insert((vector.clone().as_slice(), id as usize));
        }

        // 4. 기존 인덱스를 교체
        self.index = new_index;

        Ok(())
    }

    pub fn save_to_bytes(&self) -> Result<Vec<u8>, String> {
        // 1. engine의 documents 해시맵을 벡터로 변환
        let documents_to_save = self.documents
            .iter()
            .map(|(&id, vector)| {
                Document {
                    id,
                    vector: vector.clone(),
                }
            })
            .collect();
        
        // 2. 변환된 데이터를 포함하는 EngineState 인스턴스 생성
        let current_engine_state: EngineState = EngineState {
            format_version: 1,
            documents: documents_to_save,
        };

        // 3. EngineState를 바이트 배열로 직렬화
        let mut buf: Vec<u8> = Vec::new();
        current_engine_state.encode(&mut buf).map_err(|e| e.to_string())?;
        
        // 4. 직렬화된 바이트 벡터 반환
        Ok(buf)
    }

    pub fn load_from_bytes(bytes: &[u8], dimension: usize) -> Result<VectorEngine<'static>, String> {
        // 1. byte 슬라이스를 EngineState 구조체로 디코딩
        let state = EngineState::decode(bytes).map_err(|e| e.to_string())?;

        // 2. 새로운 VectorEngine 인스턴스 생성
        let mut engine: VectorEngine<'static> = VectorEngine::new(dimension);

        // 3. 복원된 document목록을 순회하며 엔진에 추가
        for doc in state.documents {
            // add_document가 Result를 반환하므로 ?로 에러 처리
            engine.add_document(doc.id, doc.vector)?;
        }
        
        // 4. 복원 완료된 엔진 반환
        Ok(engine)
    }
}
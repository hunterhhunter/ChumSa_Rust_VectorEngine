use rust_vector_engine::models::document::{Document, EngineState};
use rust_vector_engine::models::VectorEngine;
use rust_vector_engine::models::errors::VectorEngineError;

// 테스트에 사용할 기본 파라미터
const TEST_DIM: usize = 3;
const EF_SEARCH: usize = 10;

#[test]
fn test_vector_engine_creation() {
    let engine = VectorEngine::new(TEST_DIM);
    assert_eq!(engine.dimension(), TEST_DIM);
    assert_eq!(engine.document_count(), 0);
}

#[test]
fn test_vector_engine_add_document_success() {
    let mut engine = VectorEngine::new(TEST_DIM);
    let res = engine.add_document(1, vec![0.1, 0.2, 0.3]);
    assert!(res.is_ok());
    assert_eq!(engine.document_count(), 1);
}

#[test]
fn test_vector_engine_add_document_dimension_mismatch() {
    let mut engine = VectorEngine::new(TEST_DIM);
    // 차원이 2인 벡터를 추가 시도
    let res = engine.add_document(1, vec![0.1, 0.2]);
    assert!(res.is_err());
    assert_eq!(engine.document_count(), 0);
}

#[test]
fn test_vector_engine_search_on_empty_engine() {
    let mut engine = VectorEngine::new(TEST_DIM);
    let query_vector = vec![0.1, 0.2, 0.3];
    let results = engine.search(&query_vector, 3, EF_SEARCH).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_vector_engine_search_with_k_larger_than_docs() {
    let mut engine = VectorEngine::new(TEST_DIM);
    engine.add_document(1, vec![1.0, 0.1, 0.2]).unwrap();
    engine.add_document(2, vec![0.1, 1.0, 0.3]).unwrap();
    
    let query_vector = vec![1.0, 0.0, 0.1];
    // 문서 수(2)보다 큰 k(3)를 요청
    let results = engine.search(&query_vector, 3, EF_SEARCH).unwrap();

    // 전체 문서 수인 2개가 반환되어야 함
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, 1);
}

#[test]
fn test_vector_engine_search_miss_and_hit() {
    let mut engine = VectorEngine::new(TEST_DIM);
    engine.add_document(1, vec![0.1, 0.2, 0.3]).unwrap();
    engine.add_document(2, vec![0.9, 0.8, 0.7]).unwrap();
    let query_vector = vec![0.8, 0.8, 0.8];
    
    assert_eq!(engine.query_cache_len(), 0);
    
    // Cache Miss
    let first_result = engine.search(&query_vector, 1, EF_SEARCH).unwrap();
    assert_eq!(engine.query_cache_stats().misses, 1);
    assert_eq!(first_result[0].0, 2);

    // Cache Hit
    let second_result = engine.search(&query_vector, 1, EF_SEARCH).unwrap();
    assert_eq!(engine.query_cache_stats().hits, 1);
    assert_eq!(first_result, second_result);
}

#[test]
fn test_vector_engine_search_after_cache_invalidation() {
    let mut engine = VectorEngine::new(TEST_DIM);
    engine.add_document(1, vec![0.1, 0.1, 0.1]).unwrap();
    let query_vector = vec![0.9, 0.9, 0.9];

    let first_results = engine.search(&query_vector, 1, EF_SEARCH).unwrap();
    assert_eq!(first_results[0].0, 1);

    // 새 문서 추가 시 캐시가 비워짐
    engine.add_document(2, vec![0.8, 0.8, 0.8]).unwrap();
    assert_eq!(engine.query_cache_len(), 0);
    
    let second_results = engine.search(&query_vector, 1, EF_SEARCH).unwrap();
    assert_eq!(second_results[0].0, 2);
}

#[test]
fn test_vector_engine_delete_and_rebuild() {
    let mut engine = VectorEngine::new(TEST_DIM);
    engine.add_document(1, vec![0.1, 0.1, 0.1]).unwrap();
    engine.add_document(2, vec![0.9, 0.9, 0.9]).unwrap();
    
    let query = vec![0.9, 0.9, 0.9];
    // 캐시 채우기
    let _ = engine.search(&query, 1, EF_SEARCH);
    assert_eq!(engine.query_cache_len(), 1);

    // ID 2 삭제
    engine.delete_document(&2).unwrap();

    assert_eq!(engine.document_count(), 1);
    assert!(engine.documents().get(&2).is_none());
    assert_eq!(engine.query_cache_len(), 0); // 캐시 비워짐 확인

    // 재검색 시, 이제 ID 1이 가장 가까워야 함
    let results_after_delete = engine.search(&query, 1, EF_SEARCH).unwrap();
    assert_eq!(results_after_delete[0].0, 1);
}

#[test]
fn test_vector_engine_update_document() {
    let mut engine = VectorEngine::new(TEST_DIM);
    let id_to_update = 2;
    let new_vector = vec![0.9, 0.9, 0.9];
    engine.add_document(1, vec![0.1, 0.1, 0.1]).unwrap();
    engine.add_document(id_to_update, vec![0.2, 0.2, 0.2]).unwrap();

    // 업데이트 실행
    let update_result = engine.update_document(&id_to_update, new_vector.clone());
    assert!(update_result.is_ok());

    // HashMap 값 확인
    assert_eq!(engine.documents().get(&id_to_update).unwrap(), &new_vector);

    // 인덱스 재구성 확인
    let search_result = engine.search(&vec![0.9, 0.9, 0.8], 1, EF_SEARCH).unwrap();
    assert_eq!(search_result[0].0, id_to_update);
}

#[test]
fn test_vector_engine_round_trip() {
    let mut original_engine = VectorEngine::new(TEST_DIM);
    original_engine.add_document(1, vec![1.0, 0.1, 0.2]).unwrap();
    original_engine.add_document(2, vec![0.1, 1.0, 0.3]).unwrap();

    let bytes = original_engine.save_to_bytes().unwrap();
    let reloaded_engine = VectorEngine::load_from_bytes(&bytes, TEST_DIM).unwrap();

    assert_eq!(original_engine.dimension(), reloaded_engine.dimension());
    assert_eq!(original_engine.document_count(), reloaded_engine.document_count());
    assert_eq!(original_engine.documents(), reloaded_engine.documents());
}

#[test]
fn test_vector_engine_returns_dimension_mismatch_error() {
    // 준비 (Arrange)
    let mut engine = VectorEngine::new(3);
    let wrong_dim_vector = vec![0.1, 0.2]; // 엔진 차원은 3, 벡터 차원은 2

    // 실행 (Act)
    let result = engine.add_document(1, wrong_dim_vector);

    // 검증 (Assert)
    
    // 1. 우선 Err가 반환되었는지 확인합니다.
    assert!(result.is_err());
    
    // 2. 반환된 에러가 우리가 예상한 'DimensionMismatch' 종류인지 확인합니다.
    // matches! 매크로는 enum의 variant가 일치하는지 확인할 때 매우 유용합니다.
    assert!(matches!(result.unwrap_err(), VectorEngineError::DimensionMismatch(_)));
}

#[test]
fn test_vector_engine_returns_item_not_found_on_delete() {
    // 준비
    let mut engine = VectorEngine::new(3);
    engine.add_document(1, vec![0.1, 0.2, 0.3]).unwrap();

    // 실행: 존재하지 않는 ID(99)를 삭제 시도
    let result = engine.delete_document(&99);

    // 검증
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VectorEngineError::ItemNotFound(_)));
}

#[test]
fn test_vector_engine_returns_item_not_found_on_update() {
    // 준비
    let mut engine = VectorEngine::new(3);
    engine.add_document(1, vec![0.1, 0.2, 0.3]).unwrap();
    let new_vector = vec![0.4, 0.5, 0.6];

    // 실행: 존재하지 않는 ID(99)를 업데이트 시도
    let result = engine.update_document(&99, new_vector);

    // 검증
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VectorEngineError::ItemNotFound(_)));
}
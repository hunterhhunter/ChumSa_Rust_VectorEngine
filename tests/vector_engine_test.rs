use rust_vector_engine::models::errors::VectorEngineError;
use rust_vector_engine::models::VectorEngine;

// 테스트에 사용할 기본 파라미터
const TEST_DIM: usize = 3;

#[test]
fn test_vector_engine_creation() {
    let engine = VectorEngine::new(TEST_DIM);
    assert_eq!(engine.dimension(), TEST_DIM);
    assert_eq!(engine.document_count(), 0);
}

#[test]fn test_add_document_success() {
    let mut engine = VectorEngine::new(TEST_DIM);
    let res = engine.add_document(1, vec![0.1, 0.2, 0.3]);
    assert!(res.is_ok());
    assert_eq!(engine.document_count(), 1);
}

#[test]
fn test_add_document_dimension_mismatch() {
    let mut engine = VectorEngine::new(TEST_DIM);
    let res = engine.add_document(1, vec![0.1, 0.2]);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), VectorEngineError::DimensionMismatch(_)));
    assert_eq!(engine.document_count(), 0);
}

#[test]
fn test_search_on_empty_engine() {
    let mut engine = VectorEngine::new(TEST_DIM);
    let query_vector = vec![0.1, 0.2, 0.3];
    let results = engine.search(&query_vector, 3).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_search_with_k_larger_than_docs() {
    let mut engine = VectorEngine::new(TEST_DIM);
    engine.add_document(1, vec![1.0, 0.1, 0.2]).unwrap();
    engine.add_document(2, vec![0.1, 1.0, 0.3]).unwrap();
    
    let query_vector = vec![1.0, 0.0, 0.1];
    // 문서 수(2)보다 큰 k(3)를 요청
    let results = engine.search(&query_vector, 3).unwrap();

    // 전체 문서 수인 2개가 반환되어야 함
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, 1);
}

#[test]
fn test_search_miss_and_hit() {
    let mut engine = VectorEngine::new(TEST_DIM);
    engine.add_document(1, vec![0.1, 0.2, 0.3]).unwrap();
    engine.add_document(2, vec![0.9, 0.8, 0.7]).unwrap();
    let query_vector = vec![0.8, 0.8, 0.8];
    
    assert_eq!(engine.query_cache_len(), 0);
    
    // Cache Miss
    let first_result = engine.search(&query_vector, 1).unwrap();
    assert_eq!(engine.query_cache_stats().misses, 1);
    assert_eq!(engine.query_cache_len(), 1);
    assert_eq!(first_result[0].0, 2);

    // Cache Hit
    let second_result = engine.search(&query_vector, 1).unwrap();
    assert_eq!(engine.query_cache_stats().hits, 1);
    assert_eq!(first_result, second_result);
}

#[test]
fn test_search_after_cache_invalidation() {
    let mut engine = VectorEngine::new(TEST_DIM);
    
    // 쿼리는 x축 방향을 가리킴
    let query_vector = vec![1.0, 0.0, 0.0];
    // 처음 추가되는 문서는 y축 방향을 가리킴 (쿼리와 거리가 멂)
    let initial_doc_id = 1;
    engine.add_document(initial_doc_id, vec![0.0, 1.0, 0.0]).unwrap();

    // 첫 번째 검색 -> ID 1이 캐시에 저장됨
    let first_results = engine.search(&query_vector, 1).unwrap();
    assert_eq!(first_results[0].0, initial_doc_id);

    // 쿼리와 훨씬 더 유사한 새 벡터(x축 방향) 추가
    let closer_doc_id = 2;
    engine.add_document(closer_doc_id, vec![0.9, 0.1, 0.1]).unwrap();
    
    // 다시 검색
    let second_results = engine.search(&query_vector, 1).unwrap();

    // 검증: 캐시의 낡은 데이터(ID 1)가 아닌, 새로 추가된 ID 2가 반환되어야 함
    assert_eq!(second_results[0].0, closer_doc_id);
}
#[test]
fn test_delete_and_rebuild() {
    let mut engine = VectorEngine::new(TEST_DIM);
    engine.add_document(1, vec![0.1, 0.1, 0.1]).unwrap();
    engine.add_document(2, vec![0.9, 0.9, 0.9]).unwrap();
    
    // ID 2 삭제
    engine.delete_document(&2).unwrap();

    assert_eq!(engine.document_count(), 1);
    assert!(engine.documents().get(&2).is_none());

    // 재검색 시, 이제 ID 1만 남았으므로 ID 1이 검색되어야 함
    let results_after_delete = engine.search(&vec![0.9, 0.9, 0.9], 1).unwrap();
    assert_eq!(results_after_delete[0].0, 1);
}

#[test]
fn test_update_document() {
    let mut engine = VectorEngine::new(TEST_DIM);
    let id_to_update = 2;

    // 각 벡터가 서로 다른 축에 가깝도록 설정
    let other_vector = vec![-1.0, 0.0, 0.0];
    let old_vector   = vec![0.0, 1.0, 0.0];  // 업데이트 전 벡터 (y축 방향)
    let new_vector   = vec![0.0, 0.0, 1.0];  // 업데이트 후 벡터 (z축 방향)

    engine.add_document(1, other_vector).unwrap();
    engine.add_document(id_to_update, old_vector).unwrap();

    // 업데이트 실행
    let update_result = engine.update_document(&id_to_update, new_vector.clone());
    assert!(update_result.is_ok());

    // HashMap 값 확인
    assert_eq!(engine.documents().get(&id_to_update).unwrap(), &new_vector);

    // 인덱스 재구성 확인: new_vector(z축)와 가장 가까운 쿼리 사용
    let query_for_new_vector = vec![0.1, 0.1, 0.9];
    let search_result = engine.search(&query_for_new_vector, 1).unwrap();
    
    // 결과는 반드시 업데이트된 ID 2여야 함
    assert_eq!(search_result[0].0, id_to_update);
}

#[test]
fn test_round_trip_serialization() {
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
fn test_returns_item_not_found_error() {
    let mut engine = VectorEngine::new(TEST_DIM);
    let result = engine.delete_document(&99);
    assert!(matches!(result.unwrap_err(), VectorEngineError::ItemNotFound(_)));
}
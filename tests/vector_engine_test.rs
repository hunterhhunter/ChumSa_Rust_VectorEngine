use rust_vector_engine::models::{VectorEngine};
use rust_vector_engine::utils::hash_vector;

/// vector엔진 생성자 테스트
#[test]
fn test_vector_engine_creation() {
    // 1. 준비
    let dimension = 1536;

    // 2. 실행
    let engine = VectorEngine::new(dimension);

    // 3. 테스트
    println!("{{engine.document_count()}}");
    assert_eq!(engine.dimension(), dimension);
    assert_eq!(engine.document_count(), 0);
}

// VectorEngine add_document 함수 테스트
#[test]
fn test_vector_engine_add_document_success() {
    // 벡터엔진 준비
    let dimension = 3;
    let mut engine = VectorEngine::new(dimension);

    // 벡터 생성
    let input_vector = vec![0.1, 0.2, 0.3];
    let id = hash_vector(&input_vector.clone());

    // 벡터 입력
    let res = engine.add_document(id, input_vector);
    assert_eq!(engine.document_count(), 1);
    assert!(res.is_ok());
}

#[test]
fn test_vector_engine_add_document_dif_dimension() {
    // 벡터엔진 준비
    let dimension = 3;
    let mut engine = VectorEngine::new(dimension);

    // 벡터 생성
    let input_vector = vec![0.1, 0.2];
    let id = hash_vector(&input_vector.clone());

    // 벡터 입력
    // ! Err(String) 객체 반환하는게 맞음
    let res = engine.add_document(id, input_vector);

    // Err 객체인지 검증
    assert!(res.is_err());

    // ! 실패 후 문서 개수가 여전히 0인지 확인
    assert_eq!(engine.document_count(), 0);
}
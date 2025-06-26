use prost::Message;

use rust_vector_engine::Document;

// document 생성자 new 테스트
#[test]
fn test_document_creation() {
    let id = 101;
    let vector = vec![1.0, 2.0, 3.0];

    let doc = Document::new(id, vector.clone());
    
    // 생성된 구조체의 필드값이 예상과 같은지 확인
    assert_eq!(doc.id, id);
    assert_eq!(doc.vector, vector);
}

// document json 직렬화 테스트
#[test]
fn test_document_serialization_with_prost() {
    let doc = Document::new(102, vec![0.5, 1.5, 2.5]);

    // 1. prost를 사용해 바이너리 데이터 (Vec<u8>)로 인코딩
    let mut buf = Vec::new();
    doc.encode(&mut buf).unwrap();

    // 2. 결과물이 비어있지 않은지 확인
    assert!(!buf.is_empty());
}

// document 역직렬화 테스트
#[test]
fn test_document_deserialization_with_prost() {
    // 1. 원본 객체 생성
    let original_doc = Document::new(103, vec![10.0, 20.0]);
    let mut buf = Vec::new();
    original_doc.encode(&mut buf).unwrap();

    // 2. 바이너리 데이터로 역직렬화 시도
    let deserialized_doc = Document::decode(&buf[..]).unwrap();

    // 3. 복원된 객체가 원본과 동일한지 확인합니다.
    assert_eq!(deserialized_doc.id, 103);
    assert_eq!(deserialized_doc.vector, vec![10.0, 20.0]);
}

// Document 직렬화-역직렬화 테스트
#[test]
fn test_document_round_trip() {
    let original_doc = Document::new(104, vec![-1.0, -2.0]);

    let mut buf = Vec::new();
    original_doc.encode(&mut buf).unwrap();
    let deserialized_doc = Document::decode(&buf[..]).unwrap();

    assert_eq!(original_doc, deserialized_doc);
}

use prost::Message;
use std::fs::File;
use std::io::{Read, Write};
use rust_vector_engine::models::Document;

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

// Document vecbin 파일 라운드트립 테스트
#[test]
fn test_save_and_load_document_to_file() {
    // --- 준비 (Arrange) ---
    // 1. 테스트할 원본 Document 객체를 생성합니다.
    let original_doc: Document = Document::new(201, vec![10.1, 20.2, 30.3]);
    let test_file_path: &'static str = "test_document.vecbin";

    // --- 실행: 저장 (Act: Save) ---
    // 2. 객체를 prost를 이용해 바이트 벡터로 직렬화(인코딩)합니다.
    let mut buf: Vec<u8> = Vec::new();
    original_doc.encode(&mut buf).unwrap();

    // 3. 파일을 생성하고, 직렬화된 바이트를 파일에 씁니다.
    // File::create는 Result를 반환하므로 .unwrap()으로 간단히 처리 (테스트에서는 괜찮음)
    let mut file: File = File::create(test_file_path).unwrap();
    file.write_all(&buf).unwrap();

    // --- 실행: 불러오기 (Act: Load) ---
    // 4. 방금 저장한 파일을 다시 엽니다.
    let mut file: File = File::open(test_file_path).unwrap();
    let mut loaded_buf: Vec<u8> = Vec::new();
    // 5. 파일의 모든 내용을 바이트 벡터로 읽어들입니다.
    file.read_to_end(&mut loaded_buf).unwrap();

    // 6. 읽어들인 바이트를 prost를 이용해 Document 객체로 역직렬화(디코딩)합니다.
    let loaded_doc: Document = Document::decode(&loaded_buf[..]).unwrap();

    // --- 검증 (Assert) ---
    // 7. 원본 객체와 파일에서 복원된 객체가 완전히 동일한지 확인합니다.
    assert_eq!(original_doc, loaded_doc);

    // --- 정리 (Teardown) ---
    // 8. 테스트가 끝난 후 생성된 테스트 파일을 삭제합니다.
    std::fs::remove_file(test_file_path).unwrap();
}


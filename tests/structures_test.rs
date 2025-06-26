use rust_vector_engine::structures::Document;

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
fn test_document_serialization_to_json() {
    let doc = Document::new(102, vec![0.5, 1.5, 2.5]);

    let serialized = serde_json::to_string(&doc).unwrap();
    println!("{}", serialized);
    let expect_json = r#"{"id":102,"vector":[0.5,1.5,2.5]}"#;
    assert_eq!(serialized, expect_json);
}

// document 역직렬화 테스트
#[test]
fn test_document_deserialization_from_json() {
    // documnet에 #[derive(Desirialize)] 가 없다고 가정 (RED)
    let json_data = r#"{"id":103, "vector":[10.0,20.0]}"#;

    // 역직렬화 시도
    let deserialized:Document = serde_json::from_str(json_data).unwrap();
    assert_eq!(deserialized.id, 103);
    assert_eq!(deserialized.vector, vec![10.0, 20.0]);
}

// Document 직렬화-역직렬화 테스트
#[test]
fn test_document_round_trip() {
    let original_doc = Document::new(104, vec![-1.0, -2.0]);

    let serialized = serde_json::to_string(&original_doc).unwrap();
    let deserialized = serde_json::from_str(&serialized).unwrap();

    assert_eq!(original_doc, deserialized);
}

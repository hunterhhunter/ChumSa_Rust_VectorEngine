use rust_vector_engine::models::document::{EngineState, Document};
use rust_vector_engine::models::{VectorEngine};
use rust_vector_engine::utils::hash_vector;
use prost::Message;


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

// VectorEngine Search 함수 테스트
#[test]
fn test_vector_engine_search_hit() {
    // 벡터엔진 준비
    let dimension = 3;
    let mut engine = VectorEngine::new(dimension);

    // 테스트를 위해 예측 가능한 ID를 사용합니다.
    let id1 = 1;
    let id2 = 2;
    let id3 = 3;
    let id4 = 4;

    let input_vector1: Vec<f32> = vec![0.1, 0.2, 0.3];
    let input_vector2: Vec<f32> = vec![0.9, 0.8, 0.7]; // query_vector와 가장 유사한 벡터
    let input_vector3: Vec<f32> = vec![0.2, 0.4, 0.5];
    let input_vector4: Vec<f32> = vec![0.2, 0.5, 0.6];
    
    // 검색할 기준 벡터. input_vector2와 거의 동일하게 설정합니다.
    let query_vector: Vec<f32> = vec![0.9, 0.8, 0.71];

    // 벡터 입력
    engine.add_document(id1, input_vector1).unwrap();
    engine.add_document(id2, input_vector2).unwrap();
    engine.add_document(id3, input_vector3).unwrap();
    engine.add_document(id4, input_vector4).unwrap();


    // 검색 시작
    let top_k = 4;
    let results = engine.search(&query_vector, top_k);

    // 검색 성공했는지
    assert!(results.is_ok());
    let search_results = results.unwrap();

    // 1. 요청한 개수(top_k)만큼 결과가 반환되었는지
    assert_eq!(search_results.len(), top_k);

    // 2. 가장 유사한 결과 (0번 인덱스)의 ID가 예상대로 id 2인지
    assert_eq!(search_results[0].0, id2);
    println!("Search Results (ID, Distance): {:?}", search_results);
}


#[test]
fn test_vector_engine_search_miss() {
    // 1. 준비 (Arrange)
    let mut engine = VectorEngine::new(3);
    engine.add_document(1, vec![0.1, 0.2, 0.3]).unwrap();
    engine.add_document(2, vec![0.9, 0.8, 0.7]).unwrap();
    let query_vector = vec![0.8, 0.8, 0.8];

    // 실행 전, 캐시는 비어있고 통계는 모두 0이어야 함
    assert_eq!(engine.query_cache_len(), 0);
    assert_eq!(engine.query_cache_stats().hits, 0);
    assert_eq!(engine.query_cache_stats().misses, 0);

    // 2. 첫 번째 검색 (Act 1 - Cache Miss)
    let first_result = engine.search(&query_vector, 1).unwrap();

    // 3. 검증 (Assert 1)
    // 캐시 미스가 1 증가하고, 캐시에 결과가 저장되어야 함
    assert_eq!(engine.query_cache_stats().misses, 1);
    assert_eq!(engine.query_cache_len(), 1);
    assert_eq!(first_result[0].0, 2); // 가장 가까운 벡터는 ID 2

    // 4. 두 번째 검색 (Act 2 - Cache Hit)
    let second_result = engine.search(&query_vector, 1).unwrap();

    // 5. 검증 (Assert 2)
    // 캐시 히트가 1 증가하고, 미스는 그대로여야 함
    assert_eq!(engine.query_cache_stats().hits, 1);
    assert_eq!(engine.query_cache_stats().misses, 1);
    // 두 결과는 동일해야 함
    assert_eq!(first_result, second_result);
}

#[test]
fn test_vector_engine_search_on_empty_engine() {
    // 준비
    let mut engine = VectorEngine::new(3);
    let query_vector = vec![0.1, 0.2, 0.3];

    // 실행
    let results = engine.search(&query_vector, 3);

    // 검증
    assert!(results.is_ok());
    // 결과 벡터가 비어있는지 확인
    assert!(results.unwrap().is_empty());
}

#[test]
fn test_vector_engine_search_with_k_larger_than_docs() {
    // 준비
    let mut engine = VectorEngine::new(3);
    // 방향이 서로 다른 벡터들을 추가
    engine.add_document(1, vec![1.0, 0.1, 0.2]).unwrap(); // x축에 가까움
    engine.add_document(2, vec![0.1, 1.0, 0.3]).unwrap(); // y축에 가까움
    engine.add_document(3, vec![0.2, 0.1, 1.0]).unwrap(); // z축에 가까움
    
    // ID 1의 벡터와 가장 유사한 쿼리 벡터
    let query_vector = vec![0.9, 0.0, 0.1];

    // 실행
    let results = engine.search(&query_vector, 3).unwrap();

    // 검증
    assert_eq!(results.len(), 3);
    // 가장 가까운 결과(거리가 가장 작은 결과)는 ID 1이어야 함
    assert_eq!(results[0].0, 1);
}

#[test]
fn test_vector_engine_search_after_cache_invalidation() {
    // 준비
    let mut engine = VectorEngine::new(3);
    engine.add_document(1, vec![0.1, 0.1, 0.1]).unwrap(); // 덜 유사한 벡터
    let query_vector = vec![0.9, 0.9, 0.9];

    // 첫 번째 검색 -> ID 1이 캐시에 저장됨
    let first_results = engine.search(&query_vector, 1).unwrap();
    assert_eq!(first_results[0].0, 1);

    // 훨씬 더 유사한 새 벡터 추가 -> 이 때 캐시가 비워져야 함
    engine.add_document(2, vec![0.8, 0.8, 0.8]).unwrap();
    
    // 다시 검색
    let second_results = engine.search(&query_vector, 1).unwrap();

    // 검증: 캐시의 낡은 데이터가 아닌, 새로 추가된 ID 2가 반환되어야 함
    assert_eq!(second_results[0].0, 2);
}

// 직렬화 - 역직렬화 테스트
#[test]
fn test_vector_engine_serialization() {
    // 준비
    let mut engine = VectorEngine::new(3);
    engine.add_document(1, vec![1.0, 0.1, 0.2]).unwrap();
    engine.add_document(2, vec![0.1, 1.0, 0.3]).unwrap();
    engine.add_document(3, vec![0.2, 0.1, 1.0]).unwrap();

    // engine.save_to_bytes()를 통해 직렬화된 데이터를 얻음
    let saved_bytes = engine.save_to_bytes().unwrap();

    // 수동으로 비교 대상을 만듦 (순서는 다를 수 있음)
    let mut expected_state = EngineState {
        format_version: 1,
        documents: vec![
            Document { id: 1, vector: vec![1.0, 0.1, 0.2] },
            Document { id: 2, vector: vec![0.1, 1.0, 0.3] },
            Document { id: 3, vector: vec![0.2, 0.1, 1.0] },
        ],
    };

    // 실행: 저장된 바이트를 다시 역직렬화
    let mut reloaded_state = EngineState::decode(saved_bytes.as_slice()).unwrap();

    // 검증: 내용을 비교하기 전, 두 벡터를 ID 기준으로 정렬
    expected_state.documents.sort_by_key(|d| d.id);
    reloaded_state.documents.sort_by_key(|d| d.id);

    // 정렬된 두 documents 벡터의 내용이 같은지 비교
    assert_eq!(expected_state.documents, reloaded_state.documents);
}

#[test]
fn test_vector_engine_deserialization() {
        // 준비
    let mut engine = VectorEngine::new(3);
    // 벡터들을 추가
    engine.add_document(1, vec![1.0, 0.1, 0.2]).unwrap(); // x축에 가까움
    engine.add_document(2, vec![0.1, 1.0, 0.3]).unwrap(); // y축에 가까움
    engine.add_document(3, vec![0.2, 0.1, 1.0]).unwrap(); // z축에 가까움

    let state: EngineState = EngineState {
        format_version: 1, 
        documents: vec![
                Document{
                    id: 1,
                    vector: vec![1.0, 0.1, 0.2]
                }, 
                Document{
                    id: 2,
                    vector: vec![0.1, 1.0, 0.3]
                }, 
                Document{
                    id: 3,
                    vector: vec![0.2, 0.1, 1.0]
                }, 
            ]
        };

    let mut buf: Vec<u8> =  Vec::new();
    state.encode(&mut buf).unwrap();
    
    let serialized_engine = engine.save_to_bytes().unwrap();

    let engine_v1 = VectorEngine::load_from_bytes(buf.as_slice(), 3).unwrap();
    let engine_v2 = VectorEngine::load_from_bytes(serialized_engine.as_slice(), 3).unwrap();

    assert_eq!(engine_v1.document_count(), engine_v2.document_count());
    assert_eq!(engine_v1.dimension(), engine_v2.dimension());

    let doc1 = engine_v1.documents();
    let doc2 = engine_v2.documents();
    assert_eq!(doc1.get(&1).unwrap(),  doc2.get(&1).unwrap());
}

#[test]
fn test_vector_engine_round_trip() {
    // 1. 준비 (Arrange): 원본 엔진을 만들고 데이터를 추가합니다.
    let dimension = 3;
    let mut original_engine = VectorEngine::new(dimension);
    original_engine.add_document(1, vec![1.0, 0.1, 0.2]).unwrap();
    original_engine.add_document(2, vec![0.1, 1.0, 0.3]).unwrap();

    // 2. 실행 1 (Save): 원본 엔진을 바이트로 직렬화합니다.
    let bytes = original_engine.save_to_bytes().unwrap();

    // 3. 실행 2 (Load): 저장된 바이트로부터 새로운 엔진을 복원합니다.
    let reloaded_engine = VectorEngine::load_from_bytes(&bytes, dimension).unwrap();

    // 4. 검증 (Assert): 원본 엔진과 복원된 엔진의 상태가 완전히 동일한지 확인합니다.
    assert_eq!(original_engine.dimension(), reloaded_engine.dimension());
    assert_eq!(original_engine.document_count(), reloaded_engine.document_count());
    
    // documents getter를 사용하여 두 해시맵이 동일한지 직접 비교합니다.
    assert_eq!(original_engine.documents(), reloaded_engine.documents());
}
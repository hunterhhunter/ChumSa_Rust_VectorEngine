use criterion::{criterion_group, criterion_main, Criterion};
use prost::Message;
use rand::{rngs::StdRng, Rng, SeedableRng};
use rust_vector_engine::models::{
    document::{Document, EngineState},
    VectorEngine,
};
use std::hint::black_box;

// 벤치마크에 사용할 상수 정의
const DIMENSION: usize = 1536;
const NUM_VECTORS: usize = 1000;

/// 벤치마크용 직렬화된 EngineState 바이트 데이터를 생성하는 함수
fn get_serialized_data() -> Vec<u8> {
    let mut rng = StdRng::from_seed([0; 32]);
    let documents: Vec<Document> = (0..NUM_VECTORS)
        .map(|i| {
            let vector: Vec<f32> = (0..DIMENSION).map(|_| rng.random::<f32>()).collect();
            Document { id: i as u64, vector }
        })
        .collect();
    
    let state = EngineState {
        format_version: 1,
        documents,
    };

    let mut buf = Vec::new();
    state.encode(&mut buf).unwrap();
    buf
}

/// Criterion 벤치마크를 설정하고 실행하는 함수
pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("VectorEngine Realistic Scenarios");
    group.sample_size(10);

    // --- 시나리오 1: 초기 로딩 (Deserialization + Full Build) ---
    group.bench_function("1_load_from_bytes_1000_docs", |b| {
        // 미리 직렬화된 데이터를 한 번만 생성
        let bytes = get_serialized_data();
        // iter 안에서 load_from_bytes만 반복 측정
        b.iter(|| {
            VectorEngine::load_from_bytes(black_box(&bytes), black_box(DIMENSION)).unwrap();
        });
    });

    // --- 시나리오 2: 문서 1개 추가 및 재구성 (Incremental Add + Rebuild) ---
    group.bench_function("2_add_and_rebuild_on_1000_docs", |b| {
        let new_doc_id = (NUM_VECTORS + 1) as u64;
        let new_vector: Vec<f32> = {
            let mut rng = StdRng::from_seed([1; 32]);
            (0..DIMENSION).map(|_| rng.random::<f32>()).collect()
        };
        
        // 매 측정마다 1000개의 문서가 있는 엔진을 새로 준비
        b.iter_with_setup(
            || {
                let bytes = get_serialized_data();
                VectorEngine::load_from_bytes(&bytes, DIMENSION).unwrap()
            },
            // 준비된 엔진에 문서 1개만 추가하는 시간을 측정
            |mut engine| {
                engine.add_document(black_box(new_doc_id), black_box(new_vector.clone())).unwrap();
            }
        );
    });

    // --- 시나리오 3: 순수 검색 (Pure Search) ---
    group.bench_function("3_search_in_1000_docs", |b| {
        // 미리 1,000개의 문서가 채워진 엔진을 준비
        let bytes = get_serialized_data();
        let mut engine = VectorEngine::load_from_bytes(&bytes, DIMENSION).unwrap();
        let query_vector: Vec<f32> = {
            let mut rng = StdRng::from_seed([2; 32]);
            (0..DIMENSION).map(|_| rng.random()).collect()
        };
        
        // 준비된 엔진에서 search 메서드만 반복 측정
        b.iter(|| {
            engine.search(black_box(&query_vector), black_box(10))
        });
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
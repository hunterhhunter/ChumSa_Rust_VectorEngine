use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rust_vector_engine::models::VectorEngine; 
use std::hint::black_box; // criterion::black_box 대신 표준 라이브러리 사용

const DIMENSION: usize = 1536;
const NUM_VECTORS: usize = 1000;

/// 벤치마크용 랜덤 데이터를 생성하는 함수
fn generate_random_data(num: usize) -> Vec<(u64, Vec<f32>)> {
    let mut rng = StdRng::from_seed([0; 32]);
    (0..num)
        .map(|i| {
            let vector: Vec<f32> = (0..DIMENSION).map(|_| rng.r#gen::<f32>()).collect();
            (i as u64, vector)
        })
        .collect()
}

/// Criterion 벤치마크를 설정하고 실행하는 함수
pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("VectorEngine Benchmarks");

    // --- 1. add_document 5,000개 실행 성능 측정 ---
    group.bench_function("add_1000_docs", |b| {
        let data = generate_random_data(NUM_VECTORS);
        b.iter_with_setup(
            || VectorEngine::new(DIMENSION),
            |mut engine| {
                for (id, vector) in &data {
                    // black_box로 감싸서 컴파일러 최적화 방지
                    engine.add_document(black_box(*id), black_box(vector.clone())).unwrap();
                }
            },
        );
    });

    // --- 2. 5,000개 문서가 있는 상태에서 search 1회 실행 성능 측정 ---
    group.bench_function("search_in_1000_docs", |b| {
        let mut engine = VectorEngine::new(DIMENSION);
        let data = generate_random_data(NUM_VECTORS);
        for (id, vector) in data {
            engine.add_document(id, vector).unwrap();
        }
        let query_vector = generate_random_data(1).pop().unwrap().1;
        
        b.iter(|| {
            engine.search(black_box(&query_vector), black_box(10))
        });
    });

    group.finish();
}

// Criterion 벤치마크 실행을 위한 필수 매크로
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
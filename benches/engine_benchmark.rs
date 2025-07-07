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
    group.sample_size(10); // 각 벤치마크의 샘플 수를 10으로 줄여서 실행 시간 단축

    // --- 1. Create: add_document 1,000개 실행 성능 측정 ---
    group.bench_function("C: add_1000_docs", |b| {
        let data = generate_random_data(NUM_VECTORS);
        b.iter_with_setup(
            || VectorEngine::new(DIMENSION),
            |mut engine| {
                for (id, vector) in &data {
                    engine.add_document(black_box(*id), black_box(vector.clone())).unwrap();
                }
            },
        );
    });

    // --- 2. Read: 1,000개 문서가 있는 상태에서 search 1회 실행 성능 측정 ---
    group.bench_function("R: search_in_1000_docs", |b| {
        let mut engine = VectorEngine::new(DIMENSION);
        let data = generate_random_data(NUM_VECTORS);
        for (id, vector) in data {
            engine.add_document(id, vector).unwrap();
        }
        let query_vector = generate_random_data(1).pop().unwrap().1;
        
        b.iter(|| {
            engine.search(black_box(&query_vector), black_box(10), black_box(30))
        });
    });

    // --- 3. Update: 1,000개 문서 중 하나를 업데이트하는 성능 측정 ---
    group.bench_function("U: update_doc_in_1000_docs", |b| {
        let data = generate_random_data(NUM_VECTORS);
        let new_vector = generate_random_data(1).pop().unwrap().1;
        let update_id = (NUM_VECTORS / 2) as u64;

        // 매 측정마다 1000개의 문서가 있는 엔진을 새로 준비
        b.iter_with_setup(
            || {
                let mut engine = VectorEngine::new(DIMENSION);
                for (id, vector) in &data {
                    engine.add_document(*id, vector.clone()).unwrap();
                }
                engine
            },
            // 준비된 엔진에서 update 메서드만 실행하여 시간 측정
            |mut engine| {
                engine.update_document(black_box(&update_id), black_box(new_vector.clone()));
            }
        );
    });


    // --- 4. Delete: 1,000개 문서 중 하나를 삭제하는 성능 측정 ---
    group.bench_function("D: delete_doc_in_1000_docs", |b| {
        let data = generate_random_data(NUM_VECTORS);
        let delete_id = (NUM_VECTORS / 2) as u64;

        // 매 측정마다 1000개의 문서가 있는 엔진을 새로 준비
        b.iter_with_setup(
            || {
                let mut engine = VectorEngine::new(DIMENSION);
                for (id, vector) in &data {
                    engine.add_document(*id, vector.clone()).unwrap();
                }
                engine
            },
            // 준비된 엔진에서 delete 메서드만 실행하여 시간 측정
            |mut engine| {
                engine.delete_document(black_box(&delete_id));
            }
        );
    });


    group.finish();
}

// Criterion 벤치마크 실행을 위한 필수 매크로
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
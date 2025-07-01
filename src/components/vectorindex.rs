use hnsw_rs::prelude::*;


pub type VectorIndex<'a> = Hnsw<'a, f32, DistCosine>;

// 2. 생성자(new) 함수도 직관적으로 바뀌었습니다.
pub fn create_vector_index() -> VectorIndex<'static> {
    // Hnsw::new(M, M0, ef_construction, capacity, distance_type)
    // M: 최대 이웃 수
    // M0: 기본 레이어 최대 이웃 수
    // ef_construction: 검색 품질/속도 트레이드오프 파라미터
    // capacity: 예상되는 데이터의 최대 개수
    // distance: 거리 계산 방식 인스턴스
    Hnsw::new(16, 32, 200, 500, DistCosine)
}
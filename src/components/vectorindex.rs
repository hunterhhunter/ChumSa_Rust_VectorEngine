use hnsw_rs::prelude::*;


pub type VectorIndex<'a> = Hnsw<'a, f32, DistCosine>;

pub fn create_vector_index<'b>() -> VectorIndex<'b> {
    // Hnsw::new(M, M0, ef_construction, capacity, distance_type)
    // M: 최대 이웃 수
    // M0: 기본 레이어 최대 이웃 수
    // ef_construction: 검색 품질/속도 트레이드오프 파라미터
    // capacity: 예상되는 데이터의 최대 개수
    // distance: 거리 계산 방식 인스턴스
    Hnsw::new(16, 32, 200, 500, DistCosine)
}
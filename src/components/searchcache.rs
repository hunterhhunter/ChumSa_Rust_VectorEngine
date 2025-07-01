use lru::LruCache;
use std::num::NonZeroUsize;


// --- 2. LRU 쿼리 캐시 타입 정의 ---

/// 검색 쿼리 결과를 저장하는 LRU 캐시의 타입 별칭입니다.
///
/// - 제네릭 `<u64, Vec<(u64, f32)>>`:
///   - `Key (u64)`: 검색어 벡터(`query_vector`)를 해싱한 `u64` 값입니다.
///   - `Value (Vec<(u64, f32)>)`: 검색 결과. 유사도가 높은 문서들의 `(ID, 유사도 점수)` 튜플의 목록입니다.
pub type SearchCache = LruCache<u64, Vec<(u64, f32)>>;

/// LRU 캐시를 생성하는 팩토리 함수입니다.
///
/// # 인자
/// - `capacity`: 캐시가 저장할 수 있는 최대 항목 수
pub fn create_search_cache(capacity: usize) -> SearchCache {
    // 용량이 0이 되는 것을 방지하고 기본값 설정
    let non_zero_capacity = NonZeroUsize::new(capacity)
        .unwrap_or_else(|| NonZeroUsize::new(100).unwrap());

    LruCache::new(non_zero_capacity)
}
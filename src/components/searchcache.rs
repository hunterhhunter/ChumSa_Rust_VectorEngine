use lru::LruCache;
use std::num::NonZeroUsize;

pub type SearchCache = LruCache<u64, Vec<(u64, f32)>>;

pub fn create_search_cache(capacity: usize) -> SearchCache {
    // 용량이 0이 되는 것을 방지하고 기본값 설정
    let non_zero_capacity = NonZeroUsize::new(capacity)
        .unwrap_or_else(|| NonZeroUsize::new(100).unwrap());

    LruCache::new(non_zero_capacity)
}
use lru::LruCache;
use std::num::NonZeroUsize;

#[derive(Default, Debug)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64
}

pub struct SearchCache<'a, K: std::hash::Hash + Eq, V> {
    cache: LruCache<K, V>,
    stats: CacheStats,
    _phantom: std::marker::PhantomData<&'a ()>
}

impl<'a, K: std::hash::Hash + Eq, V> SearchCache<'a, K, V> {
    /// 통계 기능이 추가된 새 캐시를 생성합니다.
    pub fn new(capacity: usize) -> Self {
        // 용량이 0이 되는 것을 방지
        let non_zero_capacity = NonZeroUsize::new(capacity)
            .unwrap_or_else(|| NonZeroUsize::new(1).unwrap()); // 0일 경우 기본값 1

        SearchCache {
            cache: LruCache::new(non_zero_capacity),
            stats: CacheStats::default(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 데이터를 추가합니다. (내부 캐시의 put 호출)
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        self.cache.put(key, value)
    }

    /// 데이터를 조회하며 히트/미스를 기록합니다.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let result = self.cache.get(key);
        if result.is_some() {
            self.stats.hits += 1;
        } else {
            self.stats.misses += 1;
        }
        result
    }

    pub fn contains(&self, key: &K) -> bool {
        self.cache.contains(key)
    }

    /// 현재 히트율을 백분율(%)로 계산하여 반환합니다.
    pub fn hit_rate(&self) -> f64 {
        let total = self.stats.hits + self.stats.misses;
        if total == 0 {
            0.0
        } else {
            (self.stats.hits as f64 / total as f64) * 100.0
        }
    }

    /// 현재 통계 정보를 반환합니다.
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 현재 캐시 내부 요소 개수를 반환합니다.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 캐시를 초기화합니다.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.stats.hits = 0;
        self.stats.misses = 0;
    }
}
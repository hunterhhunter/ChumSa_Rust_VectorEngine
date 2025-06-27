use ahash::AHasher;
use std::hash::{Hasher};


/// 주어진 벡터에 대해 고유한 해시 키를 생성합니다.
///
/// # 인자
/// - `vec`: 해시를 생성할 벡터
///
/// # 반환값
/// - `u64`: 해시 키
pub fn hash_vector(vec: &Vec<f32>) -> u64 {
    let mut hasher = AHasher::default();
    // f32를 직접 해싱할 수 없어, 비트 표현을 u32로 변환하여 해싱
    for &v in vec.iter() {
        hasher.write_u32(v.to_bits());
    }
    hasher.finish()
}
/// 두 벡터 사이의 코사인 유사도를 계산합니다.
///
/// 코사인 유사도는 두 벡터가 같은 방향을 가리킬수록 1에,
/// 반대 방향을 가리킬수록 -1에 가까워집니다.
/// 보통 임베딩 벡터에서는 0~1 사이의 값을 가지며, 1에 가까울수록 유사합니다.
///
/// # Panics
/// 벡터의 길이가 다르면 패닉을 일으킵니다.
/// 
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vectors must have the same length");

    let dot_product = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
    
    let norm_a = a.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x.powi(2)).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0; // 제로 벡터와의 유사도는 0으로 처리
    }

    dot_product / (norm_a * norm_b)
}
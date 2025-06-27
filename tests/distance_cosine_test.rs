use crate::distance::cosine_similarity;

#[test] 
fn test_cosine_similarity_same_vector() {
    let a: Vec<f32> = vec![1.0, 2.0, 3.0];
    let b: Vec<f32> = vec![1.0, 2.0, 3.0];
    
    // 동일한 벡터의 유사도는 1.0
    assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
}

#[test]
fn test_cosine_similarity_orthogonal_vector() {
    let a: Vec<f32> = vec![1.0, 0.0];
    let b: Vec<f32> = vec![0.0, 1.0];

    assert!((cosine_similarity(&a, &b) - 0.0).abs() < 1e-6);
}
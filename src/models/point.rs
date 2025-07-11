use instant_distance::Point as IDPoint;

#[derive(Clone, Debug)]
pub struct MyPoint(pub Vec<f32>);

impl IDPoint for MyPoint {
    fn distance(&self, other: &Self) -> f32 {
        let dot_product: f32 = self.0.iter().zip(&other.0).map(|(a, b)| a * b).sum();
        let norm_a: f32 = self.0.iter().map(|a| a.powi(2)).sum::<f32>().sqrt();
        let norm_b: f32 = other.0.iter().map(|b| b.powi(2)).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { return 2.0; }
        1.0 - (dot_product / (norm_a * norm_b))
    }
}
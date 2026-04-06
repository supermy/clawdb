use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DistanceMetric {
    Euclidean,
    Cosine,
    DotProduct,
    Manhattan,
}

impl DistanceMetric {
    pub fn compute(&self, a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len(), "Vector dimensions must match");

        match self {
            DistanceMetric::Euclidean => self.euclidean_distance(a, b),
            DistanceMetric::Cosine => self.cosine_distance(a, b),
            DistanceMetric::DotProduct => self.dot_product_distance(a, b),
            DistanceMetric::Manhattan => self.manhattan_distance(a, b),
        }
    }

    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        let distance_squared: f32 = a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum();
        distance_squared.sqrt()
    }

    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 1.0;
        }

        1.0 - (dot_product / (norm_a * norm_b))
    }

    fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        -dot
    }

    fn manhattan_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
    }
}

pub fn euclidean_distance_simd(a: &[f32], b: &[f32]) -> f32 {
    use simsimd::SpatialSimilarity;

    let distance_sq: f64 = f32::sqeuclidean(a, b).unwrap_or(0.0);
    distance_sq.sqrt() as f32
}

pub fn cosine_similarity_simd(a: &[f32], b: &[f32]) -> f32 {
    use simsimd::SpatialSimilarity;

    let similarity: f64 = f32::cosine(a, b).unwrap_or(0.0);
    similarity as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let metric = DistanceMetric::Euclidean;
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];

        let distance = metric.compute(&a, &b);
        assert!((distance - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance() {
        let metric = DistanceMetric::Cosine;
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];

        let distance = metric.compute(&a, &b);
        assert!((distance - 0.0).abs() < 1e-6);

        let c = vec![0.0, 1.0];
        let distance2 = metric.compute(&a, &c);
        assert!((distance2 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product_distance() {
        let metric = DistanceMetric::DotProduct;
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];

        let distance = metric.compute(&a, &b);
        let expected_dot = -(1.0 * 4.0 + 2.0 * 5.0 + 3.0 * 6.0);
        assert!((distance - expected_dot).abs() < 1e-6);
    }

    #[test]
    fn test_manhattan_distance() {
        let metric = DistanceMetric::Manhattan;
        let a = vec![1.0, 2.0];
        let b = vec![4.0, 6.0];

        let distance = metric.compute(&a, &b);
        assert!((distance - 7.0).abs() < 1e-6);
    }
}

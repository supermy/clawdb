use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Vector {
    pub id: u64,
    pub data: Vec<f32>,
}

impl Vector {
    pub fn new(id: u64, data: Vec<f32>) -> Self {
        Self { id, data }
    }

    pub fn dimension(&self) -> usize {
        self.data.len()
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize vector")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    pub fn normalize(&mut self) {
        let norm: f32 = self.data.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut self.data {
                *x /= norm;
            }
        }
    }

    pub fn normalized(&self) -> Self {
        let mut normalized = self.clone();
        normalized.normalize();
        normalized
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector(id={}, dim={})", self.id, self.dimension())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let data = vec![1.0, 2.0, 3.0];
        let vector = Vector::new(1, data.clone());

        assert_eq!(vector.id, 1);
        assert_eq!(vector.dimension(), 3);
        assert_eq!(vector.as_slice(), data.as_slice());
    }

    #[test]
    fn test_vector_serialization() {
        let original = Vector::new(42, vec![1.0, 2.0, 3.0, 4.0]);
        let bytes = original.to_bytes();
        let decoded = Vector::from_bytes(&bytes).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_vector_normalization() {
        let mut vector = Vector::new(1, vec![3.0, 4.0]);
        vector.normalize();

        let norm: f32 = vector.data.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_normalized() {
        let vector = Vector::new(1, vec![3.0, 4.0]);
        let normalized = vector.normalized();

        let norm: f32 = normalized.data.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
        assert!((vector.data[0] - 3.0).abs() < 1e-6);
    }
}

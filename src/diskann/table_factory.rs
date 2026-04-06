use crate::diskann::{DiskAnnConfig, ProductQuantizer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskAnnMetadata {
    pub config: DiskAnnConfig,
    pub dimension: usize,
    pub num_vectors: usize,
    pub centroids: Vec<Vec<Vec<f32>>>,
}

impl DiskAnnMetadata {
    pub fn new(config: DiskAnnConfig, dimension: usize) -> Self {
        Self {
            config,
            dimension,
            num_vectors: 0,
            centroids: Vec::new(),
        }
    }

    pub fn from_quantizer(quantizer: &ProductQuantizer, num_vectors: usize) -> Self {
        let centroids = quantizer.centroids()
            .iter()
            .map(|arr| {
                arr.outer_iter()
                    .map(|row| row.to_vec())
                    .collect()
            })
            .collect();
        
        Self {
            config: quantizer.config().clone(),
            dimension: quantizer.dimension(),
            num_vectors,
            centroids,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize metadata")
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskAnnDataBlock {
    pub vector_id: u64,
    pub pq_codes: Vec<u8>,
    pub original_vector: Option<Vec<f32>>,
}

impl DiskAnnDataBlock {
    pub fn new(vector_id: u64, pq_codes: Vec<u8>) -> Self {
        Self {
            vector_id,
            pq_codes,
            original_vector: None,
        }
    }

    pub fn with_original_vector(mut self, vector: Vec<f32>) -> Self {
        self.original_vector = Some(vector);
        self
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize data block")
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }
}

pub struct DiskAnnTableFactory {
    config: DiskAnnConfig,
    dimension: usize,
}

impl DiskAnnTableFactory {
    pub fn new(config: DiskAnnConfig, dimension: usize) -> Self {
        Self { config, dimension }
    }

    pub fn create_quantizer(&self) -> ProductQuantizer {
        ProductQuantizer::new(self.config.clone(), self.dimension)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let config = DiskAnnConfig::default();
        let metadata = DiskAnnMetadata::new(config, 128);
        
        assert_eq!(metadata.dimension, 128);
        assert_eq!(metadata.num_vectors, 0);
    }

    #[test]
    fn test_metadata_serialization() {
        let config = DiskAnnConfig::default();
        let metadata = DiskAnnMetadata::new(config, 128);
        
        let bytes = metadata.to_bytes();
        let decoded = DiskAnnMetadata::from_bytes(&bytes).unwrap();
        
        assert_eq!(decoded.dimension, metadata.dimension);
    }

    #[test]
    fn test_data_block_creation() {
        let block = DiskAnnDataBlock::new(1, vec![0, 1, 2, 3]);
        
        assert_eq!(block.vector_id, 1);
        assert_eq!(block.pq_codes, vec![0, 1, 2, 3]);
        assert!(block.original_vector.is_none());
    }

    #[test]
    fn test_data_block_serialization() {
        let block = DiskAnnDataBlock::new(1, vec![0, 1, 2, 3]);
        
        let bytes = block.to_bytes();
        let decoded = DiskAnnDataBlock::from_bytes(&bytes).unwrap();
        
        assert_eq!(decoded.vector_id, block.vector_id);
        assert_eq!(decoded.pq_codes, block.pq_codes);
    }
}

use byteorder::{ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CollectionId(pub u32);

impl CollectionId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        let mut bytes = [0u8; 4];
        LittleEndian::write_u32(&mut bytes, self.0);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() >= 4 {
            let id = LittleEndian::read_u32(bytes);
            Some(Self(id))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorKey {
    pub collection_id: CollectionId,
    pub vector_id: u64,
}

impl VectorKey {
    pub fn new(collection_id: CollectionId, vector_id: u64) -> Self {
        Self {
            collection_id,
            vector_id,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(12);
        bytes.extend_from_slice(&self.collection_id.to_bytes());
        bytes.extend_from_slice(&self.vector_id.to_be_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() >= 12 {
            let collection_id = CollectionId::from_bytes(&bytes[..4])?;
            let vector_id = u64::from_be_bytes([
                bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11],
            ]);
            Some(Self {
                collection_id,
                vector_id,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorValue {
    pub metadata: VectorMetadata,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorMetadata {
    pub tags: Vec<String>,
    pub timestamp: u64,
    pub deleted: bool,
}

impl VectorValue {
    pub fn new(metadata: VectorMetadata, vector: Vec<f32>) -> Self {
        Self { metadata, vector }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize vector value")
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bincode::deserialize(bytes).ok()
    }

    pub fn tombstone() -> Vec<u8> {
        b"TOMBSTONE".to_vec()
    }

    pub fn is_tombstone(bytes: &[u8]) -> bool {
        bytes.starts_with(b"TOMBSTONE")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collection_id() {
        let id = CollectionId::new(42);
        let bytes = id.to_bytes();
        let decoded = CollectionId::from_bytes(&bytes).unwrap();
        assert_eq!(id, decoded);
    }

    #[test]
    fn test_vector_key() {
        let key = VectorKey::new(CollectionId::new(1), 12345);
        let bytes = key.to_bytes();
        let decoded = VectorKey::from_bytes(&bytes).unwrap();
        assert_eq!(key, decoded);
    }

    #[test]
    fn test_vector_value() {
        let metadata = VectorMetadata {
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            timestamp: 1234567890,
            deleted: false,
        };
        let value = VectorValue::new(metadata, vec![1.0, 2.0, 3.0]);
        let bytes = value.to_bytes();
        let decoded = VectorValue::from_bytes(&bytes).unwrap();
        assert_eq!(value.metadata.tags, decoded.metadata.tags);
        assert_eq!(value.vector, decoded.vector);
    }

    #[test]
    fn test_tombstone() {
        let tombstone = VectorValue::tombstone();
        assert!(VectorValue::is_tombstone(&tombstone));

        let normal = b"normal_data";
        assert!(!VectorValue::is_tombstone(normal));
    }
}

use crate::collection::{CollectionId, VectorKey, VectorMetadata, VectorValue};
use crate::distance::DistanceMetric;
use crate::error::{ClawError, Result};
use crate::index::VectorIndex;
use crate::vector::Vector;
use rayon::prelude::*;
use rocksdb::{ColumnFamilyDescriptor, Options, WriteBatch, DB};
use std::path::Path;
use std::sync::Arc;

pub struct AdvancedVectorStorage {
    db: Arc<DB>,
    dimension: usize,
    metric: DistanceMetric,
    index: Option<VectorIndex>,
}

impl AdvancedVectorStorage {
    pub fn open<P: AsRef<Path>>(path: P, dimension: usize, metric: DistanceMetric) -> Result<Self> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let mut cf_opts = Options::default();
        cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        let cfs = vec![
            ColumnFamilyDescriptor::new("vectors", cf_opts.clone()),
            ColumnFamilyDescriptor::new("metadata", cf_opts.clone()),
            ColumnFamilyDescriptor::new("index", cf_opts),
        ];

        let db = DB::open_cf_descriptors(&options, path, cfs)?;

        Ok(AdvancedVectorStorage {
            db: Arc::new(db),
            dimension,
            metric,
            index: None,
        })
    }

    pub fn insert(&self, collection_id: CollectionId, vector: Vector) -> Result<()> {
        if vector.dimension() != self.dimension {
            return Err(ClawError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.dimension(),
            });
        }

        let key = VectorKey::new(collection_id, vector.id);
        let metadata = VectorMetadata {
            tags: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            deleted: false,
        };
        let value = VectorValue::new(metadata, vector.data);

        let cf = self
            .db
            .cf_handle("vectors")
            .ok_or_else(|| ClawError::InvalidVectorData("Column family not found".to_string()))?;

        self.db.put_cf(cf, key.to_bytes(), value.to_bytes())?;
        Ok(())
    }

    pub fn insert_batch(&self, collection_id: CollectionId, vectors: Vec<Vector>) -> Result<()> {
        let mut batch = WriteBatch::default();
        let cf = self
            .db
            .cf_handle("vectors")
            .ok_or_else(|| ClawError::InvalidVectorData("Column family not found".to_string()))?;

        for vector in vectors {
            if vector.dimension() != self.dimension {
                return Err(ClawError::DimensionMismatch {
                    expected: self.dimension,
                    actual: vector.dimension(),
                });
            }

            let key = VectorKey::new(collection_id.clone(), vector.id);
            let metadata = VectorMetadata {
                tags: vec![],
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                deleted: false,
            };
            let value = VectorValue::new(metadata, vector.data);

            batch.put_cf(cf, key.to_bytes(), value.to_bytes());
        }

        self.db.write(batch)?;
        Ok(())
    }

    pub fn get(&self, collection_id: CollectionId, vector_id: u64) -> Result<Option<Vector>> {
        let key = VectorKey::new(collection_id, vector_id);
        let cf = self
            .db
            .cf_handle("vectors")
            .ok_or_else(|| ClawError::InvalidVectorData("Column family not found".to_string()))?;

        match self.db.get_cf(cf, key.to_bytes())? {
            Some(bytes) => {
                if VectorValue::is_tombstone(&bytes) {
                    return Ok(None);
                }

                let value = VectorValue::from_bytes(&bytes).ok_or_else(|| {
                    ClawError::InvalidVectorData("Failed to deserialize vector".to_string())
                })?;

                if value.metadata.deleted {
                    return Ok(None);
                }

                Ok(Some(Vector::new(vector_id, value.vector)))
            }
            None => Ok(None),
        }
    }

    pub fn delete(&self, collection_id: CollectionId, vector_id: u64) -> Result<()> {
        let key = VectorKey::new(collection_id, vector_id);
        let cf = self
            .db
            .cf_handle("vectors")
            .ok_or_else(|| ClawError::InvalidVectorData("Column family not found".to_string()))?;

        let tombstone = VectorValue::tombstone();
        self.db.put_cf(cf, key.to_bytes(), tombstone)?;
        Ok(())
    }

    pub fn update_metadata(
        &self,
        collection_id: CollectionId,
        vector_id: u64,
        metadata: VectorMetadata,
    ) -> Result<()> {
        let key = VectorKey::new(collection_id, vector_id);
        let cf = self
            .db
            .cf_handle("vectors")
            .ok_or_else(|| ClawError::InvalidVectorData("Column family not found".to_string()))?;

        match self.db.get_cf(cf, key.to_bytes())? {
            Some(bytes) => {
                let mut value = VectorValue::from_bytes(&bytes).ok_or_else(|| {
                    ClawError::InvalidVectorData("Failed to deserialize vector".to_string())
                })?;
                value.metadata = metadata;
                self.db.put_cf(cf, key.to_bytes(), value.to_bytes())?;
                Ok(())
            }
            None => Err(ClawError::VectorNotFound(vector_id)),
        }
    }

    pub fn search(
        &self,
        collection_id: CollectionId,
        query: &[f32],
        k: usize,
        nprobe: usize,
    ) -> Result<Vec<(u64, f32)>> {
        let index = self.index.as_ref().ok_or(ClawError::IndexNotBuilt)?;

        let candidates = index.search(query, k, nprobe)?;

        let mut results: Vec<(u64, f32)> = candidates
            .par_iter()
            .filter_map(|&id| {
                let vector = self.get(collection_id.clone(), id).ok()??;
                let distance = self.metric.compute(query, vector.as_slice());
                Some((id, distance))
            })
            .collect();

        results.par_sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);

        Ok(results)
    }

    pub fn build_index(&mut self, collection_id: CollectionId, nlist: usize) -> Result<()> {
        let vectors = self.load_collection_vectors(collection_id)?;

        if vectors.is_empty() {
            return Err(ClawError::InvalidVectorData(
                "No vectors to index".to_string(),
            ));
        }

        let mut index = VectorIndex::new(self.dimension, self.metric, nlist);
        index.build(&vectors)?;

        self.index = Some(index);
        Ok(())
    }

    fn load_collection_vectors(&self, collection_id: CollectionId) -> Result<Vec<Vector>> {
        let cf = self
            .db
            .cf_handle("vectors")
            .ok_or_else(|| ClawError::InvalidVectorData("Column family not found".to_string()))?;

        let mut vectors = Vec::new();
        let iter = self.db.iterator_cf(cf, rocksdb::IteratorMode::Start);

        let collection_prefix = collection_id.to_bytes();

        for item in iter {
            let (key, value) = item?;

            if key.starts_with(&collection_prefix) && !VectorValue::is_tombstone(&value) {
                if let Some(vector_value) = VectorValue::from_bytes(&value) {
                    if !vector_value.metadata.deleted {
                        if let Some(vector_key) = VectorKey::from_bytes(&key) {
                            vectors.push(Vector::new(vector_key.vector_id, vector_value.vector));
                        }
                    }
                }
            }
        }

        Ok(vectors)
    }

    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    pub fn compact(&self) -> Result<()> {
        let cf = self
            .db
            .cf_handle("vectors")
            .ok_or_else(|| ClawError::InvalidVectorData("Column family not found".to_string()))?;
        self.db.compact_range_cf(cf, None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (AdvancedVectorStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage =
            AdvancedVectorStorage::open(temp_dir.path(), 3, DistanceMetric::Euclidean).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_insert_and_get() {
        let (storage, _temp_dir) = create_test_storage();

        let collection_id = CollectionId::new(1);
        let vector = Vector::new(1, vec![1.0, 2.0, 3.0]);

        storage
            .insert(collection_id.clone(), vector.clone())
            .unwrap();

        let retrieved = storage.get(collection_id, 1).unwrap();
        assert_eq!(retrieved, Some(vector));
    }

    #[test]
    fn test_delete() {
        let (storage, _temp_dir) = create_test_storage();

        let collection_id = CollectionId::new(1);
        let vector = Vector::new(1, vec![1.0, 2.0, 3.0]);

        storage.insert(collection_id.clone(), vector).unwrap();
        storage.delete(collection_id.clone(), 1).unwrap();

        let retrieved = storage.get(collection_id, 1).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_collection_isolation() {
        let (storage, _temp_dir) = create_test_storage();

        let collection1 = CollectionId::new(1);
        let collection2 = CollectionId::new(2);

        let vector1 = Vector::new(1, vec![1.0, 2.0, 3.0]);
        let vector2 = Vector::new(1, vec![4.0, 5.0, 6.0]);

        storage
            .insert(collection1.clone(), vector1.clone())
            .unwrap();
        storage
            .insert(collection2.clone(), vector2.clone())
            .unwrap();

        let retrieved1 = storage.get(collection1, 1).unwrap();
        let retrieved2 = storage.get(collection2, 1).unwrap();

        assert_eq!(retrieved1, Some(vector1));
        assert_eq!(retrieved2, Some(vector2));
    }
}

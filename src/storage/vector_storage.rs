use crate::distance::DistanceMetric;
use crate::error::{ClawError, Result};
use crate::index::VectorIndex;
use crate::storage::{ColumnFamily, Storage};
use crate::vector::Vector;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;

pub struct VectorStorage {
    storage: Arc<Storage>,
    index: Option<VectorIndex>,
    dimension: usize,
    metric: DistanceMetric,
}

impl VectorStorage {
    pub fn open<P: AsRef<Path>>(path: P, dimension: usize, metric: DistanceMetric) -> Result<Self> {
        let storage = Storage::open(path)?;

        Ok(VectorStorage {
            storage: Arc::new(storage),
            index: None,
            dimension,
            metric,
        })
    }

    pub fn insert(&self, vector: Vector) -> Result<()> {
        if vector.dimension() != self.dimension {
            return Err(ClawError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.dimension(),
            });
        }

        let key = vector.id.to_be_bytes();
        let value = vector.to_bytes();

        self.storage.put(ColumnFamily::Data, key, &value)?;
        Ok(())
    }

    pub fn insert_batch(&self, vectors: Vec<Vector>) -> Result<()> {
        let batch: Vec<(ColumnFamily, Vec<u8>, Vec<u8>)> = vectors
            .iter()
            .map(|v| {
                let key = v.id.to_be_bytes().to_vec();
                let value = v.to_bytes();
                (ColumnFamily::Data, key, value)
            })
            .collect();

        self.storage.put_batch(batch)?;
        Ok(())
    }

    pub fn get(&self, id: u64) -> Result<Option<Vector>> {
        let key = id.to_be_bytes();

        match self.storage.get(ColumnFamily::Data, key)? {
            Some(bytes) => {
                let vector = Vector::from_bytes(&bytes)?;
                Ok(Some(vector))
            }
            None => Ok(None),
        }
    }

    pub fn delete(&self, id: u64) -> Result<()> {
        let key = id.to_be_bytes();
        self.storage.delete(ColumnFamily::Data, key)?;
        Ok(())
    }

    pub fn build_index(&mut self, nlist: usize) -> Result<()> {
        let all_vectors = self.load_all_vectors()?;

        if all_vectors.is_empty() {
            return Err(ClawError::InvalidVectorData(
                "No vectors to index".to_string(),
            ));
        }

        let mut index = VectorIndex::new(self.dimension, self.metric, nlist);
        index.build(&all_vectors)?;

        self.index = Some(index);
        Ok(())
    }

    fn load_all_vectors(&self) -> Result<Vec<Vector>> {
        let mut vectors = Vec::new();

        self.storage.scan(ColumnFamily::Data, |_key, value| {
            if let Ok(vector) = Vector::from_bytes(value) {
                vectors.push(vector);
            }
            true
        })?;

        Ok(vectors)
    }

    pub fn search(&self, query: &[f32], k: usize, nprobe: usize) -> Result<Vec<(u64, f32)>> {
        let index = self.index.as_ref().ok_or(ClawError::IndexNotBuilt)?;

        let candidates = index.search(query, k, nprobe)?;

        let mut results: Vec<(u64, f32)> = candidates
            .par_iter()
            .filter_map(|&id| {
                let vector = self.get(id).ok()??;
                let distance = self.metric.compute(query, vector.as_slice());
                Some((id, distance))
            })
            .collect();

        results.par_sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);

        Ok(results)
    }

    pub fn brute_force_search(&self, query: &[f32], k: usize) -> Result<Vec<(u64, f32)>> {
        let all_vectors = self.load_all_vectors()?;

        let mut results: Vec<(u64, f32)> = all_vectors
            .par_iter()
            .map(|vector| {
                let distance = self.metric.compute(query, vector.as_slice());
                (vector.id, distance)
            })
            .collect();

        results.par_sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);

        Ok(results)
    }

    pub fn count(&self) -> Result<usize> {
        let all_vectors = self.load_all_vectors()?;
        Ok(all_vectors.len())
    }

    pub fn flush(&self) -> Result<()> {
        self.storage.flush()?;
        Ok(())
    }

    pub fn compact(&self) -> Result<()> {
        self.storage.compact(ColumnFamily::Data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (VectorStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = VectorStorage::open(temp_dir.path(), 3, DistanceMetric::Euclidean).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_insert_and_get() {
        let (storage, _temp_dir) = create_test_storage();

        let vector = Vector::new(1, vec![1.0, 2.0, 3.0]);
        storage.insert(vector.clone()).unwrap();

        let retrieved = storage.get(1).unwrap();
        assert_eq!(retrieved, Some(vector));
    }

    #[test]
    fn test_insert_batch() {
        let (storage, _temp_dir) = create_test_storage();

        let vectors = vec![
            Vector::new(1, vec![1.0, 2.0, 3.0]),
            Vector::new(2, vec![4.0, 5.0, 6.0]),
        ];

        storage.insert_batch(vectors).unwrap();

        assert!(storage.get(1).unwrap().is_some());
        assert!(storage.get(2).unwrap().is_some());
    }

    #[test]
    fn test_delete() {
        let (storage, _temp_dir) = create_test_storage();

        let vector = Vector::new(1, vec![1.0, 2.0, 3.0]);
        storage.insert(vector).unwrap();

        storage.delete(1).unwrap();

        let retrieved = storage.get(1).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_dimension_mismatch() {
        let (storage, _temp_dir) = create_test_storage();

        let vector = Vector::new(1, vec![1.0, 2.0]);
        let result = storage.insert(vector);

        assert!(result.is_err());
    }

    #[test]
    fn test_count() {
        let (storage, _temp_dir) = create_test_storage();

        assert_eq!(storage.count().unwrap(), 0);

        storage.insert(Vector::new(1, vec![1.0, 2.0, 3.0])).unwrap();
        storage.insert(Vector::new(2, vec![4.0, 5.0, 6.0])).unwrap();

        assert_eq!(storage.count().unwrap(), 2);
    }

    #[test]
    fn test_brute_force_search() {
        let (storage, _temp_dir) = create_test_storage();

        storage.insert(Vector::new(1, vec![1.0, 1.0, 1.0])).unwrap();
        storage.insert(Vector::new(2, vec![2.0, 2.0, 2.0])).unwrap();
        storage
            .insert(Vector::new(3, vec![10.0, 10.0, 10.0]))
            .unwrap();

        let query = vec![1.5, 1.5, 1.5];
        let results = storage.brute_force_search(&query, 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 1);
        assert_eq!(results[1].0, 2);
    }

    #[test]
    fn test_build_and_search() {
        let (mut storage, _temp_dir) = create_test_storage();

        for i in 0..100 {
            let x = i as f32;
            storage
                .insert(Vector::new(i as u64, vec![x, x, x]))
                .unwrap();
        }

        storage.build_index(10).unwrap();

        let query = vec![5.0, 5.0, 5.0];
        let results = storage.search(&query, 5, 3).unwrap();

        assert!(!results.is_empty());
    }
}

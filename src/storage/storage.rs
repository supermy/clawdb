use crate::storage::cf::ColumnFamily;
use crate::storage::error::{Result, StorageError};
use rocksdb::{ColumnFamilyDescriptor, Options, WriteBatch, DB};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

pub struct Storage {
    db: Arc<DB>,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let cfs: Vec<ColumnFamilyDescriptor> = ColumnFamily::all_default()
            .iter()
            .map(|cf| {
                let mut cf_opts = Options::default();
                cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
                ColumnFamilyDescriptor::new(cf.name(), cf_opts)
            })
            .collect();

        let db = DB::open_cf_descriptors(&options, path, cfs)?;

        Ok(Storage { db: Arc::new(db) })
    }

    pub fn open_with_custom_cfs<P: AsRef<Path>>(path: P, custom_cfs: Vec<String>) -> Result<Self> {
        let mut options = Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let mut all_cfs: Vec<ColumnFamilyDescriptor> = ColumnFamily::all_default()
            .iter()
            .map(|cf| {
                let mut cf_opts = Options::default();
                cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
                ColumnFamilyDescriptor::new(cf.name(), cf_opts)
            })
            .collect();

        for cf_name in custom_cfs {
            let mut cf_opts = Options::default();
            cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
            all_cfs.push(ColumnFamilyDescriptor::new(cf_name, cf_opts));
        }

        let db = DB::open_cf_descriptors(&options, path, all_cfs)?;

        Ok(Storage { db: Arc::new(db) })
    }

    pub fn put<K, V>(&self, cf: ColumnFamily, key: K, value: V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let cf_handle = self.get_cf_handle(&cf)?;
        self.db.put_cf(cf_handle, key, value)?;
        Ok(())
    }

    pub fn get<K>(&self, cf: ColumnFamily, key: K) -> Result<Option<Vec<u8>>>
    where
        K: AsRef<[u8]>,
    {
        let cf_handle = self.get_cf_handle(&cf)?;
        let result = self.db.get_cf(cf_handle, key)?;
        Ok(result)
    }

    pub fn delete<K>(&self, cf: ColumnFamily, key: K) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        let cf_handle = self.get_cf_handle(&cf)?;
        self.db.delete_cf(cf_handle, key)?;
        Ok(())
    }

    pub fn put_batch<K, V>(&self, batch: Vec<(ColumnFamily, K, V)>) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let mut write_batch = WriteBatch::default();

        for (cf, key, value) in batch {
            let cf_handle = self.get_cf_handle(&cf)?;
            write_batch.put_cf(cf_handle, key, value);
        }

        self.db.write(write_batch)?;
        Ok(())
    }

    pub fn delete_batch<K>(&self, batch: Vec<(ColumnFamily, K)>) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        let mut write_batch = WriteBatch::default();

        for (cf, key) in batch {
            let cf_handle = self.get_cf_handle(&cf)?;
            write_batch.delete_cf(cf_handle, key);
        }

        self.db.write(write_batch)?;
        Ok(())
    }

    pub fn exists<K>(&self, cf: ColumnFamily, key: K) -> Result<bool>
    where
        K: AsRef<[u8]>,
    {
        let cf_handle = self.get_cf_handle(&cf)?;
        Ok(self.db.get_cf(cf_handle, key)?.is_some())
    }

    pub fn put_json<K, V>(&self, cf: ColumnFamily, key: K, value: &V) -> Result<()>
    where
        K: AsRef<[u8]>,
        V: Serialize,
    {
        let json = serde_json::to_vec(value)?;
        self.put(cf, key, json)
    }

    pub fn get_json<K, V>(&self, cf: ColumnFamily, key: K) -> Result<Option<V>>
    where
        K: AsRef<[u8]>,
        V: for<'de> Deserialize<'de>,
    {
        match self.get(cf, key)? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn compact(&self, cf: ColumnFamily) -> Result<()> {
        let cf_handle = self.get_cf_handle(&cf)?;
        self.db
            .compact_range_cf(cf_handle, None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }

    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    pub fn column_family_names(&self) -> Vec<String> {
        ColumnFamily::all_default()
            .iter()
            .map(|cf| cf.name().to_string())
            .collect()
    }

    pub fn scan<F>(&self, cf: ColumnFamily, mut f: F) -> Result<()>
    where
        F: FnMut(&[u8], &[u8]) -> bool,
    {
        let cf_handle = self.get_cf_handle(&cf)?;
        let iter = self.db.iterator_cf(cf_handle, rocksdb::IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if !f(&key, &value) {
                break;
            }
        }

        Ok(())
    }

    fn get_cf_handle(&self, cf: &ColumnFamily) -> Result<&rocksdb::ColumnFamily> {
        self.db
            .cf_handle(cf.name())
            .ok_or_else(|| StorageError::ColumnFamilyNotFound(cf.name().to_string()))
    }
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        Storage {
            db: Arc::clone(&self.db),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u64,
        name: String,
        value: f64,
    }

    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::open(temp_dir.path()).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_open_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::open(temp_dir.path());
        assert!(storage.is_ok());
    }

    #[test]
    fn test_put_and_get() {
        let (storage, _temp_dir) = create_test_storage();

        let key = b"test_key";
        let value = b"test_value";

        storage.put(ColumnFamily::Default, key, value).unwrap();

        let result = storage.get(ColumnFamily::Default, key).unwrap();
        assert_eq!(result, Some(value.to_vec()));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let (storage, _temp_dir) = create_test_storage();

        let result = storage.get(ColumnFamily::Default, b"nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete() {
        let (storage, _temp_dir) = create_test_storage();

        let key = b"test_key";
        let value = b"test_value";

        storage.put(ColumnFamily::Default, key, value).unwrap();
        assert!(storage.exists(ColumnFamily::Default, key).unwrap());

        storage.delete(ColumnFamily::Default, key).unwrap();
        assert!(!storage.exists(ColumnFamily::Default, key).unwrap());
    }

    #[test]
    fn test_put_and_get_json() {
        let (storage, _temp_dir) = create_test_storage();

        let key = b"test_key";
        let data = TestData {
            id: 42,
            name: "test".to_string(),
            value: 3.14,
        };

        storage.put_json(ColumnFamily::Data, key, &data).unwrap();

        let result: Option<TestData> = storage.get_json(ColumnFamily::Data, key).unwrap();
        assert_eq!(result, Some(data));
    }

    #[test]
    fn test_batch_operations() {
        let (storage, _temp_dir) = create_test_storage();

        let batch = vec![
            (ColumnFamily::Default, b"key1", b"value1"),
            (ColumnFamily::Default, b"key2", b"value2"),
            (ColumnFamily::Data, b"key3", b"value3"),
        ];

        storage.put_batch(batch).unwrap();

        assert_eq!(
            storage.get(ColumnFamily::Default, b"key1").unwrap(),
            Some(b"value1".to_vec())
        );
        assert_eq!(
            storage.get(ColumnFamily::Default, b"key2").unwrap(),
            Some(b"value2".to_vec())
        );
        assert_eq!(
            storage.get(ColumnFamily::Data, b"key3").unwrap(),
            Some(b"value3".to_vec())
        );
    }

    #[test]
    fn test_column_families() {
        let (storage, _temp_dir) = create_test_storage();

        let key = b"same_key";
        let value1 = b"value_in_default";
        let value2 = b"value_in_data";
        let value3 = b"value_in_metadata";

        storage.put(ColumnFamily::Default, key, value1).unwrap();
        storage.put(ColumnFamily::Data, key, value2).unwrap();
        storage.put(ColumnFamily::Metadata, key, value3).unwrap();

        assert_eq!(
            storage.get(ColumnFamily::Default, key).unwrap(),
            Some(value1.to_vec())
        );
        assert_eq!(
            storage.get(ColumnFamily::Data, key).unwrap(),
            Some(value2.to_vec())
        );
        assert_eq!(
            storage.get(ColumnFamily::Metadata, key).unwrap(),
            Some(value3.to_vec())
        );
    }

    #[test]
    fn test_exists() {
        let (storage, _temp_dir) = create_test_storage();

        let key = b"test_key";
        let value = b"test_value";

        assert!(!storage.exists(ColumnFamily::Default, key).unwrap());

        storage.put(ColumnFamily::Default, key, value).unwrap();
        assert!(storage.exists(ColumnFamily::Default, key).unwrap());
    }

    #[test]
    fn test_column_family_names() {
        let (storage, _temp_dir) = create_test_storage();

        let cfs = storage.column_family_names();
        assert!(cfs.contains(&"default".to_string()));
        assert!(cfs.contains(&"metadata".to_string()));
        assert!(cfs.contains(&"data".to_string()));
        assert!(cfs.contains(&"index".to_string()));
    }

    #[test]
    fn test_custom_column_family() {
        let temp_dir = TempDir::new().unwrap();
        let custom_cfs = vec!["custom1".to_string(), "custom2".to_string()];
        let storage = Storage::open_with_custom_cfs(temp_dir.path(), custom_cfs).unwrap();

        let key = b"test_key";
        let value = b"test_value";

        storage
            .put(ColumnFamily::Custom("custom1".to_string()), key, value)
            .unwrap();

        let result = storage
            .get(ColumnFamily::Custom("custom1".to_string()), key)
            .unwrap();
        assert_eq!(result, Some(value.to_vec()));
    }

    #[test]
    fn test_flush_and_compact() {
        let (storage, _temp_dir) = create_test_storage();

        storage
            .put(ColumnFamily::Default, b"key", b"value")
            .unwrap();

        storage.flush().unwrap();
        storage.compact(ColumnFamily::Default).unwrap();

        let result = storage.get(ColumnFamily::Default, b"key").unwrap();
        assert_eq!(result, Some(b"value".to_vec()));
    }
}

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub data: T,
    pub last_access: Instant,
    pub access_count: u64,
}

impl<T> CacheEntry<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            last_access: Instant::now(),
            access_count: 1,
        }
    }

    pub fn touch(&mut self) {
        self.last_access = Instant::now();
        self.access_count += 1;
    }
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size: usize,
    pub ttl: Option<Duration>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 10_000,
            ttl: Some(Duration::from_secs(3600)),
        }
    }
}

pub struct VectorCache<T> {
    cache: Arc<Mutex<HashMap<u64, CacheEntry<T>>>>,
    config: CacheConfig,
}

impl<T: Clone> VectorCache<T> {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn get(&self, id: u64) -> Option<T> {
        let mut cache = self.cache.lock().unwrap();
        
        if let Some(entry) = cache.get_mut(&id) {
            if let Some(ttl) = self.config.ttl {
                if entry.last_access.elapsed() > ttl {
                    cache.remove(&id);
                    return None;
                }
            }
            entry.touch();
            return Some(entry.data.clone());
        }
        None
    }

    pub fn put(&self, id: u64, data: T) {
        let mut cache = self.cache.lock().unwrap();
        
        if cache.len() >= self.config.max_size {
            self.evict_lru(&mut cache);
        }
        
        cache.insert(id, CacheEntry::new(data));
    }

    pub fn get_or_insert<F>(&self, id: u64, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if let Some(data) = self.get(id) {
            return data;
        }
        
        let data = f();
        self.put(id, data.clone());
        data
    }

    fn evict_lru(&self, cache: &mut HashMap<u64, CacheEntry<T>>) {
        if cache.is_empty() {
            return;
        }

        let mut oldest_key = None;
        let mut oldest_time = Instant::now();

        for (key, entry) in cache.iter() {
            if entry.last_access < oldest_time {
                oldest_time = entry.last_access;
                oldest_key = Some(*key);
            }
        }

        if let Some(key) = oldest_key {
            cache.remove(&key);
        }
    }

    pub fn remove(&self, id: u64) -> Option<T> {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(&id).map(|e| e.data)
    }

    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn hit_rate(&self) -> f64 {
        let cache = self.cache.lock().unwrap();
        let total_access: u64 = cache.values().map(|e| e.access_count).sum();
        let entries = cache.len() as u64;
        
        if total_access == 0 {
            return 0.0;
        }
        
        entries as f64 / total_access as f64
    }
}

pub struct MultiLevelCache<T> {
    l1: VectorCache<T>,
    l2: VectorCache<T>,
}

impl<T: Clone> MultiLevelCache<T> {
    pub fn new(l1_size: usize, l2_size: usize) -> Self {
        Self {
            l1: VectorCache::new(CacheConfig {
                max_size: l1_size,
                ttl: Some(Duration::from_secs(300)),
            }),
            l2: VectorCache::new(CacheConfig {
                max_size: l2_size,
                ttl: Some(Duration::from_secs(3600)),
            }),
        }
    }

    pub fn get(&self, id: u64) -> Option<T> {
        if let Some(data) = self.l1.get(id) {
            return Some(data);
        }
        
        if let Some(data) = self.l2.get(id) {
            self.l1.put(id, data.clone());
            return Some(data);
        }
        
        None
    }

    pub fn put(&self, id: u64, data: T) {
        self.l1.put(id, data.clone());
        self.l2.put(id, data);
    }

    pub fn get_or_insert<F>(&self, id: u64, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if let Some(data) = self.get(id) {
            return data;
        }
        
        let data = f();
        self.put(id, data.clone());
        data
    }

    pub fn clear(&self) {
        self.l1.clear();
        self.l2.clear();
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            l1_size: self.l1.len(),
            l2_size: self.l2.len(),
            l1_hit_rate: self.l1.hit_rate(),
            l2_hit_rate: self.l2.hit_rate(),
        }
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub l1_size: usize,
    pub l2_size: usize,
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let cache: VectorCache<Vec<f32>> = VectorCache::new(CacheConfig::default());
        
        cache.put(1, vec![1.0f32, 2.0, 3.0]);
        
        let data = cache.get(1);
        assert!(data.is_some());
        assert_eq!(data.unwrap(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_cache_miss() {
        let cache: VectorCache<Vec<f32>> = VectorCache::new(CacheConfig::default());
        
        let data = cache.get(999);
        assert!(data.is_none());
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            max_size: 2,
            ttl: None,
        };
        let cache: VectorCache<Vec<f32>> = VectorCache::new(config);
        
        cache.put(1, vec![1.0f32]);
        cache.put(2, vec![2.0f32]);
        cache.put(3, vec![3.0f32]);
        
        assert!(cache.get(1).is_none() || cache.get(2).is_none() || cache.get(3).is_some());
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn test_multi_level_cache() {
        let cache = MultiLevelCache::<Vec<f32>>::new(10, 100);
        
        cache.put(1, vec![1.0, 2.0, 3.0]);
        
        let data = cache.get(1);
        assert!(data.is_some());
        
        let stats = cache.stats();
        assert_eq!(stats.l1_size, 1);
        assert_eq!(stats.l2_size, 1);
    }
}

use crate::distance::DistanceMetric;
use crate::error::{ClawError, Result};
use crate::vector::Vector;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndex {
    dimension: usize,
    metric: DistanceMetric,
    nlist: usize,
    centroids: Vec<Vec<f32>>,
    inverted_lists: HashMap<usize, Vec<u64>>,
    built: bool,
}

impl VectorIndex {
    pub fn new(dimension: usize, metric: DistanceMetric, nlist: usize) -> Self {
        Self {
            dimension,
            metric,
            nlist,
            centroids: Vec::new(),
            inverted_lists: HashMap::new(),
            built: false,
        }
    }

    pub fn build(&mut self, vectors: &[Vector]) -> Result<()> {
        if vectors.is_empty() {
            return Err(ClawError::InvalidVectorData(
                "No vectors provided".to_string(),
            ));
        }

        self.centroids = self.kmeans_centroids_parallel(vectors, self.nlist)?;

        self.inverted_lists.clear();
        let assignments: Vec<usize> = vectors
            .par_iter()
            .map(|v| self.find_nearest_centroid(&v.data).unwrap_or(0))
            .collect();

        for (vector, cluster_id) in vectors.iter().zip(assignments.iter()) {
            self.inverted_lists
                .entry(*cluster_id)
                .or_default()
                .push(vector.id);
        }

        self.built = true;
        Ok(())
    }

    fn kmeans_centroids_parallel(&self, vectors: &[Vector], k: usize) -> Result<Vec<Vec<f32>>> {
        let n = vectors.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        let k = k.min(n);
        let mut centroids = self.kmeans_plusplus_init(vectors, k)?;

        let max_iterations = 20;
        let tolerance = 1e-4;

        for _ in 0..max_iterations {
            let assignments: Vec<usize> = vectors
                .par_iter()
                .map(|v| self.find_nearest_centroid_in_list(&v.data, &centroids).unwrap_or(0))
                .collect();

            let new_centroids: Vec<Vec<f32>> = (0..k)
                .into_par_iter()
                .map(|cluster_id| {
                    let mut sum = vec![0.0f32; self.dimension];
                    let mut count = 0usize;

                    for (vector, &assignment) in vectors.iter().zip(assignments.iter()) {
                        if assignment == cluster_id {
                            for (i, &val) in vector.data.iter().enumerate() {
                                sum[i] += val;
                            }
                            count += 1;
                        }
                    }

                    if count > 0 {
                        for val in sum.iter_mut() {
                            *val /= count as f32;
                        }
                    }
                    sum
                })
                .collect();

            let mut max_change = 0.0f32;
            for (old, new) in centroids.iter().zip(new_centroids.iter()) {
                let change: f32 = old
                    .iter()
                    .zip(new.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum();
                max_change = max_change.max(change.sqrt());
            }

            centroids = new_centroids;

            if max_change < tolerance {
                break;
            }
        }

        Ok(centroids)
    }

    fn kmeans_plusplus_init(&self, vectors: &[Vector], k: usize) -> Result<Vec<Vec<f32>>> {
        let n = vectors.len();
        let mut centroids: Vec<Vec<f32>> = Vec::with_capacity(k);

        let mut rng = rand::thread_rng();
        let first_idx = (rand::Rng::gen::<f64>(&mut rng) * n as f64) as usize;
        centroids.push(vectors[first_idx].data.clone());

        for _ in 1..k {
            let distances: Vec<f64> = vectors
                .par_iter()
                .map(|v| {
                    let min_dist: f64 = centroids
                        .iter()
                        .map(|c| {
                            let d = self.metric.compute(&v.data, c);
                            d as f64 * d as f64
                        })
                        .fold(f64::MAX, |a, b| a.min(b));
                    min_dist
                })
                .collect();

            let total: f64 = distances.iter().sum();
            if total == 0.0 {
                break;
            }

            let mut target = rand::Rng::gen::<f64>(&mut rng) * total;
            let mut next_idx = 0;
            for (i, &d) in distances.iter().enumerate() {
                target -= d;
                if target <= 0.0 {
                    next_idx = i;
                    break;
                }
            }

            centroids.push(vectors[next_idx].data.clone());
        }

        while centroids.len() < k {
            let idx = rand::Rng::gen::<f64>(&mut rng) * n as f64;
            centroids.push(vectors[idx as usize].data.clone());
        }

        Ok(centroids)
    }

    fn find_nearest_centroid(&self, vector: &[f32]) -> Result<usize> {
        self.find_nearest_centroid_in_list(vector, &self.centroids)
    }

    fn find_nearest_centroid_in_list(
        &self,
        vector: &[f32],
        centroids: &[Vec<f32>],
    ) -> Result<usize> {
        if centroids.is_empty() {
            return Err(ClawError::IndexNotBuilt);
        }

        let (min_idx, _) = centroids
            .par_iter()
            .enumerate()
            .map(|(i, centroid)| (i, self.metric.compute(vector, centroid)))
            .reduce(|| (0, f32::MAX), |a, b| if a.1 < b.1 { a } else { b });

        Ok(min_idx)
    }

    pub fn search(&self, query: &[f32], _k: usize, nprobe: usize) -> Result<Vec<u64>> {
        if !self.built {
            return Err(ClawError::IndexNotBuilt);
        }

        let mut cluster_distances: Vec<(usize, f32)> = self
            .centroids
            .par_iter()
            .enumerate()
            .map(|(i, centroid)| (i, self.metric.compute(query, centroid)))
            .collect();

        cluster_distances.par_sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let candidates: Vec<u64> = cluster_distances
            .iter()
            .take(nprobe)
            .filter_map(|(cluster_id, _)| self.inverted_lists.get(cluster_id))
            .flat_map(|ids| ids.iter().copied())
            .collect();

        Ok(candidates)
    }

    pub fn is_built(&self) -> bool {
        self.built
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn nlist(&self) -> usize {
        self.nlist
    }

    pub fn centroids(&self) -> &[Vec<f32>] {
        &self.centroids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vectors() -> Vec<Vector> {
        vec![
            Vector::new(1, vec![1.0, 1.0]),
            Vector::new(2, vec![1.1, 1.1]),
            Vector::new(3, vec![5.0, 5.0]),
            Vector::new(4, vec![5.1, 5.1]),
            Vector::new(5, vec![10.0, 10.0]),
        ]
    }

    #[test]
    fn test_index_creation() {
        let index = VectorIndex::new(2, DistanceMetric::Euclidean, 3);

        assert_eq!(index.dimension(), 2);
        assert_eq!(index.nlist(), 3);
        assert!(!index.is_built());
    }

    #[test]
    fn test_index_build() {
        let mut index = VectorIndex::new(2, DistanceMetric::Euclidean, 3);
        let vectors = create_test_vectors();

        index.build(&vectors).unwrap();

        assert!(index.is_built());
        assert_eq!(index.centroids.len(), 3);
    }

    #[test]
    fn test_index_search() {
        let mut index = VectorIndex::new(2, DistanceMetric::Euclidean, 3);
        let vectors = create_test_vectors();
        index.build(&vectors).unwrap();

        let query = vec![1.0, 1.0];
        let candidates = index.search(&query, 2, 2).unwrap();

        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_search_without_build() {
        let index = VectorIndex::new(2, DistanceMetric::Euclidean, 3);
        let query = vec![1.0, 1.0];

        let result = index.search(&query, 2, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_with_empty_vectors() {
        let mut index = VectorIndex::new(2, DistanceMetric::Euclidean, 3);
        let vectors: Vec<Vector> = vec![];

        let result = index.build(&vectors);
        assert!(result.is_err());
    }
}

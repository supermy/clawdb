use crate::distance::DistanceMetric;
use crate::error::{ClawError, Result};
use crate::vector::Vector;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
struct OrderedFloat(f64);

impl PartialEq for OrderedFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

#[derive(Debug, Clone)]
pub struct HnswConfig {
    pub max_elements: usize,
    pub ef_construction: usize,
    pub m_max: usize,
    pub m_max_0: usize,
    pub ml: f64,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            max_elements: 1000000,
            ef_construction: 200,
            m_max: 16,
            m_max_0: 32,
            ml: 1.0 / (16.0_f64).ln(),
        }
    }
}

impl HnswConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
struct Node {
    vector: Vec<f32>,
    neighbors: Vec<Vec<u64>>,
}

pub struct HnswIndex {
    config: HnswConfig,
    metric: DistanceMetric,
    dimension: usize,
    nodes: HashMap<u64, Node>,
    max_level: usize,
    entry_point: Option<u64>,
    built: bool,
}

impl HnswIndex {
    pub fn new(dimension: usize, metric: DistanceMetric, config: HnswConfig) -> Self {
        Self {
            config,
            metric,
            dimension,
            nodes: HashMap::new(),
            max_level: 0,
            entry_point: None,
            built: false,
        }
    }

    pub fn build(&mut self, vectors: &[Vector]) -> Result<()> {
        if vectors.is_empty() {
            return Err(ClawError::InvalidVectorData(
                "No vectors provided".to_string(),
            ));
        }

        let levels: Vec<usize> = vectors
            .iter()
            .map(|_| self.random_level())
            .collect();

        let sorted_indices: Vec<usize> = (0..vectors.len()).collect();
        
        for &idx in &sorted_indices {
            self.insert_with_level(&vectors[idx], levels[idx])?;
        }

        self.built = true;
        Ok(())
    }

    pub fn build_parallel(&mut self, vectors: &[Vector]) -> Result<()> {
        if vectors.is_empty() {
            return Err(ClawError::InvalidVectorData(
                "No vectors provided".to_string(),
            ));
        }

        let levels: Vec<usize> = vectors
            .par_iter()
            .map(|_| self.random_level())
            .collect();

        let max_level = levels.iter().max().copied().unwrap_or(0);
        let entry_idx = levels
            .iter()
            .position(|&l| l == max_level)
            .unwrap_or(0);

        self.insert_with_level(&vectors[entry_idx], levels[entry_idx])?;

        for (idx, vector) in vectors.iter().enumerate() {
            if idx != entry_idx {
                self.insert_with_level(vector, levels[idx])?;
            }
        }

        self.built = true;
        Ok(())
    }

    fn insert_with_level(&mut self, vector: &Vector, level: usize) -> Result<()> {
        if vector.dimension() != self.dimension {
            return Err(ClawError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.dimension(),
            });
        }

        let node = Node {
            vector: vector.data.clone(),
            neighbors: vec![Vec::new(); level + 1],
        };

        if self.entry_point.is_none() {
            self.entry_point = Some(vector.id);
            self.max_level = level;
            self.nodes.insert(vector.id, node);
            return Ok(());
        }

        let entry_point = self.entry_point.unwrap();

        if level > self.max_level {
            self.max_level = level;
            self.entry_point = Some(vector.id);
        }

        let mut current_node = entry_point;

        for curr_level in (level + 1..=self.max_level).rev() {
            let neighbors = self.search_layer(&vector.data, current_node, 1, curr_level)?;
            if !neighbors.is_empty() {
                current_node = neighbors[0].0;
            }
        }

        let mut neighbor_updates: Vec<(u64, usize, Vec<u64>)> = Vec::new();

        for curr_level in (0..=level.min(self.max_level)).rev() {
            let neighbors = self.search_layer(
                &vector.data,
                current_node,
                self.config.ef_construction,
                curr_level,
            )?;

            let m_max = if curr_level == 0 {
                self.config.m_max_0
            } else {
                self.config.m_max
            };
            let selected = self.select_neighbors_heuristic(&neighbors, m_max, &vector.data);

            let all_nodes: Vec<(u64, Vec<u64>, Vec<f32>)> = self
                .nodes
                .iter()
                .map(|(id, n)| {
                    let neighbors = if curr_level < n.neighbors.len() {
                        n.neighbors[curr_level].clone()
                    } else {
                        Vec::new()
                    };
                    (*id, neighbors, n.vector.clone())
                })
                .collect();

            for (neighbor_id, _) in &selected {
                if let Some((_, existing_neighbors, neighbor_vec)) = all_nodes.iter().find(|(id, _, _)| *id == *neighbor_id) {
                    if existing_neighbors.is_empty() && curr_level > 0 {
                        continue;
                    }
                    
                    let mut new_neighbors = existing_neighbors.clone();
                    new_neighbors.push(vector.id);
                    
                    let neighbor_m_max = if curr_level == 0 {
                        self.config.m_max_0
                    } else {
                        self.config.m_max
                    };
                    
                    if new_neighbors.len() > neighbor_m_max {
                        let neighbor_dists: Vec<(u64, f64)> = new_neighbors
                            .iter()
                            .filter_map(|&nid| {
                                all_nodes.iter().find(|(id, _, _)| *id == nid).map(|(_, _, vec)| {
                                    (nid, self.metric.compute(neighbor_vec, vec) as f64)
                                })
                            })
                            .collect();
                        
                        let mut sorted_dists = neighbor_dists;
                        sorted_dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                        new_neighbors = sorted_dists
                            .iter()
                            .take(neighbor_m_max)
                            .map(|(id, _)| *id)
                            .collect();
                    }
                    
                    neighbor_updates.push((*neighbor_id, curr_level, new_neighbors));
                }
            }
        }

        for (neighbor_id, curr_level, new_neighbors) in neighbor_updates {
            if let Some(neighbor_node) = self.nodes.get_mut(&neighbor_id) {
                if curr_level < neighbor_node.neighbors.len() {
                    neighbor_node.neighbors[curr_level] = new_neighbors;
                }
            }
        }

        self.nodes.insert(vector.id, node);
        Ok(())
    }

    fn search_layer(
        &self,
        query: &[f32],
        entry_point: u64,
        ef: usize,
        level: usize,
    ) -> Result<Vec<(u64, f64)>> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();

        let entry_node = self
            .nodes
            .get(&entry_point)
            .ok_or(ClawError::VectorNotFound(entry_point))?;

        let distance = self.metric.compute(query, &entry_node.vector) as f64;
        candidates.push((OrderedFloat(-distance), entry_point));
        results.push((OrderedFloat(distance), entry_point));
        visited.insert(entry_point);

        while !candidates.is_empty() {
            let (neg_dist, current_id) = candidates.pop().unwrap();
            let current_dist = neg_dist.0;

            let furthest_dist = results.peek().map(|(d, _)| d.0).unwrap_or(f64::MAX);

            if current_dist > furthest_dist {
                break;
            }

            let current_node = self
                .nodes
                .get(&current_id)
                .ok_or(ClawError::VectorNotFound(current_id))?;

            if level < current_node.neighbors.len() {
                for &neighbor_id in &current_node.neighbors[level] {
                    if !visited.contains(&neighbor_id) {
                        visited.insert(neighbor_id);

                        let neighbor_node = self
                            .nodes
                            .get(&neighbor_id)
                            .ok_or(ClawError::VectorNotFound(neighbor_id))?;

                        let neighbor_dist =
                            self.metric.compute(query, &neighbor_node.vector) as f64;

                        if neighbor_dist < furthest_dist || results.len() < ef {
                            candidates.push((OrderedFloat(-neighbor_dist), neighbor_id));
                            results.push((OrderedFloat(neighbor_dist), neighbor_id));

                            if results.len() > ef {
                                results.pop();
                            }
                        }
                    }
                }
            }
        }

        let mut result_vec: Vec<(u64, f64)> =
            results.into_iter().map(|(d, id)| (id, d.0)).collect();
        result_vec.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(result_vec)
    }

    fn select_neighbors_heuristic(
        &self,
        candidates: &[(u64, f64)],
        m: usize,
        _query: &[f32],
    ) -> Vec<(u64, f64)> {
        let mut selected: Vec<(u64, f64)> = Vec::with_capacity(m);
        let mut working_candidates: Vec<(u64, f64)> = candidates.to_vec();
        working_candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        while !working_candidates.is_empty() && selected.len() < m {
            let (candidate_id, candidate_dist) = working_candidates.remove(0);

            let candidate_node = match self.nodes.get(&candidate_id) {
                Some(n) => n,
                None => continue,
            };

            let mut good = true;
            for &(selected_id, _) in &selected {
                if let Some(selected_node) = self.nodes.get(&selected_id) {
                    let dist_to_selected =
                        self.metric.compute(&candidate_node.vector, &selected_node.vector) as f64;
                    if dist_to_selected < candidate_dist {
                        good = false;
                        break;
                    }
                }
            }

            if good {
                selected.push((candidate_id, candidate_dist));
            }
        }

        selected
    }

    fn random_level(&self) -> usize {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut level = 0;
        while rng.gen::<f64>() < 1.0 / self.config.ml && level < 16 {
            level += 1;
        }
        level
    }

    pub fn search(&self, query: &[f32], k: usize, ef: usize) -> Result<Vec<(u64, f64)>> {
        if !self.built {
            return Err(ClawError::IndexNotBuilt);
        }

        let entry_point = self.entry_point.ok_or(ClawError::IndexNotBuilt)?;

        let mut current_node = entry_point;

        for level in (1..=self.max_level).rev() {
            let neighbors = self.search_layer(query, current_node, 1, level)?;
            if !neighbors.is_empty() {
                current_node = neighbors[0].0;
            }
        }

        let neighbors = self.search_layer(query, current_node, ef.max(k), 0)?;

        Ok(neighbors.into_iter().take(k).collect())
    }

    pub fn is_built(&self) -> bool {
        self.built
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn dimension(&self) -> usize {
        self.dimension
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
    fn test_hnsw_creation() {
        let config = HnswConfig::default();
        let index = HnswIndex::new(2, DistanceMetric::Euclidean, config);

        assert_eq!(index.dimension(), 2);
        assert!(!index.is_built());
    }

    #[test]
    fn test_hnsw_build() {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(2, DistanceMetric::Euclidean, config);
        let vectors = create_test_vectors();

        index.build(&vectors).unwrap();

        assert!(index.is_built());
        assert_eq!(index.size(), 5);
    }

    #[test]
    fn test_hnsw_search() {
        let config = HnswConfig::default();
        let mut index = HnswIndex::new(2, DistanceMetric::Euclidean, config);
        let vectors = create_test_vectors();
        index.build(&vectors).unwrap();

        let query = vec![1.0, 1.0];
        let results = index.search(&query, 2, 10).unwrap();

        assert!(!results.is_empty());
    }
}

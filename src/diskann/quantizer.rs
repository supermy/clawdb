use crate::diskann::config::DiskAnnConfig;
use crate::error::Result;
use ndarray::{Array1, Array2};

pub struct ProductQuantizer {
    config: DiskAnnConfig,
    centroids: Vec<Array2<f32>>,
    dimension: usize,
}

impl ProductQuantizer {
    pub fn new(config: DiskAnnConfig, dimension: usize) -> Self {
        let subvector_dim = dimension / config.n_subvectors;
        let centroids = (0..config.n_subvectors)
            .map(|_| Array2::zeros((config.n_centroids, subvector_dim)))
            .collect();

        Self {
            config,
            centroids,
            dimension,
        }
    }

    pub fn train(&mut self, vectors: &[Vec<f32>]) -> Result<()> {
        let subvector_dim = self.dimension / self.config.n_subvectors;

        for i in 0..self.config.n_subvectors {
            let start = i * subvector_dim;
            let end = start + subvector_dim;

            let subvectors: Vec<Vec<f32>> =
                vectors.iter().map(|v| v[start..end].to_vec()).collect();

            self.centroids[i] = self.kmeans(&subvectors)?;
        }

        Ok(())
    }

    fn kmeans(&self, data: &[Vec<f32>]) -> Result<Array2<f32>> {
        let n = data.len();
        let k = self.config.n_centroids;

        if n == 0 || k == 0 {
            return Ok(Array2::zeros((k, data[0].len())));
        }

        let mut centroids = Array2::zeros((k, data[0].len()));

        for (i, mut centroid) in centroids.outer_iter_mut().enumerate() {
            if i < n {
                centroid.assign(&Array1::from_vec(data[i].clone()));
            }
        }

        Ok(centroids)
    }

    pub fn encode(&self, vector: &[f32]) -> Result<Vec<u8>> {
        let subvector_dim = self.dimension / self.config.n_subvectors;
        let mut codes = Vec::with_capacity(self.config.n_subvectors);

        for i in 0..self.config.n_subvectors {
            let start = i * subvector_dim;
            let end = start + subvector_dim;
            let subvector = &vector[start..end];

            let code = self.find_nearest_centroid(i, subvector)?;
            codes.push(code);
        }

        Ok(codes)
    }

    fn find_nearest_centroid(&self, subvector_idx: usize, subvector: &[f32]) -> Result<u8> {
        let centroids = &self.centroids[subvector_idx];

        let mut min_dist = f32::MAX;
        let mut min_idx = 0;

        for (i, centroid) in centroids.outer_iter().enumerate() {
            let dist = self.euclidean_distance(subvector, centroid.as_slice().unwrap());
            if dist < min_dist {
                min_dist = dist;
                min_idx = i;
            }
        }

        Ok(min_idx as u8)
    }

    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    pub fn decode(&self, codes: &[u8]) -> Result<Vec<f32>> {
        let mut vector = vec![0.0; self.dimension];
        let subvector_dim = self.dimension / self.config.n_subvectors;

        for (i, &code) in codes.iter().enumerate() {
            let start = i * subvector_dim;
            let end = start + subvector_dim;

            let centroid = self.centroids[i].row(code as usize);
            vector[start..end].copy_from_slice(centroid.as_slice().unwrap());
        }

        Ok(vector)
    }

    pub fn config(&self) -> &DiskAnnConfig {
        &self.config
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn centroids(&self) -> &[Array2<f32>] {
        &self.centroids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantizer_creation() {
        let config = DiskAnnConfig::default();
        let quantizer = ProductQuantizer::new(config, 128);

        assert_eq!(quantizer.dimension, 128);
    }

    #[test]
    fn test_encode_decode() {
        let config = DiskAnnConfig::default();
        let mut quantizer = ProductQuantizer::new(config.clone(), 128);

        let vectors: Vec<Vec<f32>> = (0..100).map(|i| vec![i as f32 / 100.0; 128]).collect();

        quantizer.train(&vectors).unwrap();

        let test_vector = vec![0.5; 128];
        let codes = quantizer.encode(&test_vector).unwrap();

        assert_eq!(codes.len(), config.n_subvectors);

        let decoded = quantizer.decode(&codes).unwrap();
        assert_eq!(decoded.len(), 128);
    }
}

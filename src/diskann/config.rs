use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskAnnConfig {
    pub n_subvectors: usize,
    pub n_bits: usize,
    pub n_centroids: usize,
}

impl Default for DiskAnnConfig {
    fn default() -> Self {
        Self {
            n_subvectors: 8,
            n_bits: 8,
            n_centroids: 256,
        }
    }
}

impl DiskAnnConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

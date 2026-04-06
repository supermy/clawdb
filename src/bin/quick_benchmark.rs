use clawdb::{
    distance::DistanceMetric,
    loader::SiftDataLoader,
    vector::Vector,
    HnswConfig, HnswIndex, VectorIndex,
};
use std::time::Instant;

fn main() {
    println!("=== ClawDB Quick Performance Test ===\n");

    let base_path = "data/sift_base.fvecs";
    let query_path = "data/sift_query.fvecs";

    println!("[1/4] Loading data...");
    let load_start = Instant::now();
    let (dim, base_vectors) = SiftDataLoader::load_fvecs(base_path).expect("Failed to load base");
    let (_, query_vectors) = SiftDataLoader::load_fvecs(query_path).expect("Failed to load queries");
    let load_time = load_start.elapsed();
    println!("  Loaded {} base vectors, {} queries in {:.2?}", 
             base_vectors.len(), query_vectors.len(), load_time);

    let vectors: Vec<Vector> = base_vectors
        .iter()
        .enumerate()
        .map(|(i, v)| Vector::new(i as u64, v.clone()))
        .collect();

    let test_size = vectors.len().min(50_000);
    let test_vectors: Vec<Vector> = vectors.iter().take(test_size).cloned().collect();

    println!("\n[2/4] Building IVF Index (parallel K-Means, {} vectors)...", test_size);
    let ivf_start = Instant::now();
    let mut ivf_index = VectorIndex::new(dim, DistanceMetric::Euclidean, 500);
    ivf_index.build(&test_vectors).expect("Failed to build IVF");
    let ivf_time = ivf_start.elapsed();
    let ivf_throughput = test_size as f64 / ivf_time.as_secs_f64();
    println!("  IVF build time: {:.2?} ({:.0} vectors/sec)", ivf_time, ivf_throughput);

    println!("\n[3/4] Building HNSW Index (optimized, {} vectors)...", test_size);
    let hnsw_start = Instant::now();
    let mut hnsw_config = HnswConfig::default();
    hnsw_config.max_elements = test_size;
    hnsw_config.ef_construction = 100;
    hnsw_config.m_max = 16;
    let mut hnsw_index = HnswIndex::new(dim, DistanceMetric::Euclidean, hnsw_config);
    hnsw_index.build(&test_vectors).expect("Failed to build HNSW");
    let hnsw_time = hnsw_start.elapsed();
    let hnsw_throughput = test_size as f64 / hnsw_time.as_secs_f64();
    println!("  HNSW build time: {:.2?} ({:.0} vectors/sec)", hnsw_time, hnsw_throughput);

    println!("\n[4/4] Running search benchmark...");
    let num_queries = query_vectors.len().min(100);
    
    let ivf_search_start = Instant::now();
    for query in query_vectors.iter().take(num_queries) {
        let _ = ivf_index.search(query, 10, 50).expect("IVF search failed");
    }
    let ivf_search_time = ivf_search_start.elapsed();
    let ivf_qps = num_queries as f64 / ivf_search_time.as_secs_f64();
    println!("  IVF search: {:.2?} for {} queries ({:.0} QPS)", 
             ivf_search_time, num_queries, ivf_qps);

    let hnsw_search_start = Instant::now();
    for query in query_vectors.iter().take(num_queries) {
        let _ = hnsw_index.search(query, 10, 100).expect("HNSW search failed");
    }
    let hnsw_search_time = hnsw_search_start.elapsed();
    let hnsw_qps = num_queries as f64 / hnsw_search_time.as_secs_f64();
    println!("  HNSW search: {:.2?} for {} queries ({:.0} QPS)", 
             hnsw_search_time, num_queries, hnsw_qps);

    println!("\n=== Performance Summary ===");
    println!("| Metric | Value |");
    println!("|--------|-------|");
    println!("| Data Load | {:.0} vectors/sec |", base_vectors.len() as f64 / load_time.as_secs_f64());
    println!("| IVF Build | {:.0} vectors/sec |", ivf_throughput);
    println!("| HNSW Build | {:.0} vectors/sec |", hnsw_throughput);
    println!("| IVF Search | {:.0} QPS |", ivf_qps);
    println!("| HNSW Search | {:.0} QPS |", hnsw_qps);
}

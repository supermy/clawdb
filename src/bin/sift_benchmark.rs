use clawdb::{
    distance::DistanceMetric,
    loader::SiftDataLoader,
    vector::Vector,
    HnswConfig, HnswIndex, VectorIndex, VectorStorage,
};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};

struct BenchmarkResult {
    name: String,
    duration: Duration,
    throughput: Option<f64>,
    recall: Option<f64>,
    latency_p50: Option<Duration>,
    latency_p99: Option<Duration>,
}

impl BenchmarkResult {
    fn print(&self) {
        println!("=== {} ===", self.name);
        println!("  Duration: {:.2?}", self.duration);
        if let Some(t) = self.throughput {
            println!("  Throughput: {:.2} ops/sec", t);
        }
        if let Some(r) = self.recall {
            println!("  Recall@10: {:.2}%", r * 100.0);
        }
        if let Some(l) = self.latency_p50 {
            println!("  Latency P50: {:.2?}", l);
        }
        if let Some(l) = self.latency_p99 {
            println!("  Latency P99: {:.2?}", l);
        }
        println!();
    }
}

fn calculate_recall(results: &[(u64, f64)], ground_truth: &[i32], k: usize) -> f64 {
    let result_set: HashSet<u64> = results.iter().take(k).map(|(id, _)| *id).collect();
    let gt_set: HashSet<u64> = ground_truth
        .iter()
        .take(k)
        .map(|&id| id as u64)
        .collect();

    let intersection = result_set.intersection(&gt_set).count();
    intersection as f64 / k as f64
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let data_dir = args.get(1).cloned().unwrap_or_else(|| "data".to_string());
    let data_path = Path::new(&data_dir);

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         ClawDB SIFT1M Performance Benchmark                ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let base_path = data_path.join("sift_base.fvecs");
    let query_path = data_path.join("sift_query.fvecs");
    let gt_path = data_path.join("sift_groundtruth.ivecs");

    if !base_path.exists() || !query_path.exists() || !gt_path.exists() {
        eprintln!("Error: SIFT1M dataset not found!");
        eprintln!("Please run: ./scripts/download_sift1m.sh");
        eprintln!("Or place the following files in the {} directory:", data_dir);
        eprintln!("  - sift_base.fvecs");
        eprintln!("  - sift_query.fvecs");
        eprintln!("  - sift_groundtruth.ivecs");
        std::process::exit(1);
    }

    let mut results: Vec<BenchmarkResult> = Vec::new();

    println!("[1/6] Loading SIFT1M dataset...");
    let load_start = Instant::now();

    let (dim, base_vectors) = SiftDataLoader::load_fvecs(&base_path)
        .expect("Failed to load base vectors");
    println!("  Base vectors: {} (dim: {})", base_vectors.len(), dim);

    let (_, query_vectors) = SiftDataLoader::load_fvecs(&query_path)
        .expect("Failed to load query vectors");
    println!("  Query vectors: {}", query_vectors.len());

    let ground_truth = SiftDataLoader::load_ivecs(&gt_path)
        .expect("Failed to load ground truth");
    println!("  Ground truth entries: {}", ground_truth.len());

    let load_duration = load_start.elapsed();
    println!("  Load time: {:.2?}", load_duration);

    results.push(BenchmarkResult {
        name: "Data Loading".to_string(),
        duration: load_duration,
        throughput: Some(base_vectors.len() as f64 / load_duration.as_secs_f64()),
        recall: None,
        latency_p50: None,
        latency_p99: None,
    });

    let vectors: Vec<Vector> = base_vectors
        .iter()
        .enumerate()
        .map(|(i, v)| Vector::new(i as u64, v.clone()))
        .collect();

    println!();
    println!("[2/6] Building IVF Index (nlist=1000)...");
    let ivf_build_start = Instant::now();
    let mut ivf_index = VectorIndex::new(dim, DistanceMetric::Euclidean, 1000);
    ivf_index.build(&vectors).expect("Failed to build IVF index");
    let ivf_build_duration = ivf_build_start.elapsed();

    results.push(BenchmarkResult {
        name: "IVF Index Build (nlist=1000)".to_string(),
        duration: ivf_build_duration,
        throughput: Some(vectors.len() as f64 / ivf_build_duration.as_secs_f64()),
        recall: None,
        latency_p50: None,
        latency_p99: None,
    });

    println!();
    println!("[3/6] Building HNSW Index (using subset for faster testing)...");
    let hnsw_subset_size = vectors.len().min(10_000);
    let hnsw_vectors: Vec<Vector> = vectors.iter().take(hnsw_subset_size).cloned().collect();
    let hnsw_build_start = Instant::now();
    let mut hnsw_config = HnswConfig::default();
    hnsw_config.max_elements = hnsw_subset_size;
    hnsw_config.ef_construction = 100;
    hnsw_config.m_max = 16;
    let mut hnsw_index = HnswIndex::new(dim, DistanceMetric::Euclidean, hnsw_config);
    hnsw_index.build(&hnsw_vectors).expect("Failed to build HNSW index");
    let hnsw_build_duration = hnsw_build_start.elapsed();

    results.push(BenchmarkResult {
        name: format!("HNSW Index Build ({} vectors)", hnsw_subset_size),
        duration: hnsw_build_duration,
        throughput: Some(hnsw_subset_size as f64 / hnsw_build_duration.as_secs_f64()),
        recall: None,
        latency_p50: None,
        latency_p99: None,
    });

    println!();
    println!("[4/6] Running IVF Search Benchmark...");
    let k = 10;
    let nprobe = 100;
    let mut ivf_latencies: Vec<Duration> = Vec::new();
    let mut ivf_recalls: Vec<f64> = Vec::new();

    let ivf_search_start = Instant::now();
    for (i, query) in query_vectors.iter().enumerate() {
        let query_start = Instant::now();
        let candidates = ivf_index.search(query, k, nprobe).expect("IVF search failed");

        let candidate_set: HashSet<u64> = candidates.iter().copied().collect();
        let gt_set: HashSet<u64> = ground_truth[i]
            .iter()
            .take(k)
            .map(|&id| id as u64)
            .collect();

        let recall = candidate_set.intersection(&gt_set).count() as f64 / k as f64;
        ivf_recalls.push(recall);
        ivf_latencies.push(query_start.elapsed());
    }
    let ivf_search_duration = ivf_search_start.elapsed();

    ivf_latencies.sort();
    let ivf_p50 = ivf_latencies[ivf_latencies.len() / 2];
    let ivf_p99 = ivf_latencies[(ivf_latencies.len() as f64 * 0.99) as usize];
    let ivf_avg_recall: f64 = ivf_recalls.iter().sum::<f64>() / ivf_recalls.len() as f64;

    results.push(BenchmarkResult {
        name: "IVF Search (nprobe=100)".to_string(),
        duration: ivf_search_duration,
        throughput: Some(query_vectors.len() as f64 / ivf_search_duration.as_secs_f64()),
        recall: Some(ivf_avg_recall),
        latency_p50: Some(ivf_p50),
        latency_p99: Some(ivf_p99),
    });

    println!();
    println!("[5/6] Running HNSW Search Benchmark...");
    let ef = 200;
    let mut hnsw_latencies: Vec<Duration> = Vec::new();
    let mut hnsw_recalls: Vec<f64> = Vec::new();

    let hnsw_search_start = Instant::now();
    for (i, query) in query_vectors.iter().enumerate() {
        let query_start = Instant::now();
        let search_results = hnsw_index.search(query, k, ef).expect("HNSW search failed");

        let recall = calculate_recall(&search_results, &ground_truth[i], k);
        hnsw_recalls.push(recall);
        hnsw_latencies.push(query_start.elapsed());
    }
    let hnsw_search_duration = hnsw_search_start.elapsed();

    hnsw_latencies.sort();
    let hnsw_p50 = hnsw_latencies[hnsw_latencies.len() / 2];
    let hnsw_p99 = hnsw_latencies[(hnsw_latencies.len() as f64 * 0.99) as usize];
    let hnsw_avg_recall: f64 = hnsw_recalls.iter().sum::<f64>() / hnsw_recalls.len() as f64;

    results.push(BenchmarkResult {
        name: "HNSW Search (ef=200)".to_string(),
        duration: hnsw_search_duration,
        throughput: Some(query_vectors.len() as f64 / hnsw_search_duration.as_secs_f64()),
        recall: Some(hnsw_avg_recall),
        latency_p50: Some(hnsw_p50),
        latency_p99: Some(hnsw_p99),
    });

    println!();
    println!("[6/6] Running VectorStorage Benchmark (with RocksDB)...");
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let mut storage = VectorStorage::open(temp_dir.path(), dim, DistanceMetric::Euclidean)
        .expect("Failed to open storage");

    let storage_insert_start = Instant::now();
    for v in &vectors {
        storage.insert(v.clone()).expect("Failed to insert vector");
    }
    let storage_insert_duration = storage_insert_start.elapsed();

    results.push(BenchmarkResult {
        name: "VectorStorage Insert (RocksDB)".to_string(),
        duration: storage_insert_duration,
        throughput: Some(vectors.len() as f64 / storage_insert_duration.as_secs_f64()),
        recall: None,
        latency_p50: None,
        latency_p99: None,
    });

    storage.build_index(1000).expect("Failed to build index");

    let mut storage_latencies: Vec<Duration> = Vec::new();
    let storage_search_start = Instant::now();
    for query in query_vectors.iter().take(1000) {
        let query_start = Instant::now();
        let _ = storage.search(query, k, nprobe).expect("Storage search failed");
        storage_latencies.push(query_start.elapsed());
    }
    let storage_search_duration = storage_search_start.elapsed();

    storage_latencies.sort();
    let storage_p50 = storage_latencies[storage_latencies.len() / 2];
    let storage_p99 = storage_latencies[(storage_latencies.len() as f64 * 0.99) as usize];

    results.push(BenchmarkResult {
        name: "VectorStorage Search (1000 queries)".to_string(),
        duration: storage_search_duration,
        throughput: Some(1000.0 / storage_search_duration.as_secs_f64()),
        recall: None,
        latency_p50: Some(storage_p50),
        latency_p99: Some(storage_p99),
    });

    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    BENCHMARK RESULTS                       ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    for result in &results {
        result.print();
    }

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                      SUMMARY TABLE                         ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("| {:<35} | {:>12} | {:>10} | {:>10} |", "Operation", "Throughput", "Recall@10", "P99 Latency");
    println!("|{}|{}|{}|{}|", "-".repeat(37), "-".repeat(14), "-".repeat(12), "-".repeat(12));
    for result in &results {
        let throughput = result
            .throughput
            .map(|t| format!("{:.0}/s", t))
            .unwrap_or_else(|| "-".to_string());
        let recall = result
            .recall
            .map(|r| format!("{:.2}%", r * 100.0))
            .unwrap_or_else(|| "-".to_string());
        let latency = result
            .latency_p99
            .map(|l| format!("{:.2?}", l))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "| {:<35} | {:>12} | {:>10} | {:>10} |",
            result.name, throughput, recall, latency
        );
    }

    let report_path = data_path.join("benchmark_report.md");
    let mut report = File::create(&report_path).expect("Failed to create report");
    writeln!(report, "# ClawDB SIFT1M Performance Benchmark Report").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## Test Environment").unwrap();
    writeln!(report, "- Dataset: SIFT1M (1,000,000 vectors, 128 dimensions)").unwrap();
    writeln!(report, "- Query Count: 10,000").unwrap();
    writeln!(report, "- K (top-k): 10").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "## Results").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| Operation | Duration | Throughput | Recall@10 | P50 Latency | P99 Latency |").unwrap();
    writeln!(report, "|-----------|----------|------------|-----------|-------------|-------------|").unwrap();
    for result in &results {
        let throughput = result
            .throughput
            .map(|t| format!("{:.0}/s", t))
            .unwrap_or_else(|| "-".to_string());
        let recall = result
            .recall
            .map(|r| format!("{:.2}%", r * 100.0))
            .unwrap_or_else(|| "-".to_string());
        let p50 = result
            .latency_p50
            .map(|l| format!("{:.2?}", l))
            .unwrap_or_else(|| "-".to_string());
        let p99 = result
            .latency_p99
            .map(|l| format!("{:.2?}", l))
            .unwrap_or_else(|| "-".to_string());
        writeln!(
            report,
            "| {} | {:.2?} | {} | {} | {} | {} |",
            result.name, result.duration, throughput, recall, p50, p99
        )
        .unwrap();
    }

    println!();
    println!("Report saved to: {}", report_path.display());
}

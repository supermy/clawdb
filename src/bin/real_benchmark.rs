use clawdb::{
    distance::DistanceMetric,
    loader::SiftDataLoader,
    vector::Vector,
    CacheConfig, VectorCache,
    HnswConfig, HnswIndex, VectorIndex, VectorStorage,
};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

struct PerformanceReport {
    total_duration: Duration,
    data_loading: Duration,
    ivf_build: Duration,
    hnsw_build: Duration,
    cache_operations: Duration,
    storage_write: Duration,
    storage_read: Duration,
    storage_search: Duration,
    ivf_recall: f64,
    hnsw_recall: f64,
    ivf_qps: f64,
    hnsw_qps: f64,
    storage_qps: f64,
    cache_hit_rate: f64,
}

impl PerformanceReport {
    fn save_to_file(&self, path: &PathBuf) {
        let mut file = File::create(path).expect("Failed to create report file");
        
        writeln!(file, "# ClawDB 真实性能测试报告").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "## 测试环境").unwrap();
        writeln!(file, "- 操作系统: {}", std::env::consts::OS).unwrap();
        writeln!(file, "- 测试时间: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).unwrap();
        writeln!(file).unwrap();
        
        writeln!(file, "## 总体性能").unwrap();
        writeln!(file, "- 总测试时间: {:.2?}", self.total_duration).unwrap();
        writeln!(file).unwrap();
        
        writeln!(file, "## 详细性能指标").unwrap();
        writeln!(file).unwrap();
        
        writeln!(file, "### 1. 数据加载").unwrap();
        writeln!(file, "- 加载时间: {:.2?}", self.data_loading).unwrap();
        writeln!(file).unwrap();
        
        writeln!(file, "### 2. IVF 索引（并行 K-Means 优化）").unwrap();
        writeln!(file, "- 构建时间: {:.2?}", self.ivf_build).unwrap();
        writeln!(file, "- 搜索 QPS: {:.0}", self.ivf_qps).unwrap();
        writeln!(file, "- 召回率: {:.2}%", self.ivf_recall * 100.0).unwrap();
        writeln!(file).unwrap();
        
        writeln!(file, "### 3. HNSW 索引（图索引优化）").unwrap();
        writeln!(file, "- 构建时间: {:.2?}", self.hnsw_build).unwrap();
        writeln!(file, "- 搜索 QPS: {:.0}", self.hnsw_qps).unwrap();
        writeln!(file, "- 召回率: {:.2}%", self.hnsw_recall * 100.0).unwrap();
        writeln!(file).unwrap();
        
        writeln!(file, "### 4. 缓存系统（LRU 缓存优化）").unwrap();
        writeln!(file, "- 操作时间: {:.2?}", self.cache_operations).unwrap();
        writeln!(file, "- 命中率: {:.2}%", self.cache_hit_rate * 100.0).unwrap();
        writeln!(file).unwrap();
        
        writeln!(file, "### 5. RocksDB 存储").unwrap();
        writeln!(file, "- 写入时间: {:.2?}", self.storage_write).unwrap();
        writeln!(file, "- 读取时间: {:.2?}", self.storage_read).unwrap();
        writeln!(file, "- 搜索时间: {:.2?}", self.storage_search).unwrap();
        writeln!(file, "- 搜索 QPS: {:.0}", self.storage_qps).unwrap();
        writeln!(file).unwrap();
        
        writeln!(file, "## 性能总结表").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "| 组件 | 操作 | 时间 | QPS/吞吐量 |").unwrap();
        writeln!(file, "|------|------|------|-----------|").unwrap();
        writeln!(file, "| 数据加载 | 加载 | {:.2?} | - |", self.data_loading).unwrap();
        writeln!(file, "| IVF 索引 | 构建 | {:.2?} | - |", self.ivf_build).unwrap();
        writeln!(file, "| IVF 索引 | 搜索 | - | {:.0} QPS |", self.ivf_qps).unwrap();
        writeln!(file, "| HNSW 索引 | 构建 | {:.2?} | - |", self.hnsw_build).unwrap();
        writeln!(file, "| HNSW 索引 | 搜索 | - | {:.0} QPS |", self.hnsw_qps).unwrap();
        writeln!(file, "| 缓存 | 操作 | {:.2?} | - |", self.cache_operations).unwrap();
        writeln!(file, "| RocksDB | 写入 | {:.2?} | - |", self.storage_write).unwrap();
        writeln!(file, "| RocksDB | 读取 | {:.2?} | - |", self.storage_read).unwrap();
        writeln!(file, "| RocksDB | 搜索 | {:.2?} | {:.0} QPS |", self.storage_search, self.storage_qps).unwrap();
    }
}

fn main() {
    let total_start = Instant::now();
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          ClawDB 真实性能测试 - 端到端测试                  ║");
    println!("║          数据 -> 优化 -> RocksDB 存储                      ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    let base_path = PathBuf::from("data/sift_base.fvecs");
    let query_path = PathBuf::from("data/sift_query.fvecs");
    let gt_path = PathBuf::from("data/sift_groundtruth.ivecs");

    if !base_path.exists() || !query_path.exists() || !gt_path.exists() {
        eprintln!("错误: 数据集不存在！");
        eprintln!("请运行: make generate-data");
        std::process::exit(1);
    }

    println!("═══════════════════════════════════════════════════════════");
    println!("第一阶段: 数据加载");
    println!("═══════════════════════════════════════════════════════════");
    
    let data_start = Instant::now();
    println!("\n[1/3] 加载基础向量...");
    let (dim, base_vectors) = SiftDataLoader::load_fvecs(&base_path)
        .expect("Failed to load base vectors");
    println!("  ✓ 加载 {} 个向量 (维度: {})", base_vectors.len(), dim);
    
    println!("\n[2/3] 加载查询向量...");
    let (_, query_vectors) = SiftDataLoader::load_fvecs(&query_path)
        .expect("Failed to load query vectors");
    println!("  ✓ 加载 {} 个查询向量", query_vectors.len());
    
    println!("\n[3/3] 加载 Ground Truth...");
    let ground_truth = SiftDataLoader::load_ivecs(&gt_path)
        .expect("Failed to load ground truth");
    println!("  ✓ 加载 {} 个 Ground Truth 记录", ground_truth.len());
    
    let data_loading = data_start.elapsed();
    println!("\n数据加载完成: {:.2?}", data_loading);
    println!("  吞吐量: {:.0} vectors/sec", base_vectors.len() as f64 / data_loading.as_secs_f64());

    let vectors: Vec<Vector> = base_vectors
        .iter()
        .enumerate()
        .map(|(i, v)| Vector::new(i as u64, v.clone()))
        .collect();

    let test_size = vectors.len().min(30_000);
    let test_vectors: Vec<Vector> = vectors.iter().take(test_size).cloned().collect();
    println!("\n使用 {} 个向量进行测试", test_size);

    println!("\n═══════════════════════════════════════════════════════════");
    println!("第二阶段: 优化测试");
    println!("═══════════════════════════════════════════════════════════");

    println!("\n[优化 1] IVF 索引 - 并行 K-Means");
    println!("─────────────────────────────────────");
    let ivf_start = Instant::now();
    let nlist = (test_size as f64).sqrt() as usize;
    println!("  构建索引 (nlist={})...", nlist);
    let mut ivf_index = VectorIndex::new(dim, DistanceMetric::Euclidean, nlist);
    ivf_index.build(&test_vectors).expect("Failed to build IVF");
    let ivf_build = ivf_start.elapsed();
    println!("  ✓ 构建完成: {:.2?}", ivf_build);
    println!("  ✓ 吞吐量: {:.0} vectors/sec", test_size as f64 / ivf_build.as_secs_f64());

    println!("\n  测试搜索性能...");
    let num_queries = query_vectors.len().min(100);
    let ivf_search_start = Instant::now();
    let mut ivf_recalls = Vec::new();
    for (i, query) in query_vectors.iter().take(num_queries).enumerate() {
        let candidates = ivf_index.search(query, 10, nlist / 10).expect("IVF search failed");
        let candidate_set: HashSet<u64> = candidates.iter().copied().collect();
        let gt_set: HashSet<u64> = ground_truth[i].iter().take(10).map(|&id| id as u64).collect();
        let recall = candidate_set.intersection(&gt_set).count() as f64 / 10.0;
        ivf_recalls.push(recall);
    }
    let ivf_search = ivf_search_start.elapsed();
    let ivf_qps = num_queries as f64 / ivf_search.as_secs_f64();
    let ivf_recall = ivf_recalls.iter().sum::<f64>() / ivf_recalls.len() as f64;
    println!("  ✓ 搜索 {} 个查询: {:.2?}", num_queries, ivf_search);
    println!("  ✓ QPS: {:.0}", ivf_qps);
    println!("  ✓ 召回率: {:.2}%", ivf_recall * 100.0);

    println!("\n[优化 2] HNSW 索引 - 图索引优化");
    println!("─────────────────────────────────────");
    let hnsw_test_size = test_size.min(10_000);
    let hnsw_vectors: Vec<Vector> = test_vectors.iter().take(hnsw_test_size).cloned().collect();
    
    let hnsw_start = Instant::now();
    println!("  构建索引 ({} 向量)...", hnsw_test_size);
    let mut hnsw_config = HnswConfig::default();
    hnsw_config.max_elements = hnsw_test_size;
    hnsw_config.ef_construction = 100;
    let mut hnsw_index = HnswIndex::new(dim, DistanceMetric::Euclidean, hnsw_config);
    hnsw_index.build(&hnsw_vectors).expect("Failed to build HNSW");
    let hnsw_build = hnsw_start.elapsed();
    println!("  ✓ 构建完成: {:.2?}", hnsw_build);
    println!("  ✓ 吞吐量: {:.0} vectors/sec", hnsw_test_size as f64 / hnsw_build.as_secs_f64());

    println!("\n  测试搜索性能...");
    let hnsw_search_start = Instant::now();
    let mut hnsw_recalls = Vec::new();
    for (i, query) in query_vectors.iter().take(num_queries).enumerate() {
        let results = hnsw_index.search(query, 10, 100).expect("HNSW search failed");
        let result_set: HashSet<u64> = results.iter().take(10).map(|(id, _)| *id).collect();
        let gt_set: HashSet<u64> = ground_truth[i].iter().take(10).map(|&id| id as u64).collect();
        let recall = result_set.intersection(&gt_set).count() as f64 / 10.0;
        hnsw_recalls.push(recall);
    }
    let hnsw_search = hnsw_search_start.elapsed();
    let hnsw_qps = num_queries as f64 / hnsw_search.as_secs_f64();
    let hnsw_recall = hnsw_recalls.iter().sum::<f64>() / hnsw_recalls.len() as f64;
    println!("  ✓ 搜索 {} 个查询: {:.2?}", num_queries, hnsw_search);
    println!("  ✓ QPS: {:.0}", hnsw_qps);
    println!("  ✓ 召回率: {:.2}%", hnsw_recall * 100.0);

    println!("\n[优化 3] LRU 缓存 - 缓存优化");
    println!("─────────────────────────────────────");
    let cache_start = Instant::now();
    let cache = VectorCache::new(CacheConfig {
        max_size: 5_000,
        ttl: Some(Duration::from_secs(3600)),
    });

    println!("  测试缓存写入...");
    for (i, vec) in test_vectors.iter().take(5_000).enumerate() {
        cache.put(vec.id, vec.data.clone());
    }

    println!("  测试缓存命中...");
    let mut hits = 0;
    for vec in test_vectors.iter().take(3_000) {
        if cache.get(vec.id).is_some() {
            hits += 1;
        }
    }

    println!("  测试缓存未命中...");
    for i in 10_000..13_000 {
        let _ = cache.get(i as u64);
    }

    let cache_operations = cache_start.elapsed();
    let cache_hit_rate = hits as f64 / 3_000.0;
    println!("  ✓ 缓存操作完成: {:.2?}", cache_operations);
    println!("  ✓ 命中率: {:.2}%", cache_hit_rate * 100.0);

    println!("\n═══════════════════════════════════════════════════════════");
    println!("第三阶段: RocksDB 存储测试");
    println!("═══════════════════════════════════════════════════════════");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    println!("\n初始化 RocksDB 存储...");
    let mut storage = VectorStorage::open(temp_dir.path(), dim, DistanceMetric::Euclidean)
        .expect("Failed to open storage");

    let storage_vectors: Vec<Vector> = test_vectors.iter().take(10_000).cloned().collect();

    println!("\n[RocksDB] 写入测试");
    println!("─────────────────────────────────────");
    let write_start = Instant::now();
    for v in &storage_vectors {
        storage.insert(v.clone()).expect("Failed to insert");
    }
    storage.flush().expect("Failed to flush");
    let storage_write = write_start.elapsed();
    let write_throughput = storage_vectors.len() as f64 / storage_write.as_secs_f64();
    println!("  ✓ 写入 {} 个向量: {:.2?}", storage_vectors.len(), storage_write);
    println!("  ✓ 吞吐量: {:.0} vectors/sec", write_throughput);

    println!("\n[RocksDB] 读取测试");
    println!("─────────────────────────────────────");
    let read_start = Instant::now();
    for i in 0..1_000 {
        let _ = storage.get(i).expect("Failed to get");
    }
    let storage_read = read_start.elapsed();
    let read_throughput = 1_000.0 / storage_read.as_secs_f64();
    println!("  ✓ 读取 1,000 个向量: {:.2?}", storage_read);
    println!("  ✓ 吞吐量: {:.0} ops/sec", read_throughput);

    println!("\n[RocksDB] 索引构建");
    println!("─────────────────────────────────────");
    let index_start = Instant::now();
    storage.build_index(100).expect("Failed to build index");
    let index_time = index_start.elapsed();
    println!("  ✓ 索引构建完成: {:.2?}", index_time);

    println!("\n[RocksDB] 搜索测试");
    println!("─────────────────────────────────────");
    let storage_search_start = Instant::now();
    for query in query_vectors.iter().take(num_queries) {
        let _ = storage.search(query, 10, 10).expect("Search failed");
    }
    let storage_search = storage_search_start.elapsed();
    let storage_qps = num_queries as f64 / storage_search.as_secs_f64();
    println!("  ✓ 搜索 {} 个查询: {:.2?}", num_queries, storage_search);
    println!("  ✓ QPS: {:.0}", storage_qps);

    let total_duration = total_start.elapsed();

    println!("\n═══════════════════════════════════════════════════════════");
    println!("性能测试总结");
    println!("═══════════════════════════════════════════════════════════");

    println!("\n## 测试配置");
    println!("- 向量数量: {}", test_size);
    println!("- 查询数量: {}", num_queries);
    println!("- 向量维度: {}", dim);

    println!("\n## 性能指标");
    println!("| 组件 | 操作 | 时间 | 性能 |");
    println!("|------|------|------|------|");
    println!("| 数据加载 | 加载 | {:.2?} | {:.0} v/s |", data_loading, base_vectors.len() as f64 / data_loading.as_secs_f64());
    println!("| IVF 索引 | 构建 | {:.2?} | {:.0} v/s |", ivf_build, test_size as f64 / ivf_build.as_secs_f64());
    println!("| IVF 索引 | 搜索 | - | {:.0} QPS |", ivf_qps);
    println!("| HNSW 索引 | 构建 | {:.2?} | {:.0} v/s |", hnsw_build, hnsw_test_size as f64 / hnsw_build.as_secs_f64());
    println!("| HNSW 索引 | 搜索 | - | {:.0} QPS |", hnsw_qps);
    println!("| 缓存 | 操作 | {:.2?} | {:.0}% 命中 |", cache_operations, cache_hit_rate * 100.0);
    println!("| RocksDB | 写入 | {:.2?} | {:.0} v/s |", storage_write, write_throughput);
    println!("| RocksDB | 读取 | {:.2?} | {:.0} ops/s |", storage_read, read_throughput);
    println!("| RocksDB | 搜索 | {:.2?} | {:.0} QPS |", storage_search, storage_qps);

    println!("\n## 优化效果验证");
    println!("✓ 并行 K-Means: IVF 构建吞吐量 {:.0} vectors/sec", test_size as f64 / ivf_build.as_secs_f64());
    println!("✓ HNSW 图索引: 召回率 {:.2}%, QPS {:.0}", hnsw_recall * 100.0, hnsw_qps);
    println!("✓ LRU 缓存: 命中率 {:.2}%", cache_hit_rate * 100.0);
    println!("✓ RocksDB 存储: 写入 {:.0} v/s, 读取 {:.0} ops/s", write_throughput, read_throughput);

    println!("\n总测试时间: {:.2?}", total_duration);

    let report = PerformanceReport {
        total_duration,
        data_loading,
        ivf_build,
        hnsw_build,
        cache_operations,
        storage_write,
        storage_read,
        storage_search,
        ivf_recall,
        hnsw_recall,
        ivf_qps,
        hnsw_qps,
        storage_qps,
        cache_hit_rate,
    };

    let report_path = PathBuf::from("data/real_performance_report.md");
    report.save_to_file(&report_path);
    println!("\n报告已保存到: {}", report_path.display());

    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║              真实性能测试完成！                            ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}

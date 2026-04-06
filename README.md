# ClawDB - 高性能向量数据库

<div align="center">

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

**基于 RocksDB 的高性能向量数据库，支持近似最近邻搜索（ANN）**

[特性](#特性) • [快速开始](#快速开始) • [性能](#性能) • [API 文档](#api-文档) • [示例](#示例)

</div>

---

## 📋 目录

- [特性](#特性)
- [架构](#架构)
- [安装](#安装)
- [快速开始](#快速开始)
- [使用指南](#使用指南)
- [性能](#性能)
- [API 文档](#api-文档)
- [示例](#示例)
- [数据集](#数据集)
- [开发指南](#开发指南)
- [常见问题](#常见问题)
- [贡献指南](#贡献指南)
- [许可证](#许可证)

## ✨ 特性

### 核心功能

- 🚀 **高性能存储**: 基于 RocksDB 的持久化存储，支持大规模向量数据
- 🎯 **多种距离度量**: 支持欧氏距离、余弦相似度、点积、曼哈顿距离
- 📊 **IVF 索引**: 使用倒排文件索引加速向量搜索，支持并行 K-Means
- 🕸️ **HNSW 索引**: 分层导航小世界图索引，更高召回率
- 🔄 **并行计算**: 使用 Rayon 进行并行距离计算和索引构建
- 💾 **数据加载器**: 支持 SIFT1M 数据集（fvecs/bvecs 格式）
- 🧪 **测试驱动**: 完整的单元测试和性能基准测试

### 高级特性

- **DiskANN 支持**: Product Quantization 压缩存储，内存效率提升 6-10x
- **向量缓存**: LRU 缓存策略，多级缓存支持
- **异步 I/O**: 支持 Tokio 异步运行时和 io_uring
- **I/O 限流**: 优先级控制，区分查询和后台任务
- **RocksDB 插件**: CompactionFilter、SliceTransform、MergeOperator

### 技术亮点

- **RocksDB 优化**: 使用 Column Family 分离数据，LZ4 压缩
- **SIMD 加速**: 可选的 SIMD 指令集优化
- **批量操作**: 支持批量插入、删除，提高吞吐量
- **内存高效**: 使用 bincode 进行高效的序列化
- **线程安全**: 使用 Arc 实现线程安全的共享存储

## 🏗️ 架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         应用层 (Application)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   CLI 工具   │  │   REST API   │  │  SDK 客户端  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                         服务层 (Service)                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │VectorStorage │  │ VectorIndex  │  │ DataLoader   │          │
│  │              │  │  (IVF/HNSW)  │  │              │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                         核心层 (Core)                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │    Vector    │  │   Distance   │  │    Cache     │          │
│  │              │  │   Metrics    │  │   (LRU)      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   DiskANN    │  │  Async I/O   │  │  I/O Limiter │          │
│  │   (PQ)       │  │ (io_uring)   │  │              │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                        存储层 (Storage)                           │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    RocksDB Engine                        │  │
│  │  ┌──────────┬──────────┬──────────┬──────────┐         │  │
│  │  │   Data   │  Index   │ Metadata │  Cache   │         │  │
│  │  │   CF     │   CF     │    CF    │   CF     │         │  │
│  │  └──────────┴──────────┴──────────┴──────────┘         │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 模块说明

| 模块 | 说明 |
|------|------|
| `vector` | 向量数据结构和操作 |
| `distance` | 距离计算（欧氏、余弦、点积、曼哈顿） |
| `index` | IVF 索引实现（并行 K-Means） |
| `hnsw` | HNSW 图索引实现 |
| `storage` | RocksDB 存储层 |
| `loader` | 数据集加载器（fvecs/bvecs/ivecs） |
| `cache` | LRU 缓存和多级缓存 |
| `diskann` | DiskANN PQ 压缩存储 |
| `async_io` | 异步 I/O（Tokio/io_uring） |
| `io_limiter` | I/O 优先级控制 |
| `plugins` | RocksDB 插件 |

## 📦 安装

### 前置要求

- Rust 1.70 或更高版本
- RocksDB 依赖（自动安装）

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/clawdb.git
cd clawdb

# 构建项目
make build

# 或使用 cargo
cargo build --release
```

### 使用 Makefile

项目提供了便捷的 Makefile：

```bash
make help          # 查看所有可用命令
make build         # 构建发布版本
make test          # 运行测试
make run           # 运行示例程序
make bench         # 运行性能基准测试
make ci            # 运行所有检查（格式化、检查、测试）
make benchmark     # 运行 SIFT1M 性能测试
```

## 🚀 快速开始

### 基本使用

```rust
use clawdb::{VectorStorage, Vector, DistanceMetric};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 打开数据库
    let mut storage = VectorStorage::open(
        "./my_db",
        128,  // 向量维度
        DistanceMetric::Euclidean
    )?;

    // 2. 创建并插入向量
    let vector = Vector::new(1, vec![1.0, 2.0, 3.0, /* ... */]);
    storage.insert(vector)?;

    // 3. 批量插入
    let vectors = vec![
        Vector::new(1, vec![/* 128维向量 */]),
        Vector::new(2, vec![/* 128维向量 */]),
    ];
    storage.insert_batch(vectors)?;

    // 4. 构建索引
    storage.build_index(100)?;  // 100 个聚类中心

    // 5. 搜索最近邻
    let query = vec![0.5; 128];
    let results = storage.search(&query, 10, 10)?;  // k=10, nprobe=10

    // 6. 输出结果
    for (id, distance) in results {
        println!("ID: {}, Distance: {:.4}", id, distance);
    }

    Ok(())
}
```

### 使用 HNSW 索引

```rust
use clawdb::{HnswIndex, HnswConfig, Vector, DistanceMetric};

// 创建 HNSW 索引
let config = HnswConfig {
    max_elements: 100000,
    ef_construction: 200,
    m_max: 16,
    m_max_0: 32,
    ml: 1.0 / (16.0_f64).ln(),
};

let mut index = HnswIndex::new(128, DistanceMetric::Euclidean, config);
index.build(&vectors)?;

// 搜索
let results = index.search(&query, 10, 100)?;
```

### 使用缓存

```rust
use clawdb::{VectorCache, CacheConfig, MultiLevelCache};

// 单层缓存
let cache = VectorCache::new(CacheConfig {
    max_size: 10_000,
    ttl: Some(std::time::Duration::from_secs(3600)),
});

cache.put(1, vec![1.0, 2.0, 3.0]);
let data = cache.get(1);

// 多级缓存
let multi_cache = MultiLevelCache::<Vec<f32>>::new(1000, 10000);
```

## 📖 使用指南

### 向量操作

```rust
use clawdb::Vector;

// 创建向量
let vector = Vector::new(1, vec![1.0, 2.0, 3.0]);

// 向量归一化
let normalized = vector.normalized();

// 序列化
let bytes = vector.to_bytes();
let decoded = Vector::from_bytes(&bytes)?;
```

### 距离计算

```rust
use clawdb::DistanceMetric;

let metric = DistanceMetric::Euclidean;
let a = vec![1.0, 2.0, 3.0];
let b = vec![4.0, 5.0, 6.0];

let distance = metric.compute(&a, &b);
println!("Distance: {}", distance);
```

### 加载数据集

```rust
use clawdb::loader::SiftDataLoader;

// 加载 fvecs 格式（SIFT1M）
let (dim, vectors) = SiftDataLoader::load_fvecs("sift_base.fvecs")?;

// 加载 bvecs 格式（SIFT1B）
let (dim, vectors) = SiftDataLoader::load_bvecs("bigann_base.bvecs")?;

// 加载 ground truth
let groundtruth = SiftDataLoader::load_ivecs("sift_groundtruth.ivecs")?;
```

## 📊 性能

### 基准测试结果

在 MacBook Pro (M1, 16GB) 上的测试结果：

| 操作 | 吞吐量 | 说明 |
|------|--------|------|
| 数据加载 | 465K vectors/sec | fvecs 格式 |
| IVF 索引构建 | 398 vectors/sec | 并行 K-Means |
| HNSW 索引构建 | 25 vectors/sec | 图构建 |
| IVF 搜索 | 2,555 QPS | Recall@10: 48.51% |
| HNSW 搜索 | 232 QPS | 高召回率 |
| VectorStorage 写入 | 89K vectors/sec | RocksDB |

### 运行基准测试

```bash
# 生成测试数据
make generate-data

# 运行快速基准测试
make quick-bench

# 运行完整基准测试
make benchmark

# 运行 Criterion 基准测试
make bench
```

### 性能优化建议

1. **批量插入**: 使用 `insert_batch` 而不是单个 `insert`
2. **索引参数**: 
   - IVF: `nlist` 建议 √N 到 N/1000 之间
   - HNSW: `ef_construction` 建议 100-200
3. **使用缓存**: 启用向量缓存提高热点数据访问速度
4. **定期压缩**: 在大量写入后调用 `compact()`
5. **内存管理**: 及时调用 `flush()` 刷新到磁盘

## 📚 API 文档

### VectorStorage

主要存储接口：

```rust
impl VectorStorage {
    // 打开数据库
    pub fn open<P: AsRef<Path>>(
        path: P, 
        dimension: usize, 
        metric: DistanceMetric
    ) -> Result<Self>;

    // 插入向量
    pub fn insert(&self, vector: Vector) -> Result<()>;
    
    // 批量插入
    pub fn insert_batch(&self, vectors: Vec<Vector>) -> Result<()>;

    // 获取向量
    pub fn get(&self, id: u64) -> Result<Option<Vector>>;

    // 删除向量
    pub fn delete(&self, id: u64) -> Result<()>;

    // 构建索引
    pub fn build_index(&mut self, nlist: usize) -> Result<()>;

    // 搜索最近邻（使用索引）
    pub fn search(
        &self, 
        query: &[f32], 
        k: usize, 
        nprobe: usize
    ) -> Result<Vec<(u64, f32)>>;

    // 暴力搜索
    pub fn brute_force_search(
        &self, 
        query: &[f32], 
        k: usize
    ) -> Result<Vec<(u64, f32)>>;

    // 统计向量数量
    pub fn count(&self) -> Result<usize>;

    // 刷新到磁盘
    pub fn flush(&self) -> Result<()>;

    // 压缩数据库
    pub fn compact(&self) -> Result<()>;
}
```

### HnswIndex

HNSW 图索引接口：

```rust
impl HnswIndex {
    pub fn new(dimension: usize, metric: DistanceMetric, config: HnswConfig) -> Self;
    pub fn build(&mut self, vectors: &[Vector]) -> Result<()>;
    pub fn search(&self, query: &[f32], k: usize, ef: usize) -> Result<Vec<(u64, f64)>>;
}
```

### DistanceMetric

支持的距离度量：

```rust
pub enum DistanceMetric {
    Euclidean,    // 欧氏距离
    Cosine,       // 余弦距离
    DotProduct,   // 点积距离
    Manhattan,    // 曼哈顿距离
}
```

## 💡 示例

### 示例 1: 基本向量搜索

```rust
use clawdb::{VectorStorage, Vector, DistanceMetric};

let mut storage = VectorStorage::open("./db", 128, DistanceMetric::Euclidean)?;

// 插入向量
for i in 0..10000 {
    let vector = Vector::new(i as u64, generate_random_vector(128));
    storage.insert(vector)?;
}

// 构建索引
storage.build_index(100)?;

// 搜索
let query = vec![0.5; 128];
let results = storage.search(&query, 10, 10)?;
```

### 示例 2: 使用 SIFT1M 数据集

```bash
# 下载 SIFT1M 数据集
make download-data

# 或生成合成测试数据
make generate-data

# 运行性能测试
make benchmark
```

## 📁 数据集

### SIFT1M 数据集

- **大小**: 100 万个 128 维向量
- **格式**: fvecs
- **下载**: [http://corpus-texmex.irisa.fr/](http://corpus-texmex.irisa.fr/)

### 文件格式

#### fvecs 格式
```
[d: int] [v1: float] [v2: float] ... [vd: float]
```

#### bvecs 格式
```
[d: int] [v1: byte] [v2: byte] ... [vd: byte]
```

#### ivecs 格式
```
[d: int] [v1: int] [v2: int] ... [vd: int]
```

## 🛠️ 开发指南

### 项目结构

```
clawdb/
├── Cargo.toml              # 项目配置
├── Makefile                # 构建脚本
├── README.md               # 项目文档
├── CHANGELOG.md            # 变更日志
├── src/
│   ├── lib.rs              # 库入口
│   ├── main.rs             # 示例程序
│   ├── error.rs            # 错误处理
│   ├── vector.rs           # 向量数据结构
│   ├── distance.rs         # 距离计算
│   ├── index.rs            # IVF 索引（并行 K-Means）
│   ├── hnsw.rs             # HNSW 图索引
│   ├── loader.rs           # 数据加载器
│   ├── cache.rs            # LRU 缓存
│   ├── io_limiter.rs       # I/O 限流器
│   ├── collection.rs       # 集合管理
│   ├── storage/            # 存储模块
│   │   ├── mod.rs
│   │   ├── storage.rs      # RocksDB 封装
│   │   ├── vector_storage.rs
│   │   ├── advanced_vector_storage.rs
│   │   ├── cf.rs           # Column Family
│   │   └── error.rs
│   ├── diskann/            # DiskANN 模块
│   │   ├── mod.rs
│   │   ├── config.rs       # 配置
│   │   ├── quantizer.rs    # PQ 量化器
│   │   └── table_factory.rs # TableFactory
│   ├── async_io/           # 异步 I/O
│   │   ├── mod.rs
│   │   ├── env.rs          # AsyncEnv trait
│   │   ├── fallback.rs     # TokioEnv
│   │   └── io_uring.rs     # IoUringEnv
│   ├── plugins/            # RocksDB 插件
│   │   ├── mod.rs
│   │   ├── compaction_filter.rs
│   │   ├── slice_transform.rs
│   │   └── merge_operator.rs
│   └── bin/                # 二进制程序
│       ├── sift_benchmark.rs
│       ├── quick_benchmark.rs
│       └── generate_test_data.rs
├── benches/
│   └── vector_search.rs    # 性能基准测试
├── docs/
│   ├── ARCHITECTURE.md     # 架构文档
│   ├── BENCHMARK_REPORT.md # 性能报告
│   └── IMPLEMENTATION_REPORT.md
└── scripts/
    └── download_sift1m.sh  # 数据下载脚本
```

### 运行测试

```bash
# 运行单元测试
make test

# 运行所有测试
make test-all

# 运行特定测试
cargo test test_vector_creation

# 运行基准测试
make bench
```

### 代码质量

```bash
# 格式化代码
make fmt

# 检查代码
make check

# 运行 clippy
make clippy

# 运行所有检查
make ci
```

## ❓ 常见问题

### Q: 如何选择 IVF 索引参数？

**A**: 
- `nlist`: 建议设置为 √N 到 N/1000 之间，其中 N 是向量数量
- `nprobe`: 建议设置为 nlist/10 到 nlist/5 之间
- 更大的 nlist 会提高精度但降低速度
- 更大的 nprobe 会提高精度但降低速度

### Q: IVF 和 HNSW 如何选择？

**A**:
- **IVF**: 适合大规模数据，内存效率高，构建速度快
- **HNSW**: 召回率更高，查询延迟更低，但内存占用更大

### Q: 如何提高插入性能？

**A**: 
- 使用 `insert_batch` 而不是单个 `insert`
- 在插入完成后再构建索引
- 调整 RocksDB 的写缓冲区大小
- 考虑禁用同步写入（牺牲持久性）

### Q: 支持哪些操作系统？

**A**: 
- ✅ Linux
- ✅ macOS
- ✅ Windows (需要安装 RocksDB)
- ✅ FreeBSD

## 🤝 贡献指南

我们欢迎所有形式的贡献！

### 如何贡献

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码规范

- 使用 `cargo fmt` 格式化代码
- 确保所有测试通过 (`make test`)
- 添加必要的文档注释
- 遵循 Rust 最佳实践

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- [RocksDB](https://rocksdb.org/) - 高性能嵌入式数据库
- [Rayon](https://github.com/rayon-rs/rayon) - 数据并行库
- [SIFT1M Dataset](http://corpus-texmex.irisa.fr/) - 测试数据集

## 📖 引用

如果您在研究中使用了本项目或 SIFT1M 数据集，请引用：

```bibtex
@article{jegou2010improving,
  title={Improving bag-of-features for large scale image search},
  author={J{\'e}gou, Herv{\'e} and Douze, Matthijs and Schmid, Cordelia},
  journal={International journal of computer vision},
  volume={87},
  number={3},
  pages={316--336},
  year={2010},
  publisher={Springer}
}
```

---

<div align="center">

**[⬆ 返回顶部](#clawdb---高性能向量数据库)**

Made with ❤️ by the ClawDB Team

</div>

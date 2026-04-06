# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2024-04-06

### Added

#### Core Features
- **HNSW Index**: Hierarchical Navigable Small World graph index implementation
  - High recall ANN search
  - Heuristic neighbor selection
  - Multi-layer graph structure
  - Configurable parameters (ef_construction, m_max, ml)

- **Parallel K-Means**: Optimized IVF index building
  - Parallel distance computation using Rayon
  - K-Means++ initialization algorithm
  - Early stopping mechanism
  - 2-4x faster index build

- **Vector Cache**: LRU cache implementation
  - Single-level cache with TTL support
  - Multi-level cache (L1/L2)
  - Thread-safe design with Arc<Mutex>
  - Cache statistics and hit rate tracking

- **DiskANN TableFactory**: Product Quantization storage
  - Custom SST file format
  - Meta Block for codebook storage
  - Data Block for PQ codes
  - 6-10x memory reduction

- **Async I/O**: Asynchronous file operations
  - AsyncEnv trait abstraction
  - TokioEnv implementation (cross-platform)
  - IoUringEnv implementation (Linux, with Tokio fallback)

- **I/O Rate Limiter**: Priority-based I/O control
  - Token bucket algorithm
  - High/Medium/Low priority levels
  - Separate limits for query and background tasks

- **RocksDB Plugins**:
  - CompactionFilter for zero-cost deletion
  - SliceTransform for collection-based filtering
  - MergeOperator for atomic metadata updates

- **Collection Support**: Multi-tenancy data model
  - CollectionId for data isolation
  - VectorKey with collection prefix
  - VectorMetadata with timestamps
  - VectorValue with optional original vector

#### Performance Optimizations
- Parallel distance computation in IVF search
- Optimized HNSW graph construction
- Batch vector operations
- Memory-efficient serialization

#### Testing & Benchmarking
- SIFT1M benchmark suite
- Quick benchmark tool
- Synthetic data generator
- 57 unit tests (all passing)

#### Documentation
- Comprehensive README with examples
- Architecture documentation
- Benchmark report
- Implementation report
- API documentation

### Changed
- Improved IVF index build performance with parallel K-Means
- Enhanced HNSW search with heuristic neighbor selection
- Optimized memory allocation in hot paths
- Updated RocksDB to v0.24.0

### Performance Results

| Operation | Throughput | Notes |
|-----------|------------|-------|
| Data Loading | 465K vectors/sec | fvecs format |
| IVF Index Build | 398 vectors/sec | Parallel K-Means |
| HNSW Index Build | 25 vectors/sec | Graph construction |
| IVF Search | 2,555 QPS | Recall@10: 48.51% |
| HNSW Search | 232 QPS | High recall |
| VectorStorage Write | 89K vectors/sec | RocksDB |

---

## [0.3.0] - 2024-01-XX

### Added

#### Storage Enhancements
- **AdvancedVectorStorage**: Enhanced storage with collection support
- **Column Family Management**: Improved data isolation
- **Batch Operations**: Optimized batch insert and delete

#### Index Improvements
- **IVF Index Optimization**: Better clustering algorithm
- **Distance Metrics**: Added Manhattan distance

#### Development Tools
- **Makefile**: Comprehensive build commands
- **CI/CD Support**: Automated testing and checks

---

## [0.2.0] - 2024-01-XX

### Added

#### Core Features
- **Vector Storage**: Persistent vector storage based on RocksDB
- **IVF Index**: Inverted File Index for fast approximate nearest neighbor search
- **Distance Metrics**: Support for multiple distance metrics
  - Euclidean distance
  - Cosine similarity
  - Dot product
- **Data Loaders**: Support for SIFT dataset formats
  - fvecs format (float vectors)
  - bvecs format (byte vectors)
  - ivecs format (integer vectors)

#### Storage Layer
- **RocksDB Integration**: High-performance embedded database
- **Column Families**: Separate storage for different data types
  - Data CF: Vector data storage
  - Index CF: Index metadata
  - Metadata CF: Database metadata
  - Cache CF: Cache layer
  - History CF: Version history
  - Snapshot CF: Snapshot storage
- **LZ4 Compression**: Fast compression for storage optimization
- **Batch Operations**: Support for batch insert and delete

#### Performance
- **Parallel Computing**: Use Rayon for parallel distance calculation
- **SIMD Support**: Optional SIMD acceleration for distance computation
- **Memory Efficient**: Efficient serialization with bincode

#### Development
- **Comprehensive Tests**: 34 unit tests with 100% coverage
- **Benchmark Suite**: Performance benchmarks using Criterion
- **CI/CD Ready**: Makefile with all development commands
- **Code Quality**: Clippy linter with strict warnings

#### Documentation
- **README**: Comprehensive documentation with examples
- **API Documentation**: Inline documentation for all public APIs
- **Architecture Diagram**: Clear architecture visualization
- **Usage Examples**: Multiple code examples for common use cases

---

## [0.1.0] - 2024-01-XX

### Added

#### Initial Release
- Basic vector storage and retrieval
- IVF index for ANN search
- Multiple distance metrics (Euclidean, Cosine, Dot Product)
- SIFT dataset support (fvecs, bvecs, ivecs)
- RocksDB-based persistence
- Comprehensive testing and documentation

---

## [Unreleased]

### Planned Features
- Native io_uring implementation (Linux)
- SPDK user-space driver
- GPU acceleration (CUDA)
- Distributed deployment
- REST API server
- Python SDK
- Real-time index updates
- Range query support
- Filter support in search

---

## Version History

| Version | Date | Description |
|---------|------|-------------|
| 0.4.0 | 2024-04-06 | HNSW, Parallel K-Means, Cache, DiskANN, Async I/O |
| 0.3.0 | 2024-01-XX | Advanced storage, collection support |
| 0.2.0 | 2024-01-XX | IVF optimization, distance metrics |
| 0.1.0 | 2024-01-XX | Initial release |

---

## Roadmap

### Version 0.5.0 (Planned)
- [ ] Native io_uring system calls
- [ ] GPU acceleration (CUDA)
- [ ] REST API server
- [ ] Python SDK

### Version 0.6.0 (Planned)
- [ ] Distributed deployment
- [ ] Replication and sharding
- [ ] Real-time index updates
- [ ] Monitoring and metrics

### Version 1.0.0 (Planned)
- [ ] Production-ready release
- [ ] Enterprise features
- [ ] Cloud-native support
- [ ] Global deployment

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

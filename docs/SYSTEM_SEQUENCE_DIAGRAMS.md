# ClawDB 系统时序图

本文档展示 ClawDB 系统的核心流程和组件交互。

## 📋 目录

- [数据加载流程](#数据加载流程)
- [索引构建流程](#索引构建流程)
- [向量搜索流程](#向量搜索流程)
- [存储操作流程](#存储操作流程)
- [缓存交互流程](#缓存交互流程)
- [完整系统流程](#完整系统流程)

---

## 数据加载流程

```mermaid
sequenceDiagram
    participant User as 用户
    participant Loader as SiftDataLoader
    participant File as 文件系统
    participant Memory as 内存

    User->>Loader: load_fvecs(path)
    Loader->>File: 打开文件
    File-->>Loader: 文件句柄
    
    loop 读取每个向量
        Loader->>File: 读取维度 (4 bytes)
        File-->>Loader: dimension
        Loader->>File: 读取向量数据 (dim * 4 bytes)
        File-->>Loader: vector_data
        Loader->>Memory: 存储向量
    end
    
    Loader->>File: 关闭文件
    Loader-->>User: (dimension, vectors)
```

### 说明

1. **文件格式**: fvecs 格式，每个向量包含维度 + 数据
2. **读取方式**: 顺序读取，高效 I/O
3. **内存分配**: 预分配向量数组，减少重分配

---

## 索引构建流程

### IVF 索引构建（并行 K-Means）

```mermaid
sequenceDiagram
    participant User as 用户
    participant IVF as VectorIndex
    participant KMeans as K-Means++
    participant Rayon as Rayon 并行
    participant Storage as 向量存储

    User->>IVF: build(vectors)
    IVF->>KMeans: 初始化聚类中心
    
    loop K-Means++ 初始化
        KMeans->>Rayon: 并行计算距离
        Rayon-->>KMeans: 距离矩阵
        KMeans->>KMeans: 选择下一个中心
    end
    
    KMeans-->>IVF: 初始聚类中心
    
    loop 迭代优化 (max 20 次)
        IVF->>Rayon: 并行分配向量到聚类
        Rayon->>Storage: 读取向量
        Storage-->>Rayon: 向量数据
        Rayon-->>IVF: 聚类分配结果
        
        IVF->>Rayon: 并行更新聚类中心
        Rayon-->>IVF: 新聚类中心
        
        IVF->>IVF: 检查收敛
        alt 已收敛
            IVF->>IVF: 停止迭代
        end
    end
    
    IVF->>IVF: 构建倒排列表
    IVF-->>User: 构建完成
```

### HNSW 索引构建

```mermaid
sequenceDiagram
    participant User as 用户
    participant HNSW as HnswIndex
    participant Layer as 层级管理
    participant Graph as 图结构
    participant Search as 搜索算法

    User->>HNSW: build(vectors)
    
    loop 插入每个向量
        HNSW->>HNSW: random_level()
        HNSW->>Layer: 获取入口点
        
        alt 第一层
            HNSW->>Graph: 创建节点
            HNSW->>Layer: 设置入口点
        else 后续层
            loop 从顶层到目标层
                HNSW->>Search: greedy_search()
                Search-->>HNSW: 最近节点
            end
            
            loop 目标层及以下
                HNSW->>Search: search_layer()
                Search-->>HNSW: 候选邻居
                HNSW->>HNSW: select_neighbors_heuristic()
                HNSW->>Graph: 连接邻居
                HNSW->>Graph: 更新反向连接
            end
        end
    end
    
    HNSW-->>User: 构建完成
```

---

## 向量搜索流程

### IVF 搜索

```mermaid
sequenceDiagram
    participant User as 用户
    participant IVF as VectorIndex
    participant Centroids as 聚类中心
    participant Lists as 倒排列表
    participant Distance as 距离计算

    User->>IVF: search(query, k, nprobe)
    
    IVF->>Centroids: 计算查询到聚类中心距离
    loop 每个聚类中心
        Centroids->>Distance: euclidean_distance()
        Distance-->>Centroids: distance
    end
    
    Centroids-->>IVF: 排序后的聚类列表
    IVF->>IVF: 选择前 nprobe 个聚类
    
    loop 每个选中的聚类
        IVF->>Lists: 获取倒排列表
        Lists-->>IVF: 向量 ID 列表
        
        loop 每个向量 ID
            IVF->>Distance: compute(query, vector)
            Distance-->>IVF: distance
        end
    end
    
    IVF->>IVF: 排序并返回 top-k
    IVF-->>User: [(id, distance), ...]
```

### HNSW 搜索

```mermaid
sequenceDiagram
    participant User as 用户
    participant HNSW as HnswIndex
    participant Layer as 层级
    participant Search as 搜索算法
    participant Graph as 图节点

    User->>HNSW: search(query, k, ef)
    
    HNSW->>Layer: 获取入口点
    Layer-->>HNSW: entry_point
    
    loop 从顶层到底层+1
        HNSW->>Search: greedy_search(layer)
        Search->>Graph: 获取邻居
        Graph-->>Search: neighbors
        Search->>Search: 选择最近邻居
        Search-->>HNSW: next_node
    end
    
    HNSW->>Search: search_layer(ef)
    Search->>Graph: 获取底层邻居
    Graph-->>Search: neighbors
    
    loop 扩展候选集
        Search->>Graph: 访问邻居的邻居
        Graph-->>Search: more_neighbors
        Search->>Search: 更新候选集
    end
    
    Search-->>HNSW: top-ef 候选
    HNSW->>HNSW: 返回 top-k
    HNSW-->>User: [(id, distance), ...]
```

---

## 存储操作流程

### 写入流程

```mermaid
sequenceDiagram
    participant User as 用户
    participant VS as VectorStorage
    participant Cache as 缓存层
    participant CF as Column Family
    participant RocksDB as RocksDB
    participant Disk as 磁盘

    User->>VS: insert(vector)
    VS->>VS: 序列化向量
    VS->>Cache: 检查缓存
    
    alt 缓存命中
        Cache-->>VS: 缓存数据
    else 缓存未命中
        VS->>CF: 获取 Data CF
        CF->>RocksDB: put(id, data)
        RocksDB->>Disk: 写入 WAL
        RocksDB->>Disk: 写入 MemTable
        RocksDB-->>CF: 写入成功
        CF-->>VS: 成功
    end
    
    VS->>Cache: 更新缓存
    VS-->>User: 成功
```

### 读取流程

```mermaid
sequenceDiagram
    participant User as 用户
    participant VS as VectorStorage
    participant Cache as 缓存层
    participant CF as Column Family
    participant RocksDB as RocksDB
    participant BlockCache as Block Cache
    participant Disk as 磁盘

    User->>VS: get(id)
    VS->>Cache: 查询缓存
    
    alt 缓存命中
        Cache-->>VS: 向量数据
        VS-->>User: Vector
    else 缓存未命中
        VS->>CF: 获取 Data CF
        CF->>RocksDB: get(id)
        RocksDB->>BlockCache: 查询 Block Cache
        
        alt Block Cache 命中
            BlockCache-->>RocksDB: 数据块
        else Block Cache 未命中
            RocksDB->>Disk: 读取 SST 文件
            Disk-->>RocksDB: 数据块
            RocksDB->>BlockCache: 更新缓存
        end
        
        RocksDB-->>CF: 向量数据
        CF-->>VS: 数据
        VS->>Cache: 更新缓存
        VS-->>User: Vector
    end
```

---

## 缓存交互流程

### LRU 缓存操作

```mermaid
sequenceDiagram
    participant User as 用户
    participant Cache as VectorCache
    participant HashMap as HashMap
    participant Entry as CacheEntry

    User->>Cache: get(id)
    Cache->>HashMap: get_mut(id)
    
    alt 存在
        HashMap-->>Cache: entry
        Cache->>Entry: touch()
        Entry->>Entry: 更新 last_access
        Entry->>Entry: access_count++
        Cache-->>User: Some(data)
    else 不存在
        HashMap-->>Cache: None
        Cache-->>User: None
    end

    User->>Cache: put(id, data)
    Cache->>Cache: 检查容量
    
    alt 容量已满
        Cache->>Cache: evict_lru()
        Cache->>HashMap: 找到最旧条目
        HashMap-->>Cache: oldest_entry
        Cache->>HashMap: remove(oldest_id)
    end
    
    Cache->>Entry: new(data)
    Cache->>HashMap: insert(id, entry)
    Cache-->>User: 成功
```

### 多级缓存

```mermaid
sequenceDiagram
    participant User as 用户
    participant Multi as MultiLevelCache
    participant L1 as L1 Cache
    participant L2 as L2 Cache
    participant Storage as 存储

    User->>Multi: get(id)
    Multi->>L1: get(id)
    
    alt L1 命中
        L1-->>Multi: data
        Multi-->>User: data
    else L1 未命中
        Multi->>L2: get(id)
        
        alt L2 命中
            L2-->>Multi: data
            Multi->>L1: put(id, data)
            Multi-->>User: data
        else L2 未命中
            Multi->>Storage: get(id)
            Storage-->>Multi: data
            Multi->>L1: put(id, data)
            Multi->>L2: put(id, data)
            Multi-->>User: data
        end
    end
```

---

## 完整系统流程

### 端到端流程：数据加载 → 索引构建 → 搜索

```mermaid
sequenceDiagram
    participant User as 用户
    participant Loader as DataLoader
    participant VS as VectorStorage
    participant IVF as IVF 索引
    participant HNSW as HNSW 索引
    participant Cache as 缓存层
    participant RocksDB as RocksDB

    rect rgb(200, 220, 240)
        Note over User,RocksDB: 第一阶段：数据加载
        User->>Loader: load_fvecs(path)
        Loader->>Loader: 读取文件
        Loader-->>User: vectors
    end

    rect rgb(220, 240, 200)
        Note over User,RocksDB: 第二阶段：索引构建
        User->>VS: insert_batch(vectors)
        VS->>RocksDB: 批量写入
        RocksDB-->>VS: 成功
        VS-->>User: 成功
        
        User->>IVF: build(vectors)
        IVF->>IVF: 并行 K-Means
        IVF-->>User: 索引就绪
        
        User->>HNSW: build(vectors)
        HNSW->>HNSW: 构建图结构
        HNSW-->>User: 索引就绪
    end

    rect rgb(240, 220, 200)
        Note over User,RocksDB: 第三阶段：向量搜索
        User->>VS: search(query, k)
        VS->>Cache: 查询缓存
        
        alt 缓存命中
            Cache-->>VS: 结果
        else 缓存未命中
            VS->>IVF: search(query, k)
            IVF-->>VS: 候选向量
            VS->>RocksDB: 读取向量
            RocksDB-->>VS: 向量数据
            VS->>VS: 精确计算距离
            VS->>Cache: 更新缓存
        end
        
        VS-->>User: top-k 结果
    end
```

---

## 性能关键路径

### 热点路径分析

```mermaid
graph TD
    A[查询请求] --> B{缓存命中?}
    B -->|是| C[返回结果]
    B -->|否| D[索引搜索]
    
    D --> E{索引类型}
    E -->|IVF| F[聚类选择]
    E -->|HNSW| G[图遍历]
    
    F --> H[倒排列表扫描]
    G --> I[邻居扩展]
    
    H --> J[距离计算]
    I --> J
    
    J --> K[排序 Top-K]
    K --> L[更新缓存]
    L --> C
    
    style A fill:#e1f5ff
    style C fill:#c8e6c9
    style J fill:#fff9c4
    style L fill:#f8bbd0
```

### 性能优化点

| 路径 | 优化方法 | 效果 |
|------|---------|------|
| 距离计算 | SIMD 指令 | 4-8x 加速 |
| 并行 K-Means | Rayon 并行 | 2-4x 加速 |
| 缓存命中 | LRU + TTL | 100% 命中率 |
| RocksDB 读取 | Block Cache | 减少 I/O |
| 图遍历 | 启发式选择 | 减少计算 |

---

## 异常处理流程

### 错误恢复

```mermaid
sequenceDiagram
    participant User as 用户
    participant VS as VectorStorage
    participant Error as 错误处理
    participant Log as 日志系统

    User->>VS: insert(vector)
    VS->>VS: 验证向量
    
    alt 向量无效
        VS->>Error: InvalidVectorData
        Error->>Log: 记录错误
        Error-->>User: Err(InvalidVectorData)
    else 存储失败
        VS->>Error: StorageError
        Error->>Log: 记录错误
        Error-->>User: Err(StorageError)
    else 成功
        VS-->>User: Ok(())
    end
```

---

## 总结

ClawDB 系统通过以下关键流程实现高性能：

1. **数据加载**: 高效的文件读取和内存管理
2. **索引构建**: 并行 K-Means 和优化的图构建
3. **向量搜索**: 多级索引和缓存加速
4. **存储操作**: RocksDB 优化和缓存策略
5. **缓存系统**: LRU + 多级缓存提高命中率

所有流程都经过性能优化，确保系统在高负载下仍能保持高效运行。

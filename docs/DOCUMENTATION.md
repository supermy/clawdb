# ClawDB 项目文档

## 📚 文档结构

```
clawdb/
├── README.md                    # 项目介绍和快速开始
├── CHANGELOG.md                 # 版本更新日志
├── CONTRIBUTING.md              # 贡献指南
├── LICENSE                      # MIT 许可证
├── Makefile                     # 构建脚本
├── Cargo.toml                   # Rust 项目配置
├── .gitignore                   # Git 忽略规则
├── docs/                        # 文档目录
│   ├── README.md               # 文档索引和导航
│   └── ARCHITECTURE.md         # 详细技术架构文档 (44KB)
├── src/                         # 源代码
│   ├── lib.rs                  # 库入口
│   ├── main.rs                 # 示例程序
│   ├── error.rs                # 错误处理
│   ├── vector.rs               # 向量模块
│   ├── distance.rs             # 距离计算
│   ├── index.rs                # 索引模块
│   ├── loader.rs               # 数据加载器
│   └── storage/                # 存储模块
│       ├── mod.rs
│       ├── storage.rs          # RocksDB 封装
│       ├── vector_storage.rs   # 向量存储
│       ├── cf.rs               # Column Family
│       └── error.rs            # 存储错误
└── benches/                     # 性能基准测试
    └── vector_search.rs
```

## 🎯 文档使用指南

### 新用户

1. **开始使用**: 阅读 [README.md](../README.md)
2. **快速入门**: 运行 `make run` 查看示例
3. **API 文档**: 查看在线文档或运行 `make doc`

### 开发者

1. **架构理解**: 阅读 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
2. **贡献代码**: 阅读 [CONTRIBUTING.md](../CONTRIBUTING.md)
3. **开发环境**: 运行 `make install`

### 运维人员

1. **部署指南**: 查看 [ARCHITECTURE.md#部署架构](docs/ARCHITECTURE.md#部署架构)
2. **监控运维**: 查看 [ARCHITECTURE.md#监控与运维](docs/ARCHITECTURE.md#监控与运维)
3. **安全配置**: 查看 [ARCHITECTURE.md#安全设计](docs/ARCHITECTURE.md#安全设计)

## 📖 核心文档说明

### README.md
- **目标受众**: 所有用户
- **内容**: 项目介绍、安装、快速开始、基本使用
- **更新频率**: 每次版本发布

### docs/ARCHITECTURE.md
- **目标受众**: 开发者、架构师、运维人员
- **内容**: 详细的技术架构、设计决策、性能优化
- **大小**: 44KB，包含完整的架构设计
- **更新频率**: 架构变更时

### CHANGELOG.md
- **目标受众**: 所有用户
- **内容**: 版本历史、功能变更、升级指南
- **更新频率**: 每次版本发布

### CONTRIBUTING.md
- **目标受众**: 贡献者
- **内容**: 贡献流程、代码规范、提交规范
- **更新频率**: 流程变更时

## 🔍 快速查找

| 需求 | 文档位置 |
|------|---------|
| 如何安装？ | [README.md#安装](../README.md#安装) |
| 如何使用？ | [README.md#快速开始](../README.md#快速开始) |
| 架构设计？ | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) |
| 性能优化？ | [docs/ARCHITECTURE.md#性能优化](docs/ARCHITECTURE.md#性能优化) |
| 如何部署？ | [docs/ARCHITECTURE.md#部署架构](docs/ARCHITECTURE.md#部署架构) |
| 如何贡献？ | [CONTRIBUTING.md](../CONTRIBUTING.md) |
| API 文档？ | `make doc` 或 [docs.rs](https://docs.rs/clawdb) |

## 📝 文档维护

### 文档更新原则

1. **及时性**: 功能变更时同步更新文档
2. **准确性**: 确保文档与代码一致
3. **完整性**: 覆盖所有用户场景
4. **易读性**: 使用清晰的语言和示例

### 文档结构规范

- 使用 Markdown 格式
- 包含清晰的目录
- 提供代码示例
- 添加适当的图表

### 文档审查清单

- [ ] 代码示例可运行
- [ ] 链接有效
- [ ] 格式正确
- [ ] 内容准确
- [ ] 无拼写错误

## 🚀 文档生成

### API 文档

```bash
# 生成并打开 API 文档
make doc

# 或
cargo doc --no-deps --open
```

### 架构图

架构图使用 Mermaid 格式，可在支持 Mermaid 的 Markdown 查看器中查看。

## 📞 文档反馈

如果您发现文档问题或有改进建议：

1. 在 GitHub 上创建 Issue
2. 提交 Pull Request
3. 在 Discussions 中讨论

---

**维护者**: ClawDB Team  
**最后更新**: 2024-01-XX

# GitHub Actions 性能测试指南

本文档介绍如何在 GitHub 上进行自动化性能测试。

## 📋 目录

- [CI 工作流](#ci-工作流)
- [性能基准测试](#性能基准测试)
- [手动触发测试](#手动触发测试)
- [查看测试结果](#查看测试结果)
- [性能监控](#性能监控)
- [自定义配置](#自定义配置)

## 🔄 CI 工作流

### 自动触发条件

CI 工作流 (`.github/workflows/ci.yml`) 会在以下情况自动运行：

- Push 到 `main`、`master` 或 `develop` 分支
- Pull Request 到 `main`、`master` 或 `develop` 分支

### CI 检查项目

| 检查项 | 说明 |
|--------|------|
| Check | 编译检查 |
| Test | 单元测试 (Ubuntu + macOS) |
| Clippy | 代码风格检查 |
| Fmt | 格式化检查 |
| Docs | 文档生成 |
| Security | 安全审计 |

### 查看 CI 结果

1. 进入 GitHub 仓库页面
2. 点击 "Actions" 标签
3. 选择对应的 workflow 运行记录
4. 查看各项检查结果

## 📊 性能基准测试

### 自动触发条件

性能基准测试工作流 (`.github/workflows/benchmark.yml`) 会在以下情况运行：

- Push 到 `main` 或 `master` 分支
- Pull Request 到 `main` 或 `master` 分支
- 手动触发

### 测试类型

#### 1. Quick Benchmark (快速测试)

- **用途**: 快速验证性能
- **数据量**: 50,000 向量 (默认)
- **运行时间**: ~5 分钟
- **输出**: 基本性能指标

#### 2. Criterion Benchmark (详细测试)

- **用途**: 详细性能分析
- **运行条件**: Push 或手动触发
- **输出**: Criterion 报告 + 历史对比

#### 3. Memory Profile (内存分析)

- **用途**: 内存使用分析
- **运行条件**: 仅手动触发
- **工具**: Valgrind Massif

## 🚀 手动触发测试

### 通过 GitHub UI

1. 进入仓库的 "Actions" 页面
2. 选择 "Performance Benchmark" workflow
3. 点击 "Run workflow"
4. 配置参数：
   - `vector_count`: 测试向量数量 (默认: 50000)
   - `run_full_benchmark`: 是否运行完整测试 (默认: false)

### 通过 GitHub CLI

```bash
# 运行快速测试
gh workflow run benchmark.yml

# 运行完整测试
gh workflow run benchmark.yml -f run_full_benchmark=true

# 指定向量数量
gh workflow run benchmark.yml -f vector_count=100000
```

### 通过 API

```bash
curl -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: token YOUR_TOKEN" \
  https://api.github.com/repos/OWNER/REPO/actions/workflows/benchmark.yml/dispatches \
  -d '{"ref":"main","inputs":{"vector_count":"100000","run_full_benchmark":"true"}}'
```

## 📈 查看测试结果

### GitHub Summary

每次运行后，会在 workflow 页面生成 Summary，包含：

- 测试配置信息
- 性能指标表格
- 关键结果摘要

### Artifacts

测试结果会作为 Artifacts 上传，保留 30 天：

| Artifact | 内容 |
|----------|------|
| benchmark-results | 基准测试结果 |
| criterion-results | Criterion 详细报告 |
| memory-profile | 内存分析报告 |

下载方式：
1. 进入 workflow 运行页面
2. 滚动到 "Artifacts" 部分
3. 点击下载

### GitHub Pages

文档和报告可部署到 GitHub Pages：

```bash
# 启用 GitHub Pages
# Settings -> Pages -> Source: GitHub Actions

# 访问地址
https://YOUR_USERNAME.github.io/clawdb/
```

## 📉 性能监控

### 历史对比

使用 `benchmark-action/github-action-benchmark` 进行历史对比：

- 自动对比历史性能数据
- 性能回归超过 200% 时告警
- 在 PR 中添加性能对比评论

### 性能告警

当检测到性能回归时：

1. Workflow 会标记为失败
2. 自动在 PR 中添加评论
3. 通知指定用户 (@moyong)

### 自定义告警阈值

编辑 `.github/workflows/benchmark.yml`:

```yaml
- name: Store benchmark result
  uses: benchmark-action/github-action-benchmark@v1
  with:
    alert-threshold: '150%'  # 修改阈值
    fail-on-alert: true      # 是否失败
```

## ⚙️ 自定义配置

### 修改测试数据量

编辑 workflow 文件或手动触发时指定：

```yaml
inputs:
  vector_count:
    description: 'Number of vectors for testing'
    default: '100000'  # 修改默认值
```

### 添加新的测试矩阵

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta]
```

### 自定义缓存

```yaml
- name: Cache cargo build
  uses: actions/cache@v3
  with:
    path: target
    key: custom-key-${{ hashFiles('**/Cargo.lock') }}
```

### 添加通知

```yaml
- name: Notify on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: failure
    fields: repo,message,commit,author,action
  env:
    SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

## 📝 最佳实践

### 1. 定期运行完整测试

```bash
# 每周运行一次完整测试
# 使用 scheduled trigger:
on:
  schedule:
    - cron: '0 0 * * 0'  # 每周日 00:00 UTC
```

### 2. PR 中自动测试

确保每个 PR 都运行性能测试：

```yaml
on:
  pull_request:
    branches: [ main ]
```

### 3. 保存历史数据

使用 GitHub Actions Cache 或 Artifacts 保存历史数据：

```yaml
- name: Cache benchmark data
  uses: actions/cache@v3
  with:
    path: ./benchmark_history
    key: benchmark-history
```

### 4. 性能回归处理

当检测到性能回归时：

1. 检查最近的代码更改
2. 运行本地测试验证
3. 使用 Criterion 详细报告分析
4. 必要时回滚或修复

## 🔧 故障排除

### 测试超时

增加 timeout-minutes：

```yaml
jobs:
  benchmark:
    timeout-minutes: 120  # 增加到 2 小时
```

### 内存不足

减少测试数据量或使用更大 runner：

```yaml
runs-on: ubuntu-latest-m  # 更大内存的 runner
```

### RocksDB 安装失败

确保正确安装依赖：

```yaml
- name: Install RocksDB
  run: |
    sudo apt-get update
    sudo apt-get install -y librocksdb-dev
```

## 📚 参考资源

- [GitHub Actions 文档](https://docs.github.com/en/actions)
- [Criterion.rs 文档](https://bheisler.github.io/criterion.rs/book/)
- [benchmark-action 文档](https://github.com/benchmark-action/github-action-benchmark)
- [RocksDB GitHub Actions](https://github.com/facebook/rocksdb/blob/main/.github/workflows/)

---

## 快速开始

1. **推送代码** - 自动触发 CI 和性能测试
2. **查看结果** - Actions -> 选择 workflow -> 查看 Summary
3. **下载报告** - Artifacts -> 下载详细报告
4. **分析趋势** - 查看历史对比和性能趋势

如有问题，请创建 Issue 或查看 workflow 日志。

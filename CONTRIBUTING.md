# 贡献指南

感谢您有兴趣为 ClawDB 做出贡献！我们欢迎所有形式的贡献。

## 📋 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发流程](#开发流程)
- [代码规范](#代码规范)
- [提交信息](#提交信息)
- [Pull Request 流程](#pull-request-流程)
- [问题报告](#问题报告)
- [功能请求](#功能请求)

## 行为准则

### 我们的承诺

为了营造一个开放和友好的环境，我们承诺：

- 使用包容性语言
- 尊重不同的观点和经验
- 优雅地接受建设性批评
- 关注对社区最有利的事情
- 对其他社区成员表示同理心

### 我们的标准

**积极行为示例：**
- 使用友好和包容的语言
- 尊重不同的观点和经验
- 优雅地接受建设性批评
- 关注对社区最有利的事情
- 对其他社区成员表示同理心

**不可接受的行为示例：**
- 使用性化的语言或图像
- 捣乱、侮辱/贬损评论以及人身或政治攻击
- 公开或私下的骚扰
- 未经明确许可，发布他人的私人信息
- 其他在专业环境中可能被合理认为不适当的行为

## 如何贡献

### 报告 Bug

如果您发现了 bug，请创建一个 issue 并包含以下信息：

1. **标题**: 清晰简洁的描述
2. **描述**: 详细描述问题
3. **重现步骤**: 
   ```
   1. 步骤 1
   2. 步骤 2
   3. ...
   ```
4. **预期行为**: 您期望发生什么
5. **实际行为**: 实际发生了什么
6. **环境**:
   - 操作系统: [例如 macOS 12.0]
   - Rust 版本: [例如 1.70.0]
   - ClawDB 版本: [例如 0.1.0]
7. **附加信息**: 日志、截图等

### 提交功能请求

如果您有新功能的想法，请创建一个 issue 并包含：

1. **标题**: 清晰的功能描述
2. **动机**: 为什么需要这个功能
3. **详细描述**: 功能应该如何工作
4. **替代方案**: 您考虑过的其他解决方案
5. **附加信息**: 任何其他相关信息

### 提交代码

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 进行更改
4. 运行测试 (`make test`)
5. 运行代码检查 (`make ci`)
6. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
7. 推送到分支 (`git push origin feature/AmazingFeature`)
8. 开启 Pull Request

## 开发流程

### 1. 设置开发环境

```bash
# 克隆您的 fork
git clone https://github.com/your-username/clawdb.git
cd clawdb

# 安装依赖
make install

# 运行测试
make test
```

### 2. 创建分支

```bash
# 从 main 创建新分支
git checkout -b feature/your-feature-name

# 或修复 bug
git checkout -b fix/your-bug-fix
```

### 3. 进行更改

- 遵循代码规范
- 添加测试
- 更新文档
- 确保所有测试通过

### 4. 提交更改

```bash
# 查看更改
git status

# 添加文件
git add .

# 提交
git commit -m "feat: add amazing feature"
```

### 5. 推送并创建 PR

```bash
# 推送到您的 fork
git push origin feature/your-feature-name

# 在 GitHub 上创建 Pull Request
```

## 代码规范

### Rust 代码规范

1. **格式化**: 使用 `cargo fmt`
   ```bash
   make fmt
   ```

2. **Linting**: 使用 `cargo clippy`
   ```bash
   make clippy
   ```

3. **命名约定**:
   - 类型: PascalCase (例如 `VectorStorage`)
   - 函数和方法: snake_case (例如 `build_index`)
   - 常量: SCREAMING_SNAKE_CASE (例如 `MAX_RETRIES`)
   - 模块: snake_case (例如 `vector_storage`)

4. **文档注释**:
   ```rust
   /// 简短描述
   ///
   /// 详细描述
   ///
   /// # Arguments
   ///
   /// * `param` - 参数描述
   ///
   /// # Returns
   ///
   /// 返回值描述
   ///
   /// # Examples
   ///
   /// ```
   /// use clawdb::Vector;
   /// let v = Vector::new(1, vec![1.0, 2.0]);
   /// ```
   pub fn example_function(param: i32) -> Result<()> {
       // ...
   }
   ```

5. **错误处理**:
   - 使用 `Result<T, E>` 进行错误处理
   - 使用 `thiserror` 定义自定义错误
   - 提供有意义的错误信息

6. **测试**:
   - 每个公共函数都应有测试
   - 使用 `#[cfg(test)]` 模块
   - 测试名称应描述测试内容

### 项目结构

```
clawdb/
├── src/
│   ├── lib.rs           # 库入口
│   ├── main.rs          # 示例程序
│   ├── error.rs         # 错误定义
│   ├── vector.rs        # 向量模块
│   ├── distance.rs      # 距离计算
│   ├── index.rs         # 索引模块
│   ├── loader.rs        # 数据加载器
│   └── storage/         # 存储模块
│       ├── mod.rs
│       ├── storage.rs
│       ├── vector_storage.rs
│       ├── cf.rs
│       └── error.rs
├── benches/             # 性能基准测试
│   └── vector_search.rs
└── tests/               # 集成测试
```

## 提交信息

我们遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

### 格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 类型

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更改
- `style`: 代码格式（不影响代码运行的变动）
- `refactor`: 重构（既不是新增功能，也不是修改 bug 的代码变动）
- `perf`: 性能优化
- `test`: 增加测试
- `chore`: 构建过程或辅助工具的变动
- `revert`: 回退

### 示例

```
feat(index): add HNSW index implementation

Implement Hierarchical Navigable Small World index for better
search performance on high-dimensional data.

- Add HNSW graph structure
- Implement search algorithm
- Add performance benchmarks

Closes #123
```

## Pull Request 流程

### PR 检查清单

在提交 PR 之前，请确保：

- [ ] 代码通过所有测试 (`make test`)
- [ ] 代码通过 clippy 检查 (`make clippy`)
- [ ] 代码已格式化 (`make fmt`)
- [ ] 添加了必要的文档
- [ ] 添加了必要的测试
- [ ] 更新了 CHANGELOG.md
- [ ] PR 标题遵循提交信息规范

### PR 模板

```markdown
## 描述

简要描述此 PR 的更改。

## 更改类型

- [ ] Bug 修复
- [ ] 新功能
- [ ] 重构
- [ ] 文档更新
- [ ] 性能优化
- [ ] 其他

## 如何测试

描述如何测试这些更改。

## 检查清单

- [ ] 代码通过所有测试
- [ ] 代码通过 clippy 检查
- [ ] 代码已格式化
- [ ] 添加了文档
- [ ] 添加了测试
- [ ] 更新了 CHANGELOG

## 相关 Issue

Closes #issue_number
```

### 审查流程

1. **自动检查**: CI 自动运行测试和检查
2. **代码审查**: 维护者会审查您的代码
3. **讨论**: 如有必要，会进行讨论
4. **批准**: 至少需要一位维护者批准
5. **合并**: 合并到 main 分支

## 问题报告

### Bug 报告模板

```markdown
## Bug 描述

清晰简洁地描述 bug。

## 重现步骤

1. 步骤 1
2. 步骤 2
3. ...

## 预期行为

描述您期望发生什么。

## 实际行为

描述实际发生了什么。

## 环境

- OS: [例如 macOS 12.0]
- Rust: [例如 1.70.0]
- ClawDB: [例如 0.1.0]

## 附加信息

添加任何其他相关信息。
```

## 功能请求

### 功能请求模板

```markdown
## 功能描述

清晰简洁地描述您想要的功能。

## 动机

为什么需要这个功能？它解决了什么问题？

## 详细描述

详细描述功能应该如何工作。

## 替代方案

描述您考虑过的其他解决方案。

## 附加信息

添加任何其他相关信息。
```

## 获取帮助

- **GitHub Issues**: 在 GitHub 上创建 issue
- **文档**: 查看 [README.md](README.md) 和 [API 文档](https://docs.rs/clawdb)
- **讨论**: 在 GitHub Discussions 中提问

## 许可证

通过贡献代码，您同意您的代码将根据项目的 MIT 许可证进行许可。

---

再次感谢您的贡献！🎉

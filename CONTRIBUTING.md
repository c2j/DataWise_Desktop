# 贡献指南 (Contributing Guide)

感谢您对 DataWise Desktop 项目的关注！本文档将帮助您了解如何参与项目开发。

## 📋 目录

- [开发环境准备](#开发环境准备)
- [代码风格](#代码风格)
- [提交规范](#提交规范)
- [分支策略](#分支策略)
- [Pull Request 流程](#pull-request-流程)
- [测试要求](#测试要求)

## 🛠️ 开发环境准备

### 必需工具

1. **Rust 工具链** (1.75+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup component add rustfmt clippy
   ```

2. **系统依赖**
   - **macOS**: `brew install gtk+3`
   - **Ubuntu/Debian**: `sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev`
   - **Windows**: 无需额外依赖

3. **推荐工具**
   - `cargo-watch`: 自动重新编译
   - `cargo-nextest`: 更快的测试运行器

### 克隆仓库

```bash
git clone git@github.com:c2j/DataWise_Desktop.git
cd DataWise_Desktop
cargo build --workspace
cargo test --workspace
```

## 🎨 代码风格

### Rust 代码规范

1. **格式化**: 使用 `rustfmt` (自动应用)
   ```bash
   cargo fmt --all
   ```

2. **Linting**: 通过 Clippy 检查
   ```bash
   cargo clippy --workspace --all-targets -- -D warnings
   ```

3. **命名约定**
   - 类型/Trait: `PascalCase`
   - 函数/变量: `snake_case`
   - 常量: `SCREAMING_SNAKE_CASE`
   - 模块: `snake_case`

4. **文档注释**
   - 所有公共 API 必须有文档注释 (`///`)
   - 模块级文档使用 `//!`
   - 示例代码使用 ` ```rust ` 代码块

### 示例

```rust
/// 执行 SQL 查询并返回结果
///
/// # 参数
///
/// * `sql` - SQL 查询语句
///
/// # 示例
///
/// ```rust
/// let result = execute_sql("SELECT * FROM data").await?;
/// ```
pub async fn execute_sql(sql: &str) -> Result<Vec<RecordBatch>> {
    // 实现...
}
```

## 📝 提交规范

### Commit Message 格式

使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 类型

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式调整（不影响功能）
- `refactor`: 重构（不是新功能也不是 Bug 修复）
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具链相关

### Scope 范围

- `core`: datawise-core 库
- `tauri`: Tauri UI
- `egui`: egui UI
- `tui`: TUI
- `ci`: CI/CD 配置
- `docs`: 文档

### 示例

```
feat(core): 实现 CSV 文件导入功能

- 添加 CsvImporter 结构体
- 支持自动检测分隔符
- 添加进度报告

Closes #42
```

## 🌿 分支策略

### 主要分支

- `main`: 稳定版本，仅接受来自 `develop` 的合并
- `develop`: 开发分支，日常开发在此进行

### 功能分支

从 `develop` 创建，命名格式：

- `feature/<issue-number>-<short-description>` - 新功能
- `fix/<issue-number>-<short-description>` - Bug 修复
- `refactor/<short-description>` - 重构
- `docs/<short-description>` - 文档更新

示例：
```bash
git checkout develop
git pull origin develop
git checkout -b feature/123-csv-import
```

## 🔄 Pull Request 流程

### 1. 创建 PR 前检查

```bash
# 格式化代码
cargo fmt --all

# 运行 Clippy
cargo clippy --workspace --all-targets -- -D warnings

# 运行测试
cargo test --workspace

# 构建所有 crate
cargo build --workspace --all-features
```

### 2. PR 标题格式

与 Commit Message 相同：`<type>(<scope>): <subject>`

### 3. PR 描述模板

```markdown
## 变更说明

简要描述本 PR 的目的和实现方式。

## 变更类型

- [ ] 新功能
- [ ] Bug 修复
- [ ] 重构
- [ ] 文档更新
- [ ] 性能优化

## 测试

- [ ] 添加了单元测试
- [ ] 添加了集成测试
- [ ] 手动测试通过

## Checklist

- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 代码通过 `cargo clippy` 检查
- [ ] 所有测试通过
- [ ] 更新了相关文档
- [ ] 更新了 CHANGELOG.md（如适用）

## 相关 Issue

Closes #<issue-number>
```

### 4. Code Review 要求

- 至少 1 位维护者批准
- 所有 CI 检查通过
- 无未解决的讨论

## ✅ 测试要求

### 单元测试

- 所有新功能必须有单元测试
- 测试覆盖率目标：>80%
- 测试文件放在 `src/` 目录下的 `#[cfg(test)] mod tests`

### 集成测试

- 复杂功能需要集成测试
- 集成测试放在 `tests/` 目录

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定 crate 的测试
cargo test -p datawise-core

# 运行特定测试
cargo test test_csv_import
```

## 🚨 特殊注意事项

### protocol.rs 变更

`datawise-core/src/protocol.rs` 是 UI 和 Core 之间的契约，变更需要：

1. 在 PR 中明确标注 `BREAKING CHANGE`
2. 更新所有受影响的 UI 层代码
3. 更新版本号（遵循 SemVer）
4. 更新迁移文档

### 性能敏感代码

涉及大数据处理的代码需要：

1. 添加性能基准测试（使用 `criterion`）
2. 在 PR 中附上性能对比数据
3. 确保不引入性能退化

## 📞 联系方式

- GitHub Issues: 报告 Bug 或提出功能请求
- GitHub Discussions: 技术讨论和问答

---

再次感谢您的贡献！🎉


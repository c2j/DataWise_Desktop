# DataWise Desktop 测试策略
版本：v1.0  
日期：2025-11-14

---

## 1. 测试金字塔

```
        /\
       /  \  E2E 测试 (5%)
      /    \
     /------\
    /        \  集成测试 (25%)
   /          \
  /------------\
 /              \ 单元测试 (70%)
/________________\
```

### 1.1 单元测试 (70%)

**目标**：测试单个函数/模块的正确性

**覆盖范围**：
- Core 库函数
- Protocol 序列化/反序列化
- 工具函数

**当前状态**：
- ✅ 14 个单元测试通过
- ✅ 覆盖率：~70%

**新增测试**（D.6）：
- ✅ `test_concurrent_queries` - 并发查询处理
- ✅ `test_cancel_task` - 任务取消机制
- ✅ `test_invalid_sql` - SQL 错误处理
- ✅ `test_large_number_handling` - 大数字处理

### 1.2 集成测试 (25%)

**目标**：测试多个模块之间的交互

**覆盖范围**：
- 导入/导出完整流程
- SQL 执行 + 结果处理
- 事件流传递

**当前状态**：
- ✅ 10 个集成测试通过
- ✅ 覆盖率：~25%

**测试用例**：
- `test_csv_import` - CSV 导入
- `test_parquet_import_with_preview` - Parquet 导入
- `test_csv_export` - CSV 导出
- `test_import_export_roundtrip` - 导入导出往返
- `test_import_then_query` - 导入后查询
- `test_import_progress_events` - 进度事件
- `test_import_with_preview_data` - 预览数据
- `test_import_with_overwrite` - 覆盖导入
- `test_json_array_import` - JSON 数组导入
- `test_json_import_with_preview` - JSON 预览

### 1.3 端到端测试 (5%)

**目标**：测试完整的用户工作流

**覆盖范围**：
- Tauri UI 交互
- egui 窗口操作
- tui 键盘输入

**当前状态**：
- ⏳ 计划中（D.6）

**计划测试**：
- Tauri: SQL 执行、文件导入、结果导出
- egui: 窗口创建、SQL 输入、结果显示
- tui: 键盘输入、SQL 执行、结果显示

---

## 2. 测试覆盖率目标

| 模块 | 当前 | 目标 | 状态 |
|------|------|------|------|
| Core | 70% | 80% | ⏳ |
| Protocol | 90% | 95% | ✅ |
| Importer | 85% | 90% | ⏳ |
| Exporter | 85% | 90% | ⏳ |
| Executor | 75% | 85% | ⏳ |

---

## 3. 测试运行命令

### 3.1 单元测试

```bash
# 运行所有单元测试
cargo test -p datawise-core --lib

# 运行特定测试
cargo test -p datawise-core --lib test_sql_execution

# 显示输出
cargo test -p datawise-core --lib -- --nocapture
```

### 3.2 集成测试

```bash
# 运行所有集成测试
cargo test -p datawise-core --test import_export_test

# 运行特定集成测试
cargo test -p datawise-core --test import_export_test test_csv_import
```

### 3.3 性能测试

```bash
# 运行性能基准测试
cargo test -p datawise-core --test performance_benchmark -- --nocapture
```

### 3.4 所有测试

```bash
# 运行所有测试
cargo test --workspace

# 生成覆盖率报告
cargo tarpaulin --workspace --out Html
```

---

## 4. CI/CD 集成

### 4.1 GitHub Actions 工作流

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run tests
        run: cargo test --workspace
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### 4.2 测试要求

- ✅ 所有单元测试必须通过
- ✅ 所有集成测试必须通过
- ✅ 覆盖率不能下降
- ✅ 性能基准不能下降 > 10%

---

## 5. 测试数据管理

### 5.1 测试数据生成

```rust
// 在测试中生成临时数据
#[tokio::test]
async fn test_example() {
    let temp_dir = tempfile::tempdir().unwrap();
    let csv_path = temp_dir.path().join("test.csv");
    
    // 生成测试数据
    std::fs::write(&csv_path, "id,name\n1,Alice\n2,Bob").unwrap();
    
    // 使用测试数据
    // ...
}
```

### 5.2 测试数据清理

- 使用 `tempfile` crate 自动清理
- 测试完成后自动删除临时文件

---

## 6. 测试最佳实践

### 6.1 命名规范

```rust
#[test]
fn test_<function>_<scenario>_<expected_result>() {
    // test_sql_execution_valid_query_returns_results
}
```

### 6.2 测试结构

```rust
#[tokio::test]
async fn test_example() {
    // Arrange: 准备测试数据
    let core = DataWise::new().unwrap();
    
    // Act: 执行操作
    let cmd = Command { /* ... */ };
    core.handle(cmd).await.unwrap();
    
    // Assert: 验证结果
    assert_eq!(result, expected);
}
```

### 6.3 错误处理

```rust
// 使用 ? 操作符
#[tokio::test]
async fn test_example() -> anyhow::Result<()> {
    let core = DataWise::new()?;
    // ...
    Ok(())
}
```

---

## 7. 下一步计划

### 第 1 周
- [ ] 补充 Core 单元测试至 80% 覆盖率
- [ ] 设置 Playwright E2E 测试环境

### 第 2 周
- [ ] 编写 Tauri E2E 测试用例
- [ ] 配置 GitHub Actions CI/CD

### 第 3 周
- [ ] 编写 egui E2E 测试
- [ ] 编写 tui E2E 测试

### 第 4 周
- [ ] 性能回归测试
- [ ] 测试报告生成

---

## 8. 成功标准

- ✅ 单元测试覆盖率 ≥ 80%
- ✅ 集成测试通过率 100%
- ✅ E2E 测试覆盖主要工作流
- ✅ CI/CD 自动化测试
- ✅ 性能基准稳定



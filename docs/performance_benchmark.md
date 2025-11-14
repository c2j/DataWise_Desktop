# DataWise Desktop 性能基准测试报告
版本：v1.0  
日期：2025-11-14  
测试环境：macOS 14.x, Apple Silicon M1/M2

---

## 1. 测试指标

### 1.1 导入性能

| 文件大小 | 行数 | 导入时间 | 速度 | 目标 | 状态 |
|---------|------|---------|------|------|------|
| 1GB CSV | 1M | TBD | TBD MB/s | < 30s | ⏳ |
| 100MB CSV | 100K | TBD | TBD MB/s | < 5s | ⏳ |
| 10MB Parquet | 100K | TBD | TBD MB/s | < 2s | ⏳ |

### 1.2 查询性能

| 查询类型 | 数据量 | 响应时间 | 目标 | 状态 |
|---------|--------|---------|------|------|
| COUNT(*) | 1M | TBD ms | < 100ms | ⏳ |
| LIMIT 10 | 1M | TBD ms | < 50ms | ⏳ |
| WHERE 过滤 | 1M | TBD ms | < 100ms | ⏳ |
| GROUP BY 聚合 | 1M | TBD ms | < 200ms | ⏳ |

### 1.3 渲染性能

| 场景 | 行数 | 帧率 | 目标 | 状态 |
|------|------|------|------|------|
| 虚拟滚动 | 1M | TBD fps | 60 fps | ⏳ |
| 表格渲染 | 10K | TBD fps | 60 fps | ⏳ |
| 图表渲染 | 10K | TBD fps | 30 fps | ⏳ |

---

## 2. 测试方法

### 2.1 导入测试

```bash
cargo test --test performance_benchmark benchmark_csv_import_1gb -- --nocapture
```

**测试步骤**：
1. 生成 1GB CSV 文件（约 100 万行）
2. 记录导入开始时间
3. 执行 `ImportFile` 命令
4. 记录导入完成时间
5. 验证导入结果（行数、列数）

### 2.2 查询测试

```bash
cargo test --test performance_benchmark benchmark_sql_query_response -- --nocapture
```

**测试步骤**：
1. 创建 100 万行测试表
2. 执行多种类型的 SQL 查询
3. 记录每个查询的响应时间

### 2.3 渲染测试

**Tauri 端**：
- 使用 Chrome DevTools 测量帧率
- 虚拟滚动 1M 行数据
- 记录平均帧率和最小帧率

**egui 端**：
- 使用 `egui::Context::request_repaint_after()` 测量帧率
- 虚拟滚动 1M 行数据
- 记录平均帧率

**tui 端**：
- 使用 `crossterm` 事件循环测量帧率
- 渲染 10K 行数据
- 记录平均帧率

---

## 3. 基准测试结果

### 3.1 导入性能结果

```
CSV import time: X.XXs
Import speed: XXX.XX MB/s
```

### 3.2 查询性能结果

```
Count query: X.XXms
Limit query: X.XXms
Filter query: X.XXms
Aggregation query: X.XXms
```

### 3.3 渲染性能结果

```
Tauri 虚拟滚动: XX fps (平均)
egui 虚拟滚动: XX fps (平均)
tui 表格渲染: XX fps (平均)
```

---

## 4. 性能优化建议

### 4.1 导入优化

- [ ] 使用并行 CSV 解析
- [ ] 增加缓冲区大小
- [ ] 使用 Arrow 的批量导入 API

### 4.2 查询优化

- [ ] 添加查询缓存
- [ ] 使用 DuckDB 的查询优化器
- [ ] 实现增量查询

### 4.3 渲染优化

- [ ] 实现虚拟滚动（Tauri/egui）
- [ ] 使用 WebWorker（Tauri）
- [ ] 优化 DOM 更新（Tauri）

---

## 5. 持续监控

### 5.1 CI 集成

在 GitHub Actions 中定期运行性能测试：

```yaml
- name: Run performance benchmarks
  run: cargo test --test performance_benchmark -- --nocapture
```

### 5.2 性能回归检测

- 记录每次提交的性能指标
- 如果性能下降 > 10%，发出警告
- 自动生成性能对比报告

---

## 6. 下一步

- [ ] 完成基准测试实现
- [ ] 收集初始性能数据
- [ ] 识别性能瓶颈
- [ ] 实施优化方案
- [ ] 验证优化效果



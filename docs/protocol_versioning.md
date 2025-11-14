# DataWise Protocol 版本管理规范
版本：v1.0  
日期：2025-11-14  
适用范围：datawise-core/src/protocol.rs

---

## 1. 版本号规范（SemVer）

遵循 [Semantic Versioning 2.0.0](https://semver.org/)：

```
MAJOR.MINOR.PATCH
```

### 1.1 版本号含义

- **MAJOR**：不兼容的 API 变更（UI 需要更新）
- **MINOR**：向后兼容的功能添加（UI 可选更新）
- **PATCH**：向后兼容的 bug 修复（UI 无需更新）

### 1.2 初始版本

- Core 0.1.0：基础 SQL 查询
- Core 0.2.0：导入/导出功能
- Core 1.0.0：接口锁定

---

## 2. 变更流程

### 2.1 PATCH 版本（bug 修复）

**示例**：修复 Progress 事件中的 ETA 计算错误

```rust
// protocol.rs 无需修改
// 仅在 executor.rs 中修复逻辑
```

**流程**：
1. 创建 PR，标题：`fix(core): 修复 Progress ETA 计算`
2. 常规 Code Review
3. 合并后自动发布 PATCH 版本

### 2.2 MINOR 版本（功能添加）

**示例**：添加新的 EventKind::Warning

```rust
pub enum EventKind {
    // ... 现有事件
    Warning(String),  // 新增
}
```

**流程**：
1. 创建 PR，标题：`feat(protocol): 添加 Warning 事件`
2. 在 PR 描述中说明：
   - 新增字段/事件的用途
   - 向后兼容性说明
   - 受影响的 UI 列表
3. 所有 UI Owner 审核
4. 合并后自动发布 MINOR 版本

### 2.3 MAJOR 版本（破坏性变更）

**示例**：修改 EventKind::Finished 的结构

```rust
// 旧版本
Finished {
    row_count: usize,
    column_count: usize,
    preview: String,
}

// 新版本
Finished {
    row_count: usize,
    column_count: usize,
    preview: String,
    execution_time_ms: u64,  // 新增必需字段
}
```

**流程**：
1. 创建 RFC（Request for Comments）PR
2. 标题：`RFC: 修改 EventKind::Finished 结构`
3. 在 PR 描述中说明：
   - 变更原因
   - 迁移指南
   - 预计发布时间
4. 所有 UI Owner 和架构组审核
5. 至少 2 周的讨论期
6. 获得所有 UI Owner 的同意
7. 创建 ADR 记录
8. 合并后发布 MAJOR 版本

---

## 3. ADR（架构决策记录）

### 3.1 ADR 模板

```markdown
# ADR-001: 添加 Warning 事件

## 状态
已接受 / 已拒绝 / 已弃用

## 背景
为什么需要这个变更？

## 决策
具体的变更内容

## 后果
- 正面影响
- 负面影响
- 迁移成本

## 受影响的 UI
- Tauri: 需要更新
- egui: 需要更新
- tui: 需要更新

## 参考
相关的 PR、Issue 链接
```

### 3.2 ADR 存储位置

所有 ADR 存储在 `docs/adr/` 目录：

```
docs/adr/
├── ADR-001-add-warning-event.md
├── ADR-002-modify-finished-event.md
└── README.md
```

### 3.3 ADR 编号规则

- 按时间顺序编号（ADR-001, ADR-002, ...）
- 文件名格式：`ADR-NNN-short-title.md`

---

## 4. 版本发布流程

### 4.1 发布前检查清单

- [ ] 所有测试通过
- [ ] 文档已更新
- [ ] ADR 已创建（如需要）
- [ ] 所有 UI 已适配
- [ ] CHANGELOG 已更新

### 4.2 发布步骤

1. 更新 `datawise-core/Cargo.toml` 中的版本号
2. 更新 `CHANGELOG.md`
3. 创建 Git tag：`v0.2.0`
4. 推送到 GitHub
5. GitHub Actions 自动构建和发布

### 4.3 CHANGELOG 格式

```markdown
## [0.2.0] - 2025-11-14

### Added
- 导入 CSV 文件功能
- 导出 Parquet 文件功能

### Changed
- 改进 Progress 事件的 ETA 计算

### Fixed
- 修复 ExecuteSql 中的内存泄漏

### Deprecated
- 弃用 EventKind::Cancelled（使用 Error 代替）

### Breaking Changes
- 移除 EventKind::Cancelled
```

---

## 5. 兼容性保证

### 5.1 向后兼容性

- MINOR 版本必须向后兼容
- 新增字段必须有默认值
- 新增事件类型必须在 match 中处理

### 5.2 向前兼容性

- UI 应该忽略未知的事件类型
- UI 应该忽略未知的字段

---

## 6. 版本支持周期

- **最新版本**：完全支持
- **前一个 MINOR 版本**：bug 修复支持
- **更早版本**：不支持

---

## 7. 通信规范

### 7.1 版本变更通知

当发布新版本时，通知所有 UI Owner：

```
主题：DataWise Protocol v0.2.0 发布

内容：
- 新增功能：...
- 破坏性变更：...
- 迁移指南：...
- 预计更新截止日期：...
```

### 7.2 版本兼容性矩阵

维护一个兼容性矩阵：

| Core 版本 | Tauri | egui | tui |
|----------|-------|------|-----|
| 0.1.x | ✅ | ✅ | ✅ |
| 0.2.x | ✅ | ✅ | ✅ |
| 1.0.x | ✅ | ✅ | ✅ |



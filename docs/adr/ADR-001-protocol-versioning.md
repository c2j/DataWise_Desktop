# ADR-001: 引入 Protocol 版本管理和 SemVer

## 状态
✅ 已接受

## 背景

在阶段 B 和 C 的开发中，我们发现 protocol.rs 的变更频繁，导致三个 UI 框架需要频繁同步更新。为了避免接口频繁 break，我们需要建立一套严格的版本管理规范。

**问题**：
- 没有明确的版本号规范
- 变更流程不清晰
- UI 团队无法预测何时需要更新
- 没有向后兼容性保证

## 决策

采用 Semantic Versioning 2.0.0 规范，并建立以下流程：

### 版本号规范

```
MAJOR.MINOR.PATCH
```

- **MAJOR**：不兼容的 API 变更（UI 需要更新）
- **MINOR**：向后兼容的功能添加（UI 可选更新）
- **PATCH**：向后兼容的 bug 修复（UI 无需更新）

### 变更流程

1. **PATCH 版本**：常规 Code Review，自动发布
2. **MINOR 版本**：所有 UI Owner 审核，自动发布
3. **MAJOR 版本**：RFC 讨论 2 周，所有 UI Owner 同意，创建 ADR，发布

### ADR 记录

所有 MAJOR 版本变更必须创建 ADR 记录，存储在 `docs/adr/` 目录。

## 后果

### 正面影响

- ✅ 清晰的版本管理规范
- ✅ 减少接口变更频率
- ✅ UI 团队可以提前规划更新
- ✅ 向后兼容性保证
- ✅ 便于追踪变更历史

### 负面影响

- ⚠️ 增加了变更流程的复杂性
- ⚠️ MAJOR 版本变更需要 2 周讨论期
- ⚠️ 需要维护 ADR 文档

### 迁移成本

- 低：仅影响新的变更流程
- 现有代码无需修改

## 受影响的 UI

- **Tauri**：需要遵循版本管理规范
- **egui**：需要遵循版本管理规范
- **tui**：需要遵循版本管理规范

## 实施计划

1. **第 1 周**：发布 protocol_versioning.md 文档
2. **第 2 周**：创建 ADR 模板和示例
3. **第 3 周**：在 CI 中添加版本检查
4. **第 4 周**：培训所有开发者

## 参考

- [Semantic Versioning 2.0.0](https://semver.org/)
- [Architecture Decision Records](https://adr.github.io/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## 相关 Issue/PR

- 阶段 D：统一协议 & 性能/体验打磨



# DataWise Desktop 文档中心

欢迎来到 DataWise Desktop 项目文档中心。本目录包含项目的所有设计、分析和开发文档。

---

## 📚 文档导航

### 🎯 快速开始

**新成员必读**：
1. 📖 [执行摘要](./executive_summary.md) - 5 分钟了解项目全貌
2. 🚀 [快速开始指南](./quick_start_guide.md) - 搭建开发环境并开始编码
3. 📋 [可行性分析](./feasibility_analysis.md) - 深入了解技术方案

### 📐 设计文档

| 文档 | 描述 | 目标读者 |
|------|------|----------|
| [设计规格书 v1](./design_v1.md) | 架构设计、模块组织、开发指南 | 全体开发者 |
| [UI 规格说明 v1](./design_v1-ui_spec.md) | 三端 UI 交互设计、视觉规范 | UI 团队 |
| [架构决策记录](./architecture_decisions.md) | 关键技术决策及理由（ADR） | 架构师、Tech Lead |

### 📊 分析报告

| 文档 | 描述 | 目标读者 |
|------|------|----------|
| [可行性分析](./feasibility_analysis.md) | 技术可行性、风险评估、开发计划 | 项目经理、技术负责人 |
| [执行摘要](./executive_summary.md) | 高层决策摘要、资源需求、批准流程 | 管理层、决策者 |

### 🛠️ 开发指南

| 文档 | 描述 | 目标读者 |
|------|------|----------|
| [快速开始指南](./quick_start_guide.md) | 环境搭建、项目结构、开发规范 | 新加入开发者 |
| CONTRIBUTING.md *(待创建)* | 贡献指南、PR 流程、代码规范 | 所有贡献者 |

---

## 🗺️ 项目概览

### 核心理念

**一份业务逻辑，三端复用**

```
┌─────────────────────────────────────┐
│     UI Layer (可插拔)                │
│  [Tauri]  [egui]  [tui]  [未来...]  │
└──────────────┬──────────────────────┘
               │ 事件订阅 + 命令发送
┌──────────────▼──────────────────────┐
│        datawise-core                │
│  (DuckDB + Arrow + 异步任务管理)     │
└─────────────────────────────────────┘
```

### 技术栈

| 层级 | 技术 | 版本 |
|------|------|------|
| **查询引擎** | DuckDB | 1.1 |
| **数据格式** | Apache Arrow | 53 |
| **异步运行时** | Tokio | 1.x |
| **Web UI** | Tauri + React | 2.x |
| **原生 UI** | egui | 0.29+ |
| **终端 UI** | ratatui | 0.28+ |

### 开发时间线（8 周）

```
Week 1-2  : Core 基础设施（SQL 查询 + 事件流）
Week 3-4  : Core 完整功能（导入导出 + 任务管理）
Week 3-5  : UI 并行开发（三个 MVP）
Week 6-7  : 集成测试 + 性能优化
Week 8    : 文档 + 发布 v1.0-beta
```

---

## 📖 阅读路径推荐

### 路径 1: 管理层/决策者

1. [执行摘要](./executive_summary.md) - 了解项目价值和资源需求
2. [可行性分析](./feasibility_analysis.md) - 评估风险和时间规划
3. [设计规格书](./design_v1.md) - 理解技术方案

### 路径 2: 架构师/Tech Lead

1. [设计规格书](./design_v1.md) - 掌握整体架构
2. [架构决策记录](./architecture_decisions.md) - 理解关键决策
3. [可行性分析](./feasibility_analysis.md) - 了解技术挑战和解决方案

### 路径 3: Core 团队开发者

1. [快速开始指南](./quick_start_guide.md) - 搭建环境
2. [设计规格书 - 第 3 节](./design_v1.md#3-datawise-core-规格) - Core API 规格
3. [架构决策记录](./architecture_decisions.md) - 理解设计理由

### 路径 4: UI 团队开发者

1. [快速开始指南](./quick_start_guide.md) - 搭建环境
2. [UI 规格说明](./design_v1-ui_spec.md) - UI 交互设计
3. [设计规格书 - 第 4 节](./design_v1.md#4-ui-团队开发指南) - UI 开发指南

### 路径 5: QA/测试工程师

1. [执行摘要 - 成功标准](./executive_summary.md#七成功标准) - 测试目标
2. [可行性分析 - 性能指标](./feasibility_analysis.md#62-性能指标) - 性能基准
3. [设计规格书 - 第 8 节](./design_v1.md#8-风险与解除) - 风险点

---

## 🎯 关键里程碑

| 里程碑 | 日期 | 交付物 | 状态 |
|--------|------|--------|------|
| **M0: 项目启动** | Week 1 | Workspace + CI/CD | ⏳ 进行中 |
| **M1: Core 0.1.0** | Week 2 | 基础 SQL 查询 | ⏳ 待开始 |
| **M2: Core 0.2.0** | Week 4 | 导入导出功能 | ⏳ 待开始 |
| **M3: UI MVP** | Week 5 | 三个 UI 可用版本 | ⏳ 待开始 |
| **M4: 接口锁定** | Week 7 | Core 1.0.0 | ⏳ 待开始 |
| **M5: Beta 发布** | Week 8 | v1.0-beta | ⏳ 待开始 |

---

## 🔗 相关资源

### 内部资源

- 📊 [项目看板](https://github.com/your-org/datawise-desktop/projects) *(待创建)*
- 💬 [团队讨论区](https://github.com/your-org/datawise-desktop/discussions) *(待创建)*
- 🐛 [问题追踪](https://github.com/your-org/datawise-desktop/issues) *(待创建)*

### 外部文档

- [DuckDB 官方文档](https://duckdb.org/docs/)
- [Apache Arrow Rust 文档](https://docs.rs/arrow/)
- [Tauri 官方指南](https://tauri.app/v2/guides/)
- [egui 官方文档](https://docs.rs/egui/)
- [ratatui 官方文档](https://docs.rs/ratatui/)

---

## 📝 文档维护

### 更新频率

| 文档类型 | 更新频率 | 负责人 |
|----------|----------|--------|
| 设计文档 | 重大变更时 | 架构组 |
| ADR | 每次架构决策 | 架构组 |
| 开发指南 | 每周 | Tech Lead |
| 里程碑状态 | 每周 | 项目经理 |

### 版本历史

| 版本 | 日期 | 变更说明 | 作者 |
|------|------|----------|------|
| v1.0 | 2025-11-13 | 初始版本，完成可行性分析和开发计划 | 架构组 |

---

## 🤝 贡献

发现文档问题或有改进建议？

1. 提交 [Issue](https://github.com/your-org/datawise-desktop/issues)
2. 或直接提交 PR 修改文档

---

## 📧 联系方式

- **架构组**: architecture@your-org.com
- **项目经理**: pm@your-org.com
- **技术支持**: dev@your-org.com

---

**最后更新**: 2025-11-13  
**文档版本**: v1.0  
**维护者**: 架构组


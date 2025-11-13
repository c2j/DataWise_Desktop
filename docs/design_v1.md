# DataWise Desktop 设计规格书

版本：v0.1  
作者：架构组  
日期：2025-11-13  
目标：统一 Rust 工作区，支持 Tauri / egui / tui 三种 UI 并行开发；业务逻辑 100% 复用，各团队仅实现"事件订阅 + 命令发送"。

---

## 1. 总体架构

```
+-----------------------------+
|          UI Layer           |
|  [Tauri] [egui] [tui] ...   |  ← 各团队独立 crate，仅依赖 core
+------|-------------|--------+
       | 订阅 UiEvent |
       | 发送 Command |
+------v-------------v--------+
|       datawise-core         |
|  (query / import / export)  |
|  异步 tokio + DuckDB        |
+------|-------------|--------+
       | SQL + Arrow
+------v-------------v--------+
|     Data Sources            |
| CSV Parquet MySQL PG ...    |
+-----------------------------+
```

**原则**  
- core 不依赖任何 UI crate；UI 不直接调用 DuckDB / 文件 IO。  
- 所有慢任务异步，进度/结果/错误通过 broadcast channel 推送。  
- 新增第四套 UI 只需：①订阅 UiEvent ②调用 core Command ③自己画界面。

---

## 2. 模块组织（Cargo Workspace）

```
DataWise/
├─ Cargo.toml               ← workspace 定义
├─ datawise-core/           ← 业务库团队维护
├─ datawise-tauri/          ← Web UI 团队
├─ datawise-egui/           ← 原生轻量团队
├─ datawise-tui/            ← 终端团队
└─ docs/                    ← 本文档 + 协议版本记录
```

**根 Cargo.toml 关键片段**
```toml
[workspace]
members = ["datawise-core", "datawise-tauri", "datawise-egui", "datawise-tui"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
arrow = "53"
duckdb = "1.1"
```

---

## 3. datawise-core 规格

### 3.1 公共数据结构
```rust
// 统一放在 src/protocol.rs，serde 编解码保证跨语言
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UiEvent {
    pub task_id: u64,
    pub kind: EventKind,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventKind {
    Started,
    Progress { pct: u8 },
    Finished(Vec<RecordBatch>),  // Arrow 标准格式
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    pub task_id: u64,
    pub cmd_type: CmdType,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum CmdType {
    ExecuteSql { sql: String },
    ImportFile { path: String, fmt: FileFmt },
    ExportFile { id: u64, path: String, fmt: FileFmt },
    Cancel { task_id: u64 },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum FileFmt { Csv, Parquet, Json, MySql, Postgres }
```

### 3.2 核心 API（业务团队负责实现）
```rust
impl DataWise {
    pub fn new() -> Self;   // 初始化 DuckDB 内存实例
    pub fn subscribe(&self) -> broadcast::Receiver<UiEvent>;
    pub async fn handle(&self, cmd: Command) -> Result<()>;
}
```
说明：  
- `handle` 内部分发异步任务，任何进度/结果/错误均通过 `UiEvent` 广播。  
- 取消机制：内部维护 `DashMap<task_id, JoinHandle<()>>`，收到 `Cancel` 直接 `handle.abort()`。  

### 3.3 内部子模块（仅 core 可见）
```
src/
├─ lib.rs          ← 公开 API & DataWise 结构
├─ protocol.rs     ← 上述数据结构
├─ executor.rs     ← SQL 解析、DuckDB 执行
├─ importer/       ← 各格式导入，统一转 Arrow
├─ exporter/       ← Arrow → 目标格式
├─ cancel.rs       ← 任务句柄管理
└─ util.rs         ← 日志、路径、编码
```
**对外隐藏 DuckDB 细节**，保证以后可换引擎。

---

## 4. UI 团队开发指南

### 4.1 通用流程
1. 依赖 `datawise-core = { path = "../datawise-core" }`  
2. 初始化 `let core = Arc::new(DataWise::new());`  
3. 启动时 `let mut rx = core.subscribe();`  
4. 用户操作 → 生成 `Command { task_id: rand::random(), cmd_type: ... }`  
5. 调用 `tokio::spawn(core.handle(cmd));`  
6. 在事件循环里消费 `rx.try_recv()` 刷新界面。

### 4.2 线程/阻塞要求
- 禁止在 UI 线程直接调用 `core.handle()`，必须 `spawn`；  
- 收到 `EventKind::Finished(batch)` 后，UI 按需转存本地状态即可释放内存。  

### 4.3 资源与权限
- **Tauri**：前端静态文件放 `src-tauri/assets/`，权限白名单只开 `fs:read-write` 和 `dialog:open-save`。  
- **egui**：字体、图标用 `include_bytes!` 编译期嵌入，不得运行时写可执行同级目录。  
- **tui**：临时文件统一写 `std::env::temp_dir()`，退出时清理。

---

## 5. 接口版本管理

- `datawise-core` 的 `protocol.rs` 变更即**接口版本升级**。  
- 采用 **SemVer**：  
  - 新增字段/枚举变体 → 小版本（0.1 → 0.2）  
  - 删除或改名 → 大版本（0.x → 1.x）  
- 所有 UI crate 在 `Cargo.toml` 用 `exact = "0.x.y"` 锁定，升级需**联合测试**通过后方可合并主干。

---

## 6. 构建与交付

| 产物 | 构建命令 | 输出路径 | 体积目标 |
|---|---|---|---|
| Tauri 安装包 | `npm run tauri build` | `src-tauri/target/release/bundle/` | ≤ 50 MB |
| egui 单文件 | `cargo build -p datawise-egui --release` | `target/release/datawise-egui` | ≤ 5 MB |
| tui 二进制 | `cargo build -p datawise-tui --release` | `target/release/datawise-tui` | ≤ 3 MB |

CI（GitHub Actions）已配置：**push tag v* 即自动打三包并生成 SHA256 校验文件**。

---

## 7. 并行研发里程碑（供各团队排期）

| 周 | 任务 | 负责方 |
|----|------|--------|
| 0-1 | core 初版：ExecuteSql + 事件流 | 业务团队 |
| 1-2 | core 导入导出 + Cancel | 业务团队 |
| 1-2 | Tauri MVP：SQL 输入 → 表格 → 导出 | Web UI 团队 |
| 2-3 | egui MVP：同功能 | 原生团队 |
| 2-3 | tui MVP：同功能 | 终端团队 |
| 3 | 联合集成测试 & 接口锁定 0.1.0 | 全体 |
| 4 | 文档、性能压测、签名发布 1.0-beta | 全体 |

---

## 8. 风险与解除

| 风险 | 解除方案 |
|---|---|
| core 接口频繁变 | 任何 PR 改 protocol.rs 需@所有 UI owner 评审 |
| UI 框架升级 break | CI 每周跑 `cargo update` 并自动提 PR，未合并前禁止发版 |
| 大文件渲染卡顿 | 统一采用 Arrow + 虚拟滚动，core 提供 `limit/offset` 二次查询接口 |

---

## 9. 参考文档链接（内部）

- DuckDB 官方 Rust 客户端：`<internal-wiki>/duckdb-rs`  
- Tauri 3 插件列表：`<internal-wiki>/tauri-plugins`  
- egui 官方示例集：`<internal-wiki>/egui-demo`  
- ratatui  cookbook：`<internal-wiki>/ratatui-recipes`  

---

**批准**  
架构组 / 2025-11-13
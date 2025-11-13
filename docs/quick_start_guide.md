# DataWise Desktop 快速开始指南

本指南帮助各团队快速启动开发工作。

---

## 一、前置准备

### 1.1 开发环境

**必需工具**:
```bash
# Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# 版本要求
rustc --version  # >= 1.75.0
cargo --version  # >= 1.75.0
```

**Tauri 团队额外需要**:
```bash
# Node.js
node --version  # >= 18.0.0
npm --version   # >= 9.0.0

# 系统依赖（macOS）
brew install webkit2gtk

# 系统依赖（Ubuntu）
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential curl wget file libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev
```

**egui 团队额外需要**:
```bash
# Linux 需要额外依赖
sudo apt install libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

### 1.2 克隆仓库

```bash
git clone https://github.com/your-org/datawise-desktop.git
cd datawise-desktop
```

---

## 二、项目结构（待创建）

```
DataWise_Desktop/
├── Cargo.toml                 # Workspace 配置
├── .github/
│   └── workflows/
│       └── ci.yml             # CI/CD 配置
├── docs/
│   ├── design_v1.md           # 架构设计
│   ├── design_v1-ui_spec.md   # UI 规格
│   ├── feasibility_analysis.md # 可行性分析
│   └── architecture_decisions.md # ADR
├── datawise-core/             # 核心业务库
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── protocol.rs        # 公共接口
│   │   ├── executor.rs        # SQL 执行器
│   │   ├── importer/          # 导入器
│   │   ├── exporter/          # 导出器
│   │   └── cancel.rs          # 任务管理
│   └── tests/
├── datawise-tauri/            # Web UI
│   ├── Cargo.toml
│   ├── src-tauri/             # Rust 后端
│   └── src/                   # React 前端
├── datawise-egui/             # 原生 UI
│   ├── Cargo.toml
│   └── src/
└── datawise-tui/              # 终端 UI
    ├── Cargo.toml
    └── src/
```

---

## 三、Phase 0 任务清单（Week 1）

### 3.1 创建 Workspace（架构组）

**创建根 Cargo.toml**:
```toml
[workspace]
members = [
    "datawise-core",
    "datawise-tauri",
    "datawise-egui",
    "datawise-tui",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
arrow = "53"
duckdb = "1.1"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
```

### 3.2 初始化 datawise-core

```bash
cargo new --lib datawise-core
cd datawise-core
```

**编辑 Cargo.toml**:
```toml
[package]
name = "datawise-core"
version.workspace = true
edition.workspace = true

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
arrow = { workspace = true }
duckdb = { workspace = true }
anyhow = { workspace = true }
dashmap = "6"
```

**创建 src/protocol.rs**:
```rust
use serde::{Deserialize, Serialize};
use arrow::record_batch::RecordBatch;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UiEvent {
    pub task_id: u64,
    pub kind: EventKind,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventKind {
    Started,
    Progress { 
        pct: u8,
        bytes_processed: u64,
        total_bytes: u64,
    },
    Finished(Vec<RecordBatch>),
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
pub enum FileFmt {
    Csv,
    Parquet,
    Json,
}
```

### 3.3 初始化 UI Crates

**Tauri**:
```bash
npm create tauri-app@latest datawise-tauri
# 选择: React + TypeScript
```

**egui**:
```bash
cargo new datawise-egui
# 添加依赖到 Cargo.toml:
# eframe = "0.29"
# egui_extras = { version = "0.29", features = ["all_loaders"] }
```

**tui**:
```bash
cargo new datawise-tui
# 添加依赖到 Cargo.toml:
# ratatui = "0.28"
# crossterm = "0.28"
# tui-textarea = "0.6"
```

### 3.4 配置 CI/CD

**创建 .github/workflows/ci.yml**:
```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Run tests
        run: cargo test --workspace
      
      - name: Run clippy
        run: cargo clippy --workspace -- -D warnings
      
      - name: Check formatting
        run: cargo fmt --all -- --check
```

---

## 四、各团队开发指南

### 4.1 Core 团队

**优先级任务**:
1. 实现 `protocol.rs`（Week 1）
2. 集成 DuckDB（Week 1-2）
3. 实现 `ExecuteSql`（Week 2）
4. 提供 Mock 数据生成器（Week 2）

**示例代码**:
```rust
// src/lib.rs
use tokio::sync::broadcast;

pub struct DataWise {
    tx: broadcast::Sender<UiEvent>,
    // DuckDB connection
}

impl DataWise {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        // TODO: 初始化 DuckDB
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<UiEvent> {
        self.tx.subscribe()
    }

    pub async fn handle(&self, cmd: Command) -> anyhow::Result<()> {
        // TODO: 实现命令处理
        Ok(())
    }
}
```

### 4.2 Tauri 团队

**等待 Core 完成**: Week 2  
**可提前准备**:
- 搭建 React 项目结构
- 集成 Monaco Editor
- 设计组件层次结构

**示例代码**:
```rust
// src-tauri/src/main.rs
#[tauri::command]
async fn execute_sql(sql: String) -> Result<String, String> {
    // TODO: 调用 datawise-core
    Ok("[]".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![execute_sql])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 4.3 egui 团队

**等待 Core 完成**: Week 2  
**可提前准备**:
- 实现布局系统
- 测试虚拟滚动性能
- 准备字体资源

### 4.4 tui 团队

**等待 Core 完成**: Week 2  
**可提前准备**:
- 实现终端布局
- 测试 tui-textarea
- 设计快捷键映射

---

## 五、开发规范

### 5.1 代码风格

```bash
# 格式化
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings
```

### 5.2 提交规范

```
<type>(<scope>): <subject>

<body>

<footer>
```

**类型**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

**示例**:
```
feat(core): implement ExecuteSql command

- Add DuckDB integration
- Support Arrow RecordBatch output
- Add progress reporting

Closes #123
```

### 5.3 PR 流程

1. 创建 feature 分支: `git checkout -b feat/your-feature`
2. 提交代码并推送
3. 创建 PR，填写模板
4. 等待 CI 通过
5. 请求 Code Review
6. 合并到 `develop` 分支

---

## 六、常见问题

**Q: DuckDB 编译失败？**  
A: 确保安装了 C++ 编译器（MSVC/GCC/Clang）

**Q: Tauri 启动失败？**  
A: 检查系统依赖是否安装完整

**Q: 如何调试 Core？**  
A: 使用 `RUST_LOG=debug cargo test`

---

**下一步**: 查看 [可行性分析](./feasibility_analysis.md) 了解详细计划


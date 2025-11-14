# DataWise Core API 使用指南

## 概述

DataWise Core 是一个基于 DuckDB 的数据查询引擎，采用**事件驱动架构**。UI 层通过发送 `Command` 与 Core 通信，Core 通过广播 `UiEvent` 返回结果。

## 核心概念

### 架构流程

```
UI Layer (Tauri/egui/tui)
    ↓ 发送 Command
DataWise Core (处理命令)
    ↓ 广播 UiEvent
UI Layer (订阅事件)
```

### 事件驱动模型

每个命令执行会产生一系列事件：

1. **Started** - 任务开始
2. **Progress** - 进度更新（可选）
3. **Finished** - 任务完成，包含结果
4. **Error** - 任务失败

## 公开 API

### DataWise 结构体

```rust
pub struct DataWise { ... }

impl DataWise {
    /// 创建新实例，初始化 DuckDB 内存数据库
    pub fn new() -> Result<Self>
    
    /// 订阅事件流
    pub fn subscribe(&self) -> broadcast::Receiver<UiEvent>
    
    /// 处理命令（异步）
    pub async fn handle(&self, cmd: Command) -> Result<()>
}
```

### 命令类型 (CmdType)

```rust
pub enum CmdType {
    /// 执行 SQL 查询
    ExecuteSql { sql: String },
    
    /// 导入文件（CSV/Parquet）
    ImportFile { 
        path: String, 
        fmt: FileFmt, 
        table_name: Option<String> 
    },
    
    /// 导出数据
    ExportFile { 
        source: String, 
        path: String, 
        fmt: FileFmt 
    },
    
    /// 取消任务
    Cancel { task_id: u64 },
}
```

### 事件类型 (EventKind)

```rust
pub enum EventKind {
    Started,
    
    Progress {
        pct: u8,
        bytes_processed: u64,
        total_bytes: u64,
        eta_seconds: Option<u32>,
    },
    
    Finished {
        row_count: usize,
        column_count: usize,
        preview: String,  // JSON 格式
    },
    
    Error(String),
}
```

## 使用示例

### 基础 SQL 查询

```rust
use datawise_core::{DataWise, Command, CmdType};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 创建 Core 实例
    let core = DataWise::new()?;
    
    // 2. 订阅事件
    let mut rx = core.subscribe();
    
    // 3. 发送命令
    let cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ExecuteSql {
            sql: "SELECT 1 as num, 'hello' as text".to_string(),
        },
    };
    core.handle(cmd).await?;
    
    // 4. 接收事件
    while let Ok(event) = rx.recv().await {
        match event.kind {
            EventKind::Started => println!("Task started"),
            EventKind::Finished { row_count, column_count, preview } => {
                println!("Rows: {}, Cols: {}", row_count, column_count);
                println!("Preview: {}", preview);
            }
            EventKind::Error(e) => println!("Error: {}", e),
            _ => {}
        }
    }
    
    Ok(())
}
```

### 多订阅者模式

```rust
let core = DataWise::new()?;

// 多个 UI 组件可以独立订阅
let mut rx1 = core.subscribe();
let mut rx2 = core.subscribe();

// 发送命令后，所有订阅者都会收到事件
core.handle(cmd).await?;
```

## 最佳实践

### 1. 错误处理

- 始终检查 `handle()` 的返回值
- 监听 `EventKind::Error` 事件
- 为用户提供清晰的错误消息

### 2. 任务管理

- 为每个命令分配唯一的 `task_id`
- 使用 `task_id` 关联命令和事件
- 支持多个并发任务

### 3. 性能考虑

- SQL 查询在主线程执行（DuckDB 足够快）
- 预览数据限制为 10 行
- 大结果集通过分页处理

## 支持的 SQL 功能

### 已支持

- ✅ SELECT 查询（单表、多表）
- ✅ 基本聚合（COUNT, SUM, AVG, MIN, MAX）
- ✅ WHERE 条件过滤
- ✅ GROUP BY 分组
- ✅ ORDER BY 排序
- ✅ JOIN 操作
- ✅ 常见函数（CAST, COALESCE 等）

### 计划支持

- 📋 文件导入（CSV/Parquet）
- 📋 数据导出
- 📋 事务支持
- 📋 自定义函数

## 常见问题

**Q: 如何处理大结果集？**
A: Core 返回前 10 行预览，完整数据需要通过分页查询或导出功能获取。

**Q: 支持并发查询吗？**
A: 支持。使用不同的 `task_id` 发送多个命令，Core 会并发处理。

**Q: 数据持久化吗？**
A: 当前使用内存数据库，重启后数据丢失。计划支持文件导入/导出。


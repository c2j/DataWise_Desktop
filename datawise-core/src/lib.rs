//! DataWise Core - 数据分析引擎核心库
//!
//! 本库提供了基于 DuckDB 的数据查询、导入、导出功能。
//! 采用事件驱动架构，支持异步任务和进度报告。
//!
//! # 架构
//!
//! ```text
//! UI Layer (Tauri/egui/tui)
//!     ↓ Command
//! DataWise Core
//!     ↓ UiEvent
//! UI Layer (订阅事件)
//! ```
//!
//! # 示例
//!
//! ```no_run
//! use datawise_core::{DataWise, Command, CmdType};
//!
//! #[tokio::main]
//! async fn main() {
//!     let core = DataWise::new();
//!     let mut rx = core.subscribe();
//!
//!     // 发送 SQL 查询命令
//!     let cmd = Command {
//!         task_id: 1,
//!         cmd_type: CmdType::ExecuteSql {
//!             sql: "SELECT * FROM data".to_string(),
//!         },
//!     };
//!
//!     core.handle(cmd).await.unwrap();
//!
//!     // 接收事件
//!     while let Ok(event) = rx.recv().await {
//!         println!("Event: {:?}", event);
//!     }
//! }
//! ```

pub mod protocol;

pub use protocol::{Command, CmdType, EventKind, FileFmt, UiEvent};

use anyhow::Result;
use tokio::sync::broadcast;

/// DataWise 核心引擎
pub struct DataWise {
    /// 事件广播发送器
    tx: broadcast::Sender<UiEvent>,
}

impl DataWise {
    /// 创建新的 DataWise 实例
    ///
    /// 初始化 DuckDB 内存数据库和事件广播通道。
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);

        tracing::info!("DataWise initialized");

        Self { tx }
    }

    /// 订阅 UI 事件
    ///
    /// 返回一个接收器，可以接收所有 UI 事件。
    pub fn subscribe(&self) -> broadcast::Receiver<UiEvent> {
        self.tx.subscribe()
    }

    /// 处理命令
    ///
    /// 异步处理来自 UI 的命令，并通过事件通道推送进度和结果。
    pub async fn handle(&self, cmd: Command) -> Result<()> {
        tracing::info!("Handling command: {:?}", cmd);

        // 发送启动事件
        let _ = self.tx.send(UiEvent {
            task_id: cmd.task_id,
            kind: EventKind::Started,
        });

        // TODO: 实现命令处理逻辑
        match cmd.cmd_type {
            CmdType::ExecuteSql { sql } => {
                tracing::info!("Executing SQL: {}", sql);
                // TODO: 实现 SQL 执行
            }
            CmdType::ImportFile { path, fmt, table_name: _ } => {
                tracing::info!("Importing file: {} ({:?})", path, fmt);
                // TODO: 实现文件导入
            }
            CmdType::ExportFile { source: _, path, fmt } => {
                tracing::info!("Exporting to: {} ({:?})", path, fmt);
                // TODO: 实现文件导出
            }
            CmdType::Cancel { task_id } => {
                tracing::info!("Cancelling task: {}", task_id);
                // TODO: 实现任务取消
            }
        }

        Ok(())
    }
}

impl Default for DataWise {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_datawise_creation() {
        let core = DataWise::new();
        let mut rx = core.subscribe();

        // 测试事件订阅
        assert!(rx.try_recv().is_err()); // 应该没有事件
    }

    #[tokio::test]
    async fn test_command_handling() {
        let core = DataWise::new();
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 1,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT 1".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 应该收到 Started 事件
        let event = rx.recv().await.unwrap();
        assert_eq!(event.task_id, 1);
        matches!(event.kind, EventKind::Started);
    }
}

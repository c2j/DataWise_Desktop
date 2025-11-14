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
//! async fn main() -> anyhow::Result<()> {
//!     let core = DataWise::new()?;
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
//!     core.handle(cmd).await?;
//!
//!     // 接收事件
//!     while let Ok(event) = rx.recv().await {
//!         println!("Event: {:?}", event);
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod executor;
pub mod protocol;

pub use protocol::{Command, CmdType, EventKind, FileFmt, UiEvent};

use anyhow::Result;
use executor::Executor;
use std::sync::Arc;
use tokio::sync::broadcast;

/// DataWise 核心引擎
pub struct DataWise {
    /// 事件广播发送器
    tx: broadcast::Sender<UiEvent>,
    /// SQL 执行器（使用 Arc 以支持跨线程共享）
    executor: Arc<Executor>,
}

impl DataWise {
    /// 创建新的 DataWise 实例
    ///
    /// 初始化 DuckDB 内存数据库和事件广播通道。
    pub fn new() -> Result<Self> {
        let (tx, _) = broadcast::channel(100);
        let executor = Arc::new(Executor::new()?);

        tracing::info!("DataWise initialized with DuckDB executor");

        Ok(Self { tx, executor })
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

        // 处理命令
        let result = match cmd.cmd_type {
            CmdType::ExecuteSql { sql } => {
                tracing::info!("Executing SQL: {}", sql);
                self.execute_sql(cmd.task_id, &sql).await
            }
            CmdType::ImportFile { path, fmt, table_name: _ } => {
                tracing::info!("Importing file: {} ({:?})", path, fmt);
                // TODO: 实现文件导入
                Ok(())
            }
            CmdType::ExportFile { source: _, path, fmt } => {
                tracing::info!("Exporting to: {} ({:?})", path, fmt);
                // TODO: 实现文件导出
                Ok(())
            }
            CmdType::Cancel { task_id } => {
                tracing::info!("Cancelling task: {}", task_id);
                // TODO: 实现任务取消
                Ok(())
            }
        };

        // 发送结果事件
        if let Err(ref e) = result {
            let _ = self.tx.send(UiEvent {
                task_id: cmd.task_id,
                kind: EventKind::Error(e.to_string()),
            });
        }

        result
    }

    /// 执行 SQL 查询
    async fn execute_sql(&self, task_id: u64, sql: &str) -> Result<()> {
        // 直接执行 SQL（DuckDB 操作通常很快，不需要 spawn_blocking）
        let batches = self.executor.execute(sql)?;

        // 计算结果统计
        let row_count = batches.iter().map(|b| b.num_rows()).sum();
        let column_count = batches.first().map(|b| b.num_columns()).unwrap_or(0);

        // 生成预览数据（前 10 行）
        let preview = self.generate_preview(&batches)?;

        // 发送完成事件
        let _ = self.tx.send(UiEvent {
            task_id,
            kind: EventKind::Finished {
                row_count,
                column_count,
                preview,
            },
        });

        Ok(())
    }

    /// 生成数据预览（JSON 格式）
    fn generate_preview(&self, batches: &[arrow::record_batch::RecordBatch]) -> Result<String> {
        let mut preview_rows = Vec::new();
        let mut total_rows = 0;

        for batch in batches {
            if total_rows >= 10 {
                break;
            }

            let schema = batch.schema();
            let num_rows = batch.num_rows().min(10 - total_rows);

            for row_idx in 0..num_rows {
                let mut row_obj = serde_json::Map::new();

                for (col_idx, field) in schema.fields().iter().enumerate() {
                    let column = batch.column(col_idx);
                    let value = self.array_value_to_json(column, row_idx);
                    row_obj.insert(field.name().clone(), value);
                }

                preview_rows.push(serde_json::Value::Object(row_obj));
                total_rows += 1;
            }
        }

        Ok(serde_json::to_string(&preview_rows)?)
    }

    /// 将 Arrow Array 的值转换为 JSON
    fn array_value_to_json(&self, array: &dyn arrow::array::Array, index: usize) -> serde_json::Value {
        use arrow::array::*;
        use serde_json::json;

        if array.is_null(index) {
            return json!(null);
        }

        // 尝试不同的类型转换
        if let Some(arr) = array.as_any().downcast_ref::<BooleanArray>() {
            json!(arr.value(index))
        } else if let Some(arr) = array.as_any().downcast_ref::<Int32Array>() {
            json!(arr.value(index))
        } else if let Some(arr) = array.as_any().downcast_ref::<Int64Array>() {
            json!(arr.value(index))
        } else if let Some(arr) = array.as_any().downcast_ref::<Float64Array>() {
            json!(arr.value(index))
        } else if let Some(arr) = array.as_any().downcast_ref::<StringArray>() {
            json!(arr.value(index))
        } else {
            json!("<unsupported>")
        }
    }
}

impl Default for DataWise {
    fn default() -> Self {
        Self::new().expect("Failed to create default DataWise")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_datawise_creation() {
        let core = DataWise::new().unwrap();
        let mut rx = core.subscribe();

        // 测试事件订阅
        assert!(rx.try_recv().is_err()); // 应该没有事件
    }

    #[tokio::test]
    async fn test_command_handling() {
        let core = DataWise::new().unwrap();
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 1,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT 1 as num".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 应该收到 Started 事件
        let event = rx.recv().await.unwrap();
        assert_eq!(event.task_id, 1);
        assert!(matches!(event.kind, EventKind::Started));

        // 应该收到 Finished 事件
        let event = rx.recv().await.unwrap();
        assert_eq!(event.task_id, 1);
        match event.kind {
            EventKind::Finished {
                row_count,
                column_count,
                ..
            } => {
                assert_eq!(row_count, 1);
                assert_eq!(column_count, 1);
            }
            _ => panic!("Expected Finished event"),
        }
    }

    #[tokio::test]
    async fn test_sql_execution() {
        let core = DataWise::new().unwrap();
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 2,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT 1 as a, 'hello' as b, 3.14 as c".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 跳过 Started 事件
        let _ = rx.recv().await.unwrap();

        // 检查 Finished 事件
        let event = rx.recv().await.unwrap();
        match event.kind {
            EventKind::Finished {
                row_count,
                column_count,
                preview,
            } => {
                assert_eq!(row_count, 1);
                assert_eq!(column_count, 3);
                assert!(preview.contains("hello"));
            }
            _ => panic!("Expected Finished event"),
        }
    }

    #[tokio::test]
    async fn test_sql_error() {
        let core = DataWise::new().unwrap();
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 3,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT * FROM nonexistent_table".to_string(),
            },
        };

        let result = core.handle(cmd).await;
        assert!(result.is_err());

        // 跳过 Started 事件
        let _ = rx.recv().await.unwrap();

        // 应该收到 Error 事件
        let event = rx.recv().await.unwrap();
        assert_eq!(event.task_id, 3);
        assert!(matches!(event.kind, EventKind::Error(_)));
    }

    #[tokio::test]
    async fn test_multiple_rows() {
        let core = DataWise::new().unwrap();
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 4,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT * FROM (VALUES (1), (2), (3), (4), (5)) AS t(id)".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 跳过 Started 事件
        let _ = rx.recv().await.unwrap();

        // 检查 Finished 事件
        let event = rx.recv().await.unwrap();
        match event.kind {
            EventKind::Finished {
                row_count,
                column_count,
                preview,
            } => {
                assert_eq!(row_count, 5);
                assert_eq!(column_count, 1);
                assert!(preview.contains("1"));
                assert!(preview.contains("5"));
            }
            _ => panic!("Expected Finished event"),
        }
    }

    #[tokio::test]
    async fn test_multiple_columns_and_types() {
        let core = DataWise::new().unwrap();
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 5,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT 42 as int_col, 3.14 as float_col, 'text' as str_col, true as bool_col".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 跳过 Started 事件
        let _ = rx.recv().await.unwrap();

        // 检查 Finished 事件
        let event = rx.recv().await.unwrap();
        match event.kind {
            EventKind::Finished {
                row_count,
                column_count,
                preview,
            } => {
                assert_eq!(row_count, 1);
                assert_eq!(column_count, 4);
                assert!(preview.contains("42"));
                assert!(preview.contains("3.14"));
                assert!(preview.contains("text"));
                assert!(preview.contains("true"));
            }
            _ => panic!("Expected Finished event"),
        }
    }

    #[tokio::test]
    async fn test_aggregation() {
        let core = DataWise::new().unwrap();
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 6,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT COUNT(*) as cnt, SUM(id) as total FROM (VALUES (1), (2), (3)) AS t(id)".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 跳过 Started 事件
        let _ = rx.recv().await.unwrap();

        // 检查 Finished 事件
        let event = rx.recv().await.unwrap();
        match event.kind {
            EventKind::Finished {
                row_count,
                column_count,
                preview,
            } => {
                assert_eq!(row_count, 1);
                assert_eq!(column_count, 2);
                assert!(preview.contains("3")); // COUNT
                assert!(preview.contains("6")); // SUM
            }
            _ => panic!("Expected Finished event"),
        }
    }

    #[tokio::test]
    async fn test_null_values() {
        let core = DataWise::new().unwrap();
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 7,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT 1 as id, NULL as nullable_col".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 跳过 Started 事件
        let _ = rx.recv().await.unwrap();

        // 检查 Finished 事件
        let event = rx.recv().await.unwrap();
        match event.kind {
            EventKind::Finished {
                row_count,
                column_count,
                preview,
            } => {
                assert_eq!(row_count, 1);
                assert_eq!(column_count, 2);
                // JSON 中 null 值会被序列化为 null（不带引号）
                let preview_json: Vec<serde_json::Value> = serde_json::from_str(&preview).unwrap();
                assert_eq!(preview_json.len(), 1);
                assert!(preview_json[0]["nullable_col"].is_null());
            }
            _ => panic!("Expected Finished event"),
        }
    }

    #[tokio::test]
    async fn test_preview_limit() {
        let core = DataWise::new().unwrap();
        let mut rx = core.subscribe();

        // 生成 20 行数据，但预览应该只显示 10 行
        let cmd = Command {
            task_id: 8,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT * FROM (SELECT * FROM (VALUES (1), (2), (3), (4), (5), (6), (7), (8), (9), (10), (11), (12), (13), (14), (15), (16), (17), (18), (19), (20)) AS t(id))".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 跳过 Started 事件
        let _ = rx.recv().await.unwrap();

        // 检查 Finished 事件
        let event = rx.recv().await.unwrap();
        match event.kind {
            EventKind::Finished {
                row_count,
                column_count,
                preview,
            } => {
                assert_eq!(row_count, 20);
                assert_eq!(column_count, 1);
                // 预览应该包含前 10 行
                let preview_json: Vec<serde_json::Value> = serde_json::from_str(&preview).unwrap();
                assert_eq!(preview_json.len(), 10);
            }
            _ => panic!("Expected Finished event"),
        }
    }
}

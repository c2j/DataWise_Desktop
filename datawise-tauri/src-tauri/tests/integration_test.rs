/// 集成测试：验证 Tauri 命令层与 Core 的集成
/// 
/// 这个测试验证了 execute_sql 命令的基本功能
/// 注意：这是一个单元测试，不是完整的 Tauri 应用集成测试

#[cfg(test)]
mod tests {
    use datawise_core::{DataWise, Command, CmdType, EventKind};

    #[tokio::test]
    async fn test_execute_sql_integration() {
        // 创建 DataWise 实例
        let core = DataWise::new().expect("Failed to create DataWise");
        let mut rx = core.subscribe();

        // 发送 SQL 命令
        let cmd = Command {
            task_id: 1,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT 1 as num".to_string(),
            },
        };

        core.handle(cmd).await.expect("Failed to handle command");

        // 验证事件流
        let mut received_started = false;
        let mut received_finished = false;

        while let Ok(event) = rx.recv().await {
            match event.kind {
                EventKind::Started => {
                    received_started = true;
                }
                EventKind::Finished {
                    row_count,
                    column_count,
                    preview,
                } => {
                    received_finished = true;
                    assert_eq!(row_count, 1);
                    assert_eq!(column_count, 1);

                    // 验证预览数据格式
                    let preview_json: Vec<serde_json::Value> =
                        serde_json::from_str(&preview).expect("Invalid JSON");
                    assert_eq!(preview_json.len(), 1);
                    // 注意：DuckDB 使用 col_0 作为列名（因为我们使用了占位符列名）
                    assert_eq!(preview_json[0]["col_0"], 1);
                    break;
                }
                EventKind::Error(e) => {
                    panic!("Unexpected error: {}", e);
                }
                _ => {}
            }
        }

        assert!(received_started, "Did not receive Started event");
        assert!(received_finished, "Did not receive Finished event");
    }

    #[tokio::test]
    async fn test_execute_sql_multiple_columns() {
        let core = DataWise::new().expect("Failed to create DataWise");
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 2,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT 1 as id, 'hello' as name, 3.14 as value".to_string(),
            },
        };

        core.handle(cmd).await.expect("Failed to handle command");

        while let Ok(event) = rx.recv().await {
            if let EventKind::Finished {
                row_count,
                column_count,
                preview,
            } = event.kind
            {
                assert_eq!(row_count, 1);
                assert_eq!(column_count, 3);

                let preview_json: Vec<serde_json::Value> =
                    serde_json::from_str(&preview).expect("Invalid JSON");
                // 验证数据存在（使用占位符列名 col_0, col_1, col_2）
                assert_eq!(preview_json[0]["col_0"], 1);
                assert_eq!(preview_json[0]["col_1"], "hello");
                break;
            }
        }
    }

    #[tokio::test]
    async fn test_execute_sql_error_handling() {
        let core = DataWise::new().expect("Failed to create DataWise");
        let mut rx = core.subscribe();

        let cmd = Command {
            task_id: 3,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT * FROM nonexistent_table".to_string(),
            },
        };

        let result = core.handle(cmd).await;
        assert!(result.is_err(), "Should return error for invalid SQL");

        // 验证错误事件
        let mut received_error = false;
        while let Ok(event) = rx.recv().await {
            if let EventKind::Error(_) = event.kind {
                received_error = true;
                break;
            }
        }

        assert!(received_error, "Should receive Error event");
    }
}


use datawise_core::{DataWise, Command, CmdType, FileFmt, EventKind};
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_csv_import() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("test.csv");

    // 创建测试 CSV 文件
    let csv_content = "id,name,value\n1,Alice,100\n2,Bob,200\n3,Charlie,300\n";
    fs::write(&csv_path, csv_content).unwrap();

    // 创建 DataWise 实例
    let core = DataWise::new().unwrap();
    let mut rx = core.subscribe();

    // 发送导入命令
    let cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ImportFile {
            path: csv_path.to_string_lossy().to_string(),
            fmt: FileFmt::Csv,
            table_name: Some("test_data".to_string()),
            overwrite: false,
        },
    };

    core.handle(cmd).await.unwrap();

    // 验证事件流
    let mut received_started = false;
    let mut received_finished = false;

    while let Ok(event) = rx.recv().await {
        match event.kind {
            EventKind::Started => {
                received_started = true;
            }
            EventKind::Finished { column_count, .. } => {
                received_finished = true;
                // 导入后查询 COUNT(*) 返回 1 行，但列数应该是 3
                assert_eq!(column_count, 3, "Expected 3 columns");
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
async fn test_csv_export() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.csv");

    // 创建 DataWise 实例
    let core = DataWise::new().unwrap();
    let mut rx = core.subscribe();

    // 首先创建一个表
    let create_cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ExecuteSql {
            sql: "CREATE TABLE test_export AS SELECT 1 as id, 'test' as name".to_string(),
        },
    };

    core.handle(create_cmd).await.unwrap();

    // 消费事件
    while let Ok(event) = rx.recv().await {
        if let EventKind::Finished { .. } = event.kind {
            break;
        }
    }

    // 重新订阅
    let mut rx = core.subscribe();

    // 导出表
    let export_cmd = Command {
        task_id: 2,
        cmd_type: CmdType::ExportFile {
            source: "test_export".to_string(),
            path: export_path.to_string_lossy().to_string(),
            fmt: FileFmt::Csv,
        },
    };

    core.handle(export_cmd).await.unwrap();

    // 验证事件流
    let mut received_finished = false;

    while let Ok(event) = rx.recv().await {
        if let EventKind::Finished { .. } = event.kind {
            received_finished = true;
            break;
        }
    }

    assert!(received_finished, "Did not receive Finished event");

    // 验证文件存在
    assert!(export_path.exists(), "Export file not created");

    // 验证文件内容
    let content = fs::read_to_string(&export_path).unwrap();
    assert!(content.contains("id"), "CSV header missing");
    assert!(content.contains("name"), "CSV header missing");
}

#[tokio::test]
async fn test_import_then_query() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");

    // 创建测试 CSV 文件
    let csv_content = "id,name,age\n1,Alice,30\n2,Bob,25\n3,Charlie,35\n";
    fs::write(&csv_path, csv_content).unwrap();

    // 创建 DataWise 实例
    let core = DataWise::new().unwrap();

    // 第一步：导入文件
    {
        let mut rx = core.subscribe();
        let cmd = Command {
            task_id: 1,
            cmd_type: CmdType::ImportFile {
                path: csv_path.to_string_lossy().to_string(),
                fmt: FileFmt::Csv,
                table_name: Some("users".to_string()),
                overwrite: false,
            },
        };

        core.handle(cmd).await.unwrap();

        // 等待导入完成
        while let Ok(event) = rx.recv().await {
            if let EventKind::Finished { .. } = event.kind {
                break;
            }
        }
    }

    // 第二步：查询导入的数据
    {
        let mut rx = core.subscribe();
        let cmd = Command {
            task_id: 2,
            cmd_type: CmdType::ExecuteSql {
                sql: "SELECT COUNT(*) as cnt FROM users".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 等待查询完成
        let mut result_received = false;
        while let Ok(event) = rx.recv().await {
            if let EventKind::Finished { row_count, .. } = event.kind {
                assert_eq!(row_count, 1, "Expected 1 row from COUNT query");
                result_received = true;
                break;
            }
        }

        assert!(result_received, "Did not receive query result");
    }
}

#[tokio::test]
async fn test_import_export_roundtrip() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let import_path = temp_dir.path().join("import.csv");
    let export_path = temp_dir.path().join("export.csv");

    // 创建原始 CSV 文件
    let original_content = "id,name,score\n1,Alice,95\n2,Bob,87\n3,Charlie,92\n";
    fs::write(&import_path, original_content).unwrap();

    // 创建 DataWise 实例
    let core = DataWise::new().unwrap();

    // 第一步：导入文件
    {
        let mut rx = core.subscribe();
        let cmd = Command {
            task_id: 1,
            cmd_type: CmdType::ImportFile {
                path: import_path.to_string_lossy().to_string(),
                fmt: FileFmt::Csv,
                table_name: Some("scores".to_string()),
                overwrite: false,
            },
        };

        core.handle(cmd).await.unwrap();

        while let Ok(event) = rx.recv().await {
            if let EventKind::Finished { .. } = event.kind {
                break;
            }
        }
    }

    // 第二步：导出数据
    {
        let mut rx = core.subscribe();
        let cmd = Command {
            task_id: 2,
            cmd_type: CmdType::ExportFile {
                source: "scores".to_string(),
                path: export_path.to_string_lossy().to_string(),
                fmt: FileFmt::Csv,
            },
        };

        core.handle(cmd).await.unwrap();

        while let Ok(event) = rx.recv().await {
            if let EventKind::Finished { .. } = event.kind {
                break;
            }
        }
    }

    // 验证导出的文件
    assert!(export_path.exists(), "Export file not created");
    let exported_content = fs::read_to_string(&export_path).unwrap();

    // 验证关键数据存在
    assert!(exported_content.contains("Alice"), "Alice not found in export");
    assert!(exported_content.contains("95"), "Score 95 not found in export");
}

#[tokio::test]
async fn test_import_with_preview_data() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("preview_test.csv");

    // 创建测试 CSV 文件（多行数据）
    let csv_content = "id,name,age\n1,Alice,25\n2,Bob,30\n3,Charlie,35\n4,David,40\n5,Eve,28\n";
    fs::write(&csv_path, csv_content).unwrap();

    // 创建 DataWise 实例
    let core = DataWise::new().unwrap();
    let mut rx = core.subscribe();

    // 发送导入命令
    let cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ImportFile {
            path: csv_path.to_string_lossy().to_string(),
            fmt: FileFmt::Csv,
            table_name: Some("preview_data".to_string()),
            overwrite: false,
        },
    };

    core.handle(cmd).await.unwrap();

    // 验证 preview 数据
    let mut preview_received = false;
    let mut row_count = 0;

    while let Ok(event) = rx.recv().await {
        match event.kind {
            EventKind::Finished {
                row_count: rc,
                column_count: cc,
                preview,
            } => {
                preview_received = true;
                row_count = rc;

                // 验证行数和列数
                assert_eq!(cc, 3, "Expected 3 columns");
                assert_eq!(rc, 5, "Expected 5 rows");

                // 验证 preview 不是空对象
                assert_ne!(preview, "{}", "Preview should not be empty");

                // 验证 preview 是有效的 JSON 数组
                let preview_json: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&preview);
                assert!(preview_json.is_ok(), "Preview should be valid JSON");

                let rows = preview_json.unwrap();
                assert!(!rows.is_empty(), "Preview should contain at least one row");

                // 验证第一行包含预期的数据
                if let Some(first_row) = rows.first() {
                    if let Some(obj) = first_row.as_object() {
                        // 检查对象有 3 个字段（对应 3 列）
                        assert_eq!(obj.len(), 3, "Preview should have 3 columns, got: {:?}", obj.keys().collect::<Vec<_>>());

                        // 验证数据值存在（不管列名是什么）
                        let values: Vec<_> = obj.values().collect();
                        assert!(!values.is_empty(), "Preview should contain data");
                    } else {
                        panic!("First row should be an object, got: {:?}", first_row);
                    }
                } else {
                    panic!("Preview should contain at least one row");
                }

                break;
            }
            EventKind::Error(e) => {
                panic!("Unexpected error: {}", e);
            }
            _ => {}
        }
    }

    assert!(preview_received, "Did not receive Finished event with preview");
    assert_eq!(row_count, 5, "Row count should be 5");
}

#[tokio::test]
async fn test_import_with_overwrite() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("overwrite_test.csv");

    // 创建第一个 CSV 文件
    let csv_content_1 = "id,value\n1,first\n2,second\n";
    fs::write(&csv_path, csv_content_1).unwrap();

    // 创建 DataWise 实例
    let core = DataWise::new().unwrap();

    // 第一次导入
    {
        let mut rx = core.subscribe();
        let cmd = Command {
            task_id: 1,
            cmd_type: CmdType::ImportFile {
                path: csv_path.to_string_lossy().to_string(),
                fmt: FileFmt::Csv,
                table_name: Some("overwrite_table".to_string()),
                overwrite: false,
            },
        };

        core.handle(cmd).await.unwrap();

        // 等待导入完成
        while let Ok(event) = rx.recv().await {
            if let EventKind::Finished { row_count, .. } = event.kind {
                assert_eq!(row_count, 2, "First import should have 2 rows");
                break;
            }
        }
    }

    // 修改 CSV 文件
    let csv_content_2 = "id,value\n1,updated\n2,modified\n3,new\n";
    fs::write(&csv_path, csv_content_2).unwrap();

    // 第二次导入，使用 overwrite=true
    {
        let mut rx = core.subscribe();
        let cmd = Command {
            task_id: 2,
            cmd_type: CmdType::ImportFile {
                path: csv_path.to_string_lossy().to_string(),
                fmt: FileFmt::Csv,
                table_name: Some("overwrite_table".to_string()),
                overwrite: true,
            },
        };

        core.handle(cmd).await.unwrap();

        // 等待导入完成
        while let Ok(event) = rx.recv().await {
            if let EventKind::Finished { row_count, .. } = event.kind {
                assert_eq!(row_count, 3, "Second import with overwrite should have 3 rows");
                break;
            }
        }
    }
}

#[tokio::test]
async fn test_import_progress_events() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("progress_test.csv");

    // 创建测试 CSV 文件
    let csv_content = "id,name\n1,Alice\n2,Bob\n3,Charlie\n";
    fs::write(&csv_path, csv_content).unwrap();

    // 创建 DataWise 实例
    let core = DataWise::new().unwrap();
    let mut rx = core.subscribe();

    // 发送导入命令
    let cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ImportFile {
            path: csv_path.to_string_lossy().to_string(),
            fmt: FileFmt::Csv,
            table_name: Some("progress_data".to_string()),
            overwrite: false,
        },
    };

    core.handle(cmd).await.unwrap();

    // 验证事件流包含 Progress 事件
    let mut received_started = false;
    let mut received_progress = false;
    let mut received_finished = false;

    while let Ok(event) = rx.recv().await {
        match event.kind {
            EventKind::Started => {
                received_started = true;
            }
            EventKind::Progress { pct, .. } => {
                received_progress = true;
                // 进度应该在 0-100 之间
                assert!(pct <= 100, "Progress percentage should be <= 100");
            }
            EventKind::Finished { .. } => {
                received_finished = true;
                break;
            }
            EventKind::Error(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    }

    assert!(received_started, "Did not receive Started event");
    assert!(received_progress, "Did not receive Progress event");
    assert!(received_finished, "Did not receive Finished event");
}

#[tokio::test]
async fn test_parquet_import_with_preview() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let parquet_path = temp_dir.path().join("test.parquet");

    // 创建 DataWise 实例并先创建一个 Parquet 文件
    let core = DataWise::new().unwrap();

    // 使用 SQL 创建表并导出为 Parquet
    {
        let mut rx = core.subscribe();
        let cmd = Command {
            task_id: 1,
            cmd_type: CmdType::ExecuteSql {
                sql: "CREATE TABLE temp_table AS SELECT 1 as id, 'test' as name, 100 as value".to_string(),
            },
        };

        core.handle(cmd).await.unwrap();

        // 等待完成
        while let Ok(event) = rx.recv().await {
            if let EventKind::Finished { .. } = event.kind {
                break;
            }
        }
    }

    // 导出为 Parquet
    {
        let mut rx = core.subscribe();
        let cmd = Command {
            task_id: 2,
            cmd_type: CmdType::ExportFile {
                source: "temp_table".to_string(),
                path: parquet_path.to_string_lossy().to_string(),
                fmt: FileFmt::Parquet,
            },
        };

        core.handle(cmd).await.unwrap();

        // 等待导出完成
        while let Ok(event) = rx.recv().await {
            if let EventKind::Finished { .. } = event.kind {
                break;
            }
        }
    }

    // 现在导入 Parquet 文件
    {
        let mut rx = core.subscribe();
        let cmd = Command {
            task_id: 3,
            cmd_type: CmdType::ImportFile {
                path: parquet_path.to_string_lossy().to_string(),
                fmt: FileFmt::Parquet,
                table_name: Some("imported_parquet".to_string()),
                overwrite: false,
            },
        };

        core.handle(cmd).await.unwrap();

        // 验证导入成功并有 preview
        while let Ok(event) = rx.recv().await {
            match event.kind {
                EventKind::Finished {
                    row_count,
                    column_count,
                    preview,
                } => {
                    assert_eq!(row_count, 1, "Parquet import should have 1 row");
                    assert_eq!(column_count, 3, "Parquet import should have 3 columns");
                    assert_ne!(preview, "{}", "Parquet preview should not be empty");
                    break;
                }
                EventKind::Error(e) => {
                    panic!("Unexpected error: {}", e);
                }
                _ => {}
            }
        }
    }
}

#[tokio::test]
async fn test_json_import_with_preview() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let json_path = temp_dir.path().join("test.json");

    // 创建测试 JSON 文件（NDJSON 格式 - 每行一个 JSON 对象）
    let json_content = r#"{"id": 1, "name": "Alice", "age": 25}
{"id": 2, "name": "Bob", "age": 30}
{"id": 3, "name": "Charlie", "age": 35}
"#;
    fs::write(&json_path, json_content).unwrap();

    // 创建 DataWise 实例
    let core = DataWise::new().unwrap();
    let mut rx = core.subscribe();

    // 发送导入命令
    let cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ImportFile {
            path: json_path.to_string_lossy().to_string(),
            fmt: FileFmt::Json,
            table_name: Some("json_data".to_string()),
            overwrite: false,
        },
    };

    core.handle(cmd).await.unwrap();

    // 验证导入成功并有 preview
    let mut preview_received = false;
    let mut row_count = 0;

    while let Ok(event) = rx.recv().await {
        match event.kind {
            EventKind::Finished {
                row_count: rc,
                column_count: cc,
                preview,
            } => {
                preview_received = true;
                row_count = rc;

                // 验证行数和列数
                assert_eq!(cc, 3, "Expected 3 columns");
                assert_eq!(rc, 3, "Expected 3 rows");

                // 验证 preview 不是空对象
                assert_ne!(preview, "{}", "Preview should not be empty");

                // 验证 preview 是有效的 JSON 数组
                let preview_json: Result<Vec<serde_json::Value>, _> = serde_json::from_str(&preview);
                assert!(preview_json.is_ok(), "Preview should be valid JSON");

                let rows = preview_json.unwrap();
                assert_eq!(rows.len(), 3, "Preview should contain 3 rows");

                // 验证第一行包含预期的数据
                if let Some(first_row) = rows.first() {
                    if let Some(obj) = first_row.as_object() {
                        // 检查对象有 3 个字段（对应 3 列）
                        assert_eq!(obj.len(), 3, "Preview should have 3 columns");
                    } else {
                        panic!("First row should be an object, got: {:?}", first_row);
                    }
                }

                break;
            }
            EventKind::Error(e) => {
                panic!("Unexpected error: {}", e);
            }
            _ => {}
        }
    }

    assert!(preview_received, "Did not receive Finished event with preview");
    assert_eq!(row_count, 3, "Row count should be 3");
}

#[tokio::test]
async fn test_json_array_import() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let json_path = temp_dir.path().join("array.json");

    // 创建测试 JSON 文件（数组格式）
    let json_content = r#"[
  {"id": 1, "value": "first"},
  {"id": 2, "value": "second"},
  {"id": 3, "value": "third"}
]"#;
    fs::write(&json_path, json_content).unwrap();

    // 创建 DataWise 实例
    let core = DataWise::new().unwrap();
    let mut rx = core.subscribe();

    // 发送导入命令
    let cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ImportFile {
            path: json_path.to_string_lossy().to_string(),
            fmt: FileFmt::Json,
            table_name: Some("json_array_data".to_string()),
            overwrite: false,
        },
    };

    core.handle(cmd).await.unwrap();

    // 验证导入成功
    while let Ok(event) = rx.recv().await {
        match event.kind {
            EventKind::Finished {
                row_count,
                column_count,
                ..
            } => {
                assert_eq!(row_count, 3, "Expected 3 rows");
                assert_eq!(column_count, 2, "Expected 2 columns");
                break;
            }
            EventKind::Error(e) => {
                panic!("Unexpected error: {}", e);
            }
            _ => {}
        }
    }
}


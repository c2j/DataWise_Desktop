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


use datawise_core::{DataWise, Command, CmdType, EventKind, FileFmt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

/// SQL 查询结果的预览数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub row_count: usize,
    pub column_count: usize,
    pub preview: String, // JSON 格式的预览数据
}

/// 导入/导出操作的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    /// 导入操作时的表名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
    /// 导入操作时的行数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_count: Option<usize>,
    /// 导入操作时的列数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_count: Option<usize>,
}

/// 应用状态，包含 DataWise Core 实例
pub struct AppState {
    core: Arc<DataWise>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let core = Arc::new(DataWise::new()?);
        Ok(Self { core })
    }
}

/// 执行 SQL 查询命令
///
/// # 参数
/// - `sql`: SQL 查询语句
///
/// # 返回
/// 查询结果（行数、列数、预览数据）
#[tauri::command]
async fn execute_sql(
    sql: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<QueryResult, String> {
    tracing::info!("Executing SQL: {}", sql);

    let state = state.lock().await;
    let mut rx = state.core.subscribe();

    // 发送命令
    let cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ExecuteSql { sql },
    };

    state.core.handle(cmd).await.map_err(|e| e.to_string())?;

    // 等待事件
    let mut result = None;
    while let Ok(event) = rx.recv().await {
        match event.kind {
            EventKind::Started => {
                tracing::debug!("Query started");
            }
            EventKind::Finished {
                row_count,
                column_count,
                preview,
            } => {
                result = Some(QueryResult {
                    row_count,
                    column_count,
                    preview,
                });
                break;
            }
            EventKind::Error(e) => {
                return Err(format!("Query error: {}", e));
            }
            _ => {}
        }
    }

    result.ok_or_else(|| "No result received".to_string())
}

/// 导入文件命令
///
/// # 参数
/// - `path`: 文件路径
/// - `format`: 文件格式 ("csv" 或 "parquet")
/// - `table_name`: 导入到的表名（可选）
/// - `window`: Tauri 窗口（用于发送进度事件）
///
/// # 返回
/// 操作结果（包含表名、行数、列数等信息）
#[tauri::command]
async fn import_file(
    path: String,
    format: String,
    table_name: Option<String>,
    window: tauri::Window,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<OperationResult, String> {
    tracing::info!("Importing file: {} (format: {})", path, format);

    let fmt = match format.to_lowercase().as_str() {
        "csv" => FileFmt::Csv,
        "parquet" | "pq" => FileFmt::Parquet,
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    let state = state.lock().await;
    let mut rx = state.core.subscribe();

    // 发送命令
    let cmd = Command {
        task_id: 2,
        cmd_type: CmdType::ImportFile {
            path,
            fmt,
            table_name: table_name.clone(),
            overwrite: false,
        },
    };

    state.core.handle(cmd).await.map_err(|e| e.to_string())?;

    // 等待事件
    let mut success = false;
    let mut row_count = None;
    let mut column_count = None;
    let final_table_name = table_name;

    while let Ok(event) = rx.recv().await {
        match event.kind {
            EventKind::Started => {
                tracing::info!("Import started");
                let _ = window.emit("import-progress", serde_json::json!({
                    "status": "started",
                    "percentage": 0
                }));
            }
            EventKind::Progress { pct, .. } => {
                tracing::debug!("Import progress: {}%", pct);
                let _ = window.emit("import-progress", serde_json::json!({
                    "status": "in_progress",
                    "percentage": pct
                }));
            }
            EventKind::Finished {
                row_count: rc,
                column_count: cc,
                ..
            } => {
                success = true;
                row_count = Some(rc);
                column_count = Some(cc);
                let _ = window.emit("import-progress", serde_json::json!({
                    "status": "completed",
                    "percentage": 100
                }));
                break;
            }
            EventKind::Error(e) => {
                let _ = window.emit("import-progress", serde_json::json!({
                    "status": "error",
                    "message": e.clone()
                }));
                return Ok(OperationResult {
                    success: false,
                    message: format!("Import error: {}", e),
                    table_name: None,
                    row_count: None,
                    column_count: None,
                });
            }
            _ => {}
        }
    }

    Ok(OperationResult {
        success,
        message: if success {
            format!(
                "File imported successfully to table '{}' ({} rows, {} columns)",
                final_table_name.as_ref().unwrap_or(&"imported_data".to_string()),
                row_count.unwrap_or(0),
                column_count.unwrap_or(0)
            )
        } else {
            "Import failed".to_string()
        },
        table_name: final_table_name,
        row_count,
        column_count,
    })
}

/// 导出文件命令
///
/// # 参数
/// - `source`: 源表名或 SQL 查询
/// - `path`: 导出路径
/// - `format`: 导出格式 ("csv" 或 "parquet")
///
/// # 返回
/// 操作结果
#[tauri::command]
async fn export_file(
    source: String,
    path: String,
    format: String,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<OperationResult, String> {
    tracing::info!("Exporting to: {} (format: {})", path, format);

    let fmt = match format.to_lowercase().as_str() {
        "csv" => FileFmt::Csv,
        "parquet" | "pq" => FileFmt::Parquet,
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    let state = state.lock().await;
    let mut rx = state.core.subscribe();

    // 发送命令
    let cmd = Command {
        task_id: 3,
        cmd_type: CmdType::ExportFile {
            source,
            path,
            fmt,
        },
    };

    state.core.handle(cmd).await.map_err(|e| e.to_string())?;

    // 等待事件
    let mut success = false;
    while let Ok(event) = rx.recv().await {
        match event.kind {
            EventKind::Finished { .. } => {
                success = true;
                break;
            }
            EventKind::Error(e) => {
                return Ok(OperationResult {
                    success: false,
                    message: format!("Export error: {}", e),
                    table_name: None,
                    row_count: None,
                    column_count: None,
                });
            }
            _ => {}
        }
    }

    Ok(OperationResult {
        success,
        message: if success {
            "File exported successfully".to_string()
        } else {
            "Export failed".to_string()
        },
        table_name: None,
        row_count: None,
        column_count: None,
    })
}

/// 取消任务命令
///
/// # 参数
/// - `task_id`: 要取消的任务 ID
///
/// # 返回
/// 操作结果
#[tauri::command]
async fn cancel_task(
    task_id: u64,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<OperationResult, String> {
    tracing::info!("Cancelling task: {}", task_id);

    let state = state.lock().await;

    // 发送取消命令
    let cmd = Command {
        task_id,
        cmd_type: CmdType::Cancel { task_id },
    };

    state.core.handle(cmd).await.map_err(|e| e.to_string())?;

    Ok(OperationResult {
        success: true,
        message: "Task cancelled".to_string(),
        table_name: None,
        row_count: None,
        column_count: None,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建应用状态
    let app_state = Arc::new(Mutex::new(
        AppState::new().expect("Failed to initialize AppState"),
    ));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            execute_sql,
            import_file,
            export_file,
            cancel_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

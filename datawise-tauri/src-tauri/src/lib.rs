use datawise_core::{DataWise, Command, CmdType, EventKind};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// SQL 查询结果的预览数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub row_count: usize,
    pub column_count: usize,
    pub preview: String, // JSON 格式的预览数据
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
        .invoke_handler(tauri::generate_handler![execute_sql])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

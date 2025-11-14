//! 文件导入模块
//!
//! 支持 CSV、Parquet、JSON 格式的导入，带进度报告

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// 导入器配置
#[derive(Debug, Clone)]
pub struct ImportConfig {
    /// 表名
    pub table_name: String,
    /// 是否覆盖现有表
    pub overwrite: bool,
}

impl ImportConfig {
    pub fn new(table_name: String) -> Self {
        Self {
            table_name,
            overwrite: false,
        }
    }
}

/// 导入进度回调
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// 文件导入器
pub struct Importer {
    conn: Arc<std::sync::Mutex<duckdb::Connection>>,
    cancel_flag: Arc<AtomicBool>,
}

impl Importer {
    pub fn new(conn: Arc<std::sync::Mutex<duckdb::Connection>>) -> Self {
        Self {
            conn,
            cancel_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 获取取消标记
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel_flag)
    }

    /// 设置取消标记
    pub fn set_cancel(&self) {
        self.cancel_flag.store(true, Ordering::SeqCst);
    }

    /// 重置取消标记
    pub fn reset_cancel(&self) {
        self.cancel_flag.store(false, Ordering::SeqCst);
    }

    /// 检查是否已取消
    pub fn is_cancelled(&self) -> bool {
        self.cancel_flag.load(Ordering::SeqCst)
    }

    /// 导入 CSV 文件
    pub fn import_csv(
        &self,
        path: &Path,
        config: ImportConfig,
        progress: Option<ProgressCallback>,
    ) -> Result<()> {
        info!("Importing CSV from: {:?}", path);
        self.reset_cancel();

        let file_size = std::fs::metadata(path)?.len();
        let table_name = &config.table_name;

        // 使用 DuckDB 的 SQL 接口导入 CSV
        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy();

        // 删除现有表（如果需要）
        if config.overwrite {
            let _ = conn.execute(&format!("DROP TABLE IF EXISTS {}", table_name), []);
        }

        // 使用 DuckDB 的 read_csv 函数
        let sql = format!(
            "CREATE TABLE {} AS SELECT * FROM read_csv_auto('{}')",
            table_name, path_str
        );

        conn.execute(&sql, []).context("Failed to import CSV")?;

        // 报告进度
        if let Some(cb) = progress {
            cb(file_size, file_size);
        }

        info!("CSV import completed");
        Ok(())
    }

    /// 导入 Parquet 文件
    pub fn import_parquet(
        &self,
        path: &Path,
        config: ImportConfig,
        progress: Option<ProgressCallback>,
    ) -> Result<()> {
        info!("Importing Parquet from: {:?}", path);
        self.reset_cancel();

        let file_size = std::fs::metadata(path)?.len();
        let table_name = &config.table_name;

        // 使用 DuckDB 的 SQL 接口导入 Parquet
        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy();

        // 删除现有表（如果需要）
        if config.overwrite {
            let _ = conn.execute(&format!("DROP TABLE IF EXISTS {}", table_name), []);
        }

        // 使用 DuckDB 的 read_parquet 函数
        let sql = format!(
            "CREATE TABLE {} AS SELECT * FROM read_parquet('{}')",
            table_name, path_str
        );

        conn.execute(&sql, []).context("Failed to import Parquet")?;

        // 报告进度
        if let Some(cb) = progress {
            cb(file_size, file_size);
        }

        info!("Parquet import completed");
        Ok(())
    }
}


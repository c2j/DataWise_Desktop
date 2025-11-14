//! 文件导出模块
//!
//! 支持导出到 CSV、Parquet 格式，带进度报告

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// 导出器配置
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// 源表名或 SQL 查询
    pub source: String,
    /// 是否是 SQL 查询（true）还是表名（false）
    pub is_query: bool,
}

impl ExportConfig {
    pub fn new_table(table_name: String) -> Self {
        Self {
            source: table_name,
            is_query: false,
        }
    }

    pub fn new_query(sql: String) -> Self {
        Self {
            source: sql,
            is_query: true,
        }
    }
}

/// 导出进度回调
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// 文件导出器
pub struct Exporter {
    conn: Arc<std::sync::Mutex<duckdb::Connection>>,
}

impl Exporter {
    pub fn new(conn: Arc<std::sync::Mutex<duckdb::Connection>>) -> Self {
        Self { conn }
    }

    /// 导出到 CSV
    pub fn export_csv(
        &self,
        path: &Path,
        config: ExportConfig,
        _progress: Option<ProgressCallback>,
    ) -> Result<()> {
        info!("Exporting to CSV: {:?}", path);

        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy();
        let source = &config.source;

        // 使用 DuckDB 的 SQL 接口导出 CSV
        let sql = if config.is_query {
            format!("COPY ({}) TO '{}' (FORMAT CSV, HEADER TRUE)", source, path_str)
        } else {
            format!("COPY {} TO '{}' (FORMAT CSV, HEADER TRUE)", source, path_str)
        };

        conn.execute(&sql, []).context("Failed to export CSV")?;

        info!("CSV export completed");
        Ok(())
    }

    /// 导出到 Parquet
    pub fn export_parquet(
        &self,
        path: &Path,
        config: ExportConfig,
        _progress: Option<ProgressCallback>,
    ) -> Result<()> {
        info!("Exporting to Parquet: {:?}", path);

        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy();
        let source = &config.source;

        // 使用 DuckDB 的 SQL 接口导出 Parquet
        let sql = if config.is_query {
            format!("COPY ({}) TO '{}' (FORMAT PARQUET)", source, path_str)
        } else {
            format!("COPY {} TO '{}' (FORMAT PARQUET)", source, path_str)
        };

        conn.execute(&sql, []).context("Failed to export Parquet")?;

        info!("Parquet export completed");
        Ok(())
    }
}


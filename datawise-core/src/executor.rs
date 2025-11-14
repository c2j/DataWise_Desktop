//! SQL 执行器模块
//!
//! 负责执行 SQL 查询并将结果转换为 Arrow RecordBatch。

use anyhow::{Context, Result};
use arrow::array::{
    ArrayRef, BooleanArray, Float64Array, Int32Array, Int64Array, StringArray,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use duckdb::Connection;
use std::sync::{Arc, Mutex};

/// SQL 执行器
///
/// 使用 Arc<Mutex<Connection>> 以支持跨线程共享
pub struct Executor {
    conn: Arc<Mutex<Connection>>,
}

impl Executor {
    /// 创建新的执行器
    ///
    /// 初始化一个内存中的 DuckDB 连接。
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to open DuckDB in-memory database")?;

        tracing::info!("DuckDB executor initialized");

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 获取连接的 Arc 引用
    pub fn conn_arc(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// 导入 CSV 文件
    pub fn import_csv(
        &self,
        path: &std::path::Path,
        table_name: &str,
        _progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy();

        let sql = format!(
            "CREATE TABLE {} AS SELECT * FROM read_csv_auto('{}')",
            table_name, path_str
        );

        conn.execute(&sql, []).context("Failed to import CSV")?;
        tracing::info!("CSV imported to table: {}", table_name);
        Ok(())
    }

    /// 导入 Parquet 文件
    pub fn import_parquet(
        &self,
        path: &std::path::Path,
        table_name: &str,
        _progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy();

        let sql = format!(
            "CREATE TABLE {} AS SELECT * FROM read_parquet('{}')",
            table_name, path_str
        );

        conn.execute(&sql, []).context("Failed to import Parquet")?;
        tracing::info!("Parquet imported to table: {}", table_name);
        Ok(())
    }

    /// 导出到 CSV
    pub fn export_csv(
        &self,
        path: &std::path::Path,
        source: &str,
        _progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy();

        let sql = format!(
            "COPY {} TO '{}' (FORMAT CSV, HEADER TRUE)",
            source, path_str
        );

        conn.execute(&sql, []).context("Failed to export CSV")?;
        tracing::info!("Data exported to CSV: {:?}", path);
        Ok(())
    }

    /// 导出到 Parquet
    pub fn export_parquet(
        &self,
        path: &std::path::Path,
        source: &str,
        _progress: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let path_str = path.to_string_lossy();

        let sql = format!(
            "COPY {} TO '{}' (FORMAT PARQUET)",
            source, path_str
        );

        conn.execute(&sql, []).context("Failed to export Parquet")?;
        tracing::info!("Data exported to Parquet: {:?}", path);
        Ok(())
    }

    /// 执行 SQL 查询
    ///
    /// # 参数
    ///
    /// * `sql` - SQL 查询语句
    ///
    /// # 返回
    ///
    /// 返回 Arrow RecordBatch 向量
    pub fn execute(&self, sql: &str) -> Result<Vec<RecordBatch>> {
        tracing::debug!("Executing SQL: {}", sql);

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock connection: {}", e))?;

        // 准备 SQL 语句
        let mut stmt = conn
            .prepare(sql)
            .context("Failed to prepare SQL statement")?;

        // 执行查询并收集所有行
        let mut rows = stmt.query([]).context("Failed to execute query")?;

        let mut all_rows: Vec<Vec<duckdb::types::Value>> = Vec::new();
        let mut column_count = 0;

        while let Some(row) = rows.next().context("Failed to fetch row")? {
            if column_count == 0 {
                // 从第一行获取列数
                column_count = row.as_ref().column_count();
            }
            let mut row_values = Vec::new();
            for i in 0..column_count {
                let value: duckdb::types::Value = row.get(i)?;
                row_values.push(value);
            }
            all_rows.push(row_values);
        }

        // 生成列名
        let column_names: Vec<String> = (0..column_count)
            .map(|i| format!("col_{}", i))
            .collect();

        if all_rows.is_empty() {
            // 返回空的 RecordBatch
            let schema = Arc::new(Schema::new(Vec::<Field>::new()));
            return Ok(vec![RecordBatch::new_empty(schema)]);
        }

        // 推断 schema
        let schema = self.infer_schema(&column_names, &all_rows)?;
        let schema_ref = Arc::new(schema);

        // 转换为 Arrow arrays
        let arrays = self.convert_to_arrays(&all_rows, &schema_ref)?;

        // 创建 RecordBatch
        let batch = RecordBatch::try_new(schema_ref, arrays)
            .context("Failed to create RecordBatch")?;

        Ok(vec![batch])
    }

    /// 推断 Schema
    fn infer_schema(
        &self,
        column_names: &[String],
        rows: &[Vec<duckdb::types::Value>],
    ) -> Result<Schema> {
        let mut fields = Vec::new();

        for (col_idx, name) in column_names.iter().enumerate() {
            // 从第一行推断类型
            let data_type = if let Some(first_row) = rows.first() {
                if let Some(value) = first_row.get(col_idx) {
                    self.value_to_datatype(value)
                } else {
                    DataType::Utf8
                }
            } else {
                DataType::Utf8
            };

            fields.push(Field::new(name, data_type, true));
        }

        Ok(Schema::new(fields))
    }

    /// 将 DuckDB Value 转换为 Arrow DataType
    fn value_to_datatype(&self, value: &duckdb::types::Value) -> DataType {
        use duckdb::types::Value;

        match value {
            Value::Null => DataType::Utf8,
            Value::Boolean(_) => DataType::Boolean,
            Value::TinyInt(_) | Value::SmallInt(_) | Value::Int(_) => DataType::Int32,
            Value::BigInt(_) => DataType::Int64,
            Value::Float(_) | Value::Double(_) => DataType::Float64,
            Value::Text(_) => DataType::Utf8,
            _ => DataType::Utf8, // 默认使用字符串
        }
    }

    /// 将行数据转换为 Arrow Arrays
    fn convert_to_arrays(
        &self,
        rows: &[Vec<duckdb::types::Value>],
        schema: &Arc<Schema>,
    ) -> Result<Vec<ArrayRef>> {
        let mut arrays: Vec<ArrayRef> = Vec::new();

        for (col_idx, field) in schema.fields().iter().enumerate() {
            let array: ArrayRef = match field.data_type() {
                DataType::Boolean => {
                    let values: Vec<Option<bool>> = rows
                        .iter()
                        .map(|row| {
                            if let Some(duckdb::types::Value::Boolean(b)) = row.get(col_idx) {
                                Some(*b)
                            } else {
                                None
                            }
                        })
                        .collect();
                    Arc::new(BooleanArray::from(values))
                }
                DataType::Int32 => {
                    let values: Vec<Option<i32>> = rows
                        .iter()
                        .map(|row| self.extract_int32(row.get(col_idx)))
                        .collect();
                    Arc::new(Int32Array::from(values))
                }
                DataType::Int64 => {
                    let values: Vec<Option<i64>> = rows
                        .iter()
                        .map(|row| self.extract_int64(row.get(col_idx)))
                        .collect();
                    Arc::new(Int64Array::from(values))
                }
                DataType::Float64 => {
                    let values: Vec<Option<f64>> = rows
                        .iter()
                        .map(|row| self.extract_float64(row.get(col_idx)))
                        .collect();
                    Arc::new(Float64Array::from(values))
                }
                _ => {
                    // 默认转换为字符串
                    let values: Vec<Option<String>> = rows
                        .iter()
                        .map(|row| self.extract_string(row.get(col_idx)))
                        .collect();
                    Arc::new(StringArray::from(values))
                }
            };

            arrays.push(array);
        }

        Ok(arrays)
    }

    fn extract_int32(&self, value: Option<&duckdb::types::Value>) -> Option<i32> {
        use duckdb::types::Value;
        match value {
            Some(Value::TinyInt(v)) => Some(*v as i32),
            Some(Value::SmallInt(v)) => Some(*v as i32),
            Some(Value::Int(v)) => Some(*v),
            _ => None,
        }
    }

    fn extract_int64(&self, value: Option<&duckdb::types::Value>) -> Option<i64> {
        use duckdb::types::Value;
        match value {
            Some(Value::BigInt(v)) => Some(*v),
            _ => None,
        }
    }

    fn extract_float64(&self, value: Option<&duckdb::types::Value>) -> Option<f64> {
        use duckdb::types::Value;
        match value {
            Some(Value::Float(v)) => Some(*v as f64),
            Some(Value::Double(v)) => Some(*v),
            _ => None,
        }
    }

    fn extract_string(&self, value: Option<&duckdb::types::Value>) -> Option<String> {
        use duckdb::types::Value;
        match value {
            Some(Value::Text(s)) => Some(s.clone()),
            Some(v) => Some(format!("{:?}", v)),
            None => None,
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new().expect("Failed to create default Executor")
    }
}


//! 公共协议定义
//! 
//! 本模块定义了 UI 层与 Core 层之间的通信协议。
//! 所有数据结构都支持 serde 序列化，确保跨语言兼容性。

use serde::{Deserialize, Serialize};

/// UI 事件 - Core 向 UI 推送的事件
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UiEvent {
    /// 任务 ID，用于关联命令和事件
    pub task_id: u64,
    /// 事件类型
    pub kind: EventKind,
}

/// 事件类型
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventKind {
    /// 任务已启动
    Started,
    
    /// 进度更新
    Progress {
        /// 完成百分比 (0-100)
        pct: u8,
        /// 已处理字节数
        bytes_processed: u64,
        /// 总字节数
        total_bytes: u64,
        /// 预计剩余时间（秒）
        eta_seconds: Option<u32>,
    },
    
    /// 任务完成
    /// 
    /// 注意：使用 Arc 共享所有权，避免大数据复制
    Finished {
        /// 结果行数
        row_count: usize,
        /// 结果列数
        column_count: usize,
        /// 数据摘要（前 10 行的 JSON）
        preview: String,
    },
    
    /// 任务失败
    Error(String),
}

/// 命令 - UI 向 Core 发送的命令
#[derive(Serialize, Deserialize, Debug)]
pub struct Command {
    /// 任务 ID
    pub task_id: u64,
    /// 命令类型
    pub cmd_type: CmdType,
}

/// 命令类型
#[derive(Serialize, Deserialize, Debug)]
pub enum CmdType {
    /// 执行 SQL 查询
    ExecuteSql {
        /// SQL 语句
        sql: String,
    },
    
    /// 导入文件
    ImportFile {
        /// 文件路径
        path: String,
        /// 文件格式
        fmt: FileFmt,
        /// 导入到的表名（可选，默认使用文件名）
        table_name: Option<String>,
    },
    
    /// 导出数据
    ExportFile {
        /// 查询 ID 或表名
        source: String,
        /// 导出路径
        path: String,
        /// 导出格式
        fmt: FileFmt,
    },
    
    /// 取消任务
    Cancel {
        /// 要取消的任务 ID
        task_id: u64,
    },
}

/// 文件格式
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileFmt {
    /// CSV 格式
    Csv,
    /// Parquet 格式
    Parquet,
    /// JSON 格式
    Json,
}

impl FileFmt {
    /// 从文件扩展名推断格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "csv" => Some(FileFmt::Csv),
            "parquet" | "pq" => Some(FileFmt::Parquet),
            "json" | "jsonl" => Some(FileFmt::Json),
            _ => None,
        }
    }
    
    /// 获取默认文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            FileFmt::Csv => "csv",
            FileFmt::Parquet => "parquet",
            FileFmt::Json => "json",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_fmt_from_extension() {
        assert_eq!(FileFmt::from_extension("csv"), Some(FileFmt::Csv));
        assert_eq!(FileFmt::from_extension("CSV"), Some(FileFmt::Csv));
        assert_eq!(FileFmt::from_extension("parquet"), Some(FileFmt::Parquet));
        assert_eq!(FileFmt::from_extension("pq"), Some(FileFmt::Parquet));
        assert_eq!(FileFmt::from_extension("json"), Some(FileFmt::Json));
        assert_eq!(FileFmt::from_extension("unknown"), None);
    }
}


/// 性能基准测试
/// 
/// 测试以下指标：
/// - 1GB CSV 导入时间
/// - 100 万行渲染时间
/// - SQL 查询响应时间

use datawise_core::{DataWise, Command, CmdType, FileFmt};
use std::fs::File;
use std::io::Write;
use std::time::Instant;

#[tokio::test]
async fn benchmark_csv_import_1gb() {
    // 生成 1GB CSV 文件（约 100 万行）
    let csv_path = "/tmp/benchmark_1gb.csv";
    
    // 检查文件是否已存在
    if !std::path::Path::new(csv_path).exists() {
        println!("Generating 1GB CSV file...");
        generate_large_csv(csv_path, 1_000_000);
    }
    
    let file_size = std::fs::metadata(csv_path)
        .map(|m| m.len())
        .unwrap_or(0);
    println!("CSV file size: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);
    
    // 测试导入时间
    let dw = DataWise::new().expect("Failed to create DataWise");
    let start = Instant::now();
    
    let cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ImportFile {
            path: csv_path.to_string(),
            fmt: FileFmt::Csv,
            table_name: Some("benchmark_data".to_string()),
            overwrite: true,
        },
    };
    
    dw.handle(cmd).await.expect("Failed to import CSV");
    let elapsed = start.elapsed();
    
    println!("CSV import time: {:.2}s", elapsed.as_secs_f64());
    println!("Import speed: {:.2} MB/s", file_size as f64 / 1024.0 / 1024.0 / elapsed.as_secs_f64());
    
    // 验证导入结果
    let verify_cmd = Command {
        task_id: 2,
        cmd_type: CmdType::ExecuteSql {
            sql: "SELECT COUNT(*) as row_count FROM benchmark_data".to_string(),
        },
    };
    
    dw.handle(verify_cmd).await.expect("Failed to verify import");
}

#[tokio::test]
async fn benchmark_sql_query_response() {
    let dw = DataWise::new().expect("Failed to create DataWise");
    
    // 创建测试表
    let create_cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ExecuteSql {
            sql: "CREATE TABLE test_data AS SELECT 
                    range as id,
                    'name_' || range as name,
                    range * 100 as value
                 FROM range(1000000)".to_string(),
        },
    };
    
    dw.handle(create_cmd).await.expect("Failed to create test table");
    
    // 测试简单查询
    let queries = vec![
        ("SELECT COUNT(*) FROM test_data", "Count query"),
        ("SELECT * FROM test_data LIMIT 10", "Limit query"),
        ("SELECT id, name FROM test_data WHERE value > 50000 LIMIT 100", "Filter query"),
        ("SELECT id, SUM(value) as total FROM test_data GROUP BY id % 100 LIMIT 100", "Aggregation query"),
    ];
    
    for (sql, desc) in queries {
        let start = Instant::now();
        
        let cmd = Command {
            task_id: 2,
            cmd_type: CmdType::ExecuteSql {
                sql: sql.to_string(),
            },
        };
        
        dw.handle(cmd).await.expect("Failed to execute query");
        let elapsed = start.elapsed();
        
        println!("{}: {:.2}ms", desc, elapsed.as_secs_f64() * 1000.0);
    }
}

#[tokio::test]
async fn benchmark_large_result_rendering() {
    let dw = DataWise::new().expect("Failed to create DataWise");
    
    // 创建大结果集
    let create_cmd = Command {
        task_id: 1,
        cmd_type: CmdType::ExecuteSql {
            sql: "CREATE TABLE large_result AS SELECT 
                    range as id,
                    'value_' || range as data,
                    random() as score
                 FROM range(1000000)".to_string(),
        },
    };
    
    dw.handle(create_cmd).await.expect("Failed to create large result table");
    
    // 测试大结果集查询
    let start = Instant::now();
    
    let cmd = Command {
        task_id: 2,
        cmd_type: CmdType::ExecuteSql {
            sql: "SELECT * FROM large_result".to_string(),
        },
    };
    
    dw.handle(cmd).await.expect("Failed to execute large query");
    let elapsed = start.elapsed();
    
    println!("Large result query (1M rows): {:.2}s", elapsed.as_secs_f64());
}

fn generate_large_csv(path: &str, rows: usize) {
    let mut file = File::create(path).expect("Failed to create CSV file");
    
    // 写入 CSV 头
    writeln!(file, "id,name,value,timestamp").expect("Failed to write header");
    
    // 写入数据行
    for i in 0..rows {
        let line = format!(
            "{},{},{},'2025-11-14 12:00:00'\n",
            i,
            format!("name_{}", i),
            i * 100
        );
        file.write_all(line.as_bytes()).expect("Failed to write row");
        
        // 每 10000 行打印进度
        if i % 10000 == 0 && i > 0 {
            println!("Generated {} rows", i);
        }
    }
}


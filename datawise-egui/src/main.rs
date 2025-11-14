use eframe::egui;
use datawise_core::{DataWise, Command, CmdType, UiEvent, EventKind};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use std::collections::VecDeque;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "DataWise - egui",
        options,
        Box::new(|_cc| {
            Ok(Box::new(DataWiseApp::new()))
        }),
    )
}

struct DataWiseApp {
    sql_input: String,
    results: String,
    status: String,
    is_executing: bool,
    core: Arc<Mutex<Option<DataWise>>>,
    result_history: VecDeque<String>,
    task_id: u64,
    event_rx: Option<mpsc::UnboundedReceiver<UiEvent>>,
}

impl Default for DataWiseApp {
    fn default() -> Self {
        Self::new()
    }
}

impl DataWiseApp {
    fn new() -> Self {
        Self {
            sql_input: "SELECT 1 as num".to_string(),
            results: String::new(),
            status: "Ready".to_string(),
            is_executing: false,
            core: Arc::new(Mutex::new(None)),
            result_history: VecDeque::new(),
            task_id: 1,
            event_rx: None,
        }
    }
}

impl eframe::App for DataWiseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理事件
        let mut events = Vec::new();
        if let Some(ref mut rx) = self.event_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }
        for event in events {
            self.handle_event(event);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("DataWise - egui MVP");

            // 顶部工具栏
            ui.horizontal(|ui| {
                if ui.button("Execute SQL").clicked() && !self.is_executing {
                    let sql = self.sql_input.clone();
                    let core = Arc::clone(&self.core);
                    let task_id = self.task_id;
                    self.task_id += 1;

                    self.is_executing = true;
                    self.status = "Executing...".to_string();
                    self.results.clear();

                    tokio::spawn(async move {
                        // 初始化 Core 如果还没有
                        let mut core_guard = core.lock().await;
                        if core_guard.is_none() {
                            if let Ok(dw) = DataWise::new() {
                                *core_guard = Some(dw);
                            }
                        }

                        if let Some(dw) = core_guard.as_ref() {
                            let cmd = Command {
                                task_id,
                                cmd_type: CmdType::ExecuteSql { sql },
                            };

                            if let Err(e) = dw.handle(cmd).await {
                                eprintln!("Error executing SQL: {}", e);
                            }
                        }
                    });
                }

                ui.label(format!("Status: {}", self.status));
            });

            ui.separator();

            // SQL 输入区域
            ui.label("SQL Query:");
            ui.text_edit_multiline(&mut self.sql_input);

            ui.separator();

            // 结果区域
            ui.label("Results:");
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.text_edit_multiline(&mut self.results);
                });

            ui.separator();

            // 底部状态栏
            ui.horizontal(|ui| {
                ui.label(format!("Executing: {}", self.is_executing));
                if ui.button("Clear Results").clicked() {
                    self.results.clear();
                }
            });
        });
    }
}

impl DataWiseApp {
    fn handle_event(&mut self, event: UiEvent) {
        match event.kind {
            EventKind::Started => {
                self.status = "Query started...".to_string();
            }
            EventKind::Progress { pct, .. } => {
                self.status = format!("Progress: {}%", pct);
            }
            EventKind::Finished { row_count, column_count, preview } => {
                self.is_executing = false;
                self.status = format!("Completed: {} rows, {} columns", row_count, column_count);
                self.results = format!("Rows: {}\nColumns: {}\n\nPreview:\n{}",
                    row_count, column_count, preview);
            }
            EventKind::Error(e) => {
                self.is_executing = false;
                self.status = format!("Error: {}", e);
                self.results = format!("Error: {}", e);
            }
        }
    }
}

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::error::Error;
use std::io;
use datawise_core::{DataWise, Command, CmdType};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建应用
    let app = App::new();
    let res = run_app(&mut terminal, app).await;

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

struct App {
    sql_input: String,
    results: Vec<String>,
    status: String,
    is_executing: bool,
    core: Arc<Mutex<Option<DataWise>>>,
    task_id: u64,
    input_mode: bool,
    error_message: Option<String>,
}

impl App {
    fn new() -> Self {
        Self {
            sql_input: "SELECT 1 as num".to_string(),
            results: vec!["Ready to execute SQL".to_string()],
            status: "Ready".to_string(),
            is_executing: false,
            core: Arc::new(Mutex::new(None)),
            task_id: 1,
            input_mode: true,
            error_message: None,
        }
    }

    fn handle_event(&mut self, event: datawise_core::UiEvent) {
        match event.kind {
            datawise_core::EventKind::Started => {
                self.status = "Query started...".to_string();
                self.error_message = None;
            }
            datawise_core::EventKind::Progress { pct, .. } => {
                self.status = format!("Progress: {}%", pct);
            }
            datawise_core::EventKind::Finished { row_count, column_count, preview } => {
                self.is_executing = false;
                self.status = format!("Completed: {} rows, {} columns", row_count, column_count);
                self.results = vec![
                    format!("Rows: {}", row_count),
                    format!("Columns: {}", column_count),
                    "".to_string(),
                    "Preview:".to_string(),
                ];
                // 解析 JSON 预览
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&preview) {
                    if let Some(arr) = json.as_array() {
                        for (i, row) in arr.iter().take(10).enumerate() {
                            self.results.push(format!("Row {}: {}", i + 1, row));
                        }
                    }
                } else {
                    self.results.push(preview);
                }
            }
            datawise_core::EventKind::Error(e) => {
                self.is_executing = false;
                self.status = "Error".to_string();
                self.error_message = Some(e.clone());
                self.results = vec![format!("Error: {}", e)];
            }
        }
    }
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(5),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.area());

            // 顶部状态栏
            let status_text = vec![Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Yellow)),
                Span::raw(&app.status),
                Span::raw(" | Press 'r' to execute, 'q' to quit, 'c' to clear"),
            ])];
            let status = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL).title("DataWise - tui"));
            f.render_widget(status, chunks[0]);

            // 中部结果区域
            let result_lines: Vec<Line> = app
                .results
                .iter()
                .map(|s| Line::from(s.as_str()))
                .collect();
            let results = Paragraph::new(result_lines)
                .block(Block::default().borders(Borders::ALL).title("Results"));
            f.render_widget(results, chunks[1]);

            // 底部 SQL 输入区域
            let input_lines: Vec<Line> = app
                .sql_input
                .lines()
                .map(|s| Line::from(s))
                .collect();
            let input = Paragraph::new(input_lines)
                .block(Block::default().borders(Borders::ALL).title("SQL Input"))
                .style(Style::default().fg(Color::Green));
            f.render_widget(input, chunks[2]);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('r') if !app.is_executing => {
                        // 执行 SQL
                        let sql = app.sql_input.clone();
                        let core = Arc::clone(&app.core);
                        let task_id = app.task_id;
                        app.task_id += 1;
                        app.is_executing = true;
                        app.status = "Executing...".to_string();
                        app.results.clear();

                        tokio::spawn(async move {
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
                                    eprintln!("Error: {}", e);
                                }
                            }
                        });
                    }
                    KeyCode::Char('c') => {
                        app.sql_input.clear();
                    }
                    KeyCode::Enter if app.input_mode => {
                        app.sql_input.push('\n');
                    }
                    KeyCode::Backspace if app.input_mode => {
                        app.sql_input.pop();
                    }
                    KeyCode::Char(c) if app.input_mode => {
                        app.sql_input.push(c);
                    }
                    _ => {}
                }
            }
        }
    }
}



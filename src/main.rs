use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::*,
};
use ratatui::{prelude::*, widgets::*};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use sysinfo::System;

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
    }
}

struct AppState {
    table_state: TableState,
    is_sorting_active: bool,
    selected_pid: Option<sysinfo::Pid>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let _guard = TerminalGuard;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;
    let mut sys = System::new_all();
    let mut state = AppState {
        table_state: TableState::default(),
        is_sorting_active: false,
        selected_pid: None,
    };

    let tick_rate = Duration::from_millis(1000);
    let mut last_tick = Instant::now();

    rtop_lib::refresh(&mut sys);
    let mut stats = rtop_lib::get_metrics(&sys);
    let mut processes: Vec<_> = stats.processes.values().collect();

    loop {
        if last_tick.elapsed() >= tick_rate {
            rtop_lib::refresh(&mut sys);
            stats = rtop_lib::get_metrics(&sys);
            processes = stats.processes.values().collect();

            if state.is_sorting_active {
                processes.sort_by(|a, b| {
                    b.cpu_usage
                        .partial_cmp(&a.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.pid.cmp(&b.pid))
                });
            }

            if let Some(pid) = state.selected_pid {
                if let Some(new_idx) = processes.iter().position(|p| p.pid == pid) {
                    state.table_state.select(Some(new_idx));
                } else {
                    state.table_state.select(None);
                    state.selected_pid = None;
                }
            }
            last_tick = Instant::now();
        }

        let cpu_percent = stats.cpu_usage as u16;
        let mem_percent = stats.mem_usage as u16;
        let mem_label = format!("{} / {}", stats.mem_used_str, stats.mem_total_str);

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(f.area());

            let header_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[0]);

            let cpu_gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" CPU "))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(cpu_percent);

            let ram_gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" RAM "))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(mem_percent)
                .label(mem_label);

            f.render_widget(cpu_gauge, header_chunks[0]);
            f.render_widget(ram_gauge, header_chunks[1]);

            let rows = processes.iter().map(|p| {
                Row::new(vec![
                    Cell::from(p.pid.to_string()),
                    Cell::from(p.name.to_string_lossy().into_owned()),
                    Cell::from(format!("{:.1}%", p.cpu_usage)),
                    Cell::from(p.disk_usage_str.clone()),
                ])
            });

            let table = Table::new(
                rows,
                [
                    Constraint::Length(8),
                    Constraint::Min(20),
                    Constraint::Length(10),
                    Constraint::Length(10),
                ],
            )
            .header(Row::new(vec!["PID", "Name", "CPU", "Disk"]).bold())
            .row_highlight_style(Style::default().bg(Color::Blue));

            f.render_stateful_widget(table, chunks[1], &mut state.table_state);

            let footer_items = vec![
                ("F1", "Help"),
                ("F2", "Setup"),
                ("F5", "Tree"),
                ("F9", "Kill"),
                ("F10", "Quit"),
            ];
            let footer_spans: Vec<Span> = footer_items
                .iter()
                .flat_map(|(k, v)| {
                    vec![
                        Span::styled(
                            format!(" {} ", k),
                            Style::default().bg(Color::Cyan).fg(Color::Black),
                        ),
                        Span::raw(format!("{} ", v)),
                    ]
                })
                .collect();

            f.render_widget(Paragraph::new(Line::from(footer_spans)), chunks[2]);
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::F(10) => break,
                    KeyCode::Up => {
                        let i = match state.table_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    processes.len().saturating_sub(1)
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        state.table_state.select(Some(i));
                        if let Some(p) = processes.get(i) {
                            state.selected_pid = Some(p.pid);
                        }
                    }
                    KeyCode::Down => {
                        let i = match state.table_state.selected() {
                            Some(i) => {
                                if i >= processes.len().saturating_sub(1) {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        state.table_state.select(Some(i));
                        if let Some(p) = processes.get(i) {
                            state.selected_pid = Some(p.pid);
                        }
                    }
                    KeyCode::Char('k') | KeyCode::F(9) => {
                        if let Some(pid) = state.selected_pid {
                            if let Some(proc) = sys.process(pid) {
                                proc.kill();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode},
    execute,
    terminal::*,
};
use ratatui::{prelude::*, widgets::*};
use std::{error::Error, time::Duration};
use sysinfo::System;

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = std::io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;
    let mut sys = System::new_all();

    loop {
        rtop_lib::refresh(&mut sys);
        let stats = rtop_lib::get_metrics(&sys);

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // header
                    Constraint::Min(0),    // content
                    Constraint::Length(1), // footer
                ])
                .split(f.area());

            // chunks
            let header_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[0]);

            // header
            let cpu_gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" CPU "))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(stats.cpu_usage as u16);

            let ram_gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(" RAM "))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(stats.mem_usage as u16)
                .label(format!("{} / {}", stats.mem_used_str, stats.mem_total_str));

            f.render_widget(cpu_gauge, header_chunks[0]);
            f.render_widget(ram_gauge, header_chunks[1]);

            // main content
            let block = Block::default().title("> rtop <").borders(Borders::ALL);
            let text = format!("");
            f.render_widget(Paragraph::new(text).block(block), chunks[1]);

            // footer
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

        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::F(10) {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

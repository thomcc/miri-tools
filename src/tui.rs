use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use color_eyre::Report;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::backend::Backend;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::style::Style;
use tui::style::{Color, Modifier};
use tui::symbols;
use tui::text::Span;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Cell;
use tui::widgets::LineGauge;
use tui::widgets::{Gauge, Row, Table};
use tui::Frame;
use tui::{backend::CrosstermBackend, Terminal};

use crate::Crate;

pub fn run(
    crate_queue: Arc<Mutex<Vec<Crate>>>,
    crates_currently_running: Arc<Mutex<Vec<(Crate, Instant)>>>,
) -> Result<(), Report> {
    let total_num_crates = crate_queue.lock().unwrap().len();
    let start_time = Instant::now();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    'outer: loop {
        {
            let current_queue_len = crate_queue.lock().unwrap().len();
            if current_queue_len == 0 && crates_running == 0 {
                break;
            }
            let mut crates_running = crates_currently_running.lock().unwrap();
            terminal
                .draw(|f| {
                    render(
                        f,
                        &mut crates_running,
                        start_time,
                        total_num_crates,
                        current_queue_len,
                    )
                })
                .unwrap();
        }
        std::thread::sleep(Duration::from_secs(1));
        while let Ok(true) = event::poll(Duration::ZERO) {
            if let Event::Key(event) = event::read().unwrap() {
                if event.modifiers.contains(KeyModifiers::CONTROL)
                    && event.code == KeyCode::Char('c')
                {
                    break 'outer;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn render<B: Backend>(
    f: &mut Frame<B>,
    crates: &mut [(Crate, Instant)],
    start_time: Instant,
    total_crates: usize,
    current_queue_len: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Length(3), Constraint::Min(2)].as_ref())
        .split(f.size());

    let crates_completed = if total_crates == current_queue_len {
        1
    } else {
        total_crates - current_queue_len
    };

    let time_per_crate = start_time.elapsed() / crates_completed as u32;
    let total_runtime = time_per_crate * total_crates as u32;
    let total_runtime = Duration::from_secs(total_runtime.as_secs());

    let elapsed = start_time.elapsed().as_secs();
    let elapsed = Duration::from_secs(elapsed);

    let label = Span::styled(
        format!(
            "{} / {}",
            humantime::format_duration(elapsed),
            humantime::format_duration(total_runtime)
        ),
        Style::default().fg(Color::White).bg(Color::Black),
    );

    let progress = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::White).bg(Color::Black))
        .ratio(crates_completed as f64 / total_crates as f64)
        .label(label);
    f.render_widget(progress, chunks[0]);

    crates.sort_by(|a, b| a.1.cmp(&b.1));
    let table = Table::new(crates.iter().map(|(krate, start)| {
        let elapsed = start.elapsed().as_secs();
        let elapsed = Duration::from_secs(elapsed);
        Row::new([
            Cell::from(krate.to_string()),
            Cell::from(humantime::format_duration(elapsed).to_string()),
        ])
    }))
    .header(Row::new(vec!["Crate".to_string(), "Elapsed".to_string()]).bottom_margin(1))
    .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);
    f.render_widget(table, chunks[1]);
}

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::AppState;
use crate::input::handle_event;
use crate::render::draw;

pub fn run(dry_run: bool) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Clear the primary screen so preflight output doesn't persist when exiting the TUI
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Ensure the alternate screen starts clean, especially on Linux TTYs
    terminal.clear()?;

    // TODO(v0.5.0): Add unattended/automation mode that bypasses TUI and executes a config.
    let res = run_loop(&mut terminal, dry_run);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    dry_run: bool,
) -> io::Result<()> {
    let mut app = AppState::new(dry_run);
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        // Drain any pending install logs before rendering
        app.drain_install_logs();
        terminal.draw(|frame| draw(frame, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            let ev: Event = event::read()?;
            if handle_event(&mut app, ev) {
                break;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}

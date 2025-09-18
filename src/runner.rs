use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{execute, terminal};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::AppState;
use crate::common::utils::redact_command_for_logging;
use crate::input::handle_event;
use crate::render::draw;
use std::process::Command;

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

        if app.exit_tui_after_install {
            break;
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        // If requested, leave TUI and run install plan with inherited stdio
        if app.exit_tui_after_install
            && let Some(sections) = app.pending_install_sections.take()
        {
            // Ensure normal terminal restored
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
            terminal.show_cursor()?;

            println!("Starting installation...\n");
            let mut any_error = None::<String>;
            'outer: for (title, cmds) in sections {
                println!("=== {} ===", title);
                for c in cmds {
                    println!("$ {}", redact_command_for_logging(&c));
                    let status = Command::new("bash").arg("-lc").arg(&c).status();
                    match status {
                        Ok(st) if st.success() => {}
                        Ok(st) => {
                            any_error = Some(format!(
                                "Command failed (exit {}): {}",
                                st.code().unwrap_or(-1),
                                redact_command_for_logging(&c)
                            ));
                            eprintln!("{}", any_error.as_ref().unwrap());
                            break 'outer;
                        }
                        Err(e) => {
                            any_error = Some(format!(
                                "Failed to run: {} ({})",
                                redact_command_for_logging(&c),
                                e
                            ));
                            eprintln!("{}", any_error.as_ref().unwrap());
                            break 'outer;
                        }
                    }
                }
                println!();
            }
            if any_error.is_none() {
                println!("Installation completed.");
                print!("Do you want to reboot now? [Y/n] ");
                io::stdout().flush()?;
                let mut answer = String::new();
                io::stdin().read_line(&mut answer)?;
                let ans = answer.trim();
                if ans.is_empty()
                    || ans.eq_ignore_ascii_case("y")
                    || ans.eq_ignore_ascii_case("yes")
                {
                    let _ = Command::new("bash").arg("-lc").arg("reboot").status();
                }
            }
        }
    }

    Ok(())
}

use std::io::{self};
use std::time::{Duration, Instant};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event};
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
    execute!(stdout, terminal::EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Ensure the alternate screen starts clean, especially on Linux TTYs
    terminal.clear()?;

    // TODO(v0.5.0): Add unattended/automation mode that bypasses TUI and executes a config.
    let res = run_loop(&mut terminal, dry_run);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        terminal::LeaveAlternateScreen
    )?;
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

    // For logging to file (only for non-dry-run)
    let mut log_file = if !dry_run {
        Some(std::fs::File::create("run.log").expect("Failed to create run.log"))
    } else {
        None
    };
    let mut last_logged_line: usize = 0;

    loop {
        // Drain any pending install logs before rendering
        app.drain_install_logs();

        // If install just finished (not dry-run), trigger reboot prompt
        if !app.dry_run && app.install_completed && !app.reboot_prompt_open && app.reboot_confirmed.is_none() {
            app.reboot_prompt_open = true;
        }

        // Write new log lines to file if not dry_run
        if let Some(file) = log_file.as_mut() {
            while last_logged_line < app.install_log.len() {
                if let Some(line) = app.install_log.get(last_logged_line) {
                    use std::io::Write;
                    let _ = writeln!(file, "{}", line);
                }
                last_logged_line += 1;
            }
        }

        terminal.draw(|frame| draw(frame, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            let ev: Event = event::read()?;
            // If reboot prompt is open, handle Y/N keys
            if app.reboot_prompt_open {
                if let crossterm::event::Event::Key(key) = &ev {
                    use crossterm::event::KeyCode;
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            app.reboot_confirmed = Some(true);
                            app.reboot_prompt_open = false;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.reboot_confirmed = Some(false);
                            app.reboot_prompt_open = false;
                        }
                        _ => {}
                    }
                }
            } else if handle_event(&mut app, ev) {
                break;
            }
        }

        // After reboot prompt, exit TUI and perform reboot if confirmed
        if let Some(confirmed) = app.reboot_confirmed {
            if confirmed {
                // Exit TUI and reboot
                break;
            } else {
                // Exit TUI, no reboot
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
                // Mark install as completed for reboot prompt
                app.install_completed = true;
            }
        }
    }

    // After TUI closes, perform reboot if confirmed
    if let Some(true) = app.reboot_confirmed {
        let _ = Command::new("bash").arg("-lc").arg("reboot").status();
    }
    Ok(())
}

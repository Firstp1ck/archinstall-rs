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

fn debug_log(enabled: bool, msg: &str) {
    if !enabled {
        return;
    }
    let now = chrono::Local::now();
    let ts = now.format("%Y-%m-%d %H:%M:%S");
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
        .and_then(|mut f| {
            use std::io::Write;
            writeln!(f, "[DEBUG {}] {}", ts, msg)
        });
}

// Prettify noisy pacman/pacstrap output for run.log
fn format_runlog_line(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return Some(String::new());
    }
    // Collapse pacman download lines: "<pkg> downloading..."
    if let Some(pos) = s.find(" downloading...") {
        let pkg = s[..pos].trim();
        if !pkg.is_empty() {
            return Some(format!("download: {}", pkg));
        }
    }
    // Collapse pacman installing lines: "installing <pkg>..."
    let sl = s.to_lowercase();
    if let Some(rest) = sl.strip_prefix("installing ") {
        // Find token up to dots or space
        let end = rest.find(' ').or_else(|| rest.find('.')).unwrap_or(rest.len());
        let pkg = &s["installing ".len()..("installing ".len() + end)];
        return Some(format!("install: {}", pkg));
    }
    // Warnings
    if sl.starts_with("warning:") {
        return Some(format!("warn: {}", s[8..].trim()));
    }
    // Total sizes
    if let Some(rest) = s.strip_prefix("Total Download Size:") {
        return Some(format!("download size:{}", rest));
    }
    if let Some(rest) = s.strip_prefix("Total Installed Size:") {
        return Some(format!("installed size:{}", rest));
    }
    // Proceed question is irrelevant in non-interactive mode
    if s.starts_with(":: Proceed with installation?") {
        return None;
    }
    Some(raw.to_string())
}

pub fn run_with_debug(dry_run: bool, debug_enabled: bool) -> io::Result<()> {
    debug_log(debug_enabled, "TUI init: enable_raw_mode");
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Clear the primary screen so preflight output doesn't persist when exiting the TUI
    debug_log(debug_enabled, "TUI init: clearing primary screen");
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    debug_log(
        debug_enabled,
        "TUI init: enter alternate screen + enable mouse",
    );
    execute!(stdout, terminal::EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Ensure the alternate screen starts clean, especially on Linux TTYs
    terminal.clear()?;
    debug_log(debug_enabled, "TUI init: terminal initialized and cleared");

    debug_log(debug_enabled, "run loop: start run_loop_with_debug");
    let res = run_loop_with_debug(&mut terminal, dry_run, debug_enabled);
    debug_log(
        debug_enabled,
        &format!(
            "run loop: finished with {}",
            if res.is_ok() { "Ok" } else { "Err" }
        ),
    );

    debug_log(
        debug_enabled,
        "TUI teardown: disable_raw_mode, leave alternate screen",
    );
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    debug_log(debug_enabled, "TUI teardown: completed");

    res
}

pub fn run(dry_run: bool) -> io::Result<()> {
    run_with_debug(dry_run, false)
}

fn run_loop_with_debug(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    dry_run: bool,
    debug_enabled: bool,
) -> io::Result<()> {
    debug_log(
        debug_enabled,
        &format!(
            "run_loop_with_debug: creating AppState dry_run={} debug_enabled={}",
            dry_run, debug_enabled
        ),
    );
    let mut app = AppState::new(dry_run);
    app.debug_enabled = debug_enabled;
    debug_log(
        debug_enabled,
        "run_loop_with_debug: entering run_loop_inner",
    );
    run_loop_inner(terminal, &mut app)
}

fn run_loop_inner(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    // Throttle Resize logs
    let mut last_resize_log_ts = Instant::now();
    let mut last_resize_w: u16 = 0;
    let mut last_resize_h: u16 = 0;

    // For logging to file (only for non-dry-run)
    let mut log_file = if !app.dry_run {
        debug_log(app.debug_enabled, "run.log: creating run.log");
        Some(std::fs::File::create("run.log").expect("Failed to create run.log"))
    } else {
        None
    };
    let mut last_logged_line: usize = 0;
    let mut first_log_write_done = false;
    let mut prev_runlog_line: Option<String> = None;

    let mut install_was_running = false;
    debug_log(app.debug_enabled, "run_loop_inner: start main loop");
    loop {
        // Drain any pending install logs before rendering
        app.drain_install_logs();

        // Detect install_running transition
        if app.install_running != install_was_running {
            debug_log(
                app.debug_enabled,
                &format!(
                    "state: install_running changed {} -> {}",
                    install_was_running, app.install_running
                ),
            );
        }

        // Detect install completion in TUI-driven flow
        if !app.dry_run
            && install_was_running
            && !app.install_running
            && !app.install_completed
            && !app.install_section_titles.is_empty()
            && app.install_section_done.iter().all(|&done| done)
        {
            app.install_completed = true;
            debug_log(app.debug_enabled, "state: install_completed=true");
        }
        install_was_running = app.install_running;

        // If install just finished (not dry-run), trigger reboot prompt
        if !app.dry_run
            && app.install_completed
            && !app.reboot_prompt_open
            && app.reboot_confirmed.is_none()
        {
            app.reboot_prompt_open = true;
            debug_log(app.debug_enabled, "state: reboot_prompt_open=true");
        }

        // Write new log lines to file if not dry_run
        if let Some(file) = log_file.as_mut() {
            use std::io::Write;
            while last_logged_line < app.install_log.len() {
                if let Some(line) = app.install_log.get(last_logged_line) {
                    if let Some(pretty) = format_runlog_line(line) {
                        if prev_runlog_line.as_deref() != Some(pretty.as_str()) {
                            let _ = writeln!(file, "{}", pretty);
                            prev_runlog_line = Some(pretty);
                            if !first_log_write_done {
                                debug_log(app.debug_enabled, "run.log: first write");
                                first_log_write_done = true;
                            }
                        }
                    }
                }
                last_logged_line += 1;
            }
        }

        terminal.draw(|frame| draw(frame, app))?;

        let elapsed = last_tick.elapsed();
        let timeout = tick_rate
            .checked_sub(elapsed)
            .unwrap_or_else(|| Duration::from_secs(0));
        // removed verbose heartbeat debug log

        if event::poll(timeout)? {
            let ev: Event = event::read()?;
            // Reduce noisy Resize spam: log first and then at most twice per second and only on meaningful change
            if let Event::Resize(w, h) = &ev {
                let significant = (i32::from(*w) - i32::from(last_resize_w)).abs()
                    + (i32::from(*h) - i32::from(last_resize_h)).abs()
                    >= 5;
                if last_resize_w == 0 && last_resize_h == 0
                    || (significant && last_resize_log_ts.elapsed() > Duration::from_millis(500))
                {
                    debug_log(app.debug_enabled, &format!("event: Resize {}x{}", w, h));
                    last_resize_log_ts = Instant::now();
                    last_resize_w = *w;
                    last_resize_h = *h;
                }
            }

            // If reboot prompt is open, handle Y/N keys
            if app.reboot_prompt_open {
                if let crossterm::event::Event::Key(key) = &ev {
                    use crossterm::event::KeyCode;
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            app.reboot_confirmed = Some(true);
                            app.reboot_prompt_open = false;
                            debug_log(app.debug_enabled, "state: reboot_confirmed=true");
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.reboot_confirmed = Some(false);
                            app.reboot_prompt_open = false;
                            debug_log(app.debug_enabled, "state: reboot_confirmed=false");
                        }
                        _ => {}
                    }
                }
            } else if handle_event(app, ev) {
                debug_log(app.debug_enabled, "loop: handle_event requested exit");
                break;
            }
        }

        // After reboot prompt, exit TUI and perform reboot if confirmed
        if let Some(confirmed) = app.reboot_confirmed {
            debug_log(
                app.debug_enabled,
                &format!("post-tui: reboot decision confirmed={}", confirmed),
            );
            if confirmed {
                // Exit TUI and reboot
                break;
            } else {
                // Exit TUI, no reboot
                break;
            }
        }
        if app.exit_tui_after_install {
            debug_log(
                app.debug_enabled,
                "post-tui: exit_tui_after_install set, leaving loop",
            );
            break;
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        // If requested, leave TUI and run install plan with inherited stdio
        if app.exit_tui_after_install
            && let Some(sections) = app.pending_install_sections.take()
        {
            debug_log(
                app.debug_enabled,
                &format!(
                    "stdout-mode: taking pending_install_sections count={}",
                    sections.len()
                ),
            );
            // Ensure normal terminal restored
            disable_raw_mode()?;
            // Ensure we disable mouse capture and leave alt screen cleanly
            execute!(
                terminal.backend_mut(),
                DisableMouseCapture,
                terminal::LeaveAlternateScreen
            )?;
            terminal.show_cursor()?;

            // Hard clear and reset primary screen to avoid leftover TUI borders/artifacts
            use crossterm::{cursor, style::ResetColor};
            let mut out = io::stdout();
            execute!(
                out,
                ResetColor,
                terminal::Clear(terminal::ClearType::All),
                cursor::MoveTo(0, 0)
            )?;

            println!("Starting installation...\n");
            let mut any_error = None::<String>;
            'outer: for (title, cmds) in sections {
                debug_log(
                    app.debug_enabled,
                    &format!(
                        "stdout-mode: section '{}' with {} commands",
                        title,
                        cmds.len()
                    ),
                );
                println!("=== {} ===", title);
                for c in cmds {
                    let red = redact_command_for_logging(&c);
                    println!("$ {}", red);
                    debug_log(app.debug_enabled, &format!("stdout-mode: run '{}'", red));
                    let status = Command::new("bash").arg("-lc").arg(&c).status();
                    match status {
                        Ok(st) if st.success() => {
                            debug_log(app.debug_enabled, "stdout-mode: command OK");
                        }
                        Ok(st) => {
                            let code = st.code().unwrap_or(-1);
                            let msg = format!("Command failed (exit {}): {}", code, red);
                            any_error = Some(msg.clone());
                            eprintln!("{}", any_error.as_ref().unwrap());
                            debug_log(
                                app.debug_enabled,
                                &format!("stdout-mode: command failed exit_code={}", code),
                            );
                            break 'outer;
                        }
                        Err(e) => {
                            let msg = format!("Failed to run: {} ({})", red, e);
                            any_error = Some(msg.clone());
                            eprintln!("{}", any_error.as_ref().unwrap());
                            debug_log(
                                app.debug_enabled,
                                &format!("stdout-mode: command spawn error: {}", e),
                            );
                            break 'outer;
                        }
                    }
                }
                println!();
            }
            if any_error.is_none() {
                // Mark install as completed for reboot prompt
                app.install_completed = true;
                debug_log(app.debug_enabled, "stdout-mode: install_completed=true");
            }
        }
    }

    // After TUI closes, perform reboot if confirmed
    if let Some(true) = app.reboot_confirmed {
        debug_log(app.debug_enabled, "post-tui: attempting reboot");
        let res = Command::new("bash").arg("-lc").arg("reboot").status();
        match res {
            Ok(st) if st.success() => {
                debug_log(app.debug_enabled, "post-tui: reboot command succeeded")
            }
            Ok(st) => debug_log(
                app.debug_enabled,
                &format!("post-tui: reboot command exited with code {:?}", st.code()),
            ),
            Err(e) => debug_log(
                app.debug_enabled,
                &format!("post-tui: reboot command error: {}", e),
            ),
        }
    }
    debug_log(app.debug_enabled, "run_loop_inner: end");
    // run.log will be closed when file handle drops
    Ok(())
}

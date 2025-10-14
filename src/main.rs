pub mod app;
pub mod common;
pub mod core;
pub mod input;
pub mod render;
pub mod runner;

use std::io::Write;

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
            writeln!(f, "[DEBUG {ts}] {msg}")
        });
}

fn print_info(debug_enabled: bool, msg: &str) {
    println!("{msg}");
    debug_log(debug_enabled, &format!("info: {msg}"));
}

fn print_error(debug_enabled: bool, msg: &str) {
    eprintln!("{msg}");
    debug_log(debug_enabled, &format!("error: {msg}"));
}

fn main() -> std::io::Result<()> {
    // Detect flags first
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|arg| arg == "--dry-run" || arg == "--dry");
    let debug_enabled = args.iter().any(|arg| arg == "--debug");
    debug_log(
        debug_enabled,
        &format!("main: parsed flags dry_run={dry_run} debug_enabled={debug_enabled}"),
    );

    debug_log(debug_enabled, "preflight: start");
    let had_warnings = run_preflight_checks(dry_run, debug_enabled);
    debug_log(
        debug_enabled,
        &format!("preflight: end had_warnings={had_warnings}"),
    );

    if had_warnings {
        print_info(
            debug_enabled,
            "\nOne or more preflight warnings were detected.",
        );
        debug_log(debug_enabled, "preflight: warnings detected");
        // In dry-run, don't block with a prompt; just continue for TUI testing
        let will_prompt = !(dry_run && cfg!(windows));
        debug_log(
            debug_enabled,
            &format!(
                "preflight: prompt gating will_prompt={} (dry_run && windows = {})",
                will_prompt,
                dry_run && cfg!(windows)
            ),
        );
        if will_prompt {
            print!("Proceed anyway? [y/N]: ");
            let _ = std::io::stdout().flush();
            debug_log(
                debug_enabled,
                "preflight: prompting user for proceed anyway",
            );

            let mut answer = String::new();
            if let Err(err) = std::io::stdin().read_line(&mut answer) {
                print_error(debug_enabled, &format!("Failed to read input: {err}"));
                debug_log(debug_enabled, &format!("preflight: read_line error: {err}"));
                return Ok(());
            }

            let answer_trimmed = answer.trim().to_lowercase();
            let proceed = answer_trimmed == "y" || answer_trimmed == "yes";
            debug_log(
                debug_enabled,
                &format!("preflight: user decision proceed={proceed}"),
            );
            if !proceed {
                print_info(debug_enabled, "Aborted by user due to preflight warnings.");
                debug_log(debug_enabled, "preflight: aborted by user");
                return Ok(());
            }
        }
    }
    // TODO(v0.5.0): Parse config path and unattended flags to run non-interactively.
    runner::run_with_debug(dry_run, debug_enabled)
}

fn run_preflight_checks(dry_run: bool, debug_enabled: bool) -> bool {
    // On Windows with dry-run, skip preflight warnings entirely for smoother TUI testing
    if dry_run && cfg!(windows) {
        debug_log(
            debug_enabled,
            "preflight: skipping on Windows dry-run for smoother TUI testing",
        );
        return false;
    }

    let mut had_warning = false;
    // Check EFI platform size (Linux-only path)
    match std::fs::read_to_string("/sys/firmware/efi/fw_platform_size") {
        Ok(contents) => {
            let value = contents.trim();
            debug_log(
                debug_enabled,
                &format!("preflight: fw_platform_size read='{value}'"),
            );
            if value != "64" {
                print_info(debug_enabled, &format!("EFI fw_platform_size: {value}"));
                print_info(debug_enabled, "Warning: Bootmode is not 64-bit");
                debug_log(
                    debug_enabled,
                    "preflight: fw_platform_size != 64, warning set",
                );
                had_warning = true;
            }
        }
        Err(err) => {
            // If we can't read the file, surface a warning but don't block
            print_info(debug_enabled, "EFI fw_platform_size: unavailable");
            print_info(
                debug_enabled,
                "Warning: Bootmode is not 64-bit (could not determine)",
            );
            debug_log(
                debug_enabled,
                &format!("preflight: failed to read fw_platform_size: {err}"),
            );
            had_warning = true;
        }
    }

    // Check internet connectivity (use platform-appropriate ping)
    let archlinux_ok = check_host_connectivity("archlinux.org", debug_enabled);
    let google_ok = check_host_connectivity("google.com", debug_enabled);
    debug_log(
        debug_enabled,
        &format!(
            "preflight: connectivity results archlinux.org={archlinux_ok} google.com={google_ok}"
        ),
    );
    if !(archlinux_ok || google_ok) {
        had_warning = true;
    }

    // Check terminal color capabilities and advise for best experience
    let term = std::env::var("TERM").unwrap_or_default();
    let colorterm = std::env::var("COLORTERM").unwrap_or_default();
    let is_linux_console = term == "linux" || term == "dumb";
    let has_truecolor = colorterm.to_ascii_lowercase().contains("truecolor")
        || colorterm.to_ascii_lowercase().contains("24bit");
    let has_256 = has_truecolor || term.to_ascii_lowercase().contains("256color");
    debug_log(
        debug_enabled,
        &format!(
            "preflight: TERM='{}' COLORTERM='{}' linux_console={} has_256={} has_truecolor={}",
            term, colorterm, is_linux_console, has_256, has_truecolor
        ),
    );
    if is_linux_console || !has_256 {
        had_warning = true;
        print_info(
            debug_enabled,
            "Note: For proper colors, run in a terminal emulator (not raw TTY).",
        );
        print_info(
            debug_enabled,
            "Recommended: export TERM=xterm-256color and COLORTERM=truecolor",
        );
    }

    had_warning
}

fn check_host_connectivity(host: &str, debug_enabled: bool) -> bool {
    // Use platform-appropriate ping flags
    #[cfg(windows)]
    let args: [&str; 4] = ["-n", "1", "-w", "2000"]; // -n count, -w timeout(ms)
    #[cfg(not(windows))]
    let args: [&str; 4] = ["-c", "1", "-W", "2"]; // -c count, -W timeout(s)

    debug_log(
        debug_enabled,
        &format!("preflight: ping '{}' with args {:?}", host, &args),
    );

    let status = std::process::Command::new("ping")
        .args(args)
        .arg(host)
        .status();

    match status {
        Ok(s) if s.success() => {
            debug_log(debug_enabled, &format!("preflight: '{host}' reachable"));
            true
        }
        Ok(s) => {
            let code = s.code().unwrap_or(-1);
            print_info(
                debug_enabled,
                &format!("Network check failed: cannot reach {host}"),
            );
            debug_log(
                debug_enabled,
                &format!("preflight: '{host}' unreachable, exit_code={code}"),
            );
            false
        }
        Err(err) => {
            print_error(
                debug_enabled,
                &format!("Network check error for {host}: {err}"),
            );
            debug_log(
                debug_enabled,
                &format!("preflight: ping error for '{host}': {err}"),
            );
            false
        }
    }
}

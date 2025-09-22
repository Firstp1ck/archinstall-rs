pub mod app;
pub mod common;
pub mod core;
pub mod input;
pub mod render;
pub mod runner;

use std::io::Write;

fn main() -> std::io::Result<()> {
    // Detect flags first
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|arg| arg == "--dry-run" || arg == "--dry");
    let debug_enabled = args.iter().any(|arg| arg == "--debug");

    let had_warnings = run_preflight_checks(dry_run);
    if had_warnings {
        println!("\nOne or more preflight warnings were detected.");
        // In dry-run, don't block with a prompt; just continue for TUI testing
        if !(dry_run && cfg!(windows)) {
            print!("Proceed anyway? [y/N]: ");
            let _ = std::io::stdout().flush();

            let mut answer = String::new();
            if let Err(err) = std::io::stdin().read_line(&mut answer) {
                eprintln!("Failed to read input: {}", err);
                return Ok(());
            }

            let answer_trimmed = answer.trim().to_lowercase();
            if !(answer_trimmed == "y" || answer_trimmed == "yes") {
                println!("Aborted by user due to preflight warnings.");
                return Ok(());
            }
        }
    }
    // TODO(v0.5.0): Parse config path and unattended flags to run non-interactively.
    runner::run_with_debug(dry_run, debug_enabled)
}

fn run_preflight_checks(dry_run: bool) -> bool {
    // On Windows with dry-run, skip preflight warnings entirely for smoother TUI testing
    if dry_run && cfg!(windows) {
        return false;
    }

    let mut had_warning = false;
    // Check EFI platform size (Linux-only path)
    match std::fs::read_to_string("/sys/firmware/efi/fw_platform_size") {
        Ok(contents) => {
            let value = contents.trim();
            if value != "64" {
                println!("EFI fw_platform_size: {}", value);
                println!("Warning: Bootmode is not 64-bit");
                had_warning = true;
            }
        }
        Err(_err) => {
            // If we can't read the file, surface a warning but don't block
            println!("EFI fw_platform_size: unavailable");
            println!("Warning: Bootmode is not 64-bit (could not determine)");
            had_warning = true;
        }
    }

    // Check internet connectivity (use platform-appropriate ping)
    let archlinux_ok = check_host_connectivity("archlinux.org");
    let google_ok = check_host_connectivity("google.com");
    if !(archlinux_ok || google_ok) {
        had_warning = true;
    }

    had_warning
}

fn check_host_connectivity(host: &str) -> bool {
    // Use platform-appropriate ping flags
    #[cfg(windows)]
    let args: [&str; 4] = ["-n", "1", "-w", "2000"]; // -n count, -w timeout(ms)
    #[cfg(not(windows))]
    let args: [&str; 4] = ["-c", "1", "-W", "2"]; // -c count, -W timeout(s)

    let status = std::process::Command::new("ping")
        .args(args)
        .arg(host)
        .status();

    match status {
        Ok(s) if s.success() => true,
        Ok(_) => {
            println!("Network check failed: cannot reach {}", host);
            false
        }
        Err(err) => {
            println!("Network check error for {}: {}", host, err);
            false
        }
    }
}

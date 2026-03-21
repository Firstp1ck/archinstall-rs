pub mod app;
pub mod common;
pub mod core;
pub mod input;
pub mod render;
pub mod runner;

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
            "\nOne or more preflight warnings were detected. Continuing.",
        );
        debug_log(
            debug_enabled,
            "preflight: warnings detected; continuing without prompt",
        );
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

    // Check internet connectivity in parallel via TCP connect (fast, no subprocess overhead)
    let (archlinux_ok, google_ok) = std::thread::scope(|s| {
        let h1 = s.spawn(|| check_host_connectivity("archlinux.org", debug_enabled));
        let h2 = s.spawn(|| check_host_connectivity("google.com", debug_enabled));
        (h1.join().unwrap_or(false), h2.join().unwrap_or(false))
    });
    debug_log(
        debug_enabled,
        &format!(
            "preflight: connectivity results archlinux.org={archlinux_ok} google.com={google_ok}"
        ),
    );
    if !(archlinux_ok || google_ok) {
        had_warning = true;
    }

    // Check for kernel/module version mismatch (common on older Arch ISOs)
    if !dry_run {
        let kver = std::fs::read_to_string("/proc/version")
            .ok()
            .and_then(|v| v.split_whitespace().nth(2).map(String::from));
        if let Some(ref kver) = kver {
            let mod_dir = format!("/lib/modules/{kver}");
            debug_log(
                debug_enabled,
                &format!("preflight: checking module dir {mod_dir}"),
            );
            if !std::path::Path::new(&mod_dir).is_dir() {
                // List what module dirs DO exist
                let available: Vec<String> = std::fs::read_dir("/lib/modules")
                    .ok()
                    .map(|rd| {
                        rd.filter_map(|e| e.ok())
                            .filter_map(|e| e.file_name().into_string().ok())
                            .collect()
                    })
                    .unwrap_or_default();
                print_info(
                    debug_enabled,
                    &format!(
                        "Warning: Kernel/module version mismatch -- running kernel {kver} but modules on disk: {}",
                        if available.is_empty() {
                            "none".into()
                        } else {
                            available.join(", ")
                        }
                    ),
                );
                print_info(
                    debug_enabled,
                    "FAT/vfat modules may not load. Consider downloading a current Arch ISO.",
                );
                debug_log(
                    debug_enabled,
                    &format!(
                        "preflight: kernel/module mismatch kver={kver} available={available:?}"
                    ),
                );
                had_warning = true;
            }
        }
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
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    debug_log(
        debug_enabled,
        &format!("preflight: TCP connect check for '{host}:443'"),
    );

    let timeout = Duration::from_secs(2);

    match (host, 443u16).to_socket_addrs() {
        Ok(addrs) => {
            for addr in addrs {
                if TcpStream::connect_timeout(&addr, timeout).is_ok() {
                    debug_log(
                        debug_enabled,
                        &format!("preflight: '{host}' reachable via TCP"),
                    );
                    return true;
                }
            }
            print_info(
                debug_enabled,
                &format!("Network check failed: cannot reach {host}"),
            );
            debug_log(
                debug_enabled,
                &format!("preflight: '{host}' unreachable via TCP"),
            );
            false
        }
        Err(err) => {
            print_info(
                debug_enabled,
                &format!("Network check failed: cannot resolve {host}"),
            );
            debug_log(
                debug_enabled,
                &format!("preflight: DNS resolution failed for '{host}': {err}"),
            );
            false
        }
    }
}

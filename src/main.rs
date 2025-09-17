mod ui;

use std::io::Write;

fn main() -> std::io::Result<()> {
    let had_warnings = run_preflight_checks();
    if had_warnings {
        println!("\nOne or more preflight warnings were detected.");
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
    let dry_run = std::env::args().any(|arg| arg == "--dry-run" || arg == "--dry");
    ui::run(dry_run)
}

fn run_preflight_checks() -> bool {
    let mut had_warning = false;
    // Check EFI platform size
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

    // Check internet connectivity
    let archlinux_ok = check_host_connectivity("archlinux.org");
    let google_ok = check_host_connectivity("google.com");
    if !(archlinux_ok || google_ok) {
        had_warning = true;
    }

    had_warning
}

fn check_host_connectivity(host: &str) -> bool {
    let status = std::process::Command::new("ping")
        .args(["-c", "1", "-W", "2", host])
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

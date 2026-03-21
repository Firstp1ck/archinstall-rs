//! Install-time commands: mostly opaque shell strings, plus LUKS steps where the
//! passphrase must never be concatenated into a logged `bash -lc` script.

use std::io::{self, Write};
use std::process::{Command, Stdio};

/// One step of the installation plan.
#[derive(Clone, Debug)]
pub enum InstallCmd {
    /// Arbitrary shell fragment run under `bash -lc` inside `script(1)`.
    Shell(String),
    /// `cryptsetup luksFormat` with passphrase supplied via stdin (not in the shell string).
    CryptsetupLuksFormat { device: String, passphrase: String },
    /// `cryptsetup open` with passphrase supplied via stdin.
    CryptsetupOpen {
        device: String,
        mapper: String,
        passphrase: String,
    },
}

impl InstallCmd {
    pub fn shell(cmd: impl Into<String>) -> Self {
        Self::Shell(cmd.into())
    }

    /// Safe text for dry-run logs, live install logs, and debug (no raw passphrase).
    pub fn for_log(&self) -> String {
        match self {
            InstallCmd::Shell(s) => crate::common::utils::redact_command_for_logging(s),
            InstallCmd::CryptsetupLuksFormat { device, .. } => format!(
                "cryptsetup luksFormat --type luks2 -q --key-file=- {}",
                shell_single_quote(device)
            ),
            InstallCmd::CryptsetupOpen { device, mapper, .. } => format!(
                "cryptsetup open --type luks --key-file=- {} {}",
                shell_single_quote(device),
                shell_single_quote(mapper)
            ),
        }
    }

    pub fn is_thin_pacstrap(&self) -> bool {
        match self {
            InstallCmd::Shell(s) => s.contains("pacstrap"),
            _ => false,
        }
    }

    /// Shell steps run as `bash -lc 'script -qfec "inner" /dev/null 2>&1'` so output is
    /// captured without `/dev/tty`. LUKS steps spawn `cryptsetup` directly with a piped stdin:
    /// wrapping them in `script` would give `cryptsetup` a PTY as stdin, so `--key-file=-`
    /// would not receive the passphrase and would error (e.g. "Error reading passphrase from terminal").
    ///
    /// For LUKS variants, [`Self::write_passphrase_to_stdin`] must be called after spawn.
    pub fn spawn_script_pipeline(&self, stdout: Stdio) -> io::Result<std::process::Child> {
        match self {
            InstallCmd::CryptsetupLuksFormat { device, .. } => {
                let mut cmd = Command::new("cryptsetup");
                cmd.args([
                    "luksFormat",
                    "--type",
                    "luks2",
                    "-q",
                    "--key-file=-",
                    device.as_str(),
                ])
                .stdin(Stdio::piped())
                .stdout(stdout);
                configure_install_command(&mut cmd);
                cmd.spawn()
            }
            InstallCmd::CryptsetupOpen {
                device,
                mapper,
                ..
            } => {
                let mut cmd = Command::new("cryptsetup");
                cmd.args([
                    "open",
                    "--type",
                    "luks",
                    "--key-file=-",
                    device.as_str(),
                    mapper.as_str(),
                ])
                .stdin(Stdio::piped())
                .stdout(stdout);
                configure_install_command(&mut cmd);
                cmd.spawn()
            }
            InstallCmd::Shell(c) => {
                let escaped = c.replace('"', "\\\"");
                let pipeline = format!("script -qfec \"{escaped}\" /dev/null 2>&1");
                let mut cmd = Command::new("bash");
                cmd.arg("-lc")
                    .arg(&pipeline)
                    .stdin(Stdio::null())
                    .stdout(stdout);
                configure_install_command(&mut cmd);
                cmd.spawn()
            }
        }
    }

    pub fn write_passphrase_to_stdin(&self, child: &mut std::process::Child) -> io::Result<()> {
        match self {
            InstallCmd::Shell(_) => Ok(()),
            InstallCmd::CryptsetupLuksFormat { passphrase, .. }
            | InstallCmd::CryptsetupOpen { passphrase, .. } => {
                let mut stdin = child
                    .stdin
                    .take()
                    .ok_or_else(|| io::Error::other("cryptsetup child missing stdin pipe"))?;
                stdin.write_all(passphrase.as_bytes())?;
                Ok(())
            }
        }
    }
}

pub fn configure_install_command(cmd: &mut Command) -> &mut Command {
    cmd.env("TERM", "dumb")
        .env("NO_COLOR", "1")
        .env("PACMAN_COLOR", "never")
        .env("SYSTEMD_PAGER", "cat")
        .env("SYSTEMD_COLORS", "0")
        .env("PAGER", "cat")
        .env("LESS", "FRX")
        .env(
            "PACMAN",
            "pacman --noconfirm --noprogressbar --color never --quiet",
        )
}

fn shell_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

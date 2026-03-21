/// Messages from the install thread into the TUI install log.
#[derive(Debug, Clone)]
pub enum InstallLogMsg {
    /// Append a new log line.
    Line(String),
    /// Replace the last log line (in-place progress updates, e.g. `[###---] 42%`).
    ReplaceLastLine(String),
}

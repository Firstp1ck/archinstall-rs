//! Decode install child stdout and split on `\\n` / `\\r` for TUI logging.
//!
//! Matches pacsea-style heuristics: lines that look like bracketed progress with a `%` update the
//! last log row instead of appending a new one.

use std::io::{self, Read};

use crate::common::utils::sanitize_terminal_output_line;

/// Same heuristic as pacsea: bracketed bar plus a percent sign.
#[inline]
pub fn looks_like_tty_progress_line(s: &str) -> bool {
    let t = s.trim();
    !t.is_empty() && t.contains('[') && t.contains(']') && t.contains('%')
}

/// Feed decoded text into line assembly; `on_full_line` receives **raw** lines (caller sanitizes).
pub fn process_install_stdout_text_chunk<F, G>(text: &str, line_buf: &mut String, mut on_full_line: F, mut on_replace_last: G)
where
    F: FnMut(&str),
    G: FnMut(&str),
{
    for ch in text.chars() {
        match ch {
            '\n' => {
                let raw = std::mem::take(line_buf);
                if !raw.trim().is_empty() {
                    on_full_line(&raw);
                }
            }
            '\r' => {
                let raw = std::mem::take(line_buf);
                if raw.trim().is_empty() {
                    continue;
                }
                let clean = sanitize_terminal_output_line(&raw);
                if clean.is_empty() {
                    continue;
                }
                if looks_like_tty_progress_line(&clean) {
                    on_replace_last(&clean);
                } else {
                    on_full_line(&raw);
                }
            }
            _ => line_buf.push(ch),
        }
    }
}

fn decode_and_process_bytes<F, G>(byte_buf: &mut Vec<u8>, line_buf: &mut String, on_full_line: &mut F, on_replace_last: &mut G)
where
    F: FnMut(&str),
    G: FnMut(&str),
{
    loop {
        if let Ok(text) = String::from_utf8(byte_buf.clone()) {
            byte_buf.clear();
            process_install_stdout_text_chunk(&text, line_buf, |s| on_full_line(s), |s| on_replace_last(s));
            break;
        }
        if byte_buf.len() < 4 {
            break;
        }
        let mut found_valid = false;
        for trim_len in 1..=4.min(byte_buf.len()) {
            let test_len = byte_buf.len().saturating_sub(trim_len);
            if test_len == 0 {
                break;
            }
            if let Ok(text) = String::from_utf8(byte_buf[..test_len].to_vec()) {
                process_install_stdout_text_chunk(&text, line_buf, |s| on_full_line(s), |s| on_replace_last(s));
                byte_buf.drain(..test_len);
                found_valid = true;
                break;
            }
        }
        if !found_valid {
            let text = String::from_utf8_lossy(byte_buf);
            process_install_stdout_text_chunk(&text, line_buf, |s| on_full_line(s), |s| on_replace_last(s));
            byte_buf.clear();
            break;
        }
    }
}

/// Read all bytes from `reader`, decode UTF-8, and emit log lines / in-place progress updates.
pub fn pump_install_stdout<R, F, G>(mut reader: R, line_buf: &mut String, byte_buf: &mut Vec<u8>, scratch: &mut [u8], mut on_full_line: F, mut on_replace_last: G) -> io::Result<()>
where
    R: Read,
    F: FnMut(&str),
    G: FnMut(&str),
{
    loop {
        let n = match reader.read(scratch) {
            Ok(0) => {
                decode_and_process_bytes(byte_buf, line_buf, &mut on_full_line, &mut on_replace_last);
                break;
            }
            Ok(n) => n,
            Err(e) => return Err(e),
        };
        byte_buf.extend_from_slice(&scratch[..n]);
        decode_and_process_bytes(byte_buf, line_buf, &mut on_full_line, &mut on_replace_last);
    }
    if !line_buf.trim().is_empty() {
        on_full_line(line_buf.as_str());
        line_buf.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_carriage_return_replaces_not_duplicates() {
        let mut line_buf = String::new();
        let mut lines: Vec<String> = Vec::new();
        let mut last_replace: Option<String> = None;
        // No trailing newline: each progress frame ends with `\r`; final `\r` emits last replace.
        process_install_stdout_text_chunk(
            "\r[##------] 25%\r[####----] 50%\r",
            &mut line_buf,
            |raw| {
                lines.push(crate::common::utils::sanitize_terminal_output_line(raw));
            },
            |clean| {
                last_replace = Some(clean.to_string());
            },
        );
        assert!(lines.is_empty());
        assert_eq!(last_replace.as_deref(), Some("[####----] 50%"));
        assert!(line_buf.is_empty());
    }

    #[test]
    fn newline_after_progress_commits_final_line() {
        let mut line_buf = String::new();
        let mut lines: Vec<String> = Vec::new();
        process_install_stdout_text_chunk(
            "[####----] 50%\n",
            &mut line_buf,
            |raw| {
                lines.push(crate::common::utils::sanitize_terminal_output_line(raw));
            },
            |_| panic!("no replace"),
        );
        assert_eq!(lines, vec!["[####----] 50%"]);
    }

    #[test]
    fn newline_flushes_line() {
        let mut line_buf = String::new();
        let mut lines: Vec<String> = Vec::new();
        process_install_stdout_text_chunk(
            "hello world\n",
            &mut line_buf,
            |raw| {
                lines.push(crate::common::utils::sanitize_terminal_output_line(raw));
            },
            |_| panic!("no replace"),
        );
        assert_eq!(lines, vec!["hello world"]);
    }

    #[test]
    fn non_progress_cr_becomes_full_line() {
        let mut line_buf = String::new();
        let mut lines: Vec<String> = Vec::new();
        process_install_stdout_text_chunk(
            "loading\r",
            &mut line_buf,
            |raw| {
                lines.push(crate::common::utils::sanitize_terminal_output_line(raw));
            },
            |_| panic!("no replace"),
        );
        assert_eq!(lines, vec!["loading"]);
    }
}

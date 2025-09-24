use crate::app::AppState;

impl AppState {
    pub fn is_ascii_only(input: &str) -> bool {
        input.is_ascii()
    }

    pub fn is_ascii_lowercase_only(input: &str) -> bool {
        input.chars().all(|c| c.is_ascii_lowercase())
    }
}

/// Redact sensitive data (passwords) from shell command strings before logging/printing.
///
/// Currently handles:
/// - echo "user:password" | chpasswd  -> echo "user:<REDACTED>" | chpasswd
/// - echo 'user:password' | chpasswd  -> echo 'user:<REDACTED>' | chpasswd
/// - echo "pass" | cryptsetup ...     -> echo "<REDACTED>" | cryptsetup ...
/// - echo 'pass' | cryptsetup ...     -> echo '<REDACTED>' | cryptsetup ...
pub fn redact_command_for_logging(command: &str) -> String {
    let mut redacted = command.to_string();

    // Helper to redact the first quoted string after an `echo ` segment
    fn redact_after_echo_segment(s: &mut String, replacement_fn: impl FnOnce(&str) -> String) {
        if let Some(echo_idx) = s.find("echo ") {
            let rest = &s[echo_idx + 5..];
            // Find opening quote ' or "
            let mut open_quote_char = '\0';
            let mut rel_start = 0usize;
            for (i, ch) in rest.char_indices() {
                if ch == '"' || ch == '\'' {
                    open_quote_char = ch;
                    rel_start = i + 1; // start of inner content
                    break;
                }
            }
            if open_quote_char != '\0'
                && let Some(rel_end) = rest[rel_start..].find(open_quote_char)
            {
                let start_abs = echo_idx + 5 + rel_start;
                let end_abs = start_abs + rel_end;
                let inner = s[start_abs..end_abs].to_string();
                let replacement = replacement_fn(&inner);
                s.replace_range(start_abs..end_abs, &replacement);
            }
        }
    }

    // Redact passwords piped to chpasswd (keep username)
    if redacted.contains("chpasswd") {
        redact_after_echo_segment(&mut redacted, |inner| {
            if let Some(colon_idx) = inner.find(':') {
                let user = &inner[..colon_idx];
                format!("{}:<REDACTED>", user)
            } else {
                "<REDACTED>".to_string()
            }
        });
    }

    // Redact plaintext passphrases piped to cryptsetup
    if redacted.contains("cryptsetup") && redacted.contains("echo ") {
        redact_after_echo_segment(&mut redacted, |_inner| "<REDACTED>".to_string());
    }

    redacted
}

/// Remove ANSI escape/control sequences from a single line of terminal output.
/// This strips CSI (ESC[...<final>), OSC (ESC]...BEL or ESC\), and other ESC-initiated sequences.
pub fn strip_ansi_escape_codes(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut i: usize = 0;
    let len = bytes.len();
    let mut out = String::with_capacity(len);
    while i < len {
        if bytes[i] == 0x1B {
            // ESC sequence
            if i + 1 >= len {
                // trailing ESC, drop
                break;
            }
            let next = bytes[i + 1];
            match next as char {
                '[' => {
                    // CSI: ESC [ ... <final 0x40..0x7E>
                    i += 2;
                    while i < len {
                        let b = bytes[i];
                        i += 1;
                        if (0x40..=0x7E).contains(&b) {
                            break;
                        }
                    }
                }
                ']' => {
                    // OSC: ESC ] ... BEL (0x07) or ST (ESC \)
                    i += 2;
                    while i < len {
                        if bytes[i] == 0x07 {
                            i += 1; // consume BEL
                            break;
                        }
                        if bytes[i] == 0x1B && i + 1 < len && bytes[i + 1] == b'\\' {
                            i += 2; // consume ESC \\
                            break;
                        }
                        i += 1;
                    }
                }
                'P' | 'X' | '^' | '_' => {
                    // DCS/SOS/PM/APC: ESC <type> ... ST (ESC \)
                    i += 2;
                    while i < len {
                        if bytes[i] == 0x1B && i + 1 < len && bytes[i + 1] == b'\\' {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                _ => {
                    // Single-char sequences like ESC( B etc. Skip ESC and the next byte
                    i += 2;
                }
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Sanitize a terminal output line for safe rendering inside the TUI.
/// Removes carriage returns, backspaces and ANSI escape sequences.
pub fn sanitize_terminal_output_line(input: &str) -> String {
    // 1) Turn carriage returns into newlines so progress updates become separate lines; drop BEL
    let mut tmp = input.replace('\r', "\n");
    tmp = tmp.replace('\x07', "");

    // 2) Interpret backspaces by removing the previous visible char
    let mut buf = String::with_capacity(tmp.len());
    for ch in tmp.chars() {
        match ch {
            '\u{0008}' => {
                buf.pop();
            }
            // Replace tabs with a single space to avoid layout issues
            '\t' => buf.push(' '),
            // Skip other non-printable C0 controls (except newline which we keep)
            c if (c < ' ' && c != '\n') => {}
            c => buf.push(c),
        }
    }

    // 3) Strip ANSI sequences last
    strip_ansi_escape_codes(&buf)
}

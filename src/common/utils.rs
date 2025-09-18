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

//! Log-line parsing helpers.

/// Parses a log line into (level, message).
///
/// Expects the format `[LEVEL] message`. If the line doesn't match this
/// pattern, it defaults to `("INFO", line)`.
pub(crate) fn parse_log_line(line: &str) -> (String, String) {
    // Format: [LEVEL] message
    if let Some(rest) = line.strip_prefix('[') {
        if let Some(end_bracket) = rest.find(']') {
            let level = rest[..end_bracket].to_string();
            let message = rest[end_bracket + 1..].trim().to_string();
            return (level, message);
        }
    }
    ("INFO".to_string(), line.to_string())
}

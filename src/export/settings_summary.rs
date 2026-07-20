//! Settings and configuration summary generator for layout exports.
//!
//! Generates a markdown summary of all layout settings including RGB configuration,
//! idle effect settings, and firmware information.

use crate::models::Layout;
use std::fmt::Write as _;

/// Generates a markdown settings and configuration summary section.
///
/// Creates a formatted markdown section documenting:
/// - RGB settings (saturation, brightness, timeout)
/// - Idle effect configuration (enabled, timeout, effect, duration)
/// - Firmware information (keyboard, keymap name, output format)
///
/// # Arguments
///
/// * `layout` - The Layout containing all settings to summarize
///
/// # Returns
///
/// A formatted markdown string with the configuration section
///
/// # Example
///
/// ```no_run
/// use lazyqmk::export::settings_summary::generate_settings_summary;
/// use lazyqmk::models::Layout;
///
/// let layout = Layout::new("Test").unwrap();
/// let summary = generate_settings_summary(&layout);
/// println!("{}", summary);
/// ```
pub fn generate_settings_summary(layout: &Layout) -> String {
    let mut output = String::new();

    output.push_str("## Configuration\n\n");

    // RGB Settings section
    output.push_str("### RGB Settings\n");
    let _ = writeln!(
        output,
        "- **Saturation:** {}%",
        layout.rgb_saturation.as_percent()
    );
    let _ = writeln!(
        output,
        "- **Brightness (uncolored keys):** {}%",
        layout.uncolored_key_behavior.as_percent()
    );

    // RGB Timeout
    let timeout_str = format_timeout_ms(layout.rgb_timeout_ms);
    let _ = writeln!(output, "- **Timeout:** {}", timeout_str);

    output.push('\n');

    // Idle Effect section
    output.push_str("### Idle Effect\n");
    let _ = writeln!(
        output,
        "- **Enabled:** {}",
        if layout.idle_effect_settings.enabled {
            "Yes"
        } else {
            "No"
        }
    );

    let idle_timeout_str = format_timeout_ms(layout.idle_effect_settings.idle_timeout_ms);
    let _ = writeln!(
        output,
        "- **Timeout:** {} ({} ms)",
        idle_timeout_str, layout.idle_effect_settings.idle_timeout_ms
    );

    let _ = writeln!(
        output,
        "- **Effect:** {}",
        layout.idle_effect_settings.idle_effect_mode.display_name()
    );

    let effect_duration_str =
        format_timeout_ms(layout.idle_effect_settings.idle_effect_duration_ms);
    let _ = writeln!(
        output,
        "- **Duration:** {} ({} ms)",
        effect_duration_str, layout.idle_effect_settings.idle_effect_duration_ms
    );

    output.push('\n');

    // Firmware section
    output.push_str("### Firmware\n");

    if let Some(keyboard) = &layout.metadata.keyboard {
        let _ = writeln!(output, "- **Keyboard:** {}", keyboard);
    } else {
        output.push_str("- **Keyboard:** Not specified\n");
    }

    if let Some(keymap_name) = &layout.metadata.keymap_name {
        let _ = writeln!(output, "- **Keymap Name:** {}", keymap_name);
    } else {
        output.push_str("- **Keymap Name:** Not specified\n");
    }

    if let Some(output_format) = &layout.metadata.output_format {
        let _ = writeln!(output, "- **Output Format:** {}", output_format);
    } else {
        output.push_str("- **Output Format:** Not specified\n");
    }

    output
}

/// Formats milliseconds as a human-readable duration string.
///
/// Converts milliseconds to minutes and seconds for display:
/// - 60000 ms → "1 min"
/// - 300000 ms → "5 min"
/// - 5000 ms → "5 sec"
/// - 0 ms → "Disabled"
///
/// # Arguments
///
/// * `ms` - Milliseconds to format
///
/// # Returns
///
/// A human-readable duration string
fn format_timeout_ms(ms: u32) -> String {
    if ms == 0 {
        "Disabled".to_string()
    } else if ms.is_multiple_of(60_000) {
        // Exact minutes
        let minutes = ms / 60_000;
        format!("{} min", minutes)
    } else if ms.is_multiple_of(1000) {
        // Exact seconds
        let seconds = ms / 1000;
        format!("{} sec", seconds)
    } else {
        // Milliseconds with remainder
        let seconds = ms / 1000;
        let remainder_ms = ms % 1000;
        format!("{}.{:03} sec", seconds, remainder_ms)
    }
}

#[cfg(test)]
mod tests;


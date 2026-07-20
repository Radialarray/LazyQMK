//! Low-level build helpers: `run_build`, `find_firmware_file`,
//! `enhance_qmk_error`.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;

use super::state::{BuildMessage, BuildStatus, LogLevel};

pub(super) fn enhance_qmk_error(error_str: &str) -> String {
    let error_lower = error_str.to_lowercase();

    // Check for command not found patterns across platforms
    let is_command_not_found = error_lower.contains("not found")
        || error_lower.contains("no such file")
        || error_lower.contains("is not recognized") // Windows
        || error_lower.contains("command not found")
        || error_lower.contains("qmk: command not found")
        || error_lower.contains("could not find executable");

    if is_command_not_found {
        format!(
            "{}\n\nTip: Run 'lazyqmk doctor' to check your setup and install missing dependencies.",
            error_str
        )
    } else {
        error_str.to_string()
    }
}

/// Runs the QMK build process in a background thread.
///
/// The keyboard parameter may include variant subdirectories (e.g., "`keebart/corne_choc_pro/standard`").
/// QMK's build system will use the variant-specific keyboard.json for configuration.
///
/// Uses `qmk compile` CLI command which is the standard way to build QMK firmware.
pub(super) fn run_build(
    sender: Sender<BuildMessage>,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
) -> Result<()> {
    // Send progress: Compiling
    sender
        .send(BuildMessage::Progress {
            status: BuildStatus::Compiling,
            message: format!("Compiling {keymap} keymap for {keyboard}..."),
        })
        .context("Failed to send progress message")?;

    sender
        .send(BuildMessage::Log {
            level: LogLevel::Info,
            message: format!("Running: qmk compile -kb {keyboard} -km {keymap}"),
        })
        .ok();

    // Build using qmk compile command (standard QMK CLI)
    let mut cmd = Command::new("qmk");
    cmd.arg("compile")
        .arg("-kb")
        .arg(&keyboard)
        .arg("-km")
        .arg(&keymap)
        .current_dir(&qmk_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Execute command
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            let error_msg = format!("Failed to execute qmk compile command: {e}");
            let enhanced_msg = enhance_qmk_error(&error_msg);
            sender
                .send(BuildMessage::Complete {
                    success: false,
                    firmware_path: None,
                    error: Some(enhanced_msg),
                })
                .ok();
            return Ok(());
        }
    };

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Send stdout logs
    for line in stdout.lines() {
        let level = if line.contains("error") || line.contains("Error") {
            LogLevel::Error
        } else if line.contains("warning") || line.contains("Warning") {
            LogLevel::Info
        } else {
            LogLevel::Info
        };

        sender
            .send(BuildMessage::Log {
                level,
                message: line.to_string(),
            })
            .ok();
    }

    // Send stderr logs (usually errors)
    for line in stderr.lines() {
        if !line.trim().is_empty() {
            sender
                .send(BuildMessage::Log {
                    level: LogLevel::Error,
                    message: line.to_string(),
                })
                .ok();
        }
    }

    // Check success
    if output.status.success() {
        // Find firmware file
        let firmware_path = find_firmware_file(&qmk_path, &keyboard, &keymap)?;

        sender
            .send(BuildMessage::Complete {
                success: true,
                firmware_path: Some(firmware_path),
                error: None,
            })
            .ok();
    } else {
        let base_error = "qmk compile command failed. Check build log for details.";
        let combined = format!("{base_error}\n\n{}", stderr);
        let enhanced_msg = enhance_qmk_error(&combined);
        sender
            .send(BuildMessage::Complete {
                success: false,
                firmware_path: None,
                error: Some(enhanced_msg),
            })
            .ok();
    }

    Ok(())
}

/// Finds the compiled firmware file.
///
/// QMK typically outputs to .build/{keyboard}_{keymap}.{ext}
pub(super) fn find_firmware_file(
    qmk_path: &PathBuf,
    keyboard: &str,
    keymap: &str,
) -> Result<PathBuf> {
    // Clean keyboard path (replace / with _)
    let keyboard_clean = keyboard.replace('/', "_");

    // Try common firmware extensions in order
    let extensions = ["uf2", "hex", "bin"];

    for ext in &extensions {
        let firmware_name = format!("{keyboard_clean}_{keymap}.{ext}");
        let firmware_path = qmk_path.join(".build").join(&firmware_name);

        if firmware_path.exists() {
            return Ok(firmware_path);
        }
    }

    anyhow::bail!("Could not find firmware file for {keyboard} {keymap}. Check .build/ directory.")
}

#[cfg(test)]
mod tests;


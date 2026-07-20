//! Firmware builder implementations (real & mock).
//!
//! Contains [`RealFirmwareBuilder`] which runs `qmk compile`, and
//! [`MockFirmwareBuilder`] which simulates builds for testing.
//! Also provides the artifact-discovery and SHA256 helpers.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use sha2::{Digest, Sha256};

use super::ARTIFACT_EXTENSIONS;
use super::{BuildArtifact, BuildResult, FirmwareBuilder};

// ---------------------------------------------------------------------------
// RealFirmwareBuilder
// ---------------------------------------------------------------------------

/// Real firmware builder using QMK CLI.
pub struct RealFirmwareBuilder;

impl FirmwareBuilder for RealFirmwareBuilder {
    fn build(
        &self,
        qmk_path: &PathBuf,
        keyboard: &str,
        keymap: &str,
        output_dir: &Path,
        job_id: &str,
        log_writer: &mut dyn Write,
        is_cancelled: &dyn Fn() -> bool,
    ) -> Result<BuildResult, String> {
        let _ = writeln!(log_writer, "[INFO] Starting QMK compile...");
        let _ = writeln!(
            log_writer,
            "[INFO] Running: qmk compile -kb {} -km {}",
            keyboard, keymap
        );

        // Check for cancellation before starting
        if is_cancelled() {
            let _ = writeln!(log_writer, "[INFO] Build cancelled before starting");
            return Err("Build cancelled".to_string());
        }

        let mut cmd = Command::new("qmk");
        cmd.arg("compile")
            .arg("-kb")
            .arg(keyboard)
            .arg("-km")
            .arg(keymap)
            .current_dir(qmk_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn the process instead of waiting for output
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to execute qmk: {e}"))?;

        // Get handles for stdout/stderr
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        // Stream stdout in background
        let stdout_reader = BufReader::new(stdout);
        for line in stdout_reader.lines() {
            // Check for cancellation periodically
            if is_cancelled() {
                let _ = writeln!(log_writer, "[INFO] Build cancelled, killing process...");
                let _ = child.kill();
                let _ = child.wait();
                return Err("Build cancelled".to_string());
            }

            if let Ok(line) = line {
                let level = if line.contains("error") || line.contains("Error") {
                    "ERROR"
                } else {
                    "INFO"
                };
                let _ = writeln!(log_writer, "[{level}] {line}");
            }
        }

        // Stream stderr
        let stderr_reader = BufReader::new(stderr);
        for line in stderr_reader.lines() {
            if is_cancelled() {
                let _ = writeln!(log_writer, "[INFO] Build cancelled, killing process...");
                let _ = child.kill();
                let _ = child.wait();
                return Err("Build cancelled".to_string());
            }

            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    let _ = writeln!(log_writer, "[ERROR] {line}");
                }
            }
        }

        // Wait for process to complete
        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for process: {e}"))?;

        // Final cancellation check
        if is_cancelled() {
            let _ = writeln!(log_writer, "[INFO] Build cancelled");
            return Err("Build cancelled".to_string());
        }

        if !status.success() {
            return Err("QMK compile failed. Check build log for details.".to_string());
        }

        // Discover and copy artifacts
        let _ = writeln!(log_writer, "[INFO] Discovering firmware artifacts...");
        let artifacts = discover_and_copy_artifacts(
            qmk_path, keyboard, keymap, output_dir, job_id, log_writer,
        )?;

        if artifacts.is_empty() {
            return Err(format!(
                "Could not find any firmware files for {keyboard} {keymap}"
            ));
        }

        // Use first artifact as primary firmware path (for backward compatibility)
        let primary_path = output_dir.join(&artifacts[0].filename);

        Ok(BuildResult {
            firmware_path: primary_path,
            artifacts,
        })
    }
}

// ---------------------------------------------------------------------------
// Artifact helpers
// ---------------------------------------------------------------------------

/// Discovers firmware artifacts in QMK's `.build` directory and copies them to the output directory.
///
/// Looks for files matching the pattern `<keyboard_clean>_<keymap>.<ext>` where keyboard slashes
/// are replaced with underscores. Supports multiple file extensions (uf2, bin, hex) and handles
/// variant suffixes via glob matching.
fn discover_and_copy_artifacts(
    qmk_path: &Path,
    keyboard: &str,
    keymap: &str,
    output_dir: &Path,
    job_id: &str,
    log_writer: &mut dyn Write,
) -> Result<Vec<BuildArtifact>, String> {
    let build_dir = qmk_path.join(".build");
    if !build_dir.exists() {
        return Err("QMK .build directory not found".to_string());
    }

    // Create output directory
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create output directory: {e}"))?;

    let keyboard_clean = keyboard.replace('/', "_");
    let base_prefix = format!("{keyboard_clean}_{keymap}");

    let mut artifacts = Vec::new();

    // First pass: look for exact matches
    for ext in ARTIFACT_EXTENSIONS {
        let exact_filename = format!("{base_prefix}.{ext}");
        let source_path = build_dir.join(&exact_filename);

        if source_path.exists() {
            if let Some(artifact) =
                copy_artifact(&source_path, output_dir, ext, job_id, log_writer)?
            {
                artifacts.push(artifact);
            }
        }
    }

    // Second pass: glob for variant suffixes (e.g., keyboard_keymap_avr.hex)
    // Only if we haven't found exact matches for all extensions
    if let Ok(entries) = fs::read_dir(&build_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Check if this file starts with our prefix and has a supported extension
            if !filename.starts_with(&base_prefix) {
                continue;
            }

            for ext in ARTIFACT_EXTENSIONS {
                if filename.ends_with(&format!(".{ext}")) {
                    // Skip if we already have an artifact with this extension
                    if artifacts.iter().any(|a| a.artifact_type == *ext) {
                        continue;
                    }

                    if let Some(artifact) =
                        copy_artifact(&path, output_dir, ext, job_id, log_writer)?
                    {
                        artifacts.push(artifact);
                    }
                }
            }
        }
    }

    Ok(artifacts)
}

/// Copies a single artifact file to the output directory and creates metadata.
fn copy_artifact(
    source_path: &Path,
    output_dir: &Path,
    ext: &str,
    job_id: &str,
    log_writer: &mut dyn Write,
) -> Result<Option<BuildArtifact>, String> {
    let filename = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid source filename".to_string())?;

    let dest_path = output_dir.join(filename);

    // Copy the file
    fs::copy(source_path, &dest_path).map_err(|e| format!("Failed to copy artifact: {e}"))?;

    // Get file metadata
    let metadata = fs::metadata(&dest_path).map_err(|e| format!("Failed to read metadata: {e}"))?;
    let size = metadata.len();

    // Calculate SHA256 hash
    let sha256 = calculate_sha256(&dest_path).ok();

    let _ = writeln!(
        log_writer,
        "[INFO] Copied artifact: {} ({} bytes)",
        filename, size
    );

    // Use extension as stable artifact ID
    let artifact_id = ext.to_string();
    let download_url = format!("/api/build/jobs/{job_id}/artifacts/{artifact_id}/download");

    Ok(Some(BuildArtifact {
        id: artifact_id,
        filename: filename.to_string(),
        artifact_type: ext.to_string(),
        size,
        sha256,
        download_url,
    }))
}

/// Calculates the SHA256 hash of a file.
fn calculate_sha256(path: &Path) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

// ---------------------------------------------------------------------------
// MockFirmwareBuilder
// ---------------------------------------------------------------------------

/// Mock firmware builder for testing.
pub struct MockFirmwareBuilder {
    /// Simulated build duration in milliseconds.
    pub build_duration_ms: u64,
    /// Whether the build should succeed.
    pub should_succeed: bool,
    /// Error message if build should fail.
    pub error_message: Option<String>,
}

impl Default for MockFirmwareBuilder {
    fn default() -> Self {
        Self {
            build_duration_ms: 100,
            should_succeed: true,
            error_message: None,
        }
    }
}

impl FirmwareBuilder for MockFirmwareBuilder {
    fn build(
        &self,
        _qmk_path: &PathBuf,
        keyboard: &str,
        keymap: &str,
        output_dir: &Path,
        job_id: &str,
        log_writer: &mut dyn Write,
        is_cancelled: &dyn Fn() -> bool,
    ) -> Result<BuildResult, String> {
        let _ = writeln!(log_writer, "[INFO] Mock build starting...");
        let _ = writeln!(
            log_writer,
            "[INFO] Building {} keymap for {}",
            keymap, keyboard
        );

        // Simulate build progress with cancellation checks
        let steps = 5;
        let step_duration = self.build_duration_ms / steps;

        for i in 1..=steps {
            // Check for cancellation
            if is_cancelled() {
                let _ = writeln!(log_writer, "[INFO] Mock build cancelled");
                return Err("Build cancelled".to_string());
            }

            thread::sleep(Duration::from_millis(step_duration));
            let _ = writeln!(log_writer, "[INFO] Build progress: {}%", i * 20);
        }

        if self.should_succeed {
            let keyboard_clean = keyboard.replace('/', "_");
            let filename = format!("{keyboard_clean}_{keymap}.uf2");

            // Create the output directory and mock firmware file
            let _ = fs::create_dir_all(output_dir);
            let firmware_path = output_dir.join(&filename);
            let _ = fs::write(&firmware_path, b"mock firmware content");

            let _ = writeln!(
                log_writer,
                "[INFO] Mock firmware generated: {}",
                firmware_path.display()
            );

            // Create mock artifact metadata
            let artifact_id = "uf2".to_string();
            let download_url = format!("/api/build/jobs/{job_id}/artifacts/{artifact_id}/download");

            let artifacts = vec![BuildArtifact {
                id: artifact_id,
                filename,
                artifact_type: "uf2".to_string(),
                size: 21, // "mock firmware content".len()
                sha256: None,
                download_url,
            }];

            Ok(BuildResult {
                firmware_path,
                artifacts,
            })
        } else {
            let err = self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock build failed".to_string());
            let _ = writeln!(log_writer, "[ERROR] {}", err);
            Err(err)
        }
    }
}

// ---------------------------------------------------------------------------
// Validation helper
// ---------------------------------------------------------------------------

/// Validates an artifact ID to prevent path traversal.
///
/// Valid artifact IDs are lowercase alphanumeric strings (matching file extensions).
pub(crate) fn is_valid_artifact_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 10 {
        return false;
    }
    id.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

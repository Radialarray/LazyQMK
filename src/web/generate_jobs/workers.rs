//! Generate worker implementations (real & mock).
//!
//! Contains [`RealGenerateWorker`] which runs the full firmware-generation
//! pipeline, and [`MockGenerateWorker`] which simulates generation for
//! testing.  Also provides the zip-packing helpers.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;

use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::config::Config;
use crate::firmware::generator::FirmwareGenerator;
use crate::firmware::validator::FirmwareValidator;
use crate::keycode_db::KeycodeDb;
use crate::services::geometry::{self, GeometryContext};
use crate::services::LayoutService;

use super::{GenerateCommand, GenerateWorker};

// ---------------------------------------------------------------------------
// RealGenerateWorker
// ---------------------------------------------------------------------------

/// Real generate worker that produces firmware files.
pub(crate) struct RealGenerateWorker;

impl GenerateWorker for RealGenerateWorker {
    fn generate(
        &self,
        cmd: &GenerateCommand,
        log_writer: &mut dyn Write,
        keycode_db: &KeycodeDb,
    ) -> Result<PathBuf, String> {
        let _ = writeln!(log_writer, "[INFO] Starting firmware generation...");
        let _ = writeln!(log_writer, "[INFO] Layout: {}", cmd.layout_filename);

        // Load the layout
        let _ = writeln!(log_writer, "[INFO] Loading layout...");
        let layout = LayoutService::load(&cmd.layout_path)
            .map_err(|e| format!("Failed to load layout: {e}"))?;

        // Get keyboard and layout variant
        let keyboard = layout
            .metadata
            .keyboard
            .as_ref()
            .ok_or("Layout has no keyboard defined")?;
        let layout_variant = layout
            .metadata
            .layout_variant
            .as_ref()
            .ok_or("Layout has no layout variant defined")?;

        let _ = writeln!(log_writer, "[INFO] Keyboard: {keyboard}");
        let _ = writeln!(log_writer, "[INFO] Layout variant: {layout_variant}");

        // Build config with QMK path
        let mut config = Config::load().unwrap_or_default();
        config.paths.qmk_firmware = Some(cmd.qmk_path.clone());
        config.build.output_dir.clone_from(&cmd.output_dir);

        // Build geometry
        let _ = writeln!(log_writer, "[INFO] Building keyboard geometry...");
        let geo_context = GeometryContext {
            config: &config,
            metadata: &layout.metadata,
        };

        let geo_result = geometry::build_geometry_for_layout(geo_context, layout_variant)
            .map_err(|e| format!("Failed to build geometry: {e}"))?;
        let geometry = geo_result.geometry;
        let mapping = geo_result.mapping;

        // Validate layout
        let _ = writeln!(log_writer, "[INFO] Validating layout...");
        let validator = FirmwareValidator::new(&layout, &geometry, &mapping, keycode_db);
        let report = validator
            .validate()
            .map_err(|e| format!("Validation failed: {e}"))?;

        if !report.is_valid() {
            let _ = writeln!(log_writer, "[ERROR] Layout validation failed:");
            let _ = writeln!(log_writer, "[ERROR] {}", report.format_message());
            return Err(format!(
                "Layout validation failed: {}",
                report.format_message()
            ));
        }
        let _ = writeln!(log_writer, "[INFO] Layout validation passed");

        // Generate firmware files
        let _ = writeln!(log_writer, "[INFO] Generating firmware files...");
        let generator = FirmwareGenerator::new(&layout, &geometry, &mapping, &config, keycode_db);

        let keymap_c = generator
            .generate_keymap_c()
            .map_err(|e| format!("Failed to generate keymap.c: {e}"))?;
        let config_h = generator
            .generate_merged_config_h()
            .map_err(|e| format!("Failed to generate config.h: {e}"))?;

        let _ = writeln!(
            log_writer,
            "[INFO] Generated keymap.c ({} bytes)",
            keymap_c.len()
        );
        let _ = writeln!(
            log_writer,
            "[INFO] Generated config.h ({} bytes)",
            config_h.len()
        );

        // Create output directory
        fs::create_dir_all(&cmd.output_dir)
            .map_err(|e| format!("Failed to create output directory: {e}"))?;

        // Read layout source file
        let layout_source = fs::read_to_string(&cmd.layout_path)
            .map_err(|e| format!("Failed to read layout source: {e}"))?;

        // Read logs so far
        let logs_content = fs::read_to_string(&cmd.log_path).unwrap_or_default();

        // Create manifest
        let manifest = serde_json::json!({
            "version": "1.0",
            "generator": "lazyqmk",
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "layout": {
                "name": layout.metadata.name,
                "filename": cmd.layout_filename,
                "keyboard": keyboard,
                "layout_variant": layout_variant,
            },
            "files": [
                "keymap.c",
                "config.h",
                "layout.md",
                "generate.log",
                "manifest.json"
            ]
        });

        // Create zip file
        let keyboard_clean = keyboard.replace('/', "_");
        let zip_filename = format!("{}_firmware.zip", keyboard_clean);
        let zip_path = cmd.output_dir.join(&zip_filename);

        let _ = writeln!(log_writer, "[INFO] Creating zip archive: {}", zip_filename);

        create_firmware_zip(
            &zip_path,
            &keymap_c,
            &config_h,
            &layout_source,
            &logs_content,
            &manifest,
        )?;

        let _ = writeln!(
            log_writer,
            "[INFO] Firmware generation completed successfully"
        );
        let _ = writeln!(log_writer, "[INFO] Output: {}", zip_path.display());

        Ok(zip_path)
    }
}

// ---------------------------------------------------------------------------
// Zip helpers
// ---------------------------------------------------------------------------

/// Creates a firmware zip archive with safe filename handling.
fn create_firmware_zip(
    zip_path: &Path,
    keymap_c: &str,
    config_h: &str,
    layout_source: &str,
    logs: &str,
    manifest: &serde_json::Value,
) -> Result<(), String> {
    let file = File::create(zip_path).map_err(|e| format!("Failed to create zip file: {e}"))?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Add files with safe, fixed names (no user input in filenames)
    add_file_to_zip(&mut zip, "keymap.c", keymap_c.as_bytes(), options)?;
    add_file_to_zip(&mut zip, "config.h", config_h.as_bytes(), options)?;
    add_file_to_zip(&mut zip, "layout.md", layout_source.as_bytes(), options)?;
    add_file_to_zip(&mut zip, "generate.log", logs.as_bytes(), options)?;

    let manifest_str = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {e}"))?;
    add_file_to_zip(&mut zip, "manifest.json", manifest_str.as_bytes(), options)?;

    zip.finish()
        .map_err(|e| format!("Failed to finalize zip: {e}"))?;

    Ok(())
}

/// Adds a file to a zip archive with zip-slip prevention.
pub(crate) fn add_file_to_zip(
    zip: &mut ZipWriter<File>,
    name: &str,
    content: &[u8],
    options: SimpleFileOptions,
) -> Result<(), String> {
    // Validate filename for zip-slip prevention
    if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
        return Err(format!("Invalid filename in zip: {name}"));
    }

    zip.start_file(name, options)
        .map_err(|e| format!("Failed to start file {name}: {e}"))?;
    zip.write_all(content)
        .map_err(|e| format!("Failed to write file {name}: {e}"))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// MockGenerateWorker
// ---------------------------------------------------------------------------

/// Mock generate worker for testing.
#[allow(dead_code)] // Used in tests; bin target doesn't link them
pub(crate) struct MockGenerateWorker {
    /// Simulated generation duration in milliseconds.
    pub duration_ms: u64,
    /// Whether the generation should succeed.
    pub should_succeed: bool,
    /// Error message if generation should fail.
    pub error_message: Option<String>,
}

impl Default for MockGenerateWorker {
    fn default() -> Self {
        Self {
            duration_ms: 100,
            should_succeed: true,
            error_message: None,
        }
    }
}

impl GenerateWorker for MockGenerateWorker {
    fn generate(
        &self,
        cmd: &GenerateCommand,
        log_writer: &mut dyn Write,
        _keycode_db: &KeycodeDb,
    ) -> Result<PathBuf, String> {
        let _ = writeln!(log_writer, "[INFO] Mock generation starting...");
        let _ = writeln!(log_writer, "[INFO] Layout: {}", cmd.layout_filename);

        // Simulate generation progress
        let steps = 5;
        let step_duration = self.duration_ms / steps;

        for i in 1..=steps {
            thread::sleep(std::time::Duration::from_millis(step_duration));
            let _ = writeln!(log_writer, "[INFO] Generation progress: {}%", i * 20);
        }

        if self.should_succeed {
            // Create a mock zip file
            fs::create_dir_all(&cmd.output_dir)
                .map_err(|e| format!("Failed to create output dir: {e}"))?;
            let zip_path = cmd.output_dir.join("mock_firmware.zip");

            // Create minimal mock zip
            let file =
                File::create(&zip_path).map_err(|e| format!("Failed to create mock zip: {e}"))?;
            let mut zip = ZipWriter::new(file);
            let options = SimpleFileOptions::default();
            zip.start_file("keymap.c", options)
                .map_err(|e| format!("Failed to add file: {e}"))?;
            zip.write_all(b"// Mock keymap\n")
                .map_err(|e| format!("Failed to write: {e}"))?;
            zip.finish()
                .map_err(|e| format!("Failed to finish zip: {e}"))?;

            let _ = writeln!(
                log_writer,
                "[INFO] Mock generation completed: {}",
                zip_path.display()
            );
            Ok(zip_path)
        } else {
            let err = self
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock generation failed".to_string());
            let _ = writeln!(log_writer, "[ERROR] {}", err);
            Err(err)
        }
    }
}

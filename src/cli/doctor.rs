//! Doctor command for dependency checking.

use crate::cli::common::{CliError, CliResult};
use crate::config::Config;
use crate::doctor::{DependencyChecker, DoctorFormatter, OutputFormat, ToolStatus};
use clap::Args;

/// Check development environment dependencies
#[derive(Debug, Clone, Args)]
pub struct DoctorArgs {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,
}

impl DoctorArgs {
    /// Execute the doctor command
    pub fn execute(&self) -> CliResult<()> {
        // Load configuration to get QMK firmware path
        let config = Config::load().unwrap_or_default();
        let qmk_path = config.paths.qmk_firmware.as_deref();

        // Create checker and run all checks
        let checker = DependencyChecker::new();
        let statuses = checker.check_all(qmk_path);

        // Determine output format
        let format = if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Terminal
        };

        // Format and print results
        let formatter = DoctorFormatter::with_format(format);
        let output = formatter.format_results(&statuses);
        println!("{}", output);

        // Determine exit code
        let has_missing = statuses.iter().any(|s| s.status == ToolStatus::Missing);

        if has_missing {
            Err(CliError::validation("Some dependencies are missing"))
        } else {
            Ok(())
        }
    }
}

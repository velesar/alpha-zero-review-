//! Bandit tool runner
//!
//! Runs Bandit Python security analysis with SARIF output.

use crate::domain::{ConfigValue, ToolConfig};
use crate::runner::{detect_tool, get_tool_version, parse_sarif, InstallCommand, RunnerError, ToolResult, ToolRunner};
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

/// Bandit runner
pub struct BanditRunner;

impl ToolRunner for BanditRunner {
    fn name(&self) -> &str {
        "bandit"
    }

    fn is_available(&self) -> bool {
        detect_tool("bandit")
    }

    fn version(&self) -> Option<String> {
        get_tool_version("bandit")
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["python".to_string()]
    }

    fn install_commands(&self) -> Vec<InstallCommand> {
        vec![
            InstallCommand::new("pipx", "bandit"),
            InstallCommand::new("pip", "bandit").with_args(&["--user"]),
            InstallCommand::new("pip3", "bandit").with_args(&["--user"]),
            InstallCommand::new("pip", "bandit"),
        ]
    }

    fn run(&self, path: &Path, config: Option<&ToolConfig>) -> Result<ToolResult, RunnerError> {
        if !self.is_available() {
            return Err(RunnerError::ToolNotFound("bandit".to_string()));
        }

        let output_file = NamedTempFile::new()?;

        let mut cmd = Command::new("bandit");
        cmd.arg("-r")
           .arg(path)
           .arg("-f")
           .arg("sarif")
           .arg("-o")
           .arg(output_file.path());

        // Apply configuration
        if let Some(cfg) = config {
            // Severity filter (l = low and above, m = medium and above, h = high only)
            if let Some(ref severity) = cfg.severity {
                match severity.to_lowercase().as_str() {
                    "low" => cmd.arg("-l"),
                    "medium" => cmd.arg("-ll"),
                    "high" => cmd.arg("-lll"),
                    _ => &mut cmd,
                };
            }

            // Confidence filter from options
            if let Some(ConfigValue::String(confidence)) = cfg.options.get("confidence") {
                match confidence.to_lowercase().as_str() {
                    "low" => cmd.arg("-i"),
                    "medium" => cmd.arg("-ii"),
                    "high" => cmd.arg("-iii"),
                    _ => &mut cmd,
                };
            }

            // Exclude paths (join patterns with comma for bandit)
            if !cfg.exclude_patterns.is_empty() {
                cmd.arg("-x").arg(cfg.exclude_patterns.join(","));
            }

            // Skip specific tests from options
            if let Some(ConfigValue::String(skip)) = cfg.options.get("skip") {
                cmd.arg("-s").arg(skip);
            }

            // Config file
            if let Some(ref config_file) = cfg.config_source {
                cmd.arg("-c").arg(config_file);
            }
        }

        tracing::debug!("Running bandit: {:?}", cmd);

        let output = cmd.output()?;
        let exit_code = output.status.code().unwrap_or(-1);

        // Read SARIF output
        let sarif_content = std::fs::read_to_string(output_file.path())?;
        let sarif = if sarif_content.is_empty() {
            Sarif::new()
        } else {
            parse_sarif(&sarif_content)?
        };

        let stderr = if output.stderr.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        };

        Ok(ToolResult {
            sarif,
            exit_code,
            stderr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bandit_languages() {
        let runner = BanditRunner;
        let langs = runner.supported_languages();
        assert_eq!(langs, vec!["python".to_string()]);
    }
}

//! Ruff tool runner
//!
//! Runs Ruff Python linter with SARIF output.

use crate::domain::{ConfigValue, ToolConfig};
use crate::runner::{detect_tool, get_tool_version, parse_sarif_bytes, InstallCommand, RunnerError, ToolResult, ToolRunner};
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;

/// Ruff runner
pub struct RuffRunner;

impl ToolRunner for RuffRunner {
    fn name(&self) -> &str {
        "ruff"
    }

    fn is_available(&self) -> bool {
        detect_tool("ruff")
    }

    fn version(&self) -> Option<String> {
        get_tool_version("ruff")
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["python".to_string()]
    }

    fn install_commands(&self) -> Vec<InstallCommand> {
        vec![
            InstallCommand::new("pipx", "ruff"),
            InstallCommand::new("pip", "ruff").with_args(&["--user"]),
            InstallCommand::new("pip3", "ruff").with_args(&["--user"]),
            InstallCommand::new("pip", "ruff"),
        ]
    }

    fn run(&self, path: &Path, config: Option<&ToolConfig>) -> Result<ToolResult, RunnerError> {
        if !self.is_available() {
            return Err(RunnerError::ToolNotFound("ruff".to_string()));
        }

        let mut cmd = Command::new("ruff");
        cmd.arg("check")
           .arg("--output-format")
           .arg("sarif")
           .arg(path);

        // Apply configuration
        if let Some(cfg) = config {
            // Select specific rules from options
            if let Some(ConfigValue::String(select)) = cfg.options.get("select") {
                cmd.arg("--select").arg(select);
            }

            // Ignore specific rules from options
            if let Some(ConfigValue::String(ignore)) = cfg.options.get("ignore") {
                cmd.arg("--ignore").arg(ignore);
            }

            // Exclude patterns (join with comma for ruff)
            if !cfg.exclude_patterns.is_empty() {
                cmd.arg("--exclude").arg(cfg.exclude_patterns.join(","));
            }

            // Config file
            if let Some(ref config_file) = cfg.config_source {
                cmd.arg("--config").arg(config_file);
            }

            // Line length from options
            if let Some(ConfigValue::Number(line_length)) = cfg.options.get("line_length") {
                cmd.arg("--line-length").arg((*line_length as u64).to_string());
            }

            // Target version from options
            if let Some(ConfigValue::String(target)) = cfg.options.get("target_version") {
                cmd.arg("--target-version").arg(target);
            }
        }

        tracing::debug!("Running ruff: {:?}", cmd);

        let output = cmd.output()?;
        let exit_code = output.status.code().unwrap_or(-1);

        // Ruff outputs SARIF to stdout
        let sarif = if output.stdout.is_empty() {
            Sarif::new()
        } else {
            parse_sarif_bytes(&output.stdout)?
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
    fn test_ruff_languages() {
        let runner = RuffRunner;
        let langs = runner.supported_languages();
        assert_eq!(langs, vec!["python".to_string()]);
    }
}

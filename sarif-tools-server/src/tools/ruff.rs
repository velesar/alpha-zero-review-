//! Ruff tool runner
//!
//! Runs Ruff Python linter with SARIF output.

use crate::runner::{detect_tool, get_tool_version, parse_sarif_bytes, RunnerError, ToolResult, ToolRunner};
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

    fn run(&self, path: &Path, config: Option<&serde_json::Value>) -> Result<ToolResult, RunnerError> {
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
            // Select specific rules
            if let Some(select) = cfg.get("select").and_then(|v| v.as_str()) {
                cmd.arg("--select").arg(select);
            }

            // Ignore specific rules
            if let Some(ignore) = cfg.get("ignore").and_then(|v| v.as_str()) {
                cmd.arg("--ignore").arg(ignore);
            }

            // Exclude patterns
            if let Some(exclude) = cfg.get("exclude").and_then(|v| v.as_str()) {
                cmd.arg("--exclude").arg(exclude);
            }

            // Config file
            if let Some(config_file) = cfg.get("config").and_then(|v| v.as_str()) {
                cmd.arg("--config").arg(config_file);
            }

            // Line length
            if let Some(line_length) = cfg.get("line_length").and_then(|v| v.as_u64()) {
                cmd.arg("--line-length").arg(line_length.to_string());
            }

            // Target version
            if let Some(target) = cfg.get("target_version").and_then(|v| v.as_str()) {
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

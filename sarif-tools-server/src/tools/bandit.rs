//! Bandit tool runner
//!
//! Runs Bandit Python security analysis with SARIF output.

use crate::runner::{detect_tool, get_tool_version, parse_sarif, RunnerError, ToolResult, ToolRunner};
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

    fn run(&self, path: &Path, config: Option<&serde_json::Value>) -> Result<ToolResult, RunnerError> {
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
            if let Some(severity) = cfg.get("severity").and_then(|v| v.as_str()) {
                match severity.to_lowercase().as_str() {
                    "low" => cmd.arg("-l"),
                    "medium" => cmd.arg("-ll"),
                    "high" => cmd.arg("-lll"),
                    _ => &mut cmd,
                };
            }

            // Confidence filter
            if let Some(confidence) = cfg.get("confidence").and_then(|v| v.as_str()) {
                match confidence.to_lowercase().as_str() {
                    "low" => cmd.arg("-i"),
                    "medium" => cmd.arg("-ii"),
                    "high" => cmd.arg("-iii"),
                    _ => &mut cmd,
                };
            }

            // Exclude paths
            if let Some(exclude) = cfg.get("exclude").and_then(|v| v.as_str()) {
                cmd.arg("-x").arg(exclude);
            }

            // Skip specific tests
            if let Some(skip) = cfg.get("skip").and_then(|v| v.as_str()) {
                cmd.arg("-s").arg(skip);
            }

            // Config file
            if let Some(config_file) = cfg.get("config").and_then(|v| v.as_str()) {
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

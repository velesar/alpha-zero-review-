//! Trivy tool runner
//!
//! Runs Trivy vulnerability scanner with SARIF output.

use crate::runner::{detect_tool, get_tool_version, parse_sarif_bytes, RunnerError, ToolResult, ToolRunner};
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;

/// Trivy runner
pub struct TrivyRunner;

impl ToolRunner for TrivyRunner {
    fn name(&self) -> &str {
        "trivy"
    }

    fn is_available(&self) -> bool {
        detect_tool("trivy")
    }

    fn version(&self) -> Option<String> {
        get_tool_version("trivy")
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "python", "javascript", "typescript", "java", "go",
            "ruby", "rust", "php", "dotnet",
            "dockerfile", "terraform", "kubernetes", "helm",
        ].into_iter().map(String::from).collect()
    }

    fn run(&self, path: &Path, config: Option<&serde_json::Value>) -> Result<ToolResult, RunnerError> {
        if !self.is_available() {
            return Err(RunnerError::ToolNotFound("trivy".to_string()));
        }

        let mut cmd = Command::new("trivy");
        cmd.arg("fs")
           .arg("--format")
           .arg("sarif")
           .arg(path);

        // Apply configuration
        if let Some(cfg) = config {
            // Scanners (vuln, misconfig, secret, license)
            if let Some(scanners) = cfg.get("scanners").and_then(|v| v.as_str()) {
                cmd.arg("--scanners").arg(scanners);
            }

            // Severity filter
            if let Some(severity) = cfg.get("severity").and_then(|v| v.as_str()) {
                cmd.arg("--severity").arg(severity);
            }

            // Ignore unfixed vulnerabilities
            if cfg.get("ignore_unfixed").and_then(|v| v.as_bool()).unwrap_or(false) {
                cmd.arg("--ignore-unfixed");
            }

            // Skip directories
            if let Some(skip_dirs) = cfg.get("skip_dirs").and_then(|v| v.as_str()) {
                cmd.arg("--skip-dirs").arg(skip_dirs);
            }

            // Skip files
            if let Some(skip_files) = cfg.get("skip_files").and_then(|v| v.as_str()) {
                cmd.arg("--skip-files").arg(skip_files);
            }

            // Config file
            if let Some(config_file) = cfg.get("config").and_then(|v| v.as_str()) {
                cmd.arg("--config").arg(config_file);
            }

            // Timeout
            if let Some(timeout) = cfg.get("timeout").and_then(|v| v.as_str()) {
                cmd.arg("--timeout").arg(timeout);
            }
        }

        tracing::debug!("Running trivy: {:?}", cmd);

        let output = cmd.output()?;
        let exit_code = output.status.code().unwrap_or(-1);

        // Trivy outputs SARIF to stdout
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
    fn test_trivy_languages() {
        let runner = TrivyRunner;
        let langs = runner.supported_languages();
        assert!(langs.contains(&"python".to_string()));
        assert!(langs.contains(&"dockerfile".to_string()));
        assert!(langs.contains(&"terraform".to_string()));
    }
}

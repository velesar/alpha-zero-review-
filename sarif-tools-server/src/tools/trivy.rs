//! Trivy tool runner
//!
//! Runs Trivy vulnerability scanner with SARIF output.

use crate::domain::{ConfigValue, ToolConfig};
use crate::runner::{detect_tool, get_tool_version, parse_sarif_bytes, InstallCommand, RunnerError, ToolResult, ToolRunner};
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

    fn install_command(&self) -> Option<InstallCommand> {
        // Trivy can be installed via brew on macOS or downloaded from releases
        // For broad compatibility, recommend brew
        Some(InstallCommand::new("brew", "trivy"))
    }

    fn run(&self, path: &Path, config: Option<&ToolConfig>) -> Result<ToolResult, RunnerError> {
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
            // Scanners (vuln, misconfig, secret, license) from options
            if let Some(ConfigValue::String(scanners)) = cfg.options.get("scanners") {
                cmd.arg("--scanners").arg(scanners);
            }

            // Severity filter
            if let Some(ref severity) = cfg.severity {
                cmd.arg("--severity").arg(severity);
            }

            // Ignore unfixed vulnerabilities from options
            if let Some(ConfigValue::Bool(true)) = cfg.options.get("ignore_unfixed") {
                cmd.arg("--ignore-unfixed");
            }

            // Skip directories (using exclude_patterns)
            if !cfg.exclude_patterns.is_empty() {
                cmd.arg("--skip-dirs").arg(cfg.exclude_patterns.join(","));
            }

            // Skip files from options
            if let Some(ConfigValue::String(skip_files)) = cfg.options.get("skip_files") {
                cmd.arg("--skip-files").arg(skip_files);
            }

            // Config file
            if let Some(ref config_file) = cfg.config_source {
                cmd.arg("--config").arg(config_file);
            }

            // Timeout from options
            if let Some(ConfigValue::String(timeout)) = cfg.options.get("timeout") {
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

//! Semgrep tool runner
//!
//! Runs Semgrep static analysis with SARIF output.

use crate::domain::ToolConfig;
use crate::runner::{detect_tool, get_tool_version, parse_sarif, InstallCommand, RunnerError, ToolResult, ToolRunner};
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

/// Semgrep runner
pub struct SemgrepRunner;

impl ToolRunner for SemgrepRunner {
    fn name(&self) -> &str {
        "semgrep"
    }

    fn is_available(&self) -> bool {
        detect_tool("semgrep")
    }

    fn version(&self) -> Option<String> {
        get_tool_version("semgrep")
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "python", "javascript", "typescript", "java", "go",
            "ruby", "rust", "c", "cpp", "csharp", "kotlin",
            "scala", "php", "swift", "lua", "ocaml", "r",
        ].into_iter().map(String::from).collect()
    }

    fn install_commands(&self) -> Vec<InstallCommand> {
        vec![
            InstallCommand::new("pipx", "semgrep"),
            InstallCommand::new("pip", "semgrep").with_args(&["--user"]),
            InstallCommand::new("pip3", "semgrep").with_args(&["--user"]),
            InstallCommand::new("pip", "semgrep"),
        ]
    }

    fn run(&self, path: &Path, config: Option<&ToolConfig>) -> Result<ToolResult, RunnerError> {
        if !self.is_available() {
            return Err(RunnerError::ToolNotFound("semgrep".to_string()));
        }

        let output_file = NamedTempFile::new()?;

        let mut cmd = Command::new("semgrep");
        cmd.arg("scan")
           .arg("--sarif")
           .arg("-o")
           .arg(output_file.path())
           .arg(path);

        // Apply custom config if provided
        if let Some(cfg) = config {
            if let Some(ref config_source) = cfg.config_source {
                cmd.arg("--config").arg(config_source);
            } else {
                cmd.arg("--config").arg("auto");
            }

            // Severity filter
            if let Some(ref severity) = cfg.severity {
                cmd.arg("--severity").arg(severity);
            }

            // Exclude patterns
            for pattern in &cfg.exclude_patterns {
                cmd.arg("--exclude").arg(pattern);
            }
        } else {
            cmd.arg("--config").arg("auto");
        }

        tracing::debug!("Running semgrep: {:?}", cmd);

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
    fn test_semgrep_languages() {
        let runner = SemgrepRunner;
        let langs = runner.supported_languages();
        assert!(langs.contains(&"python".to_string()));
        assert!(langs.contains(&"javascript".to_string()));
    }
}

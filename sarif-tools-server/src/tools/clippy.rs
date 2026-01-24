//! Clippy tool runner
//!
//! Runs cargo clippy and converts JSON output to SARIF format.

use crate::domain::{ConfigValue, ToolConfig};
use crate::runner::{detect_tool, get_tool_version, InstallCommand, RunnerError, ToolResult, ToolRunner};
use crate::sarif::{
    ArtifactLocation, Location, Message, PhysicalLocation, Region, Result as SarifResult, Run,
    Sarif, Tool, ToolDriver,
};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

/// Clippy runner
pub struct ClippyRunner;

/// Clippy diagnostic from JSON output
#[derive(Debug, Deserialize)]
struct ClippyDiagnostic {
    message: ClippyMessage,
}

/// Clippy message
#[derive(Debug, Deserialize)]
struct ClippyMessage {
    message: String,
    code: Option<ClippyCode>,
    level: String,
    spans: Vec<ClippySpan>,
    rendered: Option<String>,
}

/// Clippy diagnostic code
#[derive(Debug, Deserialize)]
struct ClippyCode {
    code: String,
    #[allow(dead_code)]
    explanation: Option<String>,
}

/// Clippy span (source location)
#[derive(Debug, Deserialize)]
struct ClippySpan {
    file_name: String,
    line_start: u32,
    line_end: u32,
    column_start: u32,
    column_end: u32,
    is_primary: bool,
}

impl ToolRunner for ClippyRunner {
    fn name(&self) -> &str {
        "clippy"
    }

    fn is_available(&self) -> bool {
        detect_tool("cargo")
    }

    fn version(&self) -> Option<String> {
        // Get clippy version via cargo clippy --version
        let output = Command::new("cargo")
            .args(["clippy", "--version"])
            .output()
            .ok()?;

        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            get_tool_version("rustc")
        }
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["rust".to_string()]
    }

    fn install_command(&self) -> Option<InstallCommand> {
        // Clippy is a rustup component, not a standalone install
        // Return None - user should run: rustup component add clippy
        None
    }

    fn run(&self, path: &Path, config: Option<&ToolConfig>) -> Result<ToolResult, RunnerError> {
        if !self.is_available() {
            return Err(RunnerError::ToolNotFound("cargo".to_string()));
        }

        let mut cmd = Command::new("cargo");
        cmd.arg("clippy")
            .arg("--message-format=json")
            .current_dir(path);

        // Apply configuration
        if let Some(cfg) = config {
            // All targets flag
            if let Some(ConfigValue::Bool(true)) = cfg.options.get("all_targets") {
                cmd.arg("--all-targets");
            }

            // All features flag
            if let Some(ConfigValue::Bool(true)) = cfg.options.get("all_features") {
                cmd.arg("--all-features");
            }

            // Specific features from options
            if let Some(ConfigValue::String(features)) = cfg.options.get("features") {
                cmd.arg("--features").arg(features);
            }

            // Deny warnings
            if let Some(ConfigValue::Bool(true)) = cfg.options.get("deny_warnings") {
                cmd.arg("--").arg("-D").arg("warnings");
            }

            // Specific lints to deny from options (as array)
            if let Some(ConfigValue::Array(deny_lints)) = cfg.options.get("deny") {
                for lint_name in deny_lints {
                    cmd.arg("--").arg("-D").arg(lint_name);
                }
            }
        }

        tracing::debug!("Running clippy: {:?}", cmd);

        let output = cmd.output()?;
        let exit_code = output.status.code().unwrap_or(-1);

        // Parse JSON output and convert to SARIF
        let sarif = self.parse_clippy_output(&output.stdout, path)?;

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

impl ClippyRunner {
    /// Parse clippy JSON output and convert to SARIF
    fn parse_clippy_output(&self, output: &[u8], base_path: &Path) -> Result<Sarif, RunnerError> {
        let mut results: Vec<SarifResult> = Vec::new();

        // Clippy outputs one JSON object per line
        for line in output.split(|&b| b == b'\n') {
            if line.is_empty() {
                continue;
            }

            // Try to parse as diagnostic
            if let Ok(diagnostic) = serde_json::from_slice::<ClippyDiagnostic>(line) {
                // Only include warnings and errors from clippy (skip notes, help, etc.)
                if diagnostic.message.level == "warning" || diagnostic.message.level == "error" {
                    if let Some(result) = self.diagnostic_to_sarif(&diagnostic, base_path) {
                        results.push(result);
                    }
                }
            }
        }

        // Build SARIF structure
        let tool = Tool {
            driver: ToolDriver {
                name: "clippy".to_string(),
                version: self.version(),
                information_uri: Some("https://rust-lang.github.io/rust-clippy/".to_string()),
                rules: Vec::new(),
            },
        };

        let run = Run {
            tool,
            results,
            invocations: Vec::new(),
        };

        Ok(Sarif {
            schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: vec![run],
        })
    }

    /// Convert a clippy diagnostic to a SARIF result
    fn diagnostic_to_sarif(&self, diagnostic: &ClippyDiagnostic, base_path: &Path) -> Option<SarifResult> {
        let msg = &diagnostic.message;

        // Get rule ID from code, default to "unknown" if not present
        let rule_id = msg.code.as_ref()
            .map(|c| c.code.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Get primary span for location
        let primary_span = msg.spans.iter().find(|s| s.is_primary)?;

        // Make path relative to base
        let file_path = if primary_span.file_name.starts_with('/') {
            // Try to make relative
            Path::new(&primary_span.file_name)
                .strip_prefix(base_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| primary_span.file_name.clone())
        } else {
            primary_span.file_name.clone()
        };

        // Build location
        let location = Location {
            physical_location: PhysicalLocation {
                artifact_location: ArtifactLocation {
                    uri: file_path,
                    uri_base_id: None,
                },
                region: Some(Region {
                    start_line: Some(primary_span.line_start),
                    start_column: Some(primary_span.column_start),
                    end_line: Some(primary_span.line_end),
                    end_column: Some(primary_span.column_end),
                }),
            },
        };

        // Map level to SARIF level
        let level = match msg.level.as_str() {
            "error" => Some("error".to_string()),
            "warning" => Some("warning".to_string()),
            _ => Some("note".to_string()),
        };

        // Use rendered message if available, otherwise use message
        let message_text = msg.rendered.as_ref().unwrap_or(&msg.message).clone();

        Some(SarifResult {
            rule_id,
            message: Message {
                text: message_text,
                markdown: None,
            },
            locations: vec![location],
            level,
            fingerprints: None,
            properties: serde_json::Value::Null,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clippy_languages() {
        let runner = ClippyRunner;
        let langs = runner.supported_languages();
        assert_eq!(langs, vec!["rust".to_string()]);
    }

    #[test]
    fn test_clippy_name() {
        let runner = ClippyRunner;
        assert_eq!(runner.name(), "clippy");
    }

    #[test]
    fn test_parse_empty_output() {
        let runner = ClippyRunner;
        let result = runner.parse_clippy_output(b"", Path::new("/test"));
        assert!(result.is_ok());
        let sarif = result.unwrap();
        assert!(sarif.runs[0].results.is_empty());
    }
}

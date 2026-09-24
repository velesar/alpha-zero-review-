//! Clippy tool runner
//!
//! Runs cargo clippy and converts JSON output to SARIF format.

use crate::domain::{ConfigValue, ToolConfig};
use crate::runner::{
    detect_tool, get_tool_version, InstallCommand, RunnerError, ToolResult, ToolRunner,
};
use crate::sarif::{
    ArtifactLocation, Location, Message, PhysicalLocation, Region, Result as SarifResult, Run,
    Sarif, Tool, ToolDriver,
};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

/// Clippy runner
pub struct ClippyRunner;

/// Lints the caller asked to treat as errors (`deny`, `deny_warnings`)
#[derive(Debug, Default)]
struct DenyPolicy {
    all_warnings: bool,
    lints: Vec<String>,
}

impl DenyPolicy {
    fn from_config(config: Option<&ToolConfig>) -> Self {
        let mut policy = Self::default();
        if let Some(cfg) = config {
            policy.all_warnings = matches!(
                cfg.options.get("deny_warnings"),
                Some(ConfigValue::Bool(true))
            );
            if let Some(ConfigValue::Array(lints)) = cfg.options.get("deny") {
                policy.lints = lints.clone();
            }
        }
        policy
    }

    fn escalates(&self, rule_id: &str) -> bool {
        let normalize = |s: &str| s.replace('-', "_");
        self.all_warnings
            || self
                .lints
                .iter()
                .any(|l| normalize(l) == normalize(rule_id))
    }
}

/// Build `cargo` arguments for a clippy run.
///
/// Denied lints are passed as warnings (`-W`) and escalated to SARIF errors
/// when parsing: with `-D`, rustc aborts the crate and cargo skips everything
/// depending on it, so most of the target would go unchecked. `--keep-going`
/// keeps building other units after a genuine compile error.
fn clippy_args(config: Option<&ToolConfig>) -> Vec<String> {
    let mut args = vec![
        "clippy".to_string(),
        "--message-format=json".to_string(),
        "--keep-going".to_string(),
    ];
    let mut lint_args = Vec::new();

    if let Some(cfg) = config {
        if let Some(ConfigValue::Bool(true)) = cfg.options.get("all_targets") {
            args.push("--all-targets".to_string());
        }
        if let Some(ConfigValue::Bool(true)) = cfg.options.get("all_features") {
            args.push("--all-features".to_string());
        }
        if let Some(ConfigValue::String(features)) = cfg.options.get("features") {
            args.push("--features".to_string());
            args.push(features.clone());
        }
        if let Some(ConfigValue::Array(deny_lints)) = cfg.options.get("deny") {
            for lint_name in deny_lints {
                lint_args.push("-W".to_string());
                lint_args.push(lint_name.clone());
            }
        }
    }

    if !lint_args.is_empty() {
        args.push("--".to_string());
        args.extend(lint_args);
    }
    args
}

/// Compilation units cargo reported as failed ("could not compile `x` (lib)")
fn failed_units(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .filter_map(|line| line.split_once("could not compile `"))
        .map(|(_, rest)| {
            let (name, tail) = rest.split_once('`').unwrap_or((rest, ""));
            let target = tail
                .split_once('(')
                .and_then(|(_, t)| t.split_once(')'))
                .map(|(t, _)| format!(" ({})", t))
                .unwrap_or_default();
            format!("{}{}", name, target)
        })
        .collect()
}

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

    fn install_commands(&self) -> Vec<InstallCommand> {
        // Clippy is a rustup component, not a standalone install
        // Return empty - user should run: rustup component add clippy
        vec![]
    }

    fn run(&self, path: &Path, config: Option<&ToolConfig>) -> Result<ToolResult, RunnerError> {
        if !self.is_available() {
            return Err(RunnerError::ToolNotFound("cargo".to_string()));
        }

        let mut cmd = Command::new("cargo");
        cmd.args(clippy_args(config)).current_dir(path);

        tracing::debug!("Running clippy: {:?}", cmd);

        let output = cmd.output()?;
        let exit_code = output.status.code().unwrap_or(-1);

        // Parse JSON output and convert to SARIF
        let policy = DenyPolicy::from_config(config);
        let sarif = self.parse_clippy_output(&output.stdout, path, &policy)?;

        let stderr = if output.stderr.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        };

        let failed = failed_units(stderr.as_deref().unwrap_or(""));
        let incomplete = (!failed.is_empty()).then(|| {
            format!(
                "{} compilation unit(s) failed to build, so they and units depending on them \
                 were not fully checked: {}",
                failed.len(),
                failed.join(", ")
            )
        });

        Ok(ToolResult {
            sarif,
            exit_code,
            stderr,
            incomplete,
        })
    }
}

impl ClippyRunner {
    /// Parse clippy JSON output and convert to SARIF
    fn parse_clippy_output(
        &self,
        output: &[u8],
        base_path: &Path,
        policy: &DenyPolicy,
    ) -> Result<Sarif, RunnerError> {
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
                    if let Some(mut result) = self.diagnostic_to_sarif(&diagnostic, base_path) {
                        if policy.escalates(&result.rule_id) {
                            result.level = Some("error".to_string());
                        }
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
    fn diagnostic_to_sarif(
        &self,
        diagnostic: &ClippyDiagnostic,
        base_path: &Path,
    ) -> Option<SarifResult> {
        let msg = &diagnostic.message;

        // Get rule ID from code, default to "unknown" if not present
        let rule_id = msg
            .code
            .as_ref()
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
    fn test_clippy_args_warn_instead_of_deny() {
        let mut cfg = ToolConfig::default();
        cfg.options
            .insert("all_targets".to_string(), ConfigValue::Bool(true));
        cfg.options
            .insert("deny_warnings".to_string(), ConfigValue::Bool(true));
        cfg.options.insert(
            "deny".to_string(),
            ConfigValue::Array(vec!["clippy::unwrap_used".to_string()]),
        );

        let args = clippy_args(Some(&cfg));
        assert_eq!(
            args,
            vec![
                "clippy",
                "--message-format=json",
                "--keep-going",
                "--all-targets",
                "--",
                "-W",
                "clippy::unwrap_used"
            ]
        );
        assert_eq!(
            clippy_args(None),
            vec!["clippy", "--message-format=json", "--keep-going"]
        );
    }

    #[test]
    fn test_denied_lints_are_escalated_to_errors() {
        let line = br#"{"reason":"compiler-message","message":{"message":"used unwrap","code":{"code":"clippy::unwrap_used"},"level":"warning","spans":[{"file_name":"src/lib.rs","line_start":3,"line_end":3,"column_start":1,"column_end":9,"is_primary":true}],"rendered":null}}"#;
        let runner = ClippyRunner;
        let policy = DenyPolicy {
            all_warnings: false,
            lints: vec!["clippy::unwrap-used".to_string()],
        };
        let sarif = runner
            .parse_clippy_output(line, Path::new("/p"), &policy)
            .unwrap();
        assert_eq!(sarif.runs[0].results[0].level.as_deref(), Some("error"));

        let sarif = runner
            .parse_clippy_output(line, Path::new("/p"), &DenyPolicy::default())
            .unwrap();
        assert_eq!(sarif.runs[0].results[0].level.as_deref(), Some("warning"));
    }

    #[test]
    fn test_failed_units_parsed_from_stderr() {
        let stderr = "error: could not compile `foo` (lib) due to 2 previous errors\n\
                      warning: build failed, waiting for other jobs to finish...\n\
                      error: could not compile `bar` (test \"it\") due to 1 previous error\n";
        assert_eq!(
            failed_units(stderr),
            vec!["foo (lib)".to_string(), "bar (test \"it\")".to_string()]
        );
        assert!(failed_units("    Finished `dev` profile").is_empty());
    }

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
        let result = runner.parse_clippy_output(b"", Path::new("/test"), &DenyPolicy::default());
        assert!(result.is_ok());
        let sarif = result.unwrap();
        assert!(sarif.runs[0].results.is_empty());
    }
}

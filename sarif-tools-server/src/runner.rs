//! Tool runner abstraction
//!
//! Provides a common interface for running code analysis tools
//! and capturing their SARIF output.

use crate::domain::ToolConfig;
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// Errors that can occur when running tools
#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {0}")]
    #[allow(dead_code)]
    ExecutionFailed(String),

    #[error("Failed to parse tool output: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result of running a tool
#[derive(Debug)]
pub struct ToolResult {
    /// SARIF output from the tool
    pub sarif: Sarif,
    /// Exit code from the tool
    pub exit_code: i32,
    /// Standard error output (if any)
    pub stderr: Option<String>,
}

/// Trait for tool runners
pub trait ToolRunner: Send + Sync {
    /// Get the tool name
    fn name(&self) -> &str;

    /// Check if the tool is installed and available
    fn is_available(&self) -> bool;

    /// Get the tool version
    fn version(&self) -> Option<String>;

    /// Get supported languages
    fn supported_languages(&self) -> Vec<String>;

    /// Run the tool on a path
    fn run(&self, path: &Path, config: Option<&ToolConfig>) -> Result<ToolResult, RunnerError>;
}

/// Check if a tool is available on the system
pub fn detect_tool(name: &str) -> bool {
    which::which(name).is_ok()
}

/// Get the version of a tool
pub fn get_tool_version(name: &str) -> Option<String> {
    let output = Command::new(name)
        .arg("--version")
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Try stdout first, then stderr (some tools output version to stderr)
    let version_str = if !stdout.is_empty() {
        stdout
    } else {
        stderr
    };

    // Extract first line, clean up
    version_str
        .lines()
        .next()
        .map(|s| s.trim().to_string())
}

/// Parse SARIF from JSON string
pub fn parse_sarif(content: &str) -> Result<Sarif, RunnerError> {
    serde_json::from_str(content)
        .map_err(|e| RunnerError::ParseError(format!("Invalid SARIF JSON: {}", e)))
}

/// Parse SARIF from bytes
pub fn parse_sarif_bytes(content: &[u8]) -> Result<Sarif, RunnerError> {
    serde_json::from_slice(content)
        .map_err(|e| RunnerError::ParseError(format!("Invalid SARIF JSON: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_tool_ls() {
        // 'ls' should be available on most Unix systems
        assert!(detect_tool("ls"));
    }

    #[test]
    fn test_detect_tool_nonexistent() {
        assert!(!detect_tool("this-tool-does-not-exist-xyz"));
    }
}

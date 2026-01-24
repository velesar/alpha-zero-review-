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

    /// Get install commands for this tool, ordered by preference
    /// Returns multiple options for cross-platform support
    fn install_commands(&self) -> Vec<InstallCommand>;

    /// Run the tool on a path
    fn run(&self, path: &Path, config: Option<&ToolConfig>) -> Result<ToolResult, RunnerError>;
}

/// Install command specification
#[derive(Debug, Clone)]
pub struct InstallCommand {
    /// Package manager (pip, pipx, npm, cargo, brew, apt, etc.)
    pub manager: String,
    /// Package name to install
    pub package: String,
    /// Additional arguments (e.g., --user, -g)
    pub args: Vec<String>,
}

impl InstallCommand {
    pub fn new(manager: &str, package: &str) -> Self {
        Self {
            manager: manager.to_string(),
            package: package.to_string(),
            args: vec![],
        }
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Check if this package manager is available on the system
    pub fn is_available(&self) -> bool {
        let manager_binary = match self.manager.as_str() {
            "pip" => "pip",
            "pip3" => "pip3",
            "pipx" => "pipx",
            "npm" => "npm",
            "cargo" => "cargo",
            "brew" => "brew",
            "apt" => "apt",
            "dnf" => "dnf",
            "yum" => "yum",
            "pacman" => "pacman",
            _ => &self.manager,
        };
        which::which(manager_binary).is_ok()
    }

    /// Build the full command string for display
    pub fn to_command_string(&self) -> String {
        let args_str = if self.args.is_empty() {
            String::new()
        } else {
            format!(" {}", self.args.join(" "))
        };

        match self.manager.as_str() {
            "pip" | "pip3" => format!("{} install{} {}", self.manager, args_str, self.package),
            "pipx" => format!("pipx install {}", self.package),
            "npm" => format!("npm install{} {}", args_str, self.package),
            "cargo" => format!("cargo install {}", self.package),
            "brew" => format!("brew install {}", self.package),
            "apt" => format!("sudo apt install -y {}", self.package),
            "dnf" => format!("sudo dnf install -y {}", self.package),
            "yum" => format!("sudo yum install -y {}", self.package),
            "pacman" => format!("sudo pacman -S --noconfirm {}", self.package),
            _ => format!("{} install{} {}", self.manager, args_str, self.package),
        }
    }

    /// Execute the install command
    pub fn execute(&self) -> Result<(), RunnerError> {
        if !self.is_available() {
            return Err(RunnerError::ExecutionFailed(format!(
                "Package manager '{}' not found",
                self.manager
            )));
        }

        let (program, args) = match self.manager.as_str() {
            "pip" | "pip3" => {
                let mut args = vec!["install".to_string()];
                args.extend(self.args.clone());
                args.push(self.package.clone());
                (self.manager.clone(), args)
            }
            "pipx" => (
                "pipx".to_string(),
                vec!["install".to_string(), self.package.clone()],
            ),
            "npm" => {
                let mut args = vec!["install".to_string()];
                args.extend(self.args.clone());
                args.push(self.package.clone());
                ("npm".to_string(), args)
            }
            "cargo" => (
                "cargo".to_string(),
                vec!["install".to_string(), self.package.clone()],
            ),
            "brew" => (
                "brew".to_string(),
                vec!["install".to_string(), self.package.clone()],
            ),
            "apt" => (
                "sudo".to_string(),
                vec![
                    "apt".to_string(),
                    "install".to_string(),
                    "-y".to_string(),
                    self.package.clone(),
                ],
            ),
            "dnf" => (
                "sudo".to_string(),
                vec![
                    "dnf".to_string(),
                    "install".to_string(),
                    "-y".to_string(),
                    self.package.clone(),
                ],
            ),
            "yum" => (
                "sudo".to_string(),
                vec![
                    "yum".to_string(),
                    "install".to_string(),
                    "-y".to_string(),
                    self.package.clone(),
                ],
            ),
            "pacman" => (
                "sudo".to_string(),
                vec![
                    "pacman".to_string(),
                    "-S".to_string(),
                    "--noconfirm".to_string(),
                    self.package.clone(),
                ],
            ),
            _ => {
                return Err(RunnerError::ExecutionFailed(format!(
                    "Unsupported package manager: {}",
                    self.manager
                )))
            }
        };

        tracing::info!(
            "Installing {} via {}: {} {:?}",
            self.package,
            self.manager,
            program,
            args
        );

        let output = Command::new(&program).args(&args).output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(RunnerError::ExecutionFailed(format!(
                "Install failed: {}",
                stderr.trim()
            )))
        }
    }
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

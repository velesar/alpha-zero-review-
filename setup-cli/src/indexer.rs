//! SCIP indexer management - availability check, installation, and index building

use crate::language::Language;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Default timeout for indexer commands (5 minutes)
const COMMAND_TIMEOUT_SECS: u64 = 300;

/// Indexer availability status
#[derive(Debug, Clone)]
pub struct IndexerStatus {
    pub language: Language,
    pub installed: bool,
    pub command: &'static str,
    pub version: Option<String>,
}

/// Indexer configuration for each language
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Command to check availability
    pub check_command: &'static str,
    /// Arguments for version check
    pub check_args: &'static [&'static str],
    /// Command to build index
    pub build_command: &'static str,
    /// Arguments for building (project path appended)
    pub build_args: &'static [&'static str],
    /// Output file name (relative to project)
    pub output_file: &'static str,
    /// Install instructions
    pub install_instructions: &'static str,
    /// Install command (if automatable)
    pub install_command: Option<&'static [&'static str]>,
}

impl IndexerConfig {
    pub fn for_language(lang: Language) -> Self {
        match lang {
            Language::Rust => IndexerConfig {
                check_command: "rust-analyzer",
                check_args: &["--version"],
                build_command: "rust-analyzer",
                build_args: &["scip", "."],
                output_file: "index.scip",
                install_instructions: "rustup component add rust-analyzer",
                install_command: Some(&["rustup", "component", "add", "rust-analyzer"]),
            },
            Language::TypeScript | Language::JavaScript => IndexerConfig {
                check_command: "scip-typescript",
                check_args: &["--version"],
                build_command: "scip-typescript",
                build_args: &["index"],
                output_file: "index.scip",
                install_instructions: "npm install -g @sourcegraph/scip-typescript",
                install_command: Some(&["npm", "install", "-g", "@sourcegraph/scip-typescript"]),
            },
            Language::Python => IndexerConfig {
                check_command: "scip-python",
                check_args: &["--version"],
                build_command: "scip-python",
                build_args: &["index", "."],
                output_file: "index.scip",
                install_instructions: "pip install scip-python",
                install_command: Some(&["pip", "install", "scip-python"]),
            },
            Language::Go => IndexerConfig {
                check_command: "scip-go",
                check_args: &["--version"],
                build_command: "scip-go",
                build_args: &[],
                output_file: "index.scip",
                install_instructions: "go install github.com/sourcegraph/scip-go/cmd/scip-go@latest",
                install_command: Some(&[
                    "go",
                    "install",
                    "github.com/sourcegraph/scip-go/cmd/scip-go@latest",
                ]),
            },
            Language::Java => IndexerConfig {
                check_command: "scip-java",
                check_args: &["--version"],
                build_command: "scip-java",
                build_args: &["index"],
                output_file: "index.scip",
                install_instructions: "coursier install scip-java",
                install_command: Some(&["coursier", "install", "scip-java"]),
            },
        }
    }
}

/// Run a command with timeout support
///
/// # Arguments
/// * `cmd` - The command to run
/// * `args` - Arguments to pass to the command
/// * `cwd` - Optional working directory
/// * `timeout_secs` - Timeout in seconds
///
/// # Returns
/// The command output, or an error if timeout or execution fails
fn run_command_with_timeout(
    cmd: &str,
    args: &[&str],
    cwd: Option<&Path>,
    timeout_secs: u64,
) -> Result<std::process::Output> {
    use std::process::Stdio;
    use std::io::Read;

    let mut command = Command::new(cmd);
    command.args(args);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    let mut child = command.spawn()
        .context(format!("Failed to spawn command: {}", cmd))?;

    // Wait with timeout using a simple polling approach
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process finished, collect output
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();

                if let Some(mut out) = child.stdout.take() {
                    out.read_to_end(&mut stdout)?;
                }
                if let Some(mut err) = child.stderr.take() {
                    err.read_to_end(&mut stderr)?;
                }

                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                // Still running, check timeout
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    bail!(
                        "Command '{}' timed out after {} seconds",
                        cmd,
                        timeout_secs
                    );
                }
                // Sleep briefly before checking again
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                let _ = child.kill();
                bail!("Error waiting for command '{}': {}", cmd, e);
            }
        }
    }
}

/// Check if an indexer is available
pub fn check_indexer(lang: Language) -> IndexerStatus {
    let config = IndexerConfig::for_language(lang);

    // Use short timeout for availability check (10 seconds)
    let (installed, version) = match run_command_with_timeout(
        config.check_command,
        config.check_args,
        None,
        10,
    ) {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .map(|s| s.trim().to_string());
            (true, version)
        }
        _ => (false, None),
    };

    IndexerStatus {
        language: lang,
        installed,
        command: config.check_command,
        version,
    }
}

/// Check all indexers for detected languages
pub fn check_indexers(languages: &[Language]) -> Vec<IndexerStatus> {
    languages.iter().map(|&lang| check_indexer(lang)).collect()
}

/// Install an indexer for a language
pub fn install_indexer(lang: Language) -> Result<()> {
    let config = IndexerConfig::for_language(lang);

    let install_cmd = config
        .install_command
        .context(format!("No automatic install for {}. Manual install: {}", lang, config.install_instructions))?;

    println!("  Installing {} indexer...", lang);
    println!("    Running: {}", install_cmd.join(" "));

    let (cmd, args) = install_cmd.split_first()
        .ok_or_else(|| anyhow::anyhow!("Empty install command for {}", lang))?;

    let output = run_command_with_timeout(cmd, args, None, COMMAND_TIMEOUT_SECS)
        .context(format!("Failed to run install command for {}", lang))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Failed to install {} indexer: {}",
            lang,
            stderr.lines().next().unwrap_or("unknown error")
        );
    }

    println!("    ✓ Installed {}", lang);
    Ok(())
}

/// Build SCIP index for a language
pub fn build_index(lang: Language, project_path: &Path, output_dir: &Path) -> Result<PathBuf> {
    let config = IndexerConfig::for_language(lang);

    println!("  Building {} index...", lang);

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;

    // Run indexer from project directory with timeout
    let output = run_command_with_timeout(
        config.build_command,
        config.build_args,
        Some(project_path),
        COMMAND_TIMEOUT_SECS,
    )
    .context(format!("Failed to run {} indexer", lang))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Failed to build {} index: {}",
            lang,
            stderr.lines().take(3).collect::<Vec<_>>().join("\n")
        );
    }

    // Move index file to output directory
    let source_index = project_path.join(config.output_file);
    let target_index = output_dir.join(lang.index_filename());

    if source_index.exists() {
        std::fs::rename(&source_index, &target_index).context("Failed to move index file")?;
        println!("    ✓ Created {}", target_index.display());
        Ok(target_index)
    } else {
        // Some indexers output to different locations, try common alternatives
        let alt_locations = [
            project_path.join("index.scip"),
            project_path.join(".scip/index.scip"),
            project_path.join("target/scip/index.scip"),
        ];

        for alt in &alt_locations {
            if alt.exists() {
                std::fs::rename(alt, &target_index)?;
                println!("    ✓ Created {}", target_index.display());
                return Ok(target_index);
            }
        }

        bail!(
            "Index file not found after building. Expected at: {}",
            source_index.display()
        );
    }
}

/// Get current git commit hash
#[allow(dead_code)] // Utility function for future use
pub fn get_current_commit(project_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_path)
        .output()
        .context("Failed to get git commit")?;

    if !output.status.success() {
        bail!("Not a git repository or git not available");
    }

    let commit = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    Ok(commit[..12.min(commit.len())].to_string()) // Short hash
}

/// Build indexes for all detected languages
pub fn build_all_indexes(
    languages: &[Language],
    project_path: &Path,
    output_dir: &Path,
    install_missing: bool,
) -> Result<Vec<(Language, PathBuf)>> {
    let mut results = vec![];
    let mut warnings = vec![];

    for &lang in languages {
        let status = check_indexer(lang);

        if !status.installed {
            if install_missing {
                if let Err(e) = install_indexer(lang) {
                    warnings.push(format!("  ⚠ Could not install {} indexer: {}", lang, e));
                    continue;
                }
            } else {
                let config = IndexerConfig::for_language(lang);
                warnings.push(format!(
                    "  ⚠ {} indexer not found. Install with: {}",
                    lang, config.install_instructions
                ));
                continue;
            }
        }

        match build_index(lang, project_path, output_dir) {
            Ok(path) => results.push((lang, path)),
            Err(e) => warnings.push(format!("  ⚠ Failed to build {} index: {}", lang, e)),
        }
    }

    // Print warnings at the end
    for warning in warnings {
        println!("{}", warning);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexer_config() {
        let config = IndexerConfig::for_language(Language::Rust);
        assert_eq!(config.check_command, "rust-analyzer");
        assert_eq!(config.build_command, "rust-analyzer");
    }

    #[test]
    fn test_check_indexer_rust() {
        let status = check_indexer(Language::Rust);
        // May or may not be installed, just verify it runs
        assert_eq!(status.language, Language::Rust);
        assert_eq!(status.command, "rust-analyzer");
    }
}

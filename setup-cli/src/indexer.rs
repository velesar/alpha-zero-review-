//! SCIP indexer management - availability check, installation, and index building

use crate::language::Language;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

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
    pub language: Language,
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
                language: Language::Rust,
                check_command: "rust-analyzer",
                check_args: &["--version"],
                build_command: "rust-analyzer",
                build_args: &["scip", "."],
                output_file: "index.scip",
                install_instructions: "rustup component add rust-analyzer",
                install_command: Some(&["rustup", "component", "add", "rust-analyzer"]),
            },
            Language::TypeScript | Language::JavaScript => IndexerConfig {
                language: lang,
                check_command: "scip-typescript",
                check_args: &["--version"],
                build_command: "scip-typescript",
                build_args: &["index"],
                output_file: "index.scip",
                install_instructions: "npm install -g @sourcegraph/scip-typescript",
                install_command: Some(&["npm", "install", "-g", "@sourcegraph/scip-typescript"]),
            },
            Language::Python => IndexerConfig {
                language: Language::Python,
                check_command: "scip-python",
                check_args: &["--version"],
                build_command: "scip-python",
                build_args: &["index", "."],
                output_file: "index.scip",
                install_instructions: "pip install scip-python",
                install_command: Some(&["pip", "install", "scip-python"]),
            },
            Language::Go => IndexerConfig {
                language: Language::Go,
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
                language: Language::Java,
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

/// Check if an indexer is available
pub fn check_indexer(lang: Language) -> IndexerStatus {
    let config = IndexerConfig::for_language(lang);

    let (installed, version) = match Command::new(config.check_command)
        .args(config.check_args)
        .output()
    {
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

    let (cmd, args) = install_cmd.split_first().unwrap();

    let output = Command::new(cmd)
        .args(args)
        .output()
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

    // Run indexer from project directory
    let output = Command::new(config.build_command)
        .args(config.build_args)
        .current_dir(project_path)
        .output()
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

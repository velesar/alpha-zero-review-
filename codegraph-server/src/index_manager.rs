//! Index management for auto-loading and on-demand building
//!
//! This module provides:
//! - Auto-loading indexes from .audit/indexes/
//! - Commit hash freshness checking
//! - On-demand index building when indexer is available

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Supported languages for SCIP indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
}

impl Language {
    /// Get the index filename for this language
    pub fn index_filename(&self) -> &'static str {
        match self {
            Language::Rust => "rust.scip",
            Language::TypeScript => "typescript.scip",
            Language::JavaScript => "javascript.scip",
            Language::Python => "python.scip",
            Language::Go => "go.scip",
            Language::Java => "java.scip",
        }
    }

    /// Get the indexer command for this language
    pub fn indexer_command(&self) -> &'static str {
        match self {
            Language::Rust => "rust-analyzer",
            Language::TypeScript | Language::JavaScript => "scip-typescript",
            Language::Python => "scip-python",
            Language::Go => "scip-go",
            Language::Java => "scip-java",
        }
    }

    /// Get indexer arguments
    pub fn indexer_args(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["scip", "."],
            Language::TypeScript | Language::JavaScript => &["index"],
            Language::Python => &["index", "."],
            Language::Go => &[],
            Language::Java => &["index"],
        }
    }

    /// Detect language from project path
    pub fn detect(project_path: &Path) -> Vec<Language> {
        let mut languages = vec![];

        if project_path.join("Cargo.toml").exists() {
            languages.push(Language::Rust);
        }
        if project_path.join("package.json").exists() {
            if has_extension(project_path, "ts") || has_extension(project_path, "tsx") {
                languages.push(Language::TypeScript);
            } else if has_extension(project_path, "js") || has_extension(project_path, "jsx") {
                languages.push(Language::JavaScript);
            }
        }
        if project_path.join("pyproject.toml").exists()
            || project_path.join("setup.py").exists()
            || project_path.join("requirements.txt").exists()
        {
            languages.push(Language::Python);
        }
        if project_path.join("go.mod").exists() {
            languages.push(Language::Go);
        }
        if project_path.join("pom.xml").exists()
            || project_path.join("build.gradle").exists()
            || project_path.join("build.gradle.kts").exists()
        {
            languages.push(Language::Java);
        }

        languages
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::Python => write!(f, "Python"),
            Language::Go => write!(f, "Go"),
            Language::Java => write!(f, "Java"),
        }
    }
}

/// Status of an index
#[derive(Debug, Clone, Serialize)]
pub struct IndexStatus {
    pub language: Language,
    pub path: PathBuf,
    pub exists: bool,
    pub commit: Option<String>,
    pub is_fresh: bool,
}

/// Result of loading indexes
#[derive(Debug, Serialize)]
pub struct LoadIndexesResult {
    pub loaded: Vec<IndexStatus>,
    pub missing: Vec<Language>,
    pub warnings: Vec<String>,
}

/// Index manager for a project
pub struct IndexManager {
    project_path: PathBuf,
    index_dir: PathBuf,
}

impl IndexManager {
    pub fn new(project_path: PathBuf) -> Self {
        let index_dir = project_path.join(".audit/indexes");
        Self {
            project_path,
            index_dir,
        }
    }

    /// Get the indexes directory
    pub fn index_dir(&self) -> &Path {
        &self.index_dir
    }

    /// Discover available indexes in .audit/indexes/
    pub fn discover_indexes(&self) -> HashMap<Language, PathBuf> {
        let mut indexes = HashMap::new();

        if !self.index_dir.exists() {
            return indexes;
        }

        for lang in &[
            Language::Rust,
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Go,
            Language::Java,
        ] {
            let index_path = self.index_dir.join(lang.index_filename());
            if index_path.exists() {
                indexes.insert(*lang, index_path);
            }
        }

        indexes
    }

    /// Get current git commit hash
    pub fn get_current_commit(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.project_path)
            .output()
            .context("Failed to run git")?;

        if !output.status.success() {
            bail!("Not a git repository");
        }

        let commit = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        Ok(commit[..12.min(commit.len())].to_string())
    }

    /// Read commit hash from index metadata
    pub fn get_index_commit(&self, lang: Language) -> Option<String> {
        let meta_path = self.index_dir.join(format!("{}.meta", lang.index_filename()));
        std::fs::read_to_string(meta_path).ok().map(|s| s.trim().to_string())
    }

    /// Check if index is fresh (matches current commit)
    pub fn is_index_fresh(&self, lang: Language) -> bool {
        match (self.get_current_commit(), self.get_index_commit(lang)) {
            (Ok(current), Some(index)) => current == index,
            _ => false,
        }
    }

    /// Check if indexer is available for a language
    pub fn check_indexer(&self, lang: Language) -> bool {
        Command::new(lang.indexer_command())
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Build index for a language
    pub fn build_index(&self, lang: Language) -> Result<PathBuf> {
        if !self.check_indexer(lang) {
            bail!("{} indexer ({}) not found", lang, lang.indexer_command());
        }

        std::fs::create_dir_all(&self.index_dir)?;

        let output = Command::new(lang.indexer_command())
            .args(lang.indexer_args())
            .current_dir(&self.project_path)
            .output()
            .context(format!("Failed to run {} indexer", lang))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Indexer failed: {}", stderr.lines().take(3).collect::<Vec<_>>().join("\n"));
        }

        // Find and move the index file
        let source = self.project_path.join("index.scip");
        let target = self.index_dir.join(lang.index_filename());

        if source.exists() {
            std::fs::rename(&source, &target)?;
        } else {
            bail!("Index file not found after building");
        }

        // Write commit metadata
        if let Ok(commit) = self.get_current_commit() {
            let meta_path = self.index_dir.join(format!("{}.meta", lang.index_filename()));
            let _ = std::fs::write(meta_path, commit);
        }

        Ok(target)
    }

    /// Auto-load or build indexes for detected languages
    pub fn auto_load_or_build(&self, build_if_missing: bool) -> LoadIndexesResult {
        let detected = Language::detect(&self.project_path);
        let available = self.discover_indexes();

        let mut loaded = vec![];
        let mut missing = vec![];
        let mut warnings = vec![];

        for lang in detected {
            if let Some(path) = available.get(&lang) {
                let is_fresh = self.is_index_fresh(lang);
                let commit = self.get_index_commit(lang);

                loaded.push(IndexStatus {
                    language: lang,
                    path: path.clone(),
                    exists: true,
                    commit,
                    is_fresh,
                });

                if !is_fresh {
                    warnings.push(format!(
                        "{} index may be stale (commit mismatch). Consider rebuilding with --with-index",
                        lang
                    ));
                }
            } else if build_if_missing {
                match self.build_index(lang) {
                    Ok(path) => {
                        loaded.push(IndexStatus {
                            language: lang,
                            path,
                            exists: true,
                            commit: self.get_current_commit().ok(),
                            is_fresh: true,
                        });
                    }
                    Err(e) => {
                        warnings.push(format!("Could not build {} index: {}", lang, e));
                        missing.push(lang);
                    }
                }
            } else {
                missing.push(lang);
            }
        }

        LoadIndexesResult {
            loaded,
            missing,
            warnings,
        }
    }
}

/// Check if directory contains files with given extension (shallow check)
fn has_extension(path: &Path, ext: &str) -> bool {
    let dirs = ["src", "lib", "app", "."];

    for dir in dirs {
        let check_path = if dir == "." {
            path.to_path_buf()
        } else {
            path.join(dir)
        };

        if let Ok(entries) = std::fs::read_dir(&check_path) {
            for entry in entries.flatten() {
                if entry.path().extension().map_or(false, |e| e == ext) {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_language_detection_rust() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();

        let langs = Language::detect(dir.path());
        assert!(langs.contains(&Language::Rust));
    }

    #[test]
    fn test_index_filename() {
        assert_eq!(Language::Rust.index_filename(), "rust.scip");
        assert_eq!(Language::TypeScript.index_filename(), "typescript.scip");
    }
}

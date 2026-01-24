//! Language detection for SCIP index generation

use std::path::Path;

/// Supported languages for SCIP indexing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
}

impl Language {
    /// Get the artifact key for this language
    pub fn artifact_key(&self) -> &'static str {
        match self {
            Language::Rust => "scip-rust",
            Language::TypeScript => "scip-typescript",
            Language::JavaScript => "scip-javascript",
            Language::Python => "scip-python",
            Language::Go => "scip-go",
            Language::Java => "scip-java",
        }
    }

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

    /// Get all supported languages
    pub fn all() -> &'static [Language] {
        &[
            Language::Rust,
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Go,
            Language::Java,
        ]
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

/// Detect languages used in a project
pub fn detect_languages(project_path: &Path) -> Vec<Language> {
    let mut languages = vec![];

    // Rust: Cargo.toml
    if project_path.join("Cargo.toml").exists() {
        languages.push(Language::Rust);
    }

    // TypeScript/JavaScript: package.json
    if project_path.join("package.json").exists() {
        if has_files_with_extension(project_path, "ts")
            || has_files_with_extension(project_path, "tsx")
        {
            languages.push(Language::TypeScript);
        } else if has_files_with_extension(project_path, "js")
            || has_files_with_extension(project_path, "jsx")
        {
            languages.push(Language::JavaScript);
        }
    }

    // Python: pyproject.toml, setup.py, requirements.txt
    if project_path.join("pyproject.toml").exists()
        || project_path.join("setup.py").exists()
        || project_path.join("requirements.txt").exists()
    {
        languages.push(Language::Python);
    }

    // Go: go.mod
    if project_path.join("go.mod").exists() {
        languages.push(Language::Go);
    }

    // Java: pom.xml, build.gradle, build.gradle.kts
    if project_path.join("pom.xml").exists()
        || project_path.join("build.gradle").exists()
        || project_path.join("build.gradle.kts").exists()
    {
        languages.push(Language::Java);
    }

    languages
}

/// Check if project contains files with given extension
fn has_files_with_extension(project_path: &Path, ext: &str) -> bool {
    // Check common source directories
    let dirs_to_check = ["src", "lib", "app", ".", "packages"];

    for dir in dirs_to_check {
        let check_path = if dir == "." {
            project_path.to_path_buf()
        } else {
            project_path.join(dir)
        };

        if check_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&check_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == ext) {
                        return true;
                    }
                    // Check one level deeper
                    if path.is_dir() {
                        if let Ok(subentries) = std::fs::read_dir(&path) {
                            for subentry in subentries.flatten() {
                                if subentry.path().extension().map_or(false, |e| e == ext) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]").unwrap();

        let langs = detect_languages(dir.path());
        assert!(langs.contains(&Language::Rust));
    }

    #[test]
    fn test_detect_python() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "[project]").unwrap();

        let langs = detect_languages(dir.path());
        assert!(langs.contains(&Language::Python));
    }

    #[test]
    fn test_detect_go() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("go.mod"), "module test").unwrap();

        let langs = detect_languages(dir.path());
        assert!(langs.contains(&Language::Go));
    }

    #[test]
    fn test_artifact_key() {
        assert_eq!(Language::Rust.artifact_key(), "scip-rust");
        assert_eq!(Language::TypeScript.artifact_key(), "scip-typescript");
    }
}

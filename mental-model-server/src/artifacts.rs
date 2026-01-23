//! Artifact Store for commit-indexed storage
//!
//! This module provides storage and retrieval of audit artifacts
//! (SARIF files, SCIP indices, coverage reports) indexed by commit hash.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Metadata for stored artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub commit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub artifacts: HashMap<String, ArtifactInfo>,
}

/// Information about a single artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub produced_at: DateTime<Utc>,
    pub producer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

/// Available artifact record
#[derive(Debug, Clone, Serialize)]
pub struct AvailableArtifact {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub path: String,
    pub produced_at: DateTime<Utc>,
    pub producer: String,
}

/// Store artifact input
#[derive(Debug, Clone, Deserialize)]
pub struct StoreArtifactMetadata {
    pub producer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub produced_at: Option<DateTime<Utc>>,
}

/// Artifact store implementation
pub struct ArtifactStore {
    base_path: PathBuf,
}

/// Known artifact types
const ARTIFACT_TYPES: &[&str] = &[
    "semgrep", "bandit", "ruff", "trivy", "eslint",
    "scip", "coverage", "combined", "complexity",
];

impl ArtifactStore {
    /// Create a new artifact store
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Get the artifacts directory path
    fn artifacts_dir(&self) -> PathBuf {
        self.base_path.join(".audit").join("artifacts")
    }

    /// Get the directory for a specific commit
    fn commit_dir(&self, commit: &str) -> PathBuf {
        self.artifacts_dir().join(commit)
    }

    /// Get the metadata file path for a commit
    fn meta_path(&self, commit: &str) -> PathBuf {
        self.commit_dir(commit).join("_meta.yaml")
    }

    /// Resolve a commit reference (HEAD, latest, or hash)
    pub fn resolve_commit(&self, commit: &str) -> Result<String, std::io::Error> {
        if commit == "HEAD" || commit == "latest" {
            let latest = self.artifacts_dir().join("latest");
            if latest.is_symlink() || latest.exists() {
                if let Ok(target) = fs::read_link(&latest) {
                    return Ok(target
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(commit)
                        .to_string());
                }
            }

            // Try to get current git commit
            if let Ok(output) = std::process::Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .current_dir(&self.base_path)
                .output()
            {
                if output.status.success() {
                    let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !hash.is_empty() {
                        return Ok(hash);
                    }
                }
            }
        }
        Ok(commit.to_string())
    }

    /// Get available and missing artifacts for a commit
    pub fn get_commit_artifacts(
        &self,
        commit: &str,
    ) -> Result<(Vec<AvailableArtifact>, Vec<String>), std::io::Error> {
        let commit = self.resolve_commit(commit)?;
        let meta_path = self.meta_path(&commit);

        if !meta_path.exists() {
            // No artifacts stored for this commit
            return Ok((
                vec![],
                ARTIFACT_TYPES.iter().map(|s| s.to_string()).collect(),
            ));
        }

        let content = fs::read_to_string(&meta_path)?;
        let meta: ArtifactMetadata = serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut available = Vec::new();
        let mut missing = Vec::new();

        for artifact_type in ARTIFACT_TYPES {
            if let Some(info) = meta.artifacts.get(*artifact_type) {
                let ext = Self::get_extension(artifact_type);
                let artifact_path = self.commit_dir(&commit).join(format!("{}.{}", artifact_type, ext));

                if artifact_path.exists() {
                    available.push(AvailableArtifact {
                        artifact_type: artifact_type.to_string(),
                        path: artifact_path.to_string_lossy().to_string(),
                        produced_at: info.produced_at,
                        producer: info.producer.clone(),
                    });
                } else {
                    missing.push(artifact_type.to_string());
                }
            } else {
                missing.push(artifact_type.to_string());
            }
        }

        Ok((available, missing))
    }

    /// Store an artifact for a commit
    pub fn store_artifact(
        &self,
        commit: &str,
        artifact_type: &str,
        data: &[u8],
        metadata: &StoreArtifactMetadata,
    ) -> Result<String, std::io::Error> {
        let commit = self.resolve_commit(commit)?;
        let commit_dir = self.commit_dir(&commit);
        fs::create_dir_all(&commit_dir)?;

        // Determine file extension based on artifact type
        let ext = Self::get_extension(artifact_type);
        let artifact_path = commit_dir.join(format!("{}.{}", artifact_type, ext));

        // Write the artifact data
        fs::write(&artifact_path, data)?;

        // Update metadata
        let meta_path = self.meta_path(&commit);
        let mut meta = if meta_path.exists() {
            let content = fs::read_to_string(&meta_path)?;
            serde_yaml::from_str(&content).unwrap_or_else(|_| ArtifactMetadata {
                commit: commit.clone(),
                branch: None,
                timestamp: Utc::now(),
                artifacts: HashMap::new(),
            })
        } else {
            ArtifactMetadata {
                commit: commit.clone(),
                branch: self.get_current_branch(),
                timestamp: Utc::now(),
                artifacts: HashMap::new(),
            }
        };

        meta.artifacts.insert(
            artifact_type.to_string(),
            ArtifactInfo {
                produced_at: metadata.produced_at.unwrap_or_else(Utc::now),
                producer: metadata.producer.clone(),
                size_bytes: Some(data.len() as u64),
            },
        );

        let meta_yaml = serde_yaml::to_string(&meta)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&meta_path, meta_yaml)?;

        // Update latest symlink
        self.update_latest_symlink(&commit)?;

        Ok(artifact_path.to_string_lossy().to_string())
    }

    /// Get an artifact for a commit
    pub fn get_artifact(
        &self,
        commit: &str,
        artifact_type: &str,
    ) -> Result<(Vec<u8>, ArtifactInfo), std::io::Error> {
        let commit = self.resolve_commit(commit)?;
        let meta_path = self.meta_path(&commit);

        if !meta_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("No artifacts for commit: {}", commit),
            ));
        }

        let content = fs::read_to_string(&meta_path)?;
        let meta: ArtifactMetadata = serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let info = meta.artifacts.get(artifact_type).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Artifact not found: {}", artifact_type),
            )
        })?;

        let ext = Self::get_extension(artifact_type);
        let artifact_path = self.commit_dir(&commit).join(format!("{}.{}", artifact_type, ext));

        let data = fs::read(&artifact_path)?;

        Ok((data, info.clone()))
    }

    /// Get file extension for artifact type
    fn get_extension(artifact_type: &str) -> &'static str {
        match artifact_type {
            "scip" => "scip",
            "coverage" => "json",
            "complexity" => "json",
            _ => "sarif",
        }
    }

    /// Get current git branch
    fn get_current_branch(&self) -> Option<String> {
        std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(&self.base_path)
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }

    /// Update the latest symlink to point to a commit
    fn update_latest_symlink(&self, commit: &str) -> Result<(), std::io::Error> {
        let latest = self.artifacts_dir().join("latest");
        let _ = fs::remove_file(&latest);

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(commit, &latest)?;
        }

        #[cfg(not(unix))]
        {
            // On Windows, create a file with the commit hash
            fs::write(&latest, commit)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_artifact_store_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store = ArtifactStore::new(temp_dir.path().to_path_buf());
        assert!(store.artifacts_dir().ends_with(".audit/artifacts"));
    }

    #[test]
    fn test_store_and_retrieve_artifact() {
        let temp_dir = TempDir::new().unwrap();
        let store = ArtifactStore::new(temp_dir.path().to_path_buf());

        let test_data = b"test sarif content";
        let metadata = StoreArtifactMetadata {
            producer: "test".to_string(),
            produced_at: None,
        };

        let result = store.store_artifact("abc123", "semgrep", test_data, &metadata);
        assert!(result.is_ok());

        let (data, info) = store.get_artifact("abc123", "semgrep").unwrap();
        assert_eq!(data, test_data);
        assert_eq!(info.producer, "test");
    }

    #[test]
    fn test_get_commit_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let store = ArtifactStore::new(temp_dir.path().to_path_buf());

        // Initially no artifacts
        let (available, missing) = store.get_commit_artifacts("def456").unwrap();
        assert!(available.is_empty());
        assert!(!missing.is_empty());

        // Store an artifact
        let metadata = StoreArtifactMetadata {
            producer: "test".to_string(),
            produced_at: None,
        };
        store.store_artifact("def456", "bandit", b"test", &metadata).unwrap();

        // Now we have one artifact
        let (available, missing) = store.get_commit_artifacts("def456").unwrap();
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].artifact_type, "bandit");
        assert!(!missing.contains(&"bandit".to_string()));
    }
}

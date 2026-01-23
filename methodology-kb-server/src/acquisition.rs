//! Data Acquisition Cascade
//!
//! This module implements the data acquisition cascade for metrics:
//! 1. Check artifact store for cached data
//! 2. Run appropriate tool if not cached
//! 3. Cache results for future use

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Metric data source types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DataSource {
    /// Data from cached artifact
    Cache,
    /// Data freshly acquired from tool
    Fresh,
    /// Data estimated/derived from other metrics
    Derived,
    /// Data not available
    Unavailable,
}

/// Metric data with provenance
#[derive(Debug, Clone, Serialize)]
pub struct MetricData {
    pub metric: String,
    pub value: serde_json::Value,
    pub source: DataSource,
    pub produced_at: Option<DateTime<Utc>>,
    pub producer: Option<String>,
    pub commit: Option<String>,
}

/// Tool to metric mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMapping {
    pub tool: String,
    pub metrics: Vec<String>,
    pub artifact_type: String,
}

/// Acquisition status for a commit
#[derive(Debug, Clone, Serialize)]
pub struct AcquisitionStatus {
    pub commit: String,
    pub available_metrics: Vec<String>,
    pub missing_metrics: Vec<String>,
    pub tools_needed: Vec<String>,
}

/// Data acquisition manager
pub struct DataAcquisition {
    project_path: PathBuf,
    tool_mappings: Vec<ToolMapping>,
}

impl DataAcquisition {
    /// Create a new data acquisition manager
    pub fn new(project_path: PathBuf) -> Self {
        let tool_mappings = vec![
            ToolMapping {
                tool: "semgrep".to_string(),
                metrics: vec![
                    "security_findings".to_string(),
                    "injection_risk".to_string(),
                    "authentication_issues".to_string(),
                ],
                artifact_type: "semgrep".to_string(),
            },
            ToolMapping {
                tool: "bandit".to_string(),
                metrics: vec![
                    "python_security_findings".to_string(),
                    "hardcoded_secrets".to_string(),
                    "sql_injection_risk".to_string(),
                ],
                artifact_type: "bandit".to_string(),
            },
            ToolMapping {
                tool: "ruff".to_string(),
                metrics: vec![
                    "python_code_quality".to_string(),
                    "style_violations".to_string(),
                    "import_issues".to_string(),
                ],
                artifact_type: "ruff".to_string(),
            },
            ToolMapping {
                tool: "trivy".to_string(),
                metrics: vec![
                    "vulnerability_count".to_string(),
                    "critical_cves".to_string(),
                    "dependency_risk".to_string(),
                ],
                artifact_type: "trivy".to_string(),
            },
            ToolMapping {
                tool: "scip".to_string(),
                metrics: vec![
                    "symbol_count".to_string(),
                    "reference_density".to_string(),
                    "hotspot_count".to_string(),
                ],
                artifact_type: "scip".to_string(),
            },
        ];

        Self {
            project_path,
            tool_mappings,
        }
    }

    /// Get the artifact store directory
    fn artifacts_dir(&self) -> PathBuf {
        self.project_path.join(".audit").join("artifacts")
    }

    /// Resolve commit reference to actual hash
    pub fn resolve_commit(&self, commit: &str) -> Result<String, std::io::Error> {
        if commit == "HEAD" || commit == "latest" {
            // Try to get current git commit
            if let Ok(output) = Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .current_dir(&self.project_path)
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

    /// Get tool for a metric
    pub fn get_tool_for_metric(&self, metric: &str) -> Option<&ToolMapping> {
        self.tool_mappings.iter().find(|m| m.metrics.contains(&metric.to_string()))
    }

    /// Check if artifact exists for commit
    pub fn artifact_exists(&self, commit: &str, artifact_type: &str) -> bool {
        let commit_dir = self.artifacts_dir().join(commit);
        let ext = match artifact_type {
            "scip" => "scip",
            "coverage" => "json",
            _ => "sarif",
        };
        commit_dir.join(format!("{}.{}", artifact_type, ext)).exists()
    }

    /// Load artifact data
    pub fn load_artifact(&self, commit: &str, artifact_type: &str) -> Result<(serde_json::Value, ArtifactMeta), std::io::Error> {
        let commit_dir = self.artifacts_dir().join(commit);
        let ext = match artifact_type {
            "scip" => "scip",
            "coverage" => "json",
            _ => "sarif",
        };
        let artifact_path = commit_dir.join(format!("{}.{}", artifact_type, ext));

        let data = fs::read_to_string(&artifact_path)?;
        let value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Load metadata
        let meta_path = commit_dir.join("_meta.yaml");
        let meta = if meta_path.exists() {
            let meta_content = fs::read_to_string(&meta_path)?;
            let meta_yaml: serde_yaml::Value = serde_yaml::from_str(&meta_content)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let artifacts = meta_yaml.get("artifacts").and_then(|a| a.as_mapping());
            let artifact_info = artifacts.and_then(|a| a.get(&serde_yaml::Value::String(artifact_type.to_string())));

            ArtifactMeta {
                produced_at: artifact_info
                    .and_then(|i| i.get("produced_at"))
                    .and_then(|v| v.as_str())
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                producer: artifact_info
                    .and_then(|i| i.get("producer"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
            }
        } else {
            ArtifactMeta {
                produced_at: None,
                producer: None,
            }
        };

        Ok((value, meta))
    }

    /// Extract metric value from SARIF data
    pub fn extract_metric_from_sarif(&self, sarif: &serde_json::Value, metric: &str) -> serde_json::Value {
        // Count results from SARIF
        let runs = sarif.get("runs").and_then(|r| r.as_array());

        match metric {
            "security_findings" | "python_security_findings" | "vulnerability_count" => {
                let count = runs.map(|runs| {
                    runs.iter()
                        .filter_map(|run| run.get("results").and_then(|r| r.as_array()))
                        .map(|results| results.len())
                        .sum::<usize>()
                }).unwrap_or(0);
                serde_json::json!({ "count": count })
            }
            "critical_cves" => {
                let count = runs.map(|runs| {
                    runs.iter()
                        .filter_map(|run| run.get("results").and_then(|r| r.as_array()))
                        .flat_map(|results| results.iter())
                        .filter(|r| {
                            r.get("level").and_then(|l| l.as_str()) == Some("error") ||
                            r.get("properties")
                                .and_then(|p| p.get("severity"))
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_lowercase() == "critical")
                                .unwrap_or(false)
                        })
                        .count()
                }).unwrap_or(0);
                serde_json::json!({ "count": count })
            }
            "hardcoded_secrets" => {
                let count = runs.map(|runs| {
                    runs.iter()
                        .filter_map(|run| run.get("results").and_then(|r| r.as_array()))
                        .flat_map(|results| results.iter())
                        .filter(|r| {
                            r.get("ruleId").and_then(|id| id.as_str())
                                .map(|id| id.contains("hardcoded") || id.contains("secret") || id.contains("password"))
                                .unwrap_or(false)
                        })
                        .count()
                }).unwrap_or(0);
                serde_json::json!({ "count": count })
            }
            "style_violations" | "python_code_quality" | "import_issues" => {
                let count = runs.map(|runs| {
                    runs.iter()
                        .filter_map(|run| run.get("results").and_then(|r| r.as_array()))
                        .map(|results| results.len())
                        .sum::<usize>()
                }).unwrap_or(0);
                serde_json::json!({ "count": count })
            }
            _ => {
                // Return full result count as fallback
                let count = runs.map(|runs| {
                    runs.iter()
                        .filter_map(|run| run.get("results").and_then(|r| r.as_array()))
                        .map(|results| results.len())
                        .sum::<usize>()
                }).unwrap_or(0);
                serde_json::json!({ "count": count })
            }
        }
    }

    /// Get acquisition status for a commit
    pub fn get_acquisition_status(&self, commit: &str) -> Result<AcquisitionStatus, std::io::Error> {
        let commit = self.resolve_commit(commit)?;

        let mut available_metrics = Vec::new();
        let mut missing_metrics = Vec::new();
        let mut tools_needed = Vec::new();

        for mapping in &self.tool_mappings {
            if self.artifact_exists(&commit, &mapping.artifact_type) {
                available_metrics.extend(mapping.metrics.clone());
            } else {
                missing_metrics.extend(mapping.metrics.clone());
                if !tools_needed.contains(&mapping.tool) {
                    tools_needed.push(mapping.tool.clone());
                }
            }
        }

        Ok(AcquisitionStatus {
            commit,
            available_metrics,
            missing_metrics,
            tools_needed,
        })
    }

    /// Get metric data with cascade
    pub fn get_metric_data(&self, commit: &str, metric: &str) -> Result<MetricData, std::io::Error> {
        let commit = self.resolve_commit(commit)?;

        // Find which tool provides this metric
        let mapping = self.get_tool_for_metric(metric);

        if let Some(mapping) = mapping {
            // Check if we have cached data
            if self.artifact_exists(&commit, &mapping.artifact_type) {
                let (data, meta) = self.load_artifact(&commit, &mapping.artifact_type)?;
                let value = self.extract_metric_from_sarif(&data, metric);

                return Ok(MetricData {
                    metric: metric.to_string(),
                    value,
                    source: DataSource::Cache,
                    produced_at: meta.produced_at,
                    producer: meta.producer,
                    commit: Some(commit),
                });
            }

            // Return unavailable with tool hint
            return Ok(MetricData {
                metric: metric.to_string(),
                value: serde_json::json!({
                    "status": "unavailable",
                    "tool_needed": mapping.tool,
                    "hint": format!("Run {} to acquire this metric", mapping.tool)
                }),
                source: DataSource::Unavailable,
                produced_at: None,
                producer: None,
                commit: Some(commit),
            });
        }

        // Unknown metric
        Ok(MetricData {
            metric: metric.to_string(),
            value: serde_json::json!({
                "status": "unknown_metric",
                "available_metrics": self.tool_mappings.iter()
                    .flat_map(|m| m.metrics.clone())
                    .collect::<Vec<_>>()
            }),
            source: DataSource::Unavailable,
            produced_at: None,
            producer: None,
            commit: Some(commit),
        })
    }

    /// List all available metrics
    pub fn list_available_metrics(&self) -> Vec<String> {
        self.tool_mappings.iter()
            .flat_map(|m| m.metrics.clone())
            .collect()
    }
}

/// Artifact metadata
#[derive(Debug, Clone)]
pub struct ArtifactMeta {
    pub produced_at: Option<DateTime<Utc>>,
    pub producer: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tool_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let acquisition = DataAcquisition::new(temp_dir.path().to_path_buf());

        let mapping = acquisition.get_tool_for_metric("security_findings");
        assert!(mapping.is_some());
        assert_eq!(mapping.unwrap().tool, "semgrep");
    }

    #[test]
    fn test_acquisition_status() {
        let temp_dir = TempDir::new().unwrap();
        let acquisition = DataAcquisition::new(temp_dir.path().to_path_buf());

        let status = acquisition.get_acquisition_status("abc123").unwrap();
        assert!(status.available_metrics.is_empty());
        assert!(!status.missing_metrics.is_empty());
        assert!(!status.tools_needed.is_empty());
    }

    #[test]
    fn test_list_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let acquisition = DataAcquisition::new(temp_dir.path().to_path_buf());

        let metrics = acquisition.list_available_metrics();
        assert!(metrics.contains(&"security_findings".to_string()));
        assert!(metrics.contains(&"vulnerability_count".to_string()));
    }
}

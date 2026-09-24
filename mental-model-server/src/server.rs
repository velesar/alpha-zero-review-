//! Mental Model MCP Server implementation
//!
//! This module implements the MCP server that manages the Mental Model,
//! providing tools for reading, updating, and querying the model.

use crate::artifacts::{ArtifactStore, AvailableArtifact, StoreArtifactMetadata};
use crate::error::SynthesisError;
use crate::findings_store::FindingsStore;
use crate::model::{derive_constraints, Finding, FindingContext, MentalModel, Severity};
use crate::ops;
use anyhow::Result;
use rmcp::{
    handler::server::{
        tool::{Parameters, ToolCallContext, ToolRouter},
        ServerHandler,
    },
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorData, ListToolsResult,
        PaginatedRequestParam, ServerCapabilities, ServerInfo,
    },
    schemars,
    service::{RequestContext, RoleServer},
    tool, tool_router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

/// Mental Model MCP Server
pub struct MentalModelServer {
    model_path: PathBuf,
    model: Arc<RwLock<MentalModel>>,
    findings_store: Arc<Mutex<FindingsStore>>, // ADR-0007: separate findings storage
    artifact_store: Arc<ArtifactStore>,
    audit_dir: PathBuf,
    dirty: Arc<AtomicBool>, // ADR-0006: tracks unsaved changes
    tool_router: ToolRouter<Self>,
}

// Manual Clone implementation since FindingsStore contains SQLite Connection
impl Clone for MentalModelServer {
    fn clone(&self) -> Self {
        Self {
            model_path: self.model_path.clone(),
            model: Arc::clone(&self.model),
            findings_store: Arc::clone(&self.findings_store),
            artifact_store: Arc::clone(&self.artifact_store),
            audit_dir: self.audit_dir.clone(),
            dirty: Arc::clone(&self.dirty),
            tool_router: Self::tool_router(),
        }
    }
}

/// Input for update_viewpoint tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateViewpointInput {
    /// Viewpoint ID (e.g., "VP-F01", "VP-S02")
    pub viewpoint: String,
    /// Viewpoint data as JSON object
    pub data: serde_json::Value,
}

/// Input for get_context tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetContextInput {
    /// File path to get context for
    pub file_path: String,
}

/// Input for add_finding tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddFindingInput {
    /// Viewpoint that generated this finding
    pub viewpoint: String,
    /// Finding category (security, reliability, maintainability, etc.)
    pub category: String,
    /// Short title of the finding
    pub title: String,
    /// Detailed description
    pub description: String,
    /// File path where the finding was detected
    pub file_path: String,
    /// Line number (optional)
    pub line_number: Option<u32>,
    /// Base severity before context adjustment
    pub base_severity: String,
    /// Rule ID from the tool that detected it (optional)
    pub rule_id: Option<String>,
    /// Recommendation for fixing (optional)
    pub recommendation: Option<String>,
}

/// Input for synthesize tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SynthesizeInput {
    /// Clustering algorithm: "category_based" (default) or "location_based"
    #[serde(default = "default_algorithm")]
    pub algorithm: String,
}

fn default_algorithm() -> String {
    "category_based".to_string()
}

/// Single finding input for batch operations
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindingInput {
    /// Viewpoint that generated this finding
    pub viewpoint: String,
    /// Finding category (security, reliability, maintainability, etc.)
    pub category: String,
    /// Short title of the finding
    pub title: String,
    /// Detailed description
    pub description: String,
    /// File path where the finding was detected
    pub file_path: String,
    /// Line number (optional)
    pub line_number: Option<u32>,
    /// Base severity before context adjustment
    pub base_severity: String,
    /// Rule ID from the tool that detected it (optional)
    pub rule_id: Option<String>,
    /// Recommendation for fixing (optional)
    pub recommendation: Option<String>,
}

/// Input for add_findings batch tool (ADR-0005)
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddFindingsInput {
    /// List of findings to add
    pub findings: Vec<FindingInput>,
}

/// Input for get_contexts batch tool (ADR-0005)
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetContextsInput {
    /// List of file paths to get context for
    pub file_paths: Vec<String>,
}

/// Input for get_model_section tool (ADR-0005)
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetModelSectionInput {
    /// Section to retrieve
    pub section: String,
}

// ========== Findings Query Input Types (ADR-0007) ==========

/// Input for get_findings_by_file tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFindingsByFileInput {
    /// File path to filter by (exact match or prefix)
    pub file_path: String,
}

/// Input for get_findings_by_severity tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFindingsBySeverityInput {
    /// Severity level (critical, high, medium, low, info)
    pub severity: String,
}

/// Input for get_findings_by_viewpoint tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFindingsByViewpointInput {
    /// Viewpoint ID (e.g., "VP-Q01", "VP-Q02")
    pub viewpoint: String,
}

/// Input for get_findings_by_category tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetFindingsByCategoryInput {
    /// Category to filter by (security, reliability, maintainability, etc.)
    pub category: String,
}

/// Input for export_findings tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportFindingsInput {
    /// Output file, relative to the audit directory (e.g. "reports/findings.json").
    /// Absolute paths are accepted only inside the audit directory.
    pub output_path: String,
    /// Replace the file if it already exists (default: false)
    #[serde(default)]
    pub overwrite: bool,
}

/// Input for init_model tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InitModelInput {
    /// Project name
    pub name: String,
    /// Project path
    pub path: String,
    /// Optional project description
    pub description: Option<String>,
    /// Optional repository URL
    pub repository: Option<String>,
}

/// Input for get_commit_artifacts tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetCommitArtifactsInput {
    /// Commit hash, "HEAD", or "latest"
    #[serde(default = "default_commit")]
    pub commit: String,
}

fn default_commit() -> String {
    "HEAD".to_string()
}

/// Input for store_artifact tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct StoreArtifactInput {
    /// Commit hash
    pub commit: String,
    /// Artifact type (semgrep, bandit, ruff, trivy, scip, coverage, etc.)
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// Artifact data (JSON string for SARIF, base64 for binary)
    pub data: String,
    /// Producer of the artifact
    pub producer: String,
}

/// Input for get_artifact tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetArtifactInput {
    /// Commit hash, "HEAD", or "latest"
    #[serde(default = "default_commit")]
    pub commit: String,
    /// Artifact type
    #[serde(rename = "type")]
    pub artifact_type: String,
}

/// Output for get_commit_artifacts
#[derive(Debug, Serialize)]
pub struct GetCommitArtifactsOutput {
    pub commit: String,
    pub available: Vec<AvailableArtifact>,
    pub missing: Vec<String>,
}

/// Output for store_artifact
#[derive(Debug, Serialize)]
pub struct StoreArtifactOutput {
    pub stored_at: String,
    pub commit: String,
    pub artifact_type: String,
}

/// Output for get_artifact
#[derive(Debug, Serialize)]
pub struct GetArtifactOutput {
    pub data: String,
    pub commit: String,
    pub artifact_type: String,
    pub produced_at: String,
    pub producer: String,
}

fn parse_base_severity(value: &str) -> Result<Severity, rmcp::ErrorData> {
    ops::parse_severity(value).ok_or_else(|| {
        rmcp::ErrorData::invalid_params(
            format!(
                "Invalid severity '{}': expected CRITICAL, HIGH, MEDIUM, LOW or INFO",
                value
            ),
            None,
        )
    })
}

/// Map artifact store errors: bad input and missing artifacts are the
/// caller's to fix (invalid_params); anything else is an internal error.
fn artifact_error(context: &str, e: std::io::Error) -> rmcp::ErrorData {
    let message = format!("{}: {}", context, e);
    match e.kind() {
        std::io::ErrorKind::InvalidInput | std::io::ErrorKind::NotFound => {
            rmcp::ErrorData::invalid_params(message, None)
        }
        _ => rmcp::ErrorData::internal_error(message, None),
    }
}

#[tool_router]
impl MentalModelServer {
    /// Create a new MentalModelServer
    ///
    /// # Arguments
    /// * `model_path` - Path to the mental model YAML file
    ///
    /// # Errors
    /// Returns an error if:
    /// - The model file exists but cannot be parsed (logged as warning, uses default)
    /// - The findings store cannot be created
    pub fn new(model_path: PathBuf) -> Result<Self> {
        Self::with_audit_path(model_path, None)
    }

    /// Create a server with an explicit audit directory.
    ///
    /// When `audit_path` is `None`, the audit directory defaults to
    /// `<project>/.audit`, where the project is taken from the model or the
    /// current working directory.
    pub fn with_audit_path(model_path: PathBuf, audit_path: Option<PathBuf>) -> Result<Self> {
        let model = if model_path.exists() {
            match fs::read_to_string(&model_path) {
                Ok(content) => match serde_yaml::from_str(&content) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse model file {}: {}, using default",
                            model_path.display(),
                            e
                        );
                        MentalModel::default()
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        "Failed to read model file {}: {}, using default",
                        model_path.display(),
                        e
                    );
                    MentalModel::default()
                }
            }
        } else {
            MentalModel::default()
        };

        // Get project path from model or use current directory
        let project_path = if !model.project.path.is_empty() {
            PathBuf::from(&model.project.path)
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        let audit_dir = audit_path.unwrap_or_else(|| project_path.join(".audit"));

        // ADR-0007: Create findings store in .audit directory
        let findings_db_path = audit_dir.join("findings.db");
        let findings_store = FindingsStore::new(&findings_db_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to create findings store at {}: {}",
                findings_db_path.display(),
                e
            )
        })?;

        Ok(Self {
            model_path,
            model: Arc::new(RwLock::new(model)),
            findings_store: Arc::new(Mutex::new(findings_store)),
            artifact_store: Arc::new(ArtifactStore::with_audit_dir(
                project_path,
                audit_dir.clone(),
            )),
            audit_dir,
            dirty: Arc::new(AtomicBool::new(false)), // ADR-0006
            tool_router: Self::tool_router(),
        })
    }

    fn save_model(&self) -> Result<()> {
        let model = self
            .model
            .read()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let yaml = serde_yaml::to_string(&*model)?;
        fs::write(&self.model_path, yaml)?;
        Ok(())
    }

    // ========== Deferred Persistence (ADR-0006) ==========

    /// Mark the model as having unsaved changes
    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Flush to disk if there are pending changes. Returns true if flushed.
    fn flush_if_dirty(&self) -> Result<bool> {
        if self.dirty.swap(false, Ordering::SeqCst) {
            self.save_model()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if there are unsaved changes
    #[allow(dead_code)] // Useful for testing
    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    /// Initialize a new mental model for a project
    #[tool(
        description = "Initialize a new mental model for a project. Call this before starting an audit."
    )]
    async fn init_model(
        &self,
        input: Parameters<InitModelInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let mut model = self
            .model
            .write()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        *model = MentalModel::new(input.name, input.path);
        model.project.description = input.description;
        model.project.repository = input.repository;

        drop(model);
        self.save_model()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Save error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(
            "Mental model initialized successfully",
        )]))
    }

    /// Get the current mental model state
    #[tool(
        description = "Get the current mental model state as YAML. Returns the complete model including all viewpoint data, constraints, and findings."
    )]
    async fn get_model(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let model = self
            .model
            .read()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        let yaml = serde_yaml::to_string(&*model).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(yaml)]))
    }

    /// Update the mental model with viewpoint results
    #[tool(
        description = "Update the mental model with results from a viewpoint analysis. This will also recalculate derived constraints."
    )]
    async fn update_viewpoint(
        &self,
        input: Parameters<UpdateViewpointInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let mut model = self
            .model
            .write()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        model
            .apply_viewpoint(&input.viewpoint, input.data)
            .map_err(|e| {
                rmcp::ErrorData::invalid_params(format!("Apply viewpoint error: {}", e), None)
            })?;

        // Recalculate constraints
        model.constraints = derive_constraints(&model);

        let constraints_json = serde_json::to_string_pretty(&model.constraints).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        drop(model);
        // ADR-0006: Phase boundary - flush all pending changes
        self.mark_dirty();
        self.flush_if_dirty()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Flush error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Viewpoint {} applied successfully.\n\nUpdated constraints:\n{}",
            input.viewpoint, constraints_json
        ))]))
    }

    /// Get business context for a file path
    #[tool(
        description = "Get business context for a specific file path. Returns bounded context type, architecture layer, and hotspot status."
    )]
    async fn get_context(
        &self,
        input: Parameters<GetContextInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let model = self
            .model
            .read()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        let context = model.get_context_for_path(&input.file_path);

        let json = serde_json::to_string_pretty(&context).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get derived analysis constraints
    #[tool(
        description = "Get the derived analysis constraints. These are automatically calculated paths that should receive priority attention based on the mental model."
    )]
    async fn get_constraints(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let model = self
            .model
            .read()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        let json = serde_json::to_string_pretty(&model.constraints).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Add a finding with automatic context enrichment
    #[tool(
        description = "Add a finding from quality analysis. The finding will be automatically enriched with context from the mental model and severity will be adjusted."
    )]
    async fn add_finding(
        &self,
        input: Parameters<AddFindingInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let base_severity = parse_base_severity(&input.base_severity)?;
        let mut model = self
            .model
            .write()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        // Get context for the file
        let context = model.get_context_for_path(&input.file_path);

        // Calculate adjusted severity
        let adjusted_severity = ops::adjust_severity(&base_severity, &context);

        let finding = Finding {
            id: format!("F-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            viewpoint: input.viewpoint.clone(),
            category: input.category,
            title: input.title,
            description: input.description,
            file_path: input.file_path,
            line_number: input.line_number,
            base_severity,
            adjusted_severity: adjusted_severity.clone(),
            rule_id: input.rule_id,
            context: Some(context),
            recommendation: input.recommendation,
        };

        let finding_json = serde_json::to_string_pretty(&finding).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        // ADR-0007: Store finding in SQLite database
        {
            let store = self.findings_store.lock().map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
            })?;
            store.add(&finding).map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Failed to add finding: {}", e), None)
            })?;
        }

        // Mark viewpoint as having findings
        if !model.completed_viewpoints.contains(&input.viewpoint) {
            model.completed_viewpoints.push(input.viewpoint);
        }

        drop(model);
        // ADR-0006: Defer persistence for model (findings already persisted to SQLite)
        self.mark_dirty();

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Finding added with adjusted severity: {:?}\n\n{}",
            adjusted_severity, finding_json
        ))]))
    }

    /// Get all findings from the store
    #[tool(description = "Get all findings from quality analysis viewpoints.")]
    async fn get_findings(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        // ADR-0007: Get findings from SQLite store
        let store = self.findings_store.lock().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
        })?;

        let findings = store.get_all().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to get findings: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&findings).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Total findings: {}\n\n{}",
            findings.len(),
            json
        ))]))
    }

    /// Synthesize findings into root causes
    #[tool(
        description = "Cluster findings into root causes. This analyzes patterns across findings to identify underlying issues."
    )]
    async fn synthesize(
        &self,
        input: Parameters<SynthesizeInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        // ADR-0007: Get findings from SQLite store
        let findings = {
            let store = self.findings_store.lock().map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
            })?;
            store.get_all().map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Failed to get findings: {}", e), None)
            })?
        };

        let synthesis = match input.algorithm.as_str() {
            "category_based" => ops::synthesize_by_category(&findings),
            "location_based" => ops::synthesize_by_location(&findings),
            other => {
                return Err(SynthesisError::UnknownAlgorithm(other.to_string()).into());
            }
        };

        // Store root causes in model
        let mut model = self
            .model
            .write()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;
        model.root_causes = synthesis.root_causes.clone();

        drop(model);
        // ADR-0006: End of audit - flush all pending changes
        self.mark_dirty();
        self.flush_if_dirty()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Flush error: {}", e), None))?;

        let covered: u32 = synthesis
            .root_causes
            .iter()
            .map(|rc| rc.finding_count)
            .sum();
        let omitted: Vec<serde_json::Value> = synthesis
            .omitted
            .iter()
            .map(|rc| {
                serde_json::json!({
                    "title": rc.title,
                    "impact": rc.impact,
                    "finding_count": rc.finding_count,
                    "finding_ids": rc.finding_ids,
                })
            })
            .collect();

        let response = serde_json::json!({
            "total_findings": findings.len(),
            "root_causes": synthesis.root_causes,
            "omitted_clusters": omitted,
            "ungrouped_findings": synthesis.ungrouped_findings,
        });
        let json = serde_json::to_string_pretty(&response).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Synthesized {} root causes covering {} of {} findings \
             ({} lower-ranked clusters omitted, {} findings ungrouped)\n\n{}",
            synthesis.root_causes.len(),
            covered,
            findings.len(),
            synthesis.omitted.len(),
            synthesis.ungrouped_findings,
            json
        ))]))
    }

    /// Get completed viewpoints
    #[tool(description = "Get list of viewpoints that have been completed.")]
    async fn get_completed_viewpoints(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let model = self
            .model
            .read()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        let json = serde_json::to_string_pretty(&model.completed_viewpoints).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get available and missing artifacts for a commit
    #[tool(
        description = "List available and missing artifacts for a specific commit. Use 'HEAD' or 'latest' for current commit."
    )]
    async fn get_commit_artifacts(
        &self,
        input: Parameters<GetCommitArtifactsInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let commit = self
            .artifact_store
            .resolve_commit(&input.commit)
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Failed to resolve commit: {}", e), None)
            })?;

        let (available, missing) = self
            .artifact_store
            .get_commit_artifacts(&commit)
            .map_err(|e| artifact_error("Failed to get artifacts", e))?;

        let output = GetCommitArtifactsOutput {
            commit,
            available,
            missing,
        };

        let json = serde_json::to_string_pretty(&output).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Store an artifact for a commit
    #[tool(
        description = "Store a tool output artifact (SARIF, SCIP, coverage) for a specific commit. Data should be JSON for SARIF or base64 for binary."
    )]
    async fn store_artifact(
        &self,
        input: Parameters<StoreArtifactInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let commit = self
            .artifact_store
            .resolve_commit(&input.commit)
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Failed to resolve commit: {}", e), None)
            })?;

        let metadata = StoreArtifactMetadata {
            producer: input.producer,
            produced_at: None,
        };

        let stored_at = self
            .artifact_store
            .store_artifact(
                &commit,
                &input.artifact_type,
                input.data.as_bytes(),
                &metadata,
            )
            .map_err(|e| artifact_error("Failed to store artifact", e))?;

        let output = StoreArtifactOutput {
            stored_at,
            commit,
            artifact_type: input.artifact_type,
        };

        let json = serde_json::to_string_pretty(&output).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Retrieve an artifact for a commit
    #[tool(
        description = "Retrieve a stored artifact by commit and type. Returns the artifact data along with metadata."
    )]
    async fn get_artifact(
        &self,
        input: Parameters<GetArtifactInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let commit = self
            .artifact_store
            .resolve_commit(&input.commit)
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Failed to resolve commit: {}", e), None)
            })?;

        let (data, info) = self
            .artifact_store
            .get_artifact(&commit, &input.artifact_type)
            .map_err(|e| artifact_error("Artifact not found", e))?;

        let data_str = String::from_utf8(data).unwrap_or_else(|e| {
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, e.into_bytes())
        });

        let output = GetArtifactOutput {
            data: data_str,
            commit,
            artifact_type: input.artifact_type,
            produced_at: info.produced_at.to_rfc3339(),
            producer: info.producer,
        };

        let json = serde_json::to_string_pretty(&output).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ========== Batch Operations (ADR-0005) ==========

    /// Add multiple findings in a single operation
    #[tool(
        description = "Add multiple findings from quality analysis in a single batch operation. Each finding will be automatically enriched with context and severity adjusted. More efficient than multiple add_finding calls. (ADR-0005)"
    )]
    async fn add_findings(
        &self,
        input: Parameters<AddFindingsInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let mut model = self
            .model
            .write()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        let mut added_findings = Vec::new();
        let mut viewpoints_touched = std::collections::HashSet::new();

        // Validate the whole batch before storing any of it
        let severities = input
            .findings
            .iter()
            .enumerate()
            .map(|(i, f)| {
                parse_base_severity(&f.base_severity).map_err(|e| {
                    rmcp::ErrorData::invalid_params(format!("findings[{}]: {}", i, e.message), None)
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        for (finding_input, base_severity) in input.findings.into_iter().zip(severities) {
            // Get context for the file
            let context = model.get_context_for_path(&finding_input.file_path);

            // Calculate adjusted severity
            let adjusted_severity = ops::adjust_severity(&base_severity, &context);

            let finding = Finding {
                id: format!("F-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
                viewpoint: finding_input.viewpoint.clone(),
                category: finding_input.category,
                title: finding_input.title,
                description: finding_input.description,
                file_path: finding_input.file_path,
                line_number: finding_input.line_number,
                base_severity,
                adjusted_severity,
                rule_id: finding_input.rule_id,
                context: Some(context),
                recommendation: finding_input.recommendation,
            };

            viewpoints_touched.insert(finding_input.viewpoint);
            added_findings.push(finding);
        }

        // ADR-0007: Store findings in SQLite database (batch operation)
        {
            let store = self.findings_store.lock().map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
            })?;
            store.add_batch(&added_findings).map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Failed to add findings: {}", e), None)
            })?;
        }

        // Mark viewpoints as having findings
        for vp in viewpoints_touched {
            if !model.completed_viewpoints.contains(&vp) {
                model.completed_viewpoints.push(vp);
            }
        }

        let added_count = added_findings.len();

        drop(model);
        // ADR-0006: Defer persistence for model (findings already persisted to SQLite)
        self.mark_dirty();

        let json = serde_json::to_string_pretty(&added_findings).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Added {} findings in batch\n\n{}",
            added_count, json
        ))]))
    }

    /// Get context for multiple file paths in a single operation
    #[tool(
        description = "Get business context for multiple file paths in a single batch operation. Returns a map of file paths to their context (bounded context type, architecture layer, hotspot status). More efficient than multiple get_context calls. (ADR-0005)"
    )]
    async fn get_contexts(
        &self,
        input: Parameters<GetContextsInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let model = self
            .model
            .read()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        let mut contexts: HashMap<String, FindingContext> = HashMap::new();

        for file_path in input.file_paths {
            let context = model.get_context_for_path(&file_path);
            contexts.insert(file_path, context);
        }

        let json = serde_json::to_string_pretty(&contexts).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get a specific section of the mental model
    #[tool(
        description = "Get a specific section of the mental model instead of the full model. Sections: project, tech_stack, structure, build_deploy, module_hierarchy, architecture, domain_model, entity_model, interface_surface, hotspots, constraints, findings, root_causes, completed_viewpoints. More efficient than get_model when only one section is needed. (ADR-0005)"
    )]
    async fn get_model_section(
        &self,
        input: Parameters<GetModelSectionInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        // ADR-0007: Handle findings section separately from FindingsStore
        if input.section == "findings" {
            let store = self.findings_store.lock().map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
            })?;
            let findings = store.get_all().map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Failed to get findings: {}", e), None)
            })?;
            let json = serde_json::to_string_pretty(&findings).map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
            })?;
            return Ok(CallToolResult::success(vec![Content::text(json)]));
        }

        let model = self
            .model
            .read()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None))?;

        let section_json: serde_json::Value = match input.section.as_str() {
            "project" => serde_json::to_value(&model.project),
            "tech_stack" => serde_json::to_value(&model.tech_stack),
            "structure" => serde_json::to_value(&model.structure),
            "build_deploy" => serde_json::to_value(&model.build_deploy),
            "module_hierarchy" => serde_json::to_value(&model.module_hierarchy),
            "architecture" => serde_json::to_value(&model.architecture),
            "domain_model" => serde_json::to_value(&model.domain_model),
            "entity_model" => serde_json::to_value(&model.entity_model),
            "interface_surface" => serde_json::to_value(&model.interface_surface),
            "hotspots" => serde_json::to_value(&model.hotspots),
            "constraints" => serde_json::to_value(&model.constraints),
            "root_causes" => serde_json::to_value(&model.root_causes),
            "completed_viewpoints" => serde_json::to_value(&model.completed_viewpoints),
            _ => {
                return Err(rmcp::ErrorData::invalid_params(
                    format!("Unknown section: '{}'. Valid sections: project, tech_stack, structure, build_deploy, module_hierarchy, architecture, domain_model, entity_model, interface_surface, hotspots, constraints, findings, root_causes, completed_viewpoints", input.section),
                    None,
                ));
            }
        }.map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&section_json).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ========== Deferred Persistence (ADR-0006) ==========

    /// Flush pending changes to disk
    #[tool(
        description = "Persist any pending changes to disk. Called automatically at phase boundaries (update_viewpoint, synthesize), but can be called explicitly for additional safety. (ADR-0006)"
    )]
    async fn flush(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let flushed = self
            .flush_if_dirty()
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Flush error: {}", e), None))?;

        let message = if flushed {
            "Pending changes flushed to disk"
        } else {
            "No pending changes to flush"
        };

        Ok(CallToolResult::success(vec![Content::text(format!(
            "{{\"flushed\": {}, \"message\": \"{}\"}}",
            flushed, message
        ))]))
    }

    // ========== Findings Query Tools (ADR-0007) ==========

    /// Get findings by file path
    #[tool(
        description = "Get findings filtered by file path. Returns all findings for files matching the given path. (ADR-0007)"
    )]
    async fn get_findings_by_file(
        &self,
        input: Parameters<GetFindingsByFileInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let store = self.findings_store.lock().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
        })?;

        let findings = store.get_by_file(&input.file_path).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to query findings: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&findings).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Found {} findings for file path '{}'\n\n{}",
            findings.len(),
            input.file_path,
            json
        ))]))
    }

    /// Get findings by severity level
    #[tool(
        description = "Get findings filtered by severity level (critical, high, medium, low, info). (ADR-0007)"
    )]
    async fn get_findings_by_severity(
        &self,
        input: Parameters<GetFindingsBySeverityInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let severity = match input.severity.to_uppercase().as_str() {
            "CRITICAL" => Severity::Critical,
            "HIGH" => Severity::High,
            "MEDIUM" => Severity::Medium,
            "LOW" => Severity::Low,
            "INFO" => Severity::Info,
            _ => {
                return Err(rmcp::ErrorData::invalid_params(
                    format!(
                        "Unknown severity: '{}'. Valid values: critical, high, medium, low, info",
                        input.severity
                    ),
                    None,
                ));
            }
        };

        let store = self.findings_store.lock().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
        })?;

        let findings = store.get_by_severity(&severity).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to query findings: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&findings).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Found {} findings with severity {:?}\n\n{}",
            findings.len(),
            severity,
            json
        ))]))
    }

    /// Get findings by viewpoint
    #[tool(description = "Get findings filtered by viewpoint (e.g., VP-Q01, VP-Q02). (ADR-0007)")]
    async fn get_findings_by_viewpoint(
        &self,
        input: Parameters<GetFindingsByViewpointInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let store = self.findings_store.lock().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
        })?;

        let findings = store.get_by_viewpoint(&input.viewpoint).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to query findings: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&findings).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Found {} findings for viewpoint '{}'\n\n{}",
            findings.len(),
            input.viewpoint,
            json
        ))]))
    }

    /// Get findings by category
    #[tool(
        description = "Get findings filtered by category (security, reliability, maintainability, performance, testability). (ADR-0007)"
    )]
    async fn get_findings_by_category(
        &self,
        input: Parameters<GetFindingsByCategoryInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let store = self.findings_store.lock().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
        })?;

        let findings = store.get_by_category(&input.category).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to query findings: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&findings).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Found {} findings for category '{}'\n\n{}",
            findings.len(),
            input.category,
            json
        ))]))
    }

    /// Get findings summary statistics
    #[tool(
        description = "Get summary statistics for all findings including counts by severity, category, and viewpoint. (ADR-0007)"
    )]
    async fn get_findings_summary(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let store = self.findings_store.lock().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
        })?;

        let summary = store.get_summary().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to get summary: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&summary).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Export findings to JSON file
    #[tool(
        description = "Export all findings to a JSON file. Useful for sharing or external processing. (ADR-0007)"
    )]
    async fn export_findings(
        &self,
        input: Parameters<ExportFindingsInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let store = self.findings_store.lock().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
        })?;

        let target =
            crate::utils::resolve_output_path(&self.audit_dir, &input.output_path, input.overwrite)
                .map_err(|e| artifact_error("Cannot export findings", e))?;

        let count = store.export_json(&target).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to export findings: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Exported {} findings to '{}'",
            count,
            target.display()
        ))]))
    }
}

impl ServerHandler for MentalModelServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: rmcp::model::Implementation {
                name: "mental-model".into(),
                version: "0.1.0".into(),
            },
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some("Mental Model MCP Server for AI Code Audit. Use this server to manage the central mental model artifact that accumulates understanding throughout the audit process.".into()),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _pagination: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            next_cursor: None,
        })
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        let tool_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context)
    }
}

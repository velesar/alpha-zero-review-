//! Mental Model MCP Server implementation
//!
//! This module implements the MCP server that manages the Mental Model,
//! providing tools for reading, updating, and querying the model.

use crate::artifacts::{ArtifactStore, AvailableArtifact, StoreArtifactMetadata};
use crate::findings_store::FindingsStore;
use crate::model::{derive_constraints, Finding, FindingContext, MentalModel, RootCause, Severity};
use anyhow::Result;
use std::future::Future;
use rmcp::{
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo, PaginatedRequestParam, ListToolsResult, ErrorData, CallToolRequestParam},
    schemars, tool,
    handler::server::{tool::{ToolRouter, Parameters, ToolCallContext}, ServerHandler},
    tool_router,
    service::{RequestContext, RoleServer},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

/// Mental Model MCP Server
pub struct MentalModelServer {
    model_path: PathBuf,
    model: Arc<RwLock<MentalModel>>,
    findings_store: Arc<Mutex<FindingsStore>>,  // ADR-0007: separate findings storage
    artifact_store: Arc<ArtifactStore>,
    dirty: Arc<AtomicBool>,  // ADR-0006: tracks unsaved changes
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
    /// Clustering algorithm to use
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
    /// Output file path for JSON export
    pub output_path: String,
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

#[tool_router]
impl MentalModelServer {
    pub fn new(model_path: PathBuf) -> Self {
        let model = if model_path.exists() {
            match fs::read_to_string(&model_path) {
                Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
                Err(_) => MentalModel::default(),
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

        // ADR-0007: Create findings store in .audit directory
        let findings_db_path = project_path.join(".audit").join("findings.db");
        let findings_store = FindingsStore::new(&findings_db_path)
            .expect("Failed to create findings store");

        Self {
            model_path,
            model: Arc::new(RwLock::new(model)),
            findings_store: Arc::new(Mutex::new(findings_store)),
            artifact_store: Arc::new(ArtifactStore::new(project_path)),
            dirty: Arc::new(AtomicBool::new(false)),  // ADR-0006
            tool_router: Self::tool_router(),
        }
    }

    fn save_model(&self) -> Result<()> {
        let model = self.model.read().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
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
    #[allow(dead_code)]  // Useful for testing
    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::SeqCst)
    }

    /// Initialize a new mental model for a project
    #[tool(description = "Initialize a new mental model for a project. Call this before starting an audit.")]
    async fn init_model(&self, input: Parameters<InitModelInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let mut model = self.model.write().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        *model = MentalModel::new(input.name, input.path);
        model.project.description = input.description;
        model.project.repository = input.repository;

        drop(model);
        self.save_model().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Save error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            "Mental model initialized successfully",
        )]))
    }

    /// Get the current mental model state
    #[tool(description = "Get the current mental model state as YAML. Returns the complete model including all viewpoint data, constraints, and findings.")]
    async fn get_model(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let model = self.model.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let yaml = serde_yaml::to_string(&*model).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(yaml)]))
    }

    /// Update the mental model with viewpoint results
    #[tool(description = "Update the mental model with results from a viewpoint analysis. This will also recalculate derived constraints.")]
    async fn update_viewpoint(&self, input: Parameters<UpdateViewpointInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let mut model = self.model.write().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        model.apply_viewpoint(&input.viewpoint, input.data).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Apply viewpoint error: {}", e), None)
        })?;

        // Recalculate constraints
        model.constraints = derive_constraints(&model);

        let constraints_json = serde_json::to_string_pretty(&model.constraints).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        drop(model);
        // ADR-0006: Phase boundary - flush all pending changes
        self.mark_dirty();
        self.flush_if_dirty().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Flush error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Viewpoint {} applied successfully.\n\nUpdated constraints:\n{}",
            input.viewpoint, constraints_json
        ))]))
    }

    /// Get business context for a file path
    #[tool(description = "Get business context for a specific file path. Returns bounded context type, architecture layer, and hotspot status.")]
    async fn get_context(&self, input: Parameters<GetContextInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let model = self.model.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let context = model.get_context_for_path(&input.file_path);

        let json = serde_json::to_string_pretty(&context).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get derived analysis constraints
    #[tool(description = "Get the derived analysis constraints. These are automatically calculated paths that should receive priority attention based on the mental model.")]
    async fn get_constraints(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let model = self.model.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&model.constraints).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Add a finding with automatic context enrichment
    #[tool(description = "Add a finding from quality analysis. The finding will be automatically enriched with context from the mental model and severity will be adjusted.")]
    async fn add_finding(&self, input: Parameters<AddFindingInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let mut model = self.model.write().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        // Parse base severity
        let base_severity = match input.base_severity.to_uppercase().as_str() {
            "CRITICAL" => Severity::Critical,
            "HIGH" => Severity::High,
            "MEDIUM" => Severity::Medium,
            "LOW" => Severity::Low,
            _ => Severity::Info,
        };

        // Get context for the file
        let context = model.get_context_for_path(&input.file_path);

        // Calculate adjusted severity
        let adjusted_severity = adjust_severity(&base_severity, &context);

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
    #[tool(description = "Cluster findings into root causes. This analyzes patterns across findings to identify underlying issues.")]
    async fn synthesize(&self, input: Parameters<SynthesizeInput>) -> Result<CallToolResult, rmcp::ErrorData> {
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

        let root_causes = match input.algorithm.as_str() {
            "category_based" => synthesize_by_category(&findings),
            "location_based" => synthesize_by_location(&findings),
            _ => synthesize_by_category(&findings),
        };

        // Store root causes in model
        let mut model = self.model.write().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;
        model.root_causes = root_causes.clone();

        drop(model);
        // ADR-0006: End of audit - flush all pending changes
        self.mark_dirty();
        self.flush_if_dirty().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Flush error: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&root_causes).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Synthesized {} root causes from {} findings\n\n{}",
            root_causes.len(),
            root_causes.iter().map(|rc| rc.finding_count).sum::<u32>(),
            json
        ))]))
    }

    /// Get completed viewpoints
    #[tool(description = "Get list of viewpoints that have been completed.")]
    async fn get_completed_viewpoints(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let model = self.model.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&model.completed_viewpoints).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get available and missing artifacts for a commit
    #[tool(description = "List available and missing artifacts for a specific commit. Use 'HEAD' or 'latest' for current commit.")]
    async fn get_commit_artifacts(&self, input: Parameters<GetCommitArtifactsInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let commit = self.artifact_store.resolve_commit(&input.commit)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to resolve commit: {}", e), None))?;

        let (available, missing) = self.artifact_store.get_commit_artifacts(&commit)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to get artifacts: {}", e), None))?;

        let output = GetCommitArtifactsOutput {
            commit,
            available,
            missing,
        };

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Store an artifact for a commit
    #[tool(description = "Store a tool output artifact (SARIF, SCIP, coverage) for a specific commit. Data should be JSON for SARIF or base64 for binary.")]
    async fn store_artifact(&self, input: Parameters<StoreArtifactInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let commit = self.artifact_store.resolve_commit(&input.commit)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to resolve commit: {}", e), None))?;

        let metadata = StoreArtifactMetadata {
            producer: input.producer,
            produced_at: None,
        };

        let stored_at = self.artifact_store.store_artifact(
            &commit,
            &input.artifact_type,
            input.data.as_bytes(),
            &metadata,
        ).map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to store artifact: {}", e), None))?;

        let output = StoreArtifactOutput {
            stored_at,
            commit,
            artifact_type: input.artifact_type,
        };

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Retrieve an artifact for a commit
    #[tool(description = "Retrieve a stored artifact by commit and type. Returns the artifact data along with metadata.")]
    async fn get_artifact(&self, input: Parameters<GetArtifactInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let commit = self.artifact_store.resolve_commit(&input.commit)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to resolve commit: {}", e), None))?;

        let (data, info) = self.artifact_store.get_artifact(&commit, &input.artifact_type)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Artifact not found: {}", e), None))?;

        let data_str = String::from_utf8(data)
            .unwrap_or_else(|e| base64::Engine::encode(&base64::engine::general_purpose::STANDARD, e.into_bytes()));

        let output = GetArtifactOutput {
            data: data_str,
            commit,
            artifact_type: input.artifact_type,
            produced_at: info.produced_at.to_rfc3339(),
            producer: info.producer,
        };

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    // ========== Batch Operations (ADR-0005) ==========

    /// Add multiple findings in a single operation
    #[tool(description = "Add multiple findings from quality analysis in a single batch operation. Each finding will be automatically enriched with context and severity adjusted. More efficient than multiple add_finding calls. (ADR-0005)")]
    async fn add_findings(&self, input: Parameters<AddFindingsInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let mut model = self.model.write().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let mut added_findings = Vec::new();
        let mut viewpoints_touched = std::collections::HashSet::new();

        for finding_input in input.findings {
            // Parse base severity
            let base_severity = match finding_input.base_severity.to_uppercase().as_str() {
                "CRITICAL" => Severity::Critical,
                "HIGH" => Severity::High,
                "MEDIUM" => Severity::Medium,
                "LOW" => Severity::Low,
                _ => Severity::Info,
            };

            // Get context for the file
            let context = model.get_context_for_path(&finding_input.file_path);

            // Calculate adjusted severity
            let adjusted_severity = adjust_severity(&base_severity, &context);

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
    #[tool(description = "Get business context for multiple file paths in a single batch operation. Returns a map of file paths to their context (bounded context type, architecture layer, hotspot status). More efficient than multiple get_context calls. (ADR-0005)")]
    async fn get_contexts(&self, input: Parameters<GetContextsInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let model = self.model.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

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
    #[tool(description = "Get a specific section of the mental model instead of the full model. Sections: project, tech_stack, structure, build_deploy, module_hierarchy, architecture, domain_model, entity_model, interface_surface, hotspots, constraints, findings, root_causes, completed_viewpoints. More efficient than get_model when only one section is needed. (ADR-0005)")]
    async fn get_model_section(&self, input: Parameters<GetModelSectionInput>) -> Result<CallToolResult, rmcp::ErrorData> {
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

        let model = self.model.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

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
    #[tool(description = "Persist any pending changes to disk. Called automatically at phase boundaries (update_viewpoint, synthesize), but can be called explicitly for additional safety. (ADR-0006)")]
    async fn flush(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let flushed = self.flush_if_dirty().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Flush error: {}", e), None)
        })?;

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
    #[tool(description = "Get findings filtered by file path. Returns all findings for files matching the given path. (ADR-0007)")]
    async fn get_findings_by_file(&self, input: Parameters<GetFindingsByFileInput>) -> Result<CallToolResult, rmcp::ErrorData> {
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
            findings.len(), input.file_path, json
        ))]))
    }

    /// Get findings by severity level
    #[tool(description = "Get findings filtered by severity level (critical, high, medium, low, info). (ADR-0007)")]
    async fn get_findings_by_severity(&self, input: Parameters<GetFindingsBySeverityInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let severity = match input.severity.to_uppercase().as_str() {
            "CRITICAL" => Severity::Critical,
            "HIGH" => Severity::High,
            "MEDIUM" => Severity::Medium,
            "LOW" => Severity::Low,
            "INFO" => Severity::Info,
            _ => {
                return Err(rmcp::ErrorData::invalid_params(
                    format!("Unknown severity: '{}'. Valid values: critical, high, medium, low, info", input.severity),
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
            findings.len(), severity, json
        ))]))
    }

    /// Get findings by viewpoint
    #[tool(description = "Get findings filtered by viewpoint (e.g., VP-Q01, VP-Q02). (ADR-0007)")]
    async fn get_findings_by_viewpoint(&self, input: Parameters<GetFindingsByViewpointInput>) -> Result<CallToolResult, rmcp::ErrorData> {
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
            findings.len(), input.viewpoint, json
        ))]))
    }

    /// Get findings by category
    #[tool(description = "Get findings filtered by category (security, reliability, maintainability, performance, testability). (ADR-0007)")]
    async fn get_findings_by_category(&self, input: Parameters<GetFindingsByCategoryInput>) -> Result<CallToolResult, rmcp::ErrorData> {
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
            findings.len(), input.category, json
        ))]))
    }

    /// Get findings summary statistics
    #[tool(description = "Get summary statistics for all findings including counts by severity, category, and viewpoint. (ADR-0007)")]
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
    #[tool(description = "Export all findings to a JSON file. Useful for sharing or external processing. (ADR-0007)")]
    async fn export_findings(&self, input: Parameters<ExportFindingsInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let store = self.findings_store.lock().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Findings store lock error: {}", e), None)
        })?;

        let count = store.export_json(&input.output_path).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to export findings: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Exported {} findings to '{}'",
            count, input.output_path
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

    fn list_tools(
        &self,
        _pagination: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        async move {
            Ok(ListToolsResult {
                tools: self.tool_router.list_all(),
                next_cursor: None,
            })
        }
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

/// Adjust severity based on context
fn adjust_severity(base: &Severity, context: &FindingContext) -> Severity {
    let mut multiplier = 1.0;

    // Bounded context type adjustment
    if let Some(ref bc_type) = context.bounded_context_type {
        multiplier *= match bc_type {
            crate::model::BoundedContextType::Core => 1.5,
            crate::model::BoundedContextType::Supporting => 1.0,
            crate::model::BoundedContextType::Generic => 0.7,
        };
    }

    // Architecture layer adjustment
    if let Some(ref layer) = context.layer {
        multiplier *= match layer.to_lowercase().as_str() {
            "domain" => 1.3,
            "application" => 1.1,
            "adapters" | "adapter" => 1.0,
            "infrastructure" => 0.9,
            _ => 1.0,
        };
    }

    // Hotspot adjustment
    if context.is_hotspot {
        multiplier *= 1.4;
    }

    // Calculate new severity
    let new_score = base.score() * multiplier;
    Severity::from_score(new_score)
}

/// Synthesize findings by category
fn synthesize_by_category(findings: &[Finding]) -> Vec<RootCause> {
    let mut category_groups: HashMap<String, Vec<&Finding>> = HashMap::new();

    for finding in findings {
        category_groups
            .entry(finding.category.clone())
            .or_default()
            .push(finding);
    }

    let mut root_causes = Vec::new();

    for (category, findings) in category_groups {
        if findings.is_empty() {
            continue;
        }

        // Calculate aggregate impact
        let max_severity = findings
            .iter()
            .map(|f| &f.adjusted_severity)
            .max()
            .cloned()
            .unwrap_or(Severity::Info);

        // Collect affected areas
        let affected_areas: Vec<String> = findings
            .iter()
            .filter_map(|f| {
                f.context
                    .as_ref()
                    .and_then(|c| c.bounded_context.clone())
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Generate recommendations based on category
        let recommendations = generate_recommendations(&category, &findings);

        root_causes.push(RootCause {
            id: format!("RC-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            title: format!("{} Issues", capitalize_first(&category)),
            description: format!(
                "Multiple {} issues detected across the codebase affecting code quality and maintainability.",
                category
            ),
            category: category.clone(),
            impact: max_severity,
            finding_ids: findings.iter().map(|f| f.id.clone()).collect(),
            finding_count: findings.len() as u32,
            affected_areas,
            recommendations,
        });
    }

    // Sort by impact
    root_causes.sort_by(|a, b| b.impact.cmp(&a.impact));

    // Limit to top 5
    root_causes.truncate(5);

    root_causes
}

/// Synthesize findings by location
fn synthesize_by_location(findings: &[Finding]) -> Vec<RootCause> {
    let mut location_groups: HashMap<String, Vec<&Finding>> = HashMap::new();

    for finding in findings {
        // Group by directory
        let dir = std::path::Path::new(&finding.file_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| finding.file_path.clone());

        location_groups.entry(dir).or_default().push(finding);
    }

    let mut root_causes = Vec::new();

    for (location, findings) in location_groups {
        if findings.len() < 2 {
            continue; // Need at least 2 findings to form a root cause
        }

        let max_severity = findings
            .iter()
            .map(|f| &f.adjusted_severity)
            .max()
            .cloned()
            .unwrap_or(Severity::Info);

        let categories: Vec<String> = findings
            .iter()
            .map(|f| f.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        root_causes.push(RootCause {
            id: format!("RC-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            title: format!("Quality Issues in {}", location),
            description: format!(
                "Multiple quality issues concentrated in {}. Categories: {}",
                location,
                categories.join(", ")
            ),
            category: "location_cluster".to_string(),
            impact: max_severity,
            finding_ids: findings.iter().map(|f| f.id.clone()).collect(),
            finding_count: findings.len() as u32,
            affected_areas: vec![location],
            recommendations: vec![
                "Review and refactor this area for improved quality".to_string(),
                "Consider adding tests before refactoring".to_string(),
            ],
        });
    }

    root_causes.sort_by(|a, b| b.finding_count.cmp(&a.finding_count));
    root_causes.truncate(5);

    root_causes
}

fn generate_recommendations(category: &str, _findings: &[&Finding]) -> Vec<String> {
    match category.to_lowercase().as_str() {
        "security" => vec![
            "Conduct a focused security review of affected components".to_string(),
            "Implement input validation and sanitization".to_string(),
            "Review authentication and authorization patterns".to_string(),
        ],
        "reliability" => vec![
            "Add error handling and recovery mechanisms".to_string(),
            "Implement retry logic for external dependencies".to_string(),
            "Add monitoring and alerting for critical paths".to_string(),
        ],
        "maintainability" => vec![
            "Refactor complex code into smaller, focused functions".to_string(),
            "Improve code documentation and naming".to_string(),
            "Consider extracting reusable components".to_string(),
        ],
        "performance" => vec![
            "Profile and optimize critical paths".to_string(),
            "Review database queries for N+1 issues".to_string(),
            "Consider caching frequently accessed data".to_string(),
        ],
        "testability" => vec![
            "Increase test coverage for critical paths".to_string(),
            "Add integration tests for key workflows".to_string(),
            "Refactor tightly coupled code for better testability".to_string(),
        ],
        _ => vec![
            "Review affected code and apply best practices".to_string(),
            "Consider architectural improvements".to_string(),
        ],
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::BoundedContextType;

    #[test]
    fn test_adjust_severity_core_domain_hotspot() {
        let context = FindingContext {
            bounded_context: Some("Orders".to_string()),
            bounded_context_type: Some(BoundedContextType::Core),
            layer: Some("domain".to_string()),
            is_hotspot: true,
            hotspot_score: Some(80.0),
        };

        let adjusted = adjust_severity(&Severity::Medium, &context);
        // Medium (2.0) * Core (1.5) * Domain (1.3) * Hotspot (1.4) = 5.46 → Critical
        assert_eq!(adjusted, Severity::Critical);
    }

    #[test]
    fn test_adjust_severity_generic_infrastructure() {
        let context = FindingContext {
            bounded_context: Some("Utilities".to_string()),
            bounded_context_type: Some(BoundedContextType::Generic),
            layer: Some("infrastructure".to_string()),
            is_hotspot: false,
            hotspot_score: None,
        };

        let adjusted = adjust_severity(&Severity::Medium, &context);
        // Medium (2.0) * Generic (0.7) * Infrastructure (0.9) = 1.26 → Low
        assert_eq!(adjusted, Severity::Low);
    }
}

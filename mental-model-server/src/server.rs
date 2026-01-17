//! Mental Model MCP Server implementation
//!
//! This module implements the MCP server that manages the Mental Model,
//! providing tools for reading, updating, and querying the model.

use crate::model::{derive_constraints, Constraints, Finding, FindingContext, MentalModel, RootCause, Severity};
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
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Mental Model MCP Server
#[derive(Clone)]
pub struct MentalModelServer {
    model_path: PathBuf,
    model: Arc<RwLock<MentalModel>>,
    tool_router: ToolRouter<Self>,
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

        Self {
            model_path,
            model: Arc::new(RwLock::new(model)),
            tool_router: Self::tool_router(),
        }
    }

    fn save_model(&self) -> Result<()> {
        let model = self.model.read().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        let yaml = serde_yaml::to_string(&*model)?;
        fs::write(&self.model_path, yaml)?;
        Ok(())
    }

    /// Initialize a new mental model for a project
    #[tool(description = "Initialize a new mental model for a project. Call this before starting an audit.")]
    async fn init_model(&self, input: Parameters<InitModelInput>) -> Result<CallToolResult, rmcp::Error> {
        let input = input.0;
        let mut model = self.model.write().map_err(|e| {
            rmcp::Error::internal_error(format!("Lock error: {}", e), None)
        })?;

        *model = MentalModel::new(input.name, input.path);
        model.project.description = input.description;
        model.project.repository = input.repository;

        drop(model);
        self.save_model().map_err(|e| {
            rmcp::Error::internal_error(format!("Save error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(
            "Mental model initialized successfully",
        )]))
    }

    /// Get the current mental model state
    #[tool(description = "Get the current mental model state as YAML. Returns the complete model including all viewpoint data, constraints, and findings.")]
    async fn get_model(&self) -> Result<CallToolResult, rmcp::Error> {
        let model = self.model.read().map_err(|e| {
            rmcp::Error::internal_error(format!("Lock error: {}", e), None)
        })?;

        let yaml = serde_yaml::to_string(&*model).map_err(|e| {
            rmcp::Error::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(yaml)]))
    }

    /// Update the mental model with viewpoint results
    #[tool(description = "Update the mental model with results from a viewpoint analysis. This will also recalculate derived constraints.")]
    async fn update_viewpoint(&self, input: Parameters<UpdateViewpointInput>) -> Result<CallToolResult, rmcp::Error> {
        let input = input.0;
        let mut model = self.model.write().map_err(|e| {
            rmcp::Error::internal_error(format!("Lock error: {}", e), None)
        })?;

        model.apply_viewpoint(&input.viewpoint, input.data).map_err(|e| {
            rmcp::Error::internal_error(format!("Apply viewpoint error: {}", e), None)
        })?;

        // Recalculate constraints
        model.constraints = derive_constraints(&model);

        let constraints_json = serde_json::to_string_pretty(&model.constraints).map_err(|e| {
            rmcp::Error::internal_error(format!("Serialization error: {}", e), None)
        })?;

        drop(model);
        self.save_model().map_err(|e| {
            rmcp::Error::internal_error(format!("Save error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Viewpoint {} applied successfully.\n\nUpdated constraints:\n{}",
            input.viewpoint, constraints_json
        ))]))
    }

    /// Get business context for a file path
    #[tool(description = "Get business context for a specific file path. Returns bounded context type, architecture layer, and hotspot status.")]
    async fn get_context(&self, input: Parameters<GetContextInput>) -> Result<CallToolResult, rmcp::Error> {
        let input = input.0;
        let model = self.model.read().map_err(|e| {
            rmcp::Error::internal_error(format!("Lock error: {}", e), None)
        })?;

        let context = model.get_context_for_path(&input.file_path);

        let json = serde_json::to_string_pretty(&context).map_err(|e| {
            rmcp::Error::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get derived analysis constraints
    #[tool(description = "Get the derived analysis constraints. These are automatically calculated paths that should receive priority attention based on the mental model.")]
    async fn get_constraints(&self) -> Result<CallToolResult, rmcp::Error> {
        let model = self.model.read().map_err(|e| {
            rmcp::Error::internal_error(format!("Lock error: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&model.constraints).map_err(|e| {
            rmcp::Error::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Add a finding with automatic context enrichment
    #[tool(description = "Add a finding from quality analysis. The finding will be automatically enriched with context from the mental model and severity will be adjusted.")]
    async fn add_finding(&self, input: Parameters<AddFindingInput>) -> Result<CallToolResult, rmcp::Error> {
        let input = input.0;
        let mut model = self.model.write().map_err(|e| {
            rmcp::Error::internal_error(format!("Lock error: {}", e), None)
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
            rmcp::Error::internal_error(format!("Serialization error: {}", e), None)
        })?;

        model.findings.push(finding);

        // Mark viewpoint as having findings
        if !model.completed_viewpoints.contains(&input.viewpoint) {
            model.completed_viewpoints.push(input.viewpoint);
        }

        drop(model);
        self.save_model().map_err(|e| {
            rmcp::Error::internal_error(format!("Save error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Finding added with adjusted severity: {:?}\n\n{}",
            adjusted_severity, finding_json
        ))]))
    }

    /// Get all findings from the model
    #[tool(description = "Get all findings from quality analysis viewpoints.")]
    async fn get_findings(&self) -> Result<CallToolResult, rmcp::Error> {
        let model = self.model.read().map_err(|e| {
            rmcp::Error::internal_error(format!("Lock error: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&model.findings).map_err(|e| {
            rmcp::Error::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Total findings: {}\n\n{}",
            model.findings.len(),
            json
        ))]))
    }

    /// Synthesize findings into root causes
    #[tool(description = "Cluster findings into root causes. This analyzes patterns across findings to identify underlying issues.")]
    async fn synthesize(&self, input: Parameters<SynthesizeInput>) -> Result<CallToolResult, rmcp::Error> {
        let input = input.0;
        let mut model = self.model.write().map_err(|e| {
            rmcp::Error::internal_error(format!("Lock error: {}", e), None)
        })?;

        let root_causes = match input.algorithm.as_str() {
            "category_based" => synthesize_by_category(&model.findings),
            "location_based" => synthesize_by_location(&model.findings),
            _ => synthesize_by_category(&model.findings),
        };

        model.root_causes = root_causes.clone();

        drop(model);
        self.save_model().map_err(|e| {
            rmcp::Error::internal_error(format!("Save error: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&root_causes).map_err(|e| {
            rmcp::Error::internal_error(format!("Serialization error: {}", e), None)
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
    async fn get_completed_viewpoints(&self) -> Result<CallToolResult, rmcp::Error> {
        let model = self.model.read().map_err(|e| {
            rmcp::Error::internal_error(format!("Lock error: {}", e), None)
        })?;

        let json = serde_json::to_string_pretty(&model.completed_viewpoints).map_err(|e| {
            rmcp::Error::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
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

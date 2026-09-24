//! Methodology KB MCP Server implementation
//!
//! This module implements the MCP server that provides access to the
//! methodology knowledge base for interpreting metrics, classifying findings,
//! and checking compliance.

use crate::acquisition::{DataAcquisition, DataSource};
use crate::types::*;
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
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Methodology KB MCP Server
#[derive(Clone)]
pub struct MethodologyKBServer {
    #[allow(dead_code)]
    kb_path: PathBuf,
    kb: Arc<MethodologyKB>,
    acquisition: Arc<DataAcquisition>,
    tool_router: ToolRouter<Self>,
}

/// Input for lookup_metric tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LookupMetricInput {
    /// Metric name (e.g., "cognitive_complexity", "test_coverage")
    pub metric: String,
    /// Project type for threshold selection
    #[serde(default)]
    pub project_type: Option<String>,
}

/// Input for classify_finding tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClassifyFindingInput {
    /// Tool that generated the finding (e.g., "semgrep", "sonarqube")
    #[serde(default)]
    pub tool: Option<String>,
    /// Rule ID from the tool
    #[serde(default)]
    pub rule_id: Option<String>,
    /// Finding category (if already known)
    #[serde(default)]
    pub category: Option<String>,
    /// Base severity (if already known)
    #[serde(default)]
    pub base_severity: Option<String>,
    /// Context information for adjustment
    #[serde(default)]
    pub context: Option<FindingContextInput>,
}

/// Finding context for classification
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindingContextInput {
    /// Bounded context type (core, supporting, generic)
    #[serde(default)]
    pub bounded_context_type: Option<String>,
    /// Architecture layer
    #[serde(default)]
    pub layer: Option<String>,
    /// Whether this is in a hotspot
    #[serde(default)]
    pub is_hotspot: bool,
    /// File path of the finding (used by path_pattern rules)
    #[serde(default)]
    pub file_path: Option<String>,
    /// Business context tags (e.g. "payment", "auth", "public_api")
    #[serde(default)]
    pub business_context: Vec<String>,
    /// Number of callers of the affected symbol (e.g. from codegraph)
    #[serde(default)]
    pub caller_count: Option<u32>,
}

/// Input for get_thresholds tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetThresholdsInput {
    /// Project type (greenfield, mature, legacy, startup, enterprise)
    pub project_type: String,
    /// Programming language (optional)
    #[serde(default)]
    pub language: Option<String>,
}

/// Input for check_compliance tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CheckComplianceInput {
    /// Standard to check against (e.g., "clean_architecture", "layered")
    pub standard: String,
    /// Detected architecture pattern
    pub detected_pattern: serde_json::Value,
}

/// Input for get_category tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetCategoryInput {
    /// Category name (e.g., "security", "performance", "maintainability")
    pub category: String,
}

/// Input for get_template tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTemplateInput {
    /// Template type (e.g., "executive_summary", "root_cause")
    pub template_type: String,
    /// Output format (only "markdown" is currently available)
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "markdown".to_string()
}

/// Input for get_metric_data tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetMetricDataInput {
    /// Metric name to retrieve
    pub metric: String,
    /// Commit hash (default: HEAD)
    #[serde(default = "default_commit")]
    pub commit: String,
}

fn default_commit() -> String {
    "HEAD".to_string()
}

/// Input for get_acquisition_status tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetAcquisitionStatusInput {
    /// Commit hash (default: HEAD)
    #[serde(default = "default_commit")]
    pub commit: String,
}

fn parse_project_type(name: &str) -> Result<ProjectType, rmcp::ErrorData> {
    ProjectType::parse(name).ok_or_else(|| {
        rmcp::ErrorData::invalid_params(format!("Unknown project type: {}", name), None)
    })
}

/// Match a glob against a path or any of its trailing components, so that
/// "tests/*" matches both "tests/a.py" and "src/tests/a.py".
fn path_matches(pattern: &str, path: &str) -> bool {
    let Ok(pattern) = glob::Pattern::new(pattern) else {
        return false;
    };
    let path = path.replace('\\', "/");
    let path = path.trim_start_matches("./");
    std::iter::once(path)
        .chain(path.match_indices('/').map(|(i, _)| &path[i + 1..]))
        .any(|candidate| pattern.matches(candidate))
}

#[tool_router]
impl MethodologyKBServer {
    pub fn new(kb_path: PathBuf) -> Self {
        Self::with_project_path(kb_path, None)
    }

    /// Create a server for an explicit audited project (whose
    /// `.audit/artifacts` holds cached tool output). Defaults to the current
    /// working directory.
    pub fn with_project_path(kb_path: PathBuf, project_path: Option<PathBuf>) -> Self {
        let (kb, _errors) = Self::load_kb(&kb_path);

        let project_path = project_path
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            kb_path,
            kb: Arc::new(kb),
            acquisition: Arc::new(DataAcquisition::new(project_path)),
            tool_router: Self::tool_router(),
        }
    }

    /// Load the KB from disk.
    ///
    /// Each file is loaded independently: a file that fails to read or parse
    /// is skipped and its error is logged and returned, so one bad file does
    /// not discard the rest of the knowledge base.
    pub fn load_kb(kb_path: &Path) -> (MethodologyKB, Vec<String>) {
        let mut kb = MethodologyKB::default();
        let mut errors = Vec::new();

        fn load_yaml<T: serde::de::DeserializeOwned>(
            path: &Path,
            errors: &mut Vec<String>,
        ) -> Option<T> {
            if !path.exists() {
                return None;
            }
            let parsed = fs::read_to_string(path)
                .map_err(anyhow::Error::from)
                .and_then(|content| serde_yaml::from_str(&content).map_err(Into::into));
            match parsed {
                Ok(value) => Some(value),
                Err(e) => {
                    let msg = format!("{}: {}", path.display(), e);
                    tracing::error!("Failed to load KB file {}", msg);
                    errors.push(msg);
                    None
                }
            }
        }

        if let Some(metrics) = load_yaml(&kb_path.join("glossary/metrics.yaml"), &mut errors) {
            kb.metrics = metrics;
        }
        if let Some(thresholds) = load_yaml(
            &kb_path.join("thresholds/by_project_type.yaml"),
            &mut errors,
        ) {
            kb.thresholds = thresholds;
        }
        if let Some(categories) =
            load_yaml(&kb_path.join("taxonomies/categories.yaml"), &mut errors)
        {
            kb.categories = categories;
        }
        if let Some(adjustments) = load_yaml(
            &kb_path.join("taxonomies/severity_adjustment.yaml"),
            &mut errors,
        ) {
            kb.severity_adjustments = adjustments;
        }
        if let Some(mappings) =
            load_yaml(&kb_path.join("taxonomies/rule_mapping.yaml"), &mut errors)
        {
            kb.rule_mappings = mappings;
        }

        let pattern = |sub: &str| kb_path.join(sub).to_string_lossy().to_string();

        for path in glob::glob(&pattern("standards/*.yaml"))
            .into_iter()
            .flatten()
            .flatten()
        {
            if let Some(standard) = load_yaml::<ArchitectureStandard>(&path, &mut errors) {
                kb.standards.insert(standard.id.clone(), standard);
            }
        }

        for path in glob::glob(&pattern("templates/reports/*.md"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    errors.push(format!("{}: {}", path.display(), e));
                    continue;
                }
            };
            let id = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            kb.templates.insert(
                id.clone(),
                ReportTemplate {
                    id: id.clone(),
                    name: id.replace('_', " "),
                    format: "markdown".to_string(),
                    content,
                    sections: vec![],
                },
            );
        }

        (kb, errors)
    }

    /// Look up a metric definition and thresholds
    #[tool(
        description = "Look up a metric definition including description, thresholds, and interpretation guidance."
    )]
    async fn lookup_metric(
        &self,
        input: Parameters<LookupMetricInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let metric = self.kb.metrics.get(&input.metric).cloned();

        if let Some(mut metric) = metric {
            // Get project-specific thresholds if available
            if let Some(project_type_str) = input.project_type {
                let project_type = parse_project_type(&project_type_str)?;

                for threshold_set in &self.kb.thresholds {
                    if threshold_set.project_type == project_type {
                        if let Some(threshold) = threshold_set.metrics.get(&input.metric) {
                            metric
                                .thresholds
                                .insert("current".to_string(), threshold.clone());
                        }
                    }
                }
            }

            let json = serde_json::to_string_pretty(&metric).map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
            })?;

            Ok(CallToolResult::success(vec![Content::text(json)]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Metric '{}' not found in knowledge base. Available metrics: {}",
                input.metric,
                self.kb
                    .metrics
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))]))
        }
    }

    /// Classify a finding with context-aware severity adjustment
    #[tool(
        description = "Classify a finding and calculate adjusted severity based on context (bounded context type, layer, hotspot status)."
    )]
    async fn classify_finding(
        &self,
        input: Parameters<ClassifyFindingInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let mut category = input.category.clone();
        let mut base_severity = input.base_severity.clone();

        // Try to get category from rule mapping if not provided
        if category.is_none() || base_severity.is_none() {
            if let (Some(tool), Some(rule_id)) = (&input.tool, &input.rule_id) {
                for mapping in &self.kb.rule_mappings {
                    if mapping.tool == *tool && mapping.rule_id == *rule_id {
                        if category.is_none() {
                            category = Some(mapping.category.clone());
                        }
                        if base_severity.is_none() {
                            base_severity = mapping.base_severity.clone();
                        }
                        break;
                    }
                }
            }
        }

        let category = category.unwrap_or_else(|| "unknown".to_string());
        let base_severity = base_severity.unwrap_or_else(|| "MEDIUM".to_string());

        // Calculate adjusted severity
        let mut total_multiplier = 1.0;
        let mut adjustment_factors = Vec::new();

        if let Some(context) = &input.context {
            // Path rules overlap (e.g. "tests/*" and "*_test.py"), so only the
            // first matching one is applied to avoid compounding multipliers.
            let mut path_rule_applied = false;

            // Caller-count thresholds are tiers: apply only the highest one reached.
            let caller_tier = context.caller_count.and_then(|count| {
                self.kb
                    .severity_adjustments
                    .iter()
                    .filter_map(|rule| match rule.condition {
                        AdjustmentCondition::CallerCount { min_callers } => Some(min_callers),
                        _ => None,
                    })
                    .filter(|&min| count >= min)
                    .max()
            });

            // Apply severity adjustment rules
            for rule in &self.kb.severity_adjustments {
                let applies = match &rule.condition {
                    AdjustmentCondition::BoundedContextType { value } => {
                        context.bounded_context_type.as_ref() == Some(value)
                    }
                    AdjustmentCondition::LayerType { value } => {
                        context.layer.as_ref().map(|l| l.to_lowercase())
                            == Some(value.to_lowercase())
                    }
                    AdjustmentCondition::IsHotspot => context.is_hotspot,
                    AdjustmentCondition::CategoryMatch { category: cat } => {
                        category.to_lowercase() == cat.to_lowercase()
                    }
                    AdjustmentCondition::PathPattern { pattern } => {
                        let matched = !path_rule_applied
                            && context
                                .file_path
                                .as_deref()
                                .is_some_and(|path| path_matches(pattern, path));
                        path_rule_applied |= matched;
                        matched
                    }
                    AdjustmentCondition::BusinessContext { context: tag } => context
                        .business_context
                        .iter()
                        .any(|c| c.eq_ignore_ascii_case(tag)),
                    AdjustmentCondition::CallerCount { min_callers } => {
                        caller_tier == Some(*min_callers)
                    }
                };

                if applies {
                    total_multiplier *= rule.multiplier;
                    adjustment_factors.push(AdjustmentFactor {
                        name: format!("{:?}", rule.condition),
                        multiplier: rule.multiplier,
                        reason: rule.description.clone().unwrap_or_default(),
                    });
                }
            }
        }

        let base_score = match base_severity.to_uppercase().as_str() {
            "CRITICAL" => 4.0,
            "HIGH" => 3.0,
            "MEDIUM" => 2.0,
            "LOW" => 1.0,
            _ => 0.5,
        };

        let adjusted_score = base_score * total_multiplier;
        let adjusted_severity = if adjusted_score >= 3.5 {
            "CRITICAL"
        } else if adjusted_score >= 2.5 {
            "HIGH"
        } else if adjusted_score >= 1.5 {
            "MEDIUM"
        } else if adjusted_score >= 0.75 {
            "LOW"
        } else {
            "INFO"
        };

        let result = ClassificationResult {
            category,
            base_severity,
            adjusted_severity: adjusted_severity.to_string(),
            adjustment_factors,
            total_multiplier,
        };

        let json = serde_json::to_string_pretty(&result).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get thresholds for a project type
    #[tool(
        description = "Get all metric thresholds for a specific project type and optionally language."
    )]
    async fn get_thresholds(
        &self,
        input: Parameters<GetThresholdsInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let project_type = parse_project_type(&input.project_type)?;

        let mut result_thresholds: HashMap<String, ThresholdValue> = HashMap::new();

        for threshold_set in &self.kb.thresholds {
            if threshold_set.project_type == project_type {
                // Check language match if specified
                if let Some(ref lang) = input.language {
                    if let Some(ref set_lang) = threshold_set.language {
                        if set_lang.to_lowercase() != lang.to_lowercase() {
                            continue;
                        }
                    }
                }
                result_thresholds.extend(threshold_set.metrics.clone());
            }
        }

        if result_thresholds.is_empty() {
            // Fall back to default thresholds
            for threshold_set in &self.kb.thresholds {
                if threshold_set.project_type == ProjectType::Mature
                    && threshold_set.language.is_none()
                {
                    result_thresholds.extend(threshold_set.metrics.clone());
                    break;
                }
            }
        }

        let json = serde_json::to_string_pretty(&result_thresholds).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Thresholds for {} project type:\n\n{}",
            input.project_type, json
        ))]))
    }

    /// Check compliance against an architecture standard
    #[tool(
        description = "Check if a detected architecture pattern complies with a standard (clean_architecture, layered, hexagonal)."
    )]
    async fn check_compliance(
        &self,
        input: Parameters<CheckComplianceInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let standard = self.kb.standards.get(&input.standard);

        if let Some(standard) = standard {
            let mut violations = Vec::new();
            let mut score: f64 = 100.0;

            // Parse detected pattern
            let detected_layers: Vec<String> = input
                .detected_pattern
                .get("layers")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default();

            // Check for required layers
            for required_layer in &standard.layers {
                let found = detected_layers.iter().any(|l| {
                    l.to_lowercase() == required_layer.name.to_lowercase()
                        || required_layer
                            .aliases
                            .iter()
                            .any(|a| a.to_lowercase() == l.to_lowercase())
                });

                if !found {
                    violations.push(ComplianceViolation {
                        rule: format!("Required layer: {}", required_layer.name),
                        description: format!(
                            "Missing {} layer. Purpose: {}",
                            required_layer.name, required_layer.purpose
                        ),
                        severity: "MEDIUM".to_string(),
                        location: None,
                    });
                    score -= 15.0;
                }
            }

            // Check dependency rules
            if let Some(detected_violations) = input
                .detected_pattern
                .get("violations")
                .and_then(|v| v.as_array())
            {
                for violation in detected_violations {
                    let from = violation
                        .get("from_layer")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let to = violation
                        .get("to_layer")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");

                    violations.push(ComplianceViolation {
                        rule: format!("Dependency rule: {} -> {}", from, to),
                        description: format!("Invalid dependency from {} to {}", from, to),
                        severity: "HIGH".to_string(),
                        location: violation
                            .get("file_path")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    });
                    score -= 10.0;
                }
            }

            let result = ComplianceResult {
                standard: standard.name.clone(),
                compliant: violations.is_empty(),
                score: score.max(0.0),
                violations,
                recommendations: if score < 70.0 {
                    vec![
                        "Review and enforce layer boundaries".to_string(),
                        "Consider using dependency injection to invert problematic dependencies"
                            .to_string(),
                        "Add architectural fitness functions to CI pipeline".to_string(),
                    ]
                } else {
                    vec![]
                },
            };

            let json = serde_json::to_string_pretty(&result).map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
            })?;

            Ok(CallToolResult::success(vec![Content::text(json)]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Standard '{}' not found. Available standards: {}",
                input.standard,
                self.kb
                    .standards
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))]))
        }
    }

    /// Get a report template
    #[tool(
        description = "Get a report template for generating audit outputs (executive_summary, root_cause, technical_details)."
    )]
    async fn get_template(
        &self,
        input: Parameters<GetTemplateInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        if !input.format.eq_ignore_ascii_case("markdown") {
            return Err(rmcp::ErrorData::invalid_params(
                format!(
                    "Unsupported template format: {} (only markdown is available)",
                    input.format
                ),
                None,
            ));
        }

        if let Some(template) = self.kb.templates.get(&input.template_type) {
            return Ok(CallToolResult::success(vec![Content::text(
                template.content.clone(),
            )]));
        }

        // Built-in fallbacks when the KB has no such template
        let builtin = match input.template_type.as_str() {
            "executive_summary" => include_str!("../templates/executive_summary.md"),
            "root_cause" => include_str!("../templates/root_cause.md"),
            _ => {
                let mut available: Vec<&str> =
                    self.kb.templates.keys().map(|k| k.as_str()).collect();
                for name in ["executive_summary", "root_cause"] {
                    if !available.contains(&name) {
                        available.push(name);
                    }
                }
                available.sort();
                return Err(rmcp::ErrorData::invalid_params(
                    format!(
                        "Template not found: {}. Available: {}",
                        input.template_type,
                        available.join(", ")
                    ),
                    None,
                ));
            }
        };

        Ok(CallToolResult::success(vec![Content::text(builtin)]))
    }

    /// List available metrics
    #[tool(description = "List all available metrics in the knowledge base.")]
    async fn list_metrics(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let metrics: Vec<&str> = self.kb.metrics.keys().map(|s| s.as_str()).collect();
        let json = serde_json::to_string_pretty(&metrics).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// List available standards
    #[tool(description = "List all available architecture standards.")]
    async fn list_standards(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let standards: Vec<(&String, &String)> = self
            .kb
            .standards
            .iter()
            .map(|(id, s)| (id, &s.name))
            .collect();
        let json = serde_json::to_string_pretty(&standards).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get category information
    #[tool(description = "Get detailed information about a finding category.")]
    async fn get_category(
        &self,
        input: Parameters<GetCategoryInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let category = input.0.category;
        let cat = self.kb.categories.get(&category);

        if let Some(cat) = cat {
            let json = serde_json::to_string_pretty(&cat).map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
            })?;
            Ok(CallToolResult::success(vec![Content::text(json)]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Category '{}' not found. Available categories: {}",
                category,
                self.kb
                    .categories
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))]))
        }
    }

    // =========== Data Acquisition Tools ===========

    /// Get metric data with automatic cascade
    #[tool(
        description = "Get metric data for a commit. Checks artifact cache first, returns data with provenance information. If data unavailable, indicates which tool to run."
    )]
    async fn get_metric_data(
        &self,
        input: Parameters<GetMetricDataInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let data = self
            .acquisition
            .get_metric_data(&input.commit, &input.metric)
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Acquisition error: {}", e), None)
            })?;

        let json = serde_json::to_string_pretty(&data).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        let status = match data.source {
            DataSource::Cache => "from cache",
            DataSource::Fresh => "freshly acquired",
            DataSource::Derived => "derived from other metrics",
            DataSource::Unavailable => "unavailable - tool execution needed",
        };

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Metric '{}' [{}]:\n\n{}",
            input.metric, status, json
        ))]))
    }

    /// Get acquisition status for a commit
    #[tool(
        description = "Get the status of data acquisition for a commit. Shows which metrics are available, which are missing, and which tools need to be run."
    )]
    async fn get_acquisition_status(
        &self,
        input: Parameters<GetAcquisitionStatusInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let status = self
            .acquisition
            .get_acquisition_status(&input.commit)
            .map_err(|e| {
                rmcp::ErrorData::internal_error(format!("Acquisition error: {}", e), None)
            })?;

        let json = serde_json::to_string_pretty(&status).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Acquisition status for commit {}:\n\nAvailable: {}\nMissing: {}\nTools needed: {}\n\n{}",
            status.commit,
            status.available_metrics.len(),
            status.missing_metrics.len(),
            status.tools_needed.join(", "),
            json
        ))]))
    }

    /// List acquirable metrics
    #[tool(
        description = "List all metrics that can be acquired through tool execution and caching."
    )]
    async fn list_acquirable_metrics(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let metrics = self.acquisition.list_available_metrics();

        let json = serde_json::to_string_pretty(&metrics).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Acquirable metrics ({}):\n\n{}",
            metrics.len(),
            json
        ))]))
    }
}

impl ServerHandler for MethodologyKBServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: rmcp::model::Implementation {
                name: "methodology-kb".into(),
                version: "0.1.0".into(),
            },
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some("Methodology Knowledge Base MCP Server for AI Code Audit. Use this server to look up metrics, classify findings, check compliance, and get report templates.".into()),
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

#[cfg(test)]
mod path_tests {
    use super::path_matches;

    #[test]
    fn matches_directory_patterns_at_any_depth() {
        assert!(path_matches("tests/*", "tests/test_a.py"));
        assert!(path_matches("tests/*", "src/tests/test_a.py"));
        assert!(path_matches("tests/*", "./tests/a.py"));
        assert!(!path_matches("tests/*", "src/contests.py"));
    }

    #[test]
    fn matches_file_name_patterns() {
        assert!(path_matches("*_test.py", "pkg/foo_test.py"));
        assert!(path_matches("*.spec.ts", "src/app/app.spec.ts"));
        assert!(!path_matches("*.spec.ts", "src/app/app.ts"));
    }

    #[test]
    fn project_types_parse_case_insensitively() {
        use crate::types::ProjectType;
        assert_eq!(
            ProjectType::parse("Python_Backend"),
            Some(ProjectType::PythonBackend)
        );
        assert_eq!(ProjectType::parse("bogus"), None);
    }
}

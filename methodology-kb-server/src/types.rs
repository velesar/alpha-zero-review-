//! Methodology Knowledge Base data types
//!
//! This module defines the data structures for the methodology KB,
//! including metrics, thresholds, taxonomies, and standards.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metric definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MetricDefinition {
    pub name: String,
    pub description: String,
    pub category: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub direction: MetricDirection,
    #[serde(default)]
    pub thresholds: HashMap<String, ThresholdValue>,
}

/// Direction indicating whether higher or lower values are better
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MetricDirection {
    #[default]
    LowerIsBetter,
    HigherIsBetter,
}

/// Threshold value with ranges
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThresholdValue {
    #[serde(default)]
    pub healthy: Option<f64>,
    #[serde(default)]
    pub warning: Option<f64>,
    #[serde(default)]
    pub critical: Option<f64>,
}

/// Project type for threshold selection
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Greenfield,
    Mature,
    Legacy,
    Startup,
    Enterprise,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Mature
    }
}

/// Threshold set for a specific project type and language
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThresholdSet {
    pub project_type: ProjectType,
    #[serde(default)]
    pub language: Option<String>,
    pub metrics: HashMap<String, ThresholdValue>,
}

/// Finding category definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CategoryDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub weight: f64,
}

/// Severity adjustment rule
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SeverityAdjustmentRule {
    pub condition: AdjustmentCondition,
    pub multiplier: f64,
    #[serde(default)]
    pub description: Option<String>,
}

/// Condition for severity adjustment
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AdjustmentCondition {
    BoundedContextType { value: String },
    LayerType { value: String },
    IsHotspot,
    CategoryMatch { category: String },
}

/// Rule mapping from tool rules to categories
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuleMapping {
    pub tool: String,
    pub rule_id: String,
    pub category: String,
    #[serde(default)]
    pub base_severity: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Architecture standard definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArchitectureStandard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub layers: Vec<StandardLayer>,
    #[serde(default)]
    pub dependency_rules: Vec<DependencyRule>,
}

/// Layer definition in a standard
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StandardLayer {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub typical_paths: Vec<String>,
    pub purpose: String,
    #[serde(default)]
    pub allowed_dependencies: Vec<String>,
}

/// Dependency rule for architecture compliance
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DependencyRule {
    pub from: String,
    pub to: String,
    pub allowed: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceResult {
    pub standard: String,
    pub compliant: bool,
    pub score: f64,
    #[serde(default)]
    pub violations: Vec<ComplianceViolation>,
    #[serde(default)]
    pub recommendations: Vec<String>,
}

/// Compliance violation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceViolation {
    pub rule: String,
    pub description: String,
    pub severity: String,
    #[serde(default)]
    pub location: Option<String>,
}

/// Report template
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReportTemplate {
    pub id: String,
    pub name: String,
    pub format: String,
    pub content: String,
    #[serde(default)]
    pub sections: Vec<TemplateSection>,
}

/// Template section
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TemplateSection {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub required: bool,
}

/// Finding classification result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClassificationResult {
    pub category: String,
    pub base_severity: String,
    pub adjusted_severity: String,
    pub adjustment_factors: Vec<AdjustmentFactor>,
    pub total_multiplier: f64,
}

/// Factor that contributed to severity adjustment
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AdjustmentFactor {
    pub name: String,
    pub multiplier: f64,
    pub reason: String,
}

/// OWASP category definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OwaspCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cwe_ids: Vec<u32>,
    #[serde(default)]
    pub examples: Vec<String>,
    #[serde(default)]
    pub mitigations: Vec<String>,
}

/// Glossary entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GlossaryEntry {
    pub term: String,
    pub definition: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub related_terms: Vec<String>,
    #[serde(default)]
    pub examples: Vec<String>,
}

/// Complete methodology KB
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MethodologyKB {
    #[serde(default)]
    pub metrics: HashMap<String, MetricDefinition>,
    #[serde(default)]
    pub thresholds: Vec<ThresholdSet>,
    #[serde(default)]
    pub categories: HashMap<String, CategoryDefinition>,
    #[serde(default)]
    pub severity_adjustments: Vec<SeverityAdjustmentRule>,
    #[serde(default)]
    pub rule_mappings: Vec<RuleMapping>,
    #[serde(default)]
    pub standards: HashMap<String, ArchitectureStandard>,
    #[serde(default)]
    pub owasp: HashMap<String, OwaspCategory>,
    #[serde(default)]
    pub templates: HashMap<String, ReportTemplate>,
    #[serde(default)]
    pub glossary: HashMap<String, GlossaryEntry>,
}

//! Mental Model data types and schema definitions
//!
//! This module defines the core data structures for the Mental Model,
//! which is the central artifact of the AI Code Audit system.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Confidence level for analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    #[default]
    Medium,
    Low,
}

/// Risk level for hotspots and findings
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum Risk {
    Critical,
    High,
    Medium,
    Low,
}

/// Severity level for findings
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    /// Get numeric score for severity calculation
    pub fn score(&self) -> f64 {
        match self {
            Severity::Critical => 4.0,
            Severity::High => 3.0,
            Severity::Medium => 2.0,
            Severity::Low => 1.0,
            Severity::Info => 0.5,
        }
    }

    /// Calculate severity from numeric score
    pub fn from_score(score: f64) -> Self {
        if score >= 3.5 {
            Severity::Critical
        } else if score >= 2.5 {
            Severity::High
        } else if score >= 1.5 {
            Severity::Medium
        } else if score >= 0.75 {
            Severity::Low
        } else {
            Severity::Info
        }
    }
}

/// Bounded context type in DDD terms
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BoundedContextType {
    Core,
    Supporting,
    Generic,
}

/// Root layout type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RootLayout {
    Monorepo,
    SingleApp,
    MultiPackage,
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub analyzed_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

/// Technology stack information (VP-F01)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct TechStack {
    pub primary_language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub framework_version: Option<String>,
    pub confidence: Confidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_languages: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<Dependency>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Dependency {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Project structure information (VP-F02)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Structure {
    pub root_layout: Option<RootLayout>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_points: Option<Vec<String>>,
}

/// Build and deployment information (VP-F03)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct BuildDeploy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ci_cd: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deployment_targets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containerization: Option<String>,
}

/// Architecture layer
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Layer {
    pub name: String,
    pub paths: Vec<String>,
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_dependencies: Vec<String>,
}

/// Architecture violation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Violation {
    pub from_layer: String,
    pub to_layer: String,
    pub file_path: String,
    pub import_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u32>,
    pub description: String,
}

/// Architecture information (VP-S02)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Architecture {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    pub confidence: Option<Confidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layers: Vec<Layer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<Violation>,
}

/// Bounded context in domain model
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BoundedContext {
    pub name: String,
    #[serde(rename = "type")]
    pub context_type: BoundedContextType,
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Domain model information (VP-S03)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct DomainModel {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bounded_contexts: Vec<BoundedContext>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub aggregates: HashMap<String, Vec<String>>,
}

/// Module in hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Module {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub submodules: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines_of_code: Option<u32>,
}

/// Module hierarchy (VP-S01)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ModuleHierarchy {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modules: Vec<Module>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub circular_dependencies: Vec<Vec<String>>,
}

/// Entity information (VP-S04)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Entity {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<EntityRelationship>,
}

/// Entity relationship
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityRelationship {
    pub target: String,
    pub relationship_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cardinality: Option<String>,
}

/// Entity model (VP-S04)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct EntityModel {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<Entity>,
}

/// API endpoint
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiEndpoint {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<String>,
}

/// Interface surface (VP-S05)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct InterfaceSurface {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub api_endpoints: Vec<ApiEndpoint>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub public_modules: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exported_types: Vec<String>,
}

/// Hotspot file
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Hotspot {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub churn: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<f64>,
    pub score: f64,
    pub risk: Risk,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

/// Hotspots analysis (VP-S06)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Hotspots {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<Hotspot>,
}

/// Finding from quality analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Finding {
    pub id: String,
    pub viewpoint: String,
    pub category: String,
    pub title: String,
    pub description: String,
    pub file_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u32>,
    pub base_severity: Severity,
    pub adjusted_severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<FindingContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommendation: Option<String>,
}

/// Context information for a finding
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FindingContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounded_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounded_context_type: Option<BoundedContextType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,
    pub is_hotspot: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hotspot_score: Option<f64>,
}

/// Root cause from synthesis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RootCause {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub impact: Severity,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finding_ids: Vec<String>,
    pub finding_count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_areas: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recommendations: Vec<String>,
}

/// Derived constraints for analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct Constraints {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub high_priority_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security_focus_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coverage_critical_paths: Vec<String>,
}

/// Complete Mental Model
/// Note: Findings are stored separately in SQLite (ADR-0007)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MentalModel {
    pub version: String,
    pub project: Project,
    #[serde(default)]
    pub tech_stack: TechStack,
    #[serde(default)]
    pub structure: Structure,
    #[serde(default)]
    pub build_deploy: BuildDeploy,
    #[serde(default)]
    pub module_hierarchy: ModuleHierarchy,
    #[serde(default)]
    pub architecture: Architecture,
    #[serde(default)]
    pub domain_model: DomainModel,
    #[serde(default)]
    pub entity_model: EntityModel,
    #[serde(default)]
    pub interface_surface: InterfaceSurface,
    #[serde(default)]
    pub hotspots: Hotspots,
    #[serde(default)]
    pub constraints: Constraints,
    // ADR-0007: Findings moved to separate SQLite store (findings_store.rs)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub root_causes: Vec<RootCause>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub completed_viewpoints: Vec<String>,
}

impl Default for MentalModel {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            project: Project {
                name: String::new(),
                path: String::new(),
                analyzed_at: Utc::now(),
                description: None,
                repository: None,
            },
            tech_stack: TechStack::default(),
            structure: Structure::default(),
            build_deploy: BuildDeploy::default(),
            module_hierarchy: ModuleHierarchy::default(),
            architecture: Architecture::default(),
            domain_model: DomainModel::default(),
            entity_model: EntityModel::default(),
            interface_surface: InterfaceSurface::default(),
            hotspots: Hotspots::default(),
            constraints: Constraints::default(),
            // ADR-0007: findings stored in separate SQLite store
            root_causes: Vec::new(),
            completed_viewpoints: Vec::new(),
        }
    }
}

impl MentalModel {
    /// Create a new mental model for a project
    pub fn new(name: String, path: String) -> Self {
        Self {
            project: Project {
                name,
                path,
                analyzed_at: Utc::now(),
                description: None,
                repository: None,
            },
            ..Default::default()
        }
    }

    /// Get context for a specific file path
    pub fn get_context_for_path(&self, file_path: &str) -> FindingContext {
        let mut context = FindingContext {
            bounded_context: None,
            bounded_context_type: None,
            layer: None,
            is_hotspot: false,
            hotspot_score: None,
        };

        // Find bounded context
        for bc in &self.domain_model.bounded_contexts {
            if bc.paths.iter().any(|p| file_path.starts_with(p)) {
                context.bounded_context = Some(bc.name.clone());
                context.bounded_context_type = Some(bc.context_type.clone());
                break;
            }
        }

        // Find layer
        for layer in &self.architecture.layers {
            if layer.paths.iter().any(|p| file_path.starts_with(p)) {
                context.layer = Some(layer.name.clone());
                break;
            }
        }

        // Check if hotspot
        for hotspot in &self.hotspots.files {
            if hotspot.path == file_path {
                context.is_hotspot = true;
                context.hotspot_score = Some(hotspot.score);
                break;
            }
        }

        context
    }

    /// Apply viewpoint data to the model
    pub fn apply_viewpoint(&mut self, viewpoint: &str, data: serde_json::Value) -> anyhow::Result<()> {
        match viewpoint {
            "VP-F01" => {
                if let Ok(tech_stack) = serde_json::from_value(data) {
                    self.tech_stack = tech_stack;
                }
            }
            "VP-F02" => {
                if let Ok(structure) = serde_json::from_value(data) {
                    self.structure = structure;
                }
            }
            "VP-F03" => {
                if let Ok(build_deploy) = serde_json::from_value(data) {
                    self.build_deploy = build_deploy;
                }
            }
            "VP-S01" => {
                if let Ok(module_hierarchy) = serde_json::from_value(data) {
                    self.module_hierarchy = module_hierarchy;
                }
            }
            "VP-S02" => {
                if let Ok(architecture) = serde_json::from_value(data) {
                    self.architecture = architecture;
                }
            }
            "VP-S03" => {
                if let Ok(domain_model) = serde_json::from_value(data) {
                    self.domain_model = domain_model;
                }
            }
            "VP-S04" => {
                if let Ok(entity_model) = serde_json::from_value(data) {
                    self.entity_model = entity_model;
                }
            }
            "VP-S05" => {
                if let Ok(interface_surface) = serde_json::from_value(data) {
                    self.interface_surface = interface_surface;
                }
            }
            "VP-S06" => {
                if let Ok(hotspots) = serde_json::from_value(data) {
                    self.hotspots = hotspots;
                }
            }
            _ => {
                // For quality viewpoints, findings are added via add_finding
                tracing::debug!("Viewpoint {} uses findings-based updates", viewpoint);
            }
        }

        // Mark viewpoint as completed
        if !self.completed_viewpoints.contains(&viewpoint.to_string()) {
            self.completed_viewpoints.push(viewpoint.to_string());
        }

        Ok(())
    }
}

/// Derive constraints from the current model state
pub fn derive_constraints(model: &MentalModel) -> Constraints {
    let mut high_priority = Vec::new();
    let mut security_focus = Vec::new();

    // Core bounded contexts → high priority
    for bc in &model.domain_model.bounded_contexts {
        if bc.context_type == BoundedContextType::Core {
            high_priority.extend(bc.paths.clone());
        }
    }

    // Hotspots → high priority
    for hotspot in &model.hotspots.files {
        if matches!(hotspot.risk, Risk::Critical | Risk::High)
            && !high_priority.contains(&hotspot.path) {
                high_priority.push(hotspot.path.clone());
            }
    }

    // Domain layer → security focus
    for layer in &model.architecture.layers {
        let layer_name_lower = layer.name.to_lowercase();
        if layer_name_lower == "domain" {
            security_focus.extend(layer.paths.clone());
        }
        if layer_name_lower.contains("adapter") || layer_name_lower.contains("api") {
            security_focus.extend(layer.paths.clone());
        }
    }

    // Deduplicate
    high_priority.sort();
    high_priority.dedup();
    security_focus.sort();
    security_focus.dedup();

    Constraints {
        high_priority_paths: high_priority.clone(),
        security_focus_paths: security_focus,
        coverage_critical_paths: high_priority,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_score() {
        assert_eq!(Severity::Critical.score(), 4.0);
        assert_eq!(Severity::High.score(), 3.0);
        assert_eq!(Severity::Medium.score(), 2.0);
        assert_eq!(Severity::Low.score(), 1.0);
        assert_eq!(Severity::Info.score(), 0.5);
    }

    #[test]
    fn test_severity_from_score() {
        assert_eq!(Severity::from_score(4.0), Severity::Critical);
        assert_eq!(Severity::from_score(3.0), Severity::High);
        assert_eq!(Severity::from_score(2.0), Severity::Medium);
        assert_eq!(Severity::from_score(1.0), Severity::Low);
        assert_eq!(Severity::from_score(0.5), Severity::Info);
    }

    #[test]
    fn test_derive_constraints() {
        let mut model = MentalModel::default();

        // Add a core bounded context
        model.domain_model.bounded_contexts.push(BoundedContext {
            name: "Orders".to_string(),
            context_type: BoundedContextType::Core,
            paths: vec!["src/orders".to_string()],
            entities: vec![],
            description: None,
        });

        // Add a critical hotspot
        model.hotspots.files.push(Hotspot {
            path: "src/auth/handler.rs".to_string(),
            churn: Some(50),
            complexity: Some(30.0),
            score: 85.0,
            risk: Risk::Critical,
            reasons: vec![],
        });

        // Add a domain layer
        model.architecture.layers.push(Layer {
            name: "domain".to_string(),
            paths: vec!["src/domain".to_string()],
            purpose: "Business logic".to_string(),
            allowed_dependencies: vec![],
        });

        let constraints = derive_constraints(&model);

        assert!(constraints.high_priority_paths.contains(&"src/orders".to_string()));
        assert!(constraints.high_priority_paths.contains(&"src/auth/handler.rs".to_string()));
        assert!(constraints.security_focus_paths.contains(&"src/domain".to_string()));
    }
}

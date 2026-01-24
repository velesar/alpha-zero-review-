//! Integration tests for Methodology KB Server
//!
//! These tests verify the core functionality of the methodology KB
//! including metrics, thresholds, classifications, and compliance checking.

use methodology_kb_server::types::{
    CategoryDefinition, MetricDefinition, MetricDirection, MethodologyKB,
    ProjectType, ThresholdValue, ThresholdSet, ArchitectureStandard,
    StandardLayer, DependencyRule,
};
use std::collections::HashMap;

#[test]
fn test_metric_definition() {
    let metric = MetricDefinition {
        name: "Cyclomatic Complexity".to_string(),
        description: "Measures the complexity of code".to_string(),
        category: "complexity".to_string(),
        unit: Some("per function".to_string()),
        direction: MetricDirection::LowerIsBetter,
        thresholds: HashMap::new(),
    };

    assert_eq!(metric.name, "Cyclomatic Complexity");
    assert_eq!(metric.category, "complexity");
}

#[test]
fn test_metric_direction_default() {
    let direction = MetricDirection::default();
    matches!(direction, MetricDirection::LowerIsBetter);
}

#[test]
fn test_threshold_value() {
    let threshold = ThresholdValue {
        healthy: Some(10.0),
        warning: Some(20.0),
        critical: Some(30.0),
    };

    assert_eq!(threshold.healthy, Some(10.0));
    assert_eq!(threshold.warning, Some(20.0));
    assert_eq!(threshold.critical, Some(30.0));
}

#[test]
fn test_project_type_default() {
    let project_type = ProjectType::default();
    assert_eq!(project_type, ProjectType::Mature);
}

#[test]
fn test_threshold_set() {
    let mut metrics = HashMap::new();
    metrics.insert("complexity".to_string(), ThresholdValue {
        healthy: Some(5.0),
        warning: Some(10.0),
        critical: Some(20.0),
    });

    let threshold_set = ThresholdSet {
        project_type: ProjectType::Greenfield,
        language: Some("rust".to_string()),
        metrics,
    };

    assert_eq!(threshold_set.project_type, ProjectType::Greenfield);
    assert_eq!(threshold_set.language, Some("rust".to_string()));
    assert!(threshold_set.metrics.contains_key("complexity"));
}

#[test]
fn test_category_definition() {
    let category = CategoryDefinition {
        id: "sec-injection".to_string(),
        name: "Injection Vulnerabilities".to_string(),
        description: "SQL, Command, and other injection vulnerabilities".to_string(),
        parent: Some("security".to_string()),
        weight: 1.5,
    };

    assert_eq!(category.id, "sec-injection");
    assert_eq!(category.parent, Some("security".to_string()));
    assert_eq!(category.weight, 1.5);
}

#[test]
fn test_architecture_standard() {
    let standard = ArchitectureStandard {
        id: "hexagonal".to_string(),
        name: "Hexagonal Architecture".to_string(),
        description: "Ports and adapters pattern".to_string(),
        layers: vec![
            StandardLayer {
                name: "domain".to_string(),
                aliases: vec!["core".to_string()],
                typical_paths: vec!["src/domain".to_string()],
                purpose: "Business logic".to_string(),
                allowed_dependencies: vec![],
            },
            StandardLayer {
                name: "application".to_string(),
                aliases: vec!["use_cases".to_string()],
                typical_paths: vec!["src/application".to_string()],
                purpose: "Use case orchestration".to_string(),
                allowed_dependencies: vec!["domain".to_string()],
            },
            StandardLayer {
                name: "adapter".to_string(),
                aliases: vec!["infrastructure".to_string()],
                typical_paths: vec!["src/adapters".to_string()],
                purpose: "External interfaces".to_string(),
                allowed_dependencies: vec!["application".to_string(), "domain".to_string()],
            },
        ],
        dependency_rules: vec![
            DependencyRule {
                from: "domain".to_string(),
                to: "adapter".to_string(),
                allowed: false,
                reason: Some("Domain should not depend on adapters".to_string()),
            },
        ],
    };

    assert_eq!(standard.id, "hexagonal");
    assert_eq!(standard.layers.len(), 3);
    assert_eq!(standard.dependency_rules.len(), 1);
    assert!(!standard.dependency_rules[0].allowed);
}

#[test]
fn test_methodology_kb_creation() {
    let kb = MethodologyKB::default();

    assert!(kb.metrics.is_empty());
    assert!(kb.thresholds.is_empty());
    assert!(kb.categories.is_empty());
    assert!(kb.standards.is_empty());
}

#[test]
fn test_methodology_kb_with_data() {
    let mut kb = MethodologyKB::default();

    // Add a metric
    kb.metrics.insert("loc".to_string(), MetricDefinition {
        name: "Lines of Code".to_string(),
        description: "Total lines of code".to_string(),
        category: "size".to_string(),
        unit: Some("lines".to_string()),
        direction: MetricDirection::LowerIsBetter,
        thresholds: HashMap::new(),
    });

    // Add a category
    kb.categories.insert("maintainability".to_string(), CategoryDefinition {
        id: "maintainability".to_string(),
        name: "Maintainability".to_string(),
        description: "Code maintainability issues".to_string(),
        parent: None,
        weight: 1.0,
    });

    // Add a threshold set
    let mut thresholds = HashMap::new();
    thresholds.insert("loc".to_string(), ThresholdValue {
        healthy: Some(500.0),
        warning: Some(1000.0),
        critical: Some(2000.0),
    });
    kb.thresholds.push(ThresholdSet {
        project_type: ProjectType::Mature,
        language: None,
        metrics: thresholds,
    });

    assert_eq!(kb.metrics.len(), 1);
    assert_eq!(kb.categories.len(), 1);
    assert_eq!(kb.thresholds.len(), 1);
}

#[test]
fn test_project_type_variants() {
    let types = [
        ProjectType::Greenfield,
        ProjectType::Mature,
        ProjectType::Legacy,
        ProjectType::Startup,
        ProjectType::Enterprise,
    ];

    // Verify all variants can be created
    assert_eq!(types.len(), 5);

    // Test equality
    assert_eq!(ProjectType::Mature, ProjectType::Mature);
    assert_ne!(ProjectType::Greenfield, ProjectType::Legacy);
}

#[test]
fn test_serialization_roundtrip() {
    let metric = MetricDefinition {
        name: "Test Metric".to_string(),
        description: "A test metric".to_string(),
        category: "test".to_string(),
        unit: Some("count".to_string()),
        direction: MetricDirection::HigherIsBetter,
        thresholds: {
            let mut t = HashMap::new();
            t.insert("default".to_string(), ThresholdValue {
                healthy: Some(80.0),
                warning: Some(60.0),
                critical: Some(40.0),
            });
            t
        },
    };

    // Serialize to JSON
    let json = serde_json::to_string(&metric).unwrap();

    // Deserialize back
    let deserialized: MetricDefinition = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, metric.name);
    assert_eq!(deserialized.category, metric.category);
}

#[test]
fn test_yaml_serialization() {
    let category = CategoryDefinition {
        id: "test-cat".to_string(),
        name: "Test Category".to_string(),
        description: "For testing".to_string(),
        parent: None,
        weight: 1.0,
    };

    // Serialize to YAML
    let yaml = serde_yaml::to_string(&category).unwrap();
    assert!(yaml.contains("test-cat"));

    // Deserialize back
    let deserialized: CategoryDefinition = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(deserialized.id, category.id);
}

// ============================================================================
// Server Handler Tests
// ============================================================================

use methodology_kb_server::server::MethodologyKBServer;
use rmcp::handler::server::ServerHandler;
use tempfile::TempDir;

#[test]
fn test_server_creation_and_info() {
    let temp_dir = TempDir::new().unwrap();
    let kb_path = temp_dir.path().join("kb");
    std::fs::create_dir_all(&kb_path).unwrap();

    let server = MethodologyKBServer::new(kb_path);
    let info = server.get_info();

    assert_eq!(info.server_info.name, "methodology-kb");
    assert_eq!(info.server_info.version, "0.1.0");
    assert!(info.instructions.is_some());
    assert!(info.instructions.unwrap().contains("Methodology"));
}

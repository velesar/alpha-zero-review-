//! Integration tests for Mental Model Server
//!
//! These tests verify the core functionality of the mental model
//! including initialization, viewpoint updates, finding enrichment,
//! and artifact storage.

use mental_model_server::model::{
    derive_constraints, BoundedContext, BoundedContextType,
    Hotspot, Layer, MentalModel, Risk, Severity,
};
use mental_model_server::artifacts::{ArtifactStore, StoreArtifactMetadata};
use tempfile::TempDir;

#[test]
fn test_mental_model_initialization() {
    let model = MentalModel::new("test-project".to_string(), "/test/path".to_string());

    assert_eq!(model.project.name, "test-project");
    assert_eq!(model.project.path, "/test/path");
    assert_eq!(model.version, "1.0");
    assert!(model.findings.is_empty());
    assert!(model.completed_viewpoints.is_empty());
}

#[test]
fn test_viewpoint_application() {
    let mut model = MentalModel::new("test".to_string(), "/test".to_string());

    // Apply VP-F01 (Tech Stack)
    let tech_stack_data = serde_json::json!({
        "primary_language": "rust",
        "version": "1.75",
        "framework": null,
        "confidence": "high"
    });

    model.apply_viewpoint("VP-F01", tech_stack_data).unwrap();

    assert_eq!(model.tech_stack.primary_language, "rust");
    assert!(model.completed_viewpoints.contains(&"VP-F01".to_string()));
}

#[test]
fn test_context_lookup_bounded_context() {
    let mut model = MentalModel::new("test".to_string(), "/test".to_string());

    // Add a core bounded context
    model.domain_model.bounded_contexts.push(BoundedContext {
        name: "Orders".to_string(),
        context_type: BoundedContextType::Core,
        paths: vec!["src/orders".to_string()],
        entities: vec!["Order".to_string(), "OrderItem".to_string()],
        description: Some("Order management domain".to_string()),
    });

    // Test path within context
    let context = model.get_context_for_path("src/orders/handler.rs");
    assert_eq!(context.bounded_context, Some("Orders".to_string()));
    assert_eq!(context.bounded_context_type, Some(BoundedContextType::Core));

    // Test path outside context
    let context = model.get_context_for_path("src/utils/helpers.rs");
    assert_eq!(context.bounded_context, None);
}

#[test]
fn test_context_lookup_layer() {
    let mut model = MentalModel::new("test".to_string(), "/test".to_string());

    // Add architecture layers
    model.architecture.layers.push(Layer {
        name: "domain".to_string(),
        paths: vec!["src/domain".to_string()],
        purpose: "Business logic".to_string(),
        allowed_dependencies: vec![],
    });

    model.architecture.layers.push(Layer {
        name: "adapter".to_string(),
        paths: vec!["src/adapters".to_string()],
        purpose: "External interfaces".to_string(),
        allowed_dependencies: vec!["domain".to_string()],
    });

    let context = model.get_context_for_path("src/domain/entities.rs");
    assert_eq!(context.layer, Some("domain".to_string()));

    let context = model.get_context_for_path("src/adapters/http.rs");
    assert_eq!(context.layer, Some("adapter".to_string()));
}

#[test]
fn test_context_lookup_hotspot() {
    let mut model = MentalModel::new("test".to_string(), "/test".to_string());

    // Add a hotspot
    model.hotspots.files.push(Hotspot {
        path: "src/critical/handler.rs".to_string(),
        churn: Some(100),
        complexity: Some(50.0),
        score: 85.0,
        risk: Risk::Critical,
        reasons: vec!["High churn".to_string(), "High complexity".to_string()],
    });

    let context = model.get_context_for_path("src/critical/handler.rs");
    assert!(context.is_hotspot);
    assert_eq!(context.hotspot_score, Some(85.0));

    let context = model.get_context_for_path("src/other/file.rs");
    assert!(!context.is_hotspot);
    assert_eq!(context.hotspot_score, None);
}

#[test]
fn test_derive_constraints() {
    let mut model = MentalModel::new("test".to_string(), "/test".to_string());

    // Add core bounded context -> high priority
    model.domain_model.bounded_contexts.push(BoundedContext {
        name: "Core".to_string(),
        context_type: BoundedContextType::Core,
        paths: vec!["src/core".to_string()],
        entities: vec![],
        description: None,
    });

    // Add critical hotspot -> high priority
    model.hotspots.files.push(Hotspot {
        path: "src/auth/login.rs".to_string(),
        churn: Some(50),
        complexity: Some(30.0),
        score: 80.0,
        risk: Risk::Critical,
        reasons: vec![],
    });

    // Add domain layer -> security focus
    model.architecture.layers.push(Layer {
        name: "domain".to_string(),
        paths: vec!["src/domain".to_string()],
        purpose: "Business logic".to_string(),
        allowed_dependencies: vec![],
    });

    // Add API layer -> security focus
    model.architecture.layers.push(Layer {
        name: "api".to_string(),
        paths: vec!["src/api".to_string()],
        purpose: "HTTP endpoints".to_string(),
        allowed_dependencies: vec!["domain".to_string()],
    });

    let constraints = derive_constraints(&model);

    // Check high priority paths
    assert!(constraints.high_priority_paths.contains(&"src/core".to_string()));
    assert!(constraints.high_priority_paths.contains(&"src/auth/login.rs".to_string()));

    // Check security focus paths
    assert!(constraints.security_focus_paths.contains(&"src/domain".to_string()));
    assert!(constraints.security_focus_paths.contains(&"src/api".to_string()));
}

#[test]
fn test_severity_scoring() {
    assert_eq!(Severity::Critical.score(), 4.0);
    assert_eq!(Severity::High.score(), 3.0);
    assert_eq!(Severity::Medium.score(), 2.0);
    assert_eq!(Severity::Low.score(), 1.0);
    assert_eq!(Severity::Info.score(), 0.5);

    // Test round-trip
    assert_eq!(Severity::from_score(4.0), Severity::Critical);
    assert_eq!(Severity::from_score(3.0), Severity::High);
    assert_eq!(Severity::from_score(2.0), Severity::Medium);
    assert_eq!(Severity::from_score(1.0), Severity::Low);
    assert_eq!(Severity::from_score(0.5), Severity::Info);
}

#[test]
fn test_artifact_store_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let store = ArtifactStore::new(temp_dir.path().to_path_buf());

    // Store a SARIF artifact
    let sarif_data = br#"{"version": "2.1.0", "runs": []}"#;
    let metadata = StoreArtifactMetadata {
        producer: "test-tool".to_string(),
        produced_at: None,
    };

    let path = store.store_artifact("abc123", "semgrep", sarif_data, &metadata).unwrap();
    assert!(path.contains("abc123"));
    assert!(path.contains("semgrep.sarif"));

    // Retrieve the artifact
    let (data, info) = store.get_artifact("abc123", "semgrep").unwrap();
    assert_eq!(data, sarif_data);
    assert_eq!(info.producer, "test-tool");
}

#[test]
fn test_artifact_store_commit_artifacts() {
    let temp_dir = TempDir::new().unwrap();
    let store = ArtifactStore::new(temp_dir.path().to_path_buf());

    // Initially no artifacts
    let (available, missing) = store.get_commit_artifacts("xyz789").unwrap();
    assert!(available.is_empty());
    assert!(!missing.is_empty());

    // Store some artifacts
    let metadata = StoreArtifactMetadata {
        producer: "test".to_string(),
        produced_at: None,
    };

    store.store_artifact("xyz789", "bandit", b"test1", &metadata).unwrap();
    store.store_artifact("xyz789", "ruff", b"test2", &metadata).unwrap();

    // Check available artifacts
    let (available, missing) = store.get_commit_artifacts("xyz789").unwrap();
    assert_eq!(available.len(), 2);
    assert!(available.iter().any(|a| a.artifact_type == "bandit"));
    assert!(available.iter().any(|a| a.artifact_type == "ruff"));
    assert!(!missing.contains(&"bandit".to_string()));
    assert!(!missing.contains(&"ruff".to_string()));
}

#[test]
fn test_multiple_viewpoint_updates() {
    let mut model = MentalModel::new("test".to_string(), "/test".to_string());

    // Apply multiple viewpoints
    model.apply_viewpoint("VP-F01", serde_json::json!({
        "primary_language": "python",
        "confidence": "high"
    })).unwrap();

    model.apply_viewpoint("VP-F02", serde_json::json!({
        "source_roots": ["src"],
        "test_roots": ["tests"]
    })).unwrap();

    model.apply_viewpoint("VP-F03", serde_json::json!({
        "build_tool": "cargo",
        "package_manager": "cargo"
    })).unwrap();

    assert_eq!(model.completed_viewpoints.len(), 3);
    assert!(model.completed_viewpoints.contains(&"VP-F01".to_string()));
    assert!(model.completed_viewpoints.contains(&"VP-F02".to_string()));
    assert!(model.completed_viewpoints.contains(&"VP-F03".to_string()));
}

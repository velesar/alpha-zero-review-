//! MCP-level tests: every tool is called through a real rmcp client over an
//! in-memory transport (see mcp-test-support).

use mcp_test_support::McpClient;
use methodology_kb_server::server::MethodologyKBServer;
use serde_json::json;
use std::path::PathBuf;
use tempfile::TempDir;

fn kb_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../methodology_kb")
}

async fn client() -> (McpClient, TempDir) {
    let project = TempDir::new().unwrap();
    let server =
        MethodologyKBServer::with_project_path(kb_path(), Some(project.path().to_path_buf()));
    (McpClient::start(server).await, project)
}

#[tokio::test]
async fn lists_all_tools() {
    let (c, _p) = client().await;
    assert_eq!(
        c.tool_names().await,
        vec![
            "check_compliance",
            "classify_finding",
            "get_acquisition_status",
            "get_category",
            "get_metric_data",
            "get_template",
            "get_thresholds",
            "list_acquirable_metrics",
            "list_metrics",
            "list_standards",
            "lookup_metric",
        ]
    );
}

#[tokio::test]
async fn list_tools_return_shipped_kb() {
    let (c, _p) = client().await;
    let metrics = c.ok_json("list_metrics", json!({})).await;
    assert!(metrics.as_array().unwrap().len() >= 10);
    let standards = c.ok_json("list_standards", json!({})).await;
    assert_eq!(standards.as_array().unwrap().len(), 4);
    let acquirable = c.ok("list_acquirable_metrics", json!({})).await;
    assert!(acquirable.contains("vulnerability_count") && acquirable.contains("security_findings"));
}

#[tokio::test]
async fn lookup_metric() {
    let (c, _p) = client().await;
    let text = c
        .ok(
            "lookup_metric",
            json!({"metric": "cyclomatic_complexity", "project_type": "greenfield"}),
        )
        .await;
    assert!(
        text.contains("current"),
        "project thresholds merged: {text}"
    );
    assert!(c
        .err("lookup_metric", json!({"metric": "no_such_metric"}))
        .await
        .contains("no_such_metric"));
    c.err(
        "lookup_metric",
        json!({"metric": "cyclomatic_complexity", "project_type": "bogus"}),
    )
    .await;
    c.err("lookup_metric", json!({})).await;
}

#[tokio::test]
async fn get_thresholds() {
    let (c, _p) = client().await;
    let text = c
        .ok("get_thresholds", json!({"project_type": "python_backend"}))
        .await;
    assert!(text.contains("test_coverage"));
    assert!(c
        .err("get_thresholds", json!({"project_type": "bogus"}))
        .await
        .contains("bogus"));
}

#[tokio::test]
async fn classify_finding_applies_kb_rules() {
    let (c, _p) = client().await;
    let result = c
        .ok_json(
            "classify_finding",
            json!({
                "category": "security_injection",
                "base_severity": "MEDIUM",
                "context": {"file_path": "src/payments/charge.py", "business_context": ["payment"]}
            }),
        )
        .await;
    assert_eq!(result["adjusted_severity"], "CRITICAL");
    let factors = result["adjustment_factors"].as_array().unwrap();
    assert!(factors
        .iter()
        .any(|f| f["name"].as_str().unwrap().contains("payment")));

    // Test code is downgraded once, even though several path rules match
    let result = c
        .ok_json(
            "classify_finding",
            json!({"category": "maintainability", "base_severity": "HIGH",
                   "context": {"file_path": "tests/test_x.py"}}),
        )
        .await;
    let path_rules = result["adjustment_factors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["name"].as_str().unwrap().contains("PathPattern"))
        .count();
    assert_eq!(path_rules, 1);

    c.err("classify_finding", json!({"context": {"caller_count": -1}}))
        .await;
}

#[tokio::test]
async fn get_category() {
    let (c, _p) = client().await;
    assert!(c
        .ok("get_category", json!({"category": "security"}))
        .await
        .contains("security"));
    c.err("get_category", json!({"category": "no_such_category"}))
        .await;
}

#[tokio::test]
async fn get_template() {
    let (c, _p) = client().await;
    assert!(c
        .ok(
            "get_template",
            json!({"template_type": "executive_summary"})
        )
        .await
        .contains("Executive Summary"));
    assert!(c
        .err("get_template", json!({"template_type": "nope"}))
        .await
        .contains("Available"));
    c.err(
        "get_template",
        json!({"template_type": "executive_summary", "format": "html"}),
    )
    .await;
}

#[tokio::test]
async fn check_compliance_uses_standard_rules() {
    let (c, _p) = client().await;
    let result = c
        .ok_json(
            "check_compliance",
            json!({
                "standard": "RUST-HEX-001",
                "detected_pattern": {
                    "layers": [{"name": "domain"}, {"name": "application"},
                               {"name": "inbound"}, {"name": "outbound"}],
                    "violations": [
                        {"from_layer": "inbound", "to_layer": "domain"},
                        {"from_layer": "domain", "to_layer": "outbound", "file_path": "src/a.rs"}
                    ]
                }
            }),
        )
        .await;
    assert_eq!(result["violations"].as_array().unwrap().len(), 1);
    assert_eq!(result["allowed_dependencies"].as_array().unwrap().len(), 1);

    assert!(c
        .err(
            "check_compliance",
            json!({"standard": "nope", "detected_pattern": {}})
        )
        .await
        .contains("Available"));
    c.err(
        "check_compliance",
        json!({"standard": "RUST-HEX-001",
               "detected_pattern": {"violations": [{"rule": "x"}]}}),
    )
    .await;
}

#[tokio::test]
async fn metric_acquisition_tools() {
    let (c, project) = client().await;
    let status = c
        .ok("get_acquisition_status", json!({"commit": "abc123"}))
        .await;
    assert!(status.contains("abc123"));
    let missing = c
        .ok(
            "get_metric_data",
            json!({"metric": "vulnerability_count", "commit": "abc123"}),
        )
        .await;
    assert!(
        missing.to_lowercase().contains("run") || missing.contains("tool"),
        "{missing}"
    );

    // Traversal in the commit is rejected, not resolved outside .audit
    std::fs::create_dir_all(project.path().join("outside")).unwrap();
    c.err(
        "get_metric_data",
        json!({"metric": "vulnerability_count", "commit": "../../outside"}),
    )
    .await;
    c.err("get_acquisition_status", json!({"commit": "../x"}))
        .await;
    c.err("get_metric_data", json!({"metric": "no_such_metric"}))
        .await;
}

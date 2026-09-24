//! MCP-level tests: every tool is called through a real rmcp client over an
//! in-memory transport (see mcp-test-support), following an audit flow.

use mcp_test_support::McpClient;
use mental_model_server::server::MentalModelServer;
use serde_json::{json, Value};
use tempfile::TempDir;

struct Fixture {
    client: McpClient,
    dir: TempDir,
}

async fn fixture() -> Fixture {
    let dir = TempDir::new().unwrap();
    let server = MentalModelServer::with_audit_path(
        dir.path().join("model.yaml"),
        Some(dir.path().join(".audit")),
    )
    .unwrap();
    let client = McpClient::start(server).await;
    client
        .ok(
            "init_model",
            json!({"name": "shop", "path": dir.path().to_str().unwrap(),
                   "description": "test project"}),
        )
        .await;
    Fixture { client, dir }
}

/// Architecture, domain model and hotspots as VP-S02/S03/S06 would report them
async fn apply_structure(c: &McpClient) {
    c.ok(
        "update_viewpoint",
        json!({"viewpoint": "VP-S02", "data": {
            "pattern": "hexagonal", "confidence": "high",
            "layers": [
                {"name": "domain", "paths": ["src/core"], "purpose": "logic"},
                {"name": "inbound", "paths": ["src/api"], "purpose": "http"}
            ]
        }}),
    )
    .await;
    c.ok(
        "update_viewpoint",
        json!({"viewpoint": "VP-S03", "data": {
            "bounded_contexts": [
                {"name": "Orders", "type": "core", "paths": ["src/core"],
                 "entities": ["Order"], "description": "orders"},
                {"name": "Utils", "type": "generic", "paths": ["src/util"],
                 "entities": [], "description": "helpers"}
            ]
        }}),
    )
    .await;
    c.ok(
        "update_viewpoint",
        json!({"viewpoint": "VP-S06", "data": {
            "total_modules": 3,
            "hotspots": {"files": [
                {"path": "src/core/order.rs", "score": 90.0, "risk": "HIGH"}
            ]}
        }}),
    )
    .await;
}

fn finding(title: &str, file: &str, severity: &str, category: &str) -> Value {
    json!({"viewpoint": "VP-Q01", "category": category, "title": title,
           "description": title, "file_path": file, "base_severity": severity})
}

#[tokio::test]
async fn lists_all_tools() {
    let f = fixture().await;
    assert_eq!(f.client.tool_names().await.len(), 22);
}

#[tokio::test]
async fn model_and_viewpoints() {
    let f = fixture().await;
    let c = &f.client;
    apply_structure(c).await;

    assert!(c.ok("get_model", json!({})).await.contains("shop"));
    assert!(c
        .ok("get_model_section", json!({"section": "domain_model"}))
        .await
        .contains("Orders"));
    c.err("get_model_section", json!({"section": "nope"})).await;

    let done = c.ok("get_completed_viewpoints", json!({})).await;
    for vp in ["VP-S02", "VP-S03", "VP-S06"] {
        assert!(done.contains(vp), "{done}");
    }

    // Mismatched viewpoint data is rejected instead of silently dropped
    let err = c
        .err(
            "update_viewpoint",
            json!({"viewpoint": "VP-S06", "data": {"files": [{"path": 1}]}}),
        )
        .await;
    assert!(err.contains("VP-S06"));
}

#[tokio::test]
async fn context_and_constraints() {
    let f = fixture().await;
    let c = &f.client;
    apply_structure(c).await;

    let ctx = c
        .ok_json("get_context", json!({"file_path": "src/core/order.rs"}))
        .await;
    assert_eq!(ctx["bounded_context_type"], "core");
    assert_eq!(ctx["is_hotspot"], true);

    let batch = c
        .ok_json(
            "get_contexts",
            json!({"file_paths": ["src/core/order.rs", "src/util/fmt.rs", "README.md"]}),
        )
        .await;
    assert_eq!(batch["src/util/fmt.rs"]["bounded_context_type"], "generic");
    assert_eq!(batch["README.md"]["is_hotspot"], false);

    let constraints = c.ok_json("get_constraints", json!({})).await;
    let high: Vec<&str> = constraints["high_priority_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(high.contains(&"src/core"));
    assert!(high.contains(&"src/core/order.rs"));
    let focus = constraints["security_focus_paths"].as_array().unwrap();
    assert_eq!(focus, &vec![json!("src/api")]);

    c.err("get_context", json!({})).await;
}

#[tokio::test]
async fn findings_queries_and_synthesis() {
    let f = fixture().await;
    let c = &f.client;
    apply_structure(c).await;

    // HIGH in a core-domain hotspot is escalated
    let added = c
        .ok_json(
            "add_finding",
            finding("SQL injection", "src/core/order.rs", "HIGH", "security"),
        )
        .await;
    assert_eq!(added["adjusted_severity"], "CRITICAL");

    let batch = c
        .ok_json(
            "add_findings",
            json!({"findings": [
                finding("Long function", "src/util/fmt.rs", "MEDIUM", "maintainability"),
                finding("Naming", "src/util/fmt.rs", "LOW", "maintainability"),
                finding("Missing docs", "src/api/http.rs", "LOW", "documentation")
            ]}),
        )
        .await;
    assert_eq!(batch.as_array().unwrap().len(), 3);

    let all = c.ok_json("get_findings", json!({})).await;
    assert_eq!(all.as_array().unwrap().len(), 4);
    let by_file = c
        .ok_json(
            "get_findings_by_file",
            json!({"file_path": "src/util/fmt.rs"}),
        )
        .await;
    assert_eq!(by_file.as_array().unwrap().len(), 2);
    let critical = c
        .ok_json("get_findings_by_severity", json!({"severity": "critical"}))
        .await;
    assert_eq!(critical.as_array().unwrap().len(), 1);
    c.err("get_findings_by_severity", json!({"severity": "urgent"}))
        .await;
    let by_vp = c
        .ok_json("get_findings_by_viewpoint", json!({"viewpoint": "VP-Q01"}))
        .await;
    assert_eq!(by_vp.as_array().unwrap().len(), 4);
    let by_cat = c
        .ok_json(
            "get_findings_by_category",
            json!({"category": "maintainability"}),
        )
        .await;
    assert_eq!(by_cat.as_array().unwrap().len(), 2);
    let summary = c.ok_json("get_findings_summary", json!({})).await;
    assert_eq!(summary["total"], 4);

    // Invalid severity is rejected rather than silently becoming INFO,
    // and a batch with one bad entry stores nothing
    c.err(
        "add_finding",
        finding("Bad", "src/a.rs", "SEVERE", "security"),
    )
    .await;
    let err = c
        .err(
            "add_findings",
            json!({"findings": [finding("Ok", "src/a.rs", "LOW", "style"),
                                finding("Bad", "src/a.rs", "SEVERE", "style")]}),
        )
        .await;
    assert!(err.contains("findings[1]"), "{err}");
    let summary = c.ok_json("get_findings_summary", json!({})).await;
    assert_eq!(summary["total"], 4);

    // Synthesis: most severe cluster first, everything accounted for
    let text = c.ok("synthesize", json!({})).await;
    assert!(text.contains("covering 4 of 4 findings"), "{text}");
    let synthesis = mcp_test_support::parse_json_body(&text).unwrap();
    assert_eq!(synthesis["root_causes"][0]["category"], "security");
    assert_eq!(synthesis["root_causes"][0]["impact"], "CRITICAL");
    c.ok("synthesize", json!({"algorithm": "location_based"}))
        .await;
    c.err("synthesize", json!({"algorithm": "magic"})).await;
    c.ok("flush", json!({})).await;
    assert!(f.dir.path().join("model.yaml").exists());
}

#[tokio::test]
async fn artifacts_and_export() {
    let f = fixture().await;
    let c = &f.client;

    c.ok(
        "store_artifact",
        json!({"commit": "abc123", "type": "semgrep", "data": "{\"runs\":[]}", "producer": "ci"}),
    )
    .await;
    let artifact = c
        .ok_json(
            "get_artifact",
            json!({"commit": "abc123", "type": "semgrep"}),
        )
        .await;
    assert_eq!(artifact["producer"], "ci");
    let listing = c
        .ok_json("get_commit_artifacts", json!({"commit": "abc123"}))
        .await;
    assert_eq!(listing["available"][0]["type"], "semgrep");

    // Traversal attempts from the self-audit
    c.err(
        "store_artifact",
        json!({"commit": "../../escape", "type": "semgrep", "data": "{}", "producer": "x"}),
    )
    .await;
    c.err("get_artifact", json!({"commit": "abc123", "type": "../x"}))
        .await;
    c.err("get_artifact", json!({"commit": "abc123", "type": "trivy"}))
        .await;
    assert!(!f.dir.path().join("escape").exists());

    let exported = c
        .ok(
            "export_findings",
            json!({"output_path": "reports/findings.json"}),
        )
        .await;
    assert!(exported.contains("reports/findings.json"));
    c.err(
        "export_findings",
        json!({"output_path": "reports/findings.json"}),
    )
    .await;
    c.ok(
        "export_findings",
        json!({"output_path": "reports/findings.json", "overwrite": true}),
    )
    .await;
    c.err("export_findings", json!({"output_path": "../outside.json"}))
        .await;
    assert!(!f.dir.path().join("outside.json").exists());
}

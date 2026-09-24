//! MCP-level tests: every tool is called through a real rmcp client over an
//! in-memory transport (see mcp-test-support). Only cargo/clippy are assumed
//! to be installed, as in CI.

use mcp_test_support::McpClient;
use sarif_tools_server::server::SarifToolsServer;
use serde_json::json;
use std::path::PathBuf;
use tempfile::TempDir;

fn mappings() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../methodology_kb/taxonomies/rule_mapping.yaml")
}

async fn client(root: &TempDir) -> McpClient {
    McpClient::start(SarifToolsServer::with_allowed_roots(
        Some(mappings()),
        vec![root.path().to_path_buf()],
    ))
    .await
}

fn semgrep_sarif(rule_id: &str) -> serde_json::Value {
    json!({
        "version": "2.1.0",
        "runs": [{
            "tool": {"driver": {"name": "semgrep", "rules": []}},
            "results": [{
                "ruleId": rule_id,
                "message": {"text": "finding"},
                "locations": [{"physicalLocation": {
                    "artifactLocation": {"uri": "app.py"},
                    "region": {"startLine": 3}
                }}]
            }]
        }]
    })
}

/// A one-file crate with a single `unwrap()`
fn tiny_crate(dir: &TempDir) -> PathBuf {
    let root = dir.path().join("tiny");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"tiny\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/lib.rs"),
        "pub fn first(x: Option<u8>) -> u8 {\n    x.unwrap()\n}\n",
    )
    .unwrap();
    root
}

#[tokio::test]
async fn lists_all_tools() {
    let root = TempDir::new().unwrap();
    let c = client(&root).await;
    assert_eq!(
        c.tool_names().await,
        vec![
            "get_tool_config",
            "install_tool",
            "list_available_tools",
            "merge_sarif",
            "normalize_sarif",
            "run_tool",
        ]
    );
}

#[tokio::test]
async fn tool_registry_tools() {
    let root = TempDir::new().unwrap();
    let c = client(&root).await;
    let tools = c.ok_json("list_available_tools", json!({})).await;
    let names: Vec<&str> = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    for name in ["semgrep", "bandit", "ruff", "trivy", "clippy"] {
        assert!(names.contains(&name), "{name} missing from {names:?}");
    }
    assert!(c
        .ok("get_tool_config", json!({"tool": "clippy"}))
        .await
        .contains("clippy"));
    c.err("get_tool_config", json!({"tool": "nope"})).await;
    // clippy ships with the toolchain: install is a no-op success
    assert!(c
        .ok("install_tool", json!({"tool": "clippy"}))
        .await
        .contains("true"));
    c.err("install_tool", json!({"tool": "nope"})).await;
}

#[tokio::test]
async fn run_tool_guards() {
    let root = TempDir::new().unwrap();
    let outside = TempDir::new().unwrap();
    let c = client(&root).await;
    let project = tiny_crate(&root);

    let err = c
        .err(
            "run_tool",
            json!({"tool": "clippy", "path": outside.path()}),
        )
        .await;
    assert!(err.contains("outside the allowed roots"), "{err}");
    let err = c
        .err("run_tool", json!({"tool": "clippy", "path": project}))
        .await;
    assert!(err.contains("allow_code_execution"), "{err}");
    c.err("run_tool", json!({"tool": "nope", "path": project}))
        .await;
    c.err(
        "run_tool",
        json!({"tool": "clippy", "path": root.path().join("missing")}),
    )
    .await;
}

#[tokio::test]
async fn run_tool_clippy_reports_complete_results() {
    let root = TempDir::new().unwrap();
    let c = client(&root).await;
    let project = tiny_crate(&root);

    let out = c
        .ok_json(
            "run_tool",
            json!({"tool": "clippy", "path": project, "config": {
                "allow_code_execution": true,
                "deny": ["clippy::unwrap_used"]
            }}),
        )
        .await;
    assert_eq!(out["complete"], true, "{out}");
    let results = out["sarif"]["runs"][0]["results"].as_array().unwrap();
    let unwrap = results
        .iter()
        .find(|r| r["ruleId"] == "clippy::unwrap_used")
        .expect("unwrap_used reported");
    assert_eq!(unwrap["level"], "error");
}

#[tokio::test]
async fn merge_and_normalize() {
    let root = TempDir::new().unwrap();
    let c = client(&root).await;
    let rule = "python.lang.security.audit.dangerous-subprocess-use";

    let merged = c
        .ok_json(
            "merge_sarif",
            json!({"sarif_files": [semgrep_sarif(rule), semgrep_sarif("other")]}),
        )
        .await;
    assert_eq!(merged["total_results"], 2);
    c.err("merge_sarif", json!({"sarif_files": [{"not": "sarif"}]}))
        .await;

    let normalized = c
        .ok_json("normalize_sarif", json!({"sarif": semgrep_sarif(rule)}))
        .await;
    let text = normalized.to_string();
    assert!(text.contains("security_injection"), "{text}");
    c.err(
        "normalize_sarif",
        json!({"sarif": semgrep_sarif(rule), "rule_mappings": "/etc/passwd"}),
    )
    .await;
    c.err("normalize_sarif", json!({"sarif": "nope"})).await;
}

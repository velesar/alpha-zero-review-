//! Integration tests for SARIF Tools Server
//!
//! These tests verify the core functionality of the SARIF tools
//! including tool registry, SARIF parsing, and merging.

use sarif_tools_server::sarif::{Sarif, Run, Tool, ToolDriver, Result as SarifResult, Message, Location, PhysicalLocation, ArtifactLocation};
use sarif_tools_server::tools::ToolRegistry;
use sarif_tools_server::runner::{detect_tool, parse_sarif};

#[test]
fn test_tool_registry_creation() {
    let registry = ToolRegistry::new();

    // Check all tools are registered
    assert!(registry.get("semgrep").is_some());
    assert!(registry.get("bandit").is_some());
    assert!(registry.get("ruff").is_some());
    assert!(registry.get("trivy").is_some());
    assert!(registry.get("clippy").is_some());

    // Check unknown tool returns None
    assert!(registry.get("unknown").is_none());
}

#[test]
fn test_tool_registry_list() {
    let registry = ToolRegistry::new();
    let tools = registry.list_available();

    assert_eq!(tools.len(), 5);

    // Check clippy supports Rust
    let clippy = tools.iter().find(|t| t.name == "clippy").unwrap();
    assert!(clippy.languages.contains(&"rust".to_string()));

    // Check ruff supports Python
    let ruff = tools.iter().find(|t| t.name == "ruff").unwrap();
    assert!(ruff.languages.contains(&"python".to_string()));
}

#[test]
fn test_sarif_creation() {
    let sarif = Sarif::new();

    assert_eq!(sarif.version, "2.1.0");
    assert!(sarif.runs.is_empty());
    assert_eq!(sarif.result_count(), 0);
}

#[test]
fn test_sarif_merge() {
    let mut sarif1 = Sarif::new();
    sarif1.runs.push(Run {
        tool: Tool {
            driver: ToolDriver {
                name: "tool1".to_string(),
                version: Some("1.0".to_string()),
                information_uri: None,
                rules: vec![],
            },
        },
        results: vec![],
        invocations: vec![],
    });

    let mut sarif2 = Sarif::new();
    sarif2.runs.push(Run {
        tool: Tool {
            driver: ToolDriver {
                name: "tool2".to_string(),
                version: Some("2.0".to_string()),
                information_uri: None,
                rules: vec![],
            },
        },
        results: vec![],
        invocations: vec![],
    });

    sarif1.merge(sarif2);

    assert_eq!(sarif1.runs.len(), 2);
    assert_eq!(sarif1.runs[0].tool.driver.name, "tool1");
    assert_eq!(sarif1.runs[1].tool.driver.name, "tool2");
}

#[test]
fn test_sarif_result_count() {
    let mut sarif = Sarif::new();
    sarif.runs.push(Run {
        tool: Tool {
            driver: ToolDriver {
                name: "test".to_string(),
                version: None,
                information_uri: None,
                rules: vec![],
            },
        },
        results: vec![
            SarifResult {
                rule_id: "R001".to_string(),
                level: Some("warning".to_string()),
                message: Message { text: "Test 1".to_string(), markdown: None },
                locations: vec![],
                fingerprints: None,
                properties: serde_json::Value::Null,
            },
            SarifResult {
                rule_id: "R002".to_string(),
                level: Some("error".to_string()),
                message: Message { text: "Test 2".to_string(), markdown: None },
                locations: vec![],
                fingerprints: None,
                properties: serde_json::Value::Null,
            },
        ],
        invocations: vec![],
    });

    assert_eq!(sarif.result_count(), 2);
}

#[test]
fn test_parse_sarif_json() {
    let json = r#"{
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "TestTool",
                    "version": "1.0.0",
                    "rules": []
                }
            },
            "results": [{
                "ruleId": "TEST001",
                "level": "warning",
                "message": { "text": "Test finding" },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": "src/test.rs" },
                        "region": {
                            "startLine": 10,
                            "startColumn": 5
                        }
                    }
                }]
            }]
        }]
    }"#;

    let sarif = parse_sarif(json).unwrap();

    assert_eq!(sarif.version, "2.1.0");
    assert_eq!(sarif.runs.len(), 1);
    assert_eq!(sarif.runs[0].tool.driver.name, "TestTool");
    assert_eq!(sarif.runs[0].results.len(), 1);
    assert_eq!(sarif.runs[0].results[0].rule_id, "TEST001");
}

#[test]
fn test_detect_tool_cargo() {
    // Cargo should be available in a Rust project
    assert!(detect_tool("cargo"));
}

#[test]
fn test_detect_tool_nonexistent() {
    // This tool shouldn't exist
    assert!(!detect_tool("this-tool-definitely-does-not-exist-xyz123"));
}

#[test]
fn test_sarif_with_locations() {
    let sarif = Sarif {
        schema: "https://example.com/sarif.json".to_string(),
        version: "2.1.0".to_string(),
        runs: vec![Run {
            tool: Tool {
                driver: ToolDriver {
                    name: "test".to_string(),
                    version: None,
                    information_uri: None,
                    rules: vec![],
                },
            },
            results: vec![SarifResult {
                rule_id: "R001".to_string(),
                level: Some("warning".to_string()),
                message: Message { text: "Test".to_string(), markdown: None },
                locations: vec![Location {
                    physical_location: PhysicalLocation {
                        artifact_location: ArtifactLocation {
                            uri: "src/main.rs".to_string(),
                            uri_base_id: None,
                        },
                        region: None,
                    },
                }],
                fingerprints: None,
                properties: serde_json::Value::Null,
            }],
            invocations: vec![],
        }],
    };

    assert_eq!(sarif.result_count(), 1);
    assert_eq!(
        sarif.runs[0].results[0].locations[0].physical_location.artifact_location.uri,
        "src/main.rs"
    );
}

// ============================================================================
// Server Handler Tests
// ============================================================================

use sarif_tools_server::server::SarifToolsServer;
use rmcp::handler::server::ServerHandler;

#[test]
fn test_server_creation_and_info() {
    let server = SarifToolsServer::new(None);
    let info = server.get_info();

    assert_eq!(info.server_info.name, "sarif-tools");
    assert_eq!(info.server_info.version, "0.1.0");
    assert!(info.instructions.is_some());
    assert!(info.instructions.unwrap().contains("SARIF"));
}

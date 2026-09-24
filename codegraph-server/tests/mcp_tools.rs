//! MCP-level tests: every tool is called through a real rmcp client over an
//! in-memory transport (see mcp-test-support), against a small SCIP index.

use codegraph_server::scip;
use codegraph_server::server::CodegraphServer;
use mcp_test_support::McpClient;
use prost::Message;
use serde_json::json;
use tempfile::TempDir;

const CALLER: &str = "rust-analyzer cargo shop 0.1.0 orders/place_order().";
const HELPER: &str = "rust-analyzer cargo shop 0.1.0 pricing/total().";

/// src/orders.rs: place_order() (lines 0-5) calls pricing::total() on line 2
/// and uses a local; src/pricing.rs: total() and its own local.
fn index_bytes() -> Vec<u8> {
    let def = scip::SymbolRole::Definition as i32;
    let occ = |symbol: &str, range: Vec<i32>, roles: i32, enclosing: Vec<i32>| scip::Occurrence {
        symbol: symbol.to_string(),
        range,
        symbol_roles: roles,
        enclosing_range: enclosing,
        ..Default::default()
    };
    let info = |symbol: &str| scip::SymbolInformation {
        symbol: symbol.to_string(),
        kind: scip::symbol_information::Kind::Function as i32,
        ..Default::default()
    };
    scip::Index {
        documents: vec![
            scip::Document {
                relative_path: "src/orders.rs".to_string(),
                symbols: vec![info(CALLER)],
                occurrences: vec![
                    occ(CALLER, vec![0, 7, 18], def, vec![0, 0, 5, 1]),
                    occ(HELPER, vec![2, 12, 17], 0, vec![]),
                    occ("local 0", vec![1, 8, 9], def, vec![]),
                    occ("local 0", vec![3, 4, 5], 0, vec![]),
                ],
                ..Default::default()
            },
            scip::Document {
                relative_path: "src/pricing.rs".to_string(),
                symbols: vec![info(HELPER)],
                occurrences: vec![
                    occ(HELPER, vec![0, 7, 12], def, vec![0, 0, 3, 1]),
                    occ("local 0", vec![1, 8, 9], def, vec![]),
                ],
                ..Default::default()
            },
        ],
        ..Default::default()
    }
    .encode_to_vec()
}

struct Fixture {
    client: McpClient,
    root: TempDir,
}

async fn fixture() -> Fixture {
    let root = TempDir::new().unwrap();
    std::fs::write(root.path().join("index.scip"), index_bytes()).unwrap();
    let client = McpClient::start(CodegraphServer::with_allowed_roots(vec![root
        .path()
        .to_path_buf()]))
    .await;
    Fixture { client, root }
}

async fn loaded() -> Fixture {
    let f = fixture().await;
    let out = f
        .client
        .ok_json(
            "load_index",
            json!({"scip_path": f.root.path().join("index.scip")}),
        )
        .await;
    assert_eq!(out["files_count"], 2);
    f
}

#[tokio::test]
async fn lists_all_tools() {
    let f = fixture().await;
    assert_eq!(f.client.tool_names().await.len(), 10);
}

#[tokio::test]
async fn queries_require_an_index() {
    let f = fixture().await;
    for (tool, args) in [
        ("find_symbol", json!({"pattern": "x"})),
        ("get_callers", json!({"symbol_id": HELPER})),
        ("get_module_deps", json!({"module_path": "src/"})),
        ("find_hotspot_symbols", json!({"min_callers": 1})),
    ] {
        assert!(f.client.err(tool, args).await.contains("No index loaded"));
    }
}

#[tokio::test]
async fn load_index_validates_paths() {
    let f = fixture().await;
    let outside = TempDir::new().unwrap();
    std::fs::write(outside.path().join("x.scip"), index_bytes()).unwrap();
    std::fs::write(f.root.path().join("index.txt"), "x").unwrap();

    f.client
        .err(
            "load_index",
            json!({"scip_path": outside.path().join("x.scip")}),
        )
        .await;
    f.client
        .err(
            "load_index",
            json!({"scip_path": f.root.path().join("index.txt")}),
        )
        .await;
    f.client
        .err(
            "load_index",
            json!({"scip_path": f.root.path().join("missing.scip")}),
        )
        .await;
}

#[tokio::test]
async fn symbol_queries() {
    let f = loaded().await;
    let c = &f.client;

    let found = c.ok_json("find_symbol", json!({"pattern": "total"})).await;
    assert_eq!(found.as_array().unwrap().len(), 1);
    let info = c
        .ok_json("get_symbol_info", json!({"symbol_id": HELPER}))
        .await;
    assert_eq!(info["file"], "src/pricing.rs");
    c.err("get_symbol_info", json!({"symbol_id": "nope"})).await;

    let callers = c.ok_json("get_callers", json!({"symbol_id": HELPER})).await;
    assert_eq!(callers["caller_count"], 1);
    assert_eq!(callers["callers"][0]["file"], "src/orders.rs");

    let callees = c.ok_json("get_callees", json!({"symbol_id": CALLER})).await;
    assert_eq!(
        callees["callee_count"], 1,
        "locals are not callees: {callees}"
    );
    assert_eq!(callees["callees"][0]["symbol_id"], HELPER);
    assert_eq!(callees["callees"][0]["defined_in"], "src/pricing.rs");
    c.err("get_callees", json!({"symbol_id": "nope"})).await;

    let impact = c.ok_json("get_impact", json!({"symbol_id": HELPER})).await;
    assert_eq!(impact["affected_files"], 1);

    let symbols = c
        .ok_json("get_file_symbols", json!({"file_path": "src/orders.rs"}))
        .await;
    assert_eq!(symbols.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn module_deps_and_hotspots() {
    let f = loaded().await;
    let c = &f.client;

    let deps = c
        .ok_json("get_module_deps", json!({"module_path": "src/orders.rs"}))
        .await;
    assert_eq!(deps["depends_on"]["src/pricing.rs"], 1);
    let deps = c
        .ok_json("get_module_deps", json!({"module_path": "src/pricing.rs"}))
        .await;
    assert_eq!(deps["dependents"]["src/orders.rs"], 1);

    let hotspots = c
        .ok_json("find_hotspot_symbols", json!({"min_callers": 1}))
        .await;
    let ids: Vec<&str> = hotspots
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["symbol_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec![HELPER], "no locals among hotspots");
}

#[tokio::test]
async fn load_project_indexes() {
    let f = fixture().await;
    let c = &f.client;
    let project = f.root.path().join("project");
    std::fs::create_dir_all(project.join(".audit/indexes")).unwrap();
    std::fs::write(project.join(".audit/indexes/rust.scip"), index_bytes()).unwrap();

    let out = c
        .ok_json("load_project_indexes", json!({"project_path": project}))
        .await;
    assert_eq!(out["loaded_count"], 1, "{out}");
    assert_eq!(out["total_files"], 2);
    c.ok_json("find_symbol", json!({"pattern": "place_order"}))
        .await;

    c.err(
        "load_project_indexes",
        json!({"project_path": project, "build_if_missing": true}),
    )
    .await;
    c.err("load_project_indexes", json!({"project_path": "/"}))
        .await;
    c.err(
        "load_project_indexes",
        json!({"project_path": f.root.path().join("missing")}),
    )
    .await;
}

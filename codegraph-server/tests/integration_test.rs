//! Integration tests for Codegraph Server
//!
//! These tests verify the core functionality of the codegraph
//! including symbol management, reference tracking, and impact analysis.

use codegraph_server::graph::{Codegraph, Range, Reference, ReferenceRole, Symbol, SymbolKind};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_codegraph_creation() {
    let graph = Codegraph::new();

    assert_eq!(graph.symbols_count(), 0);
    assert_eq!(graph.files_count(), 0);
}

#[test]
fn test_add_and_retrieve_symbol() {
    let mut graph = Codegraph::new();

    let symbol = Symbol {
        id: "rust-analyzer crate src/main.rs#main".to_string(),
        kind: SymbolKind::Function,
        name: "main".to_string(),
        file: "src/main.rs".to_string(),
        range: Range {
            start_line: 1,
            start_column: 0,
            end_line: 10,
            end_column: 1,
        },
        documentation: Some("Entry point".to_string()),
    };

    graph.add_symbol(symbol);

    assert_eq!(graph.symbols_count(), 1);

    let retrieved = graph.get_symbol("rust-analyzer crate src/main.rs#main");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "main");
    assert_eq!(retrieved.unwrap().kind, SymbolKind::Function);
}

#[test]
fn test_file_symbols_tracking() {
    let mut graph = Codegraph::new();

    // Add symbols to same file
    graph.add_symbol(Symbol {
        id: "sym1".to_string(),
        kind: SymbolKind::Function,
        name: "func1".to_string(),
        file: "src/lib.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    graph.add_symbol(Symbol {
        id: "sym2".to_string(),
        kind: SymbolKind::Function,
        name: "func2".to_string(),
        file: "src/lib.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    graph.add_symbol(Symbol {
        id: "sym3".to_string(),
        kind: SymbolKind::Class,
        name: "MyClass".to_string(),
        file: "src/models.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    assert_eq!(graph.files_count(), 2);

    let lib_symbols = graph.get_file_symbols("src/lib.rs");
    assert_eq!(lib_symbols.len(), 2);

    let model_symbols = graph.get_file_symbols("src/models.rs");
    assert_eq!(model_symbols.len(), 1);
}

#[test]
fn test_find_symbol_by_name() {
    let mut graph = Codegraph::new();

    graph.add_symbol(Symbol {
        id: "s1".to_string(),
        kind: SymbolKind::Function,
        name: "process_data".to_string(),
        file: "src/data.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    graph.add_symbol(Symbol {
        id: "s2".to_string(),
        kind: SymbolKind::Function,
        name: "transform_data".to_string(),
        file: "src/data.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    graph.add_symbol(Symbol {
        id: "s3".to_string(),
        kind: SymbolKind::Class,
        name: "UserHandler".to_string(),
        file: "src/handlers.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    // Case-insensitive search
    let found = graph.find_symbol("data");
    assert_eq!(found.len(), 2);

    let found = graph.find_symbol("Process");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "process_data");

    let found = graph.find_symbol("Handler");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "UserHandler");
}

#[test]
fn test_references_and_callers() {
    let mut graph = Codegraph::new();

    // Add a function
    graph.add_symbol(Symbol {
        id: "helper".to_string(),
        kind: SymbolKind::Function,
        name: "helper_func".to_string(),
        file: "src/utils.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    // Add references to it from other files
    graph.add_reference(Reference {
        symbol_id: "helper".to_string(),
        file: "src/main.rs".to_string(),
        line: 15,
        role: ReferenceRole::Call,
    });

    graph.add_reference(Reference {
        symbol_id: "helper".to_string(),
        file: "src/handler.rs".to_string(),
        line: 42,
        role: ReferenceRole::Call,
    });

    graph.add_reference(Reference {
        symbol_id: "helper".to_string(),
        file: "src/main.rs".to_string(),
        line: 28,
        role: ReferenceRole::Call,
    });

    let callers = graph.get_callers("helper");
    assert_eq!(callers.len(), 3);
}

#[test]
fn test_impact_analysis() {
    let mut graph = Codegraph::new();

    graph.add_symbol(Symbol {
        id: "critical_func".to_string(),
        kind: SymbolKind::Function,
        name: "critical_function".to_string(),
        file: "src/core.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    // Multiple references from different files
    graph.add_reference(Reference {
        symbol_id: "critical_func".to_string(),
        file: "src/service_a.rs".to_string(),
        line: 10,
        role: ReferenceRole::Call,
    });

    graph.add_reference(Reference {
        symbol_id: "critical_func".to_string(),
        file: "src/service_b.rs".to_string(),
        line: 20,
        role: ReferenceRole::Call,
    });

    graph.add_reference(Reference {
        symbol_id: "critical_func".to_string(),
        file: "src/service_a.rs".to_string(),
        line: 30,
        role: ReferenceRole::Call,
    });

    let impact = graph.get_impact("critical_func");
    assert_eq!(impact.direct_references, 3);
    assert_eq!(impact.affected_files, 2);
    assert!(impact.files.contains(&"src/service_a.rs".to_string()));
    assert!(impact.files.contains(&"src/service_b.rs".to_string()));
}

fn func(id: &str, file: &str, line: u32) -> Symbol {
    Symbol {
        id: id.to_string(),
        kind: SymbolKind::Function,
        name: id.to_string(),
        file: file.to_string(),
        range: Range {
            start_line: line,
            ..Range::default()
        },
        documentation: None,
    }
}

fn reference(symbol_id: &str, file: &str, line: u32) -> Reference {
    Reference {
        symbol_id: symbol_id.to_string(),
        file: file.to_string(),
        line,
        role: ReferenceRole::Reference,
    }
}

#[test]
fn test_module_dependencies() {
    let mut graph = Codegraph::new();

    graph.add_symbol(func("a_func1", "src/module_a.rs", 1));
    graph.add_symbol(func("a_func2", "src/module_a.rs", 10));
    graph.add_symbol(func("b_func", "src/module_b.rs", 1));
    graph.add_symbol(func("c_func", "src/module_c.rs", 1));

    // B and C use A: they are A's dependents
    graph.add_reference(reference("a_func1", "src/module_b.rs", 5));
    graph.add_reference(reference("a_func1", "src/module_c.rs", 10));
    // A uses B: A depends on B
    graph.add_reference(reference("b_func", "src/module_a.rs", 3));
    // Reference inside A itself is ignored
    graph.add_reference(reference("a_func2", "src/module_a.rs", 4));

    let deps = graph.get_module_deps("src/module_a.rs");
    assert_eq!(deps.module, "src/module_a.rs");
    assert_eq!(deps.symbols_count, 2);
    assert_eq!(deps.depends_on.len(), 1);
    assert_eq!(deps.depends_on.get("src/module_b.rs"), Some(&1));
    assert_eq!(deps.dependents.len(), 2);
    assert!(deps.dependents.contains_key("src/module_b.rs"));
    assert!(deps.dependents.contains_key("src/module_c.rs"));
}

#[test]
fn test_module_dependencies_directory_prefix() {
    let mut graph = Codegraph::new();

    graph.add_symbol(func("login", "src/auth/login.rs", 1));
    graph.add_symbol(func("token", "src/auth/token.rs", 1));
    graph.add_symbol(func("query", "src/db/query.rs", 1));
    graph.add_symbol(func("other", "src/authz.rs", 1));

    graph.add_reference(reference("token", "src/auth/login.rs", 2)); // internal
    graph.add_reference(reference("query", "src/auth/token.rs", 3)); // outgoing
    graph.add_reference(reference("login", "src/main.rs", 7)); // incoming

    let deps = graph.get_module_deps("src/auth/");
    assert_eq!(deps.files, vec!["src/auth/login.rs", "src/auth/token.rs"]);
    assert_eq!(deps.symbols_count, 2);
    assert_eq!(
        deps.depends_on.keys().collect::<Vec<_>>(),
        vec!["src/db/query.rs"]
    );
    assert_eq!(
        deps.dependents.keys().collect::<Vec<_>>(),
        vec!["src/main.rs"]
    );
}

#[test]
fn test_callees_with_enclosing_range() {
    let mut graph = Codegraph::new();

    graph.add_symbol(func("caller", "src/lib.rs", 10));
    graph.set_body(
        "caller",
        Range {
            start_line: 10,
            end_line: 15,
            ..Range::default()
        },
    );
    graph.add_symbol(func("helper", "src/util.rs", 1));
    graph.add_symbol(func("other", "src/util.rs", 5));

    graph.add_reference(reference("helper", "src/lib.rs", 12)); // inside body
    graph.add_reference(reference("other", "src/lib.rs", 20)); // after body
    graph.add_reference(reference("other", "src/main.rs", 12)); // other file
    graph.add_reference(reference("local 1", "src/lib.rs", 13)); // local, ignored

    let callees = graph.get_callees("caller");
    assert_eq!(callees.len(), 1);
    assert_eq!(callees[0].symbol_id, "helper");
    assert_eq!(callees[0].line, 12);
}

#[test]
fn test_callees_from_scip_index() {
    use codegraph_server::scip;
    use prost::Message;

    let definition = scip::SymbolRole::Definition as i32;
    let caller = "scip-python python pkg 1.0 mod/caller().";
    let helper = "scip-python python pkg 1.0 mod/helper().";
    let next = "scip-python python pkg 1.0 mod/next().";

    let occ = |symbol: &str, range: Vec<i32>, roles: i32, enclosing: Vec<i32>| scip::Occurrence {
        symbol: symbol.to_string(),
        range,
        symbol_roles: roles,
        enclosing_range: enclosing,
        ..Default::default()
    };

    let index = scip::Index {
        documents: vec![scip::Document {
            relative_path: "mod.py".to_string(),
            symbols: [caller, helper, next]
                .iter()
                .map(|s| scip::SymbolInformation {
                    symbol: s.to_string(),
                    ..Default::default()
                })
                .collect(),
            occurrences: vec![
                occ(helper, vec![0, 4, 10], definition, vec![0, 0, 1, 10]),
                occ(caller, vec![3, 4, 10], definition, vec![3, 0, 6, 0]),
                occ(helper, vec![4, 4, 10], 0, vec![]),
                occ(next, vec![8, 4, 8], definition, vec![]),
                occ(helper, vec![9, 4, 10], 0, vec![]),
            ],
            ..Default::default()
        }],
        ..Default::default()
    };

    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("index.scip");
    fs::write(&path, index.encode_to_vec()).unwrap();

    let graph = Codegraph::load_from_scip(&path).unwrap();

    // Symbol ranges come from definition occurrences
    assert_eq!(graph.get_symbol(caller).unwrap().range.start_line, 3);

    // caller's body (lines 3-6) references helper on line 4 only
    let callees = graph.get_callees(caller);
    assert_eq!(callees.len(), 1);
    assert_eq!(callees[0].symbol_id, helper);
    assert_eq!(callees[0].line, 4);

    // next has no enclosing_range: body runs to end of file
    let callees = graph.get_callees(next);
    assert_eq!(callees.len(), 1);
    assert_eq!(callees[0].line, 9);
}

#[test]
fn test_merge_graphs() {
    let mut a = Codegraph::new();
    a.add_symbol(func("py", "app.py", 1));
    a.add_reference(reference("py", "main.py", 2));

    let mut b = Codegraph::new();
    b.add_symbol(func("ts", "app.ts", 1));
    b.add_reference(reference("ts", "main.ts", 2));

    a.merge(b);
    assert_eq!(a.symbols_count(), 2);
    assert_eq!(a.files_count(), 2);
    assert_eq!(a.get_callers("ts").len(), 1);
}

#[test]
fn test_find_hotspots() {
    let mut graph = Codegraph::new();

    // Add a hotspot function (called many times)
    graph.add_symbol(Symbol {
        id: "hot".to_string(),
        kind: SymbolKind::Function,
        name: "hot_function".to_string(),
        file: "src/core.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    // Add a regular function (called few times)
    graph.add_symbol(Symbol {
        id: "cold".to_string(),
        kind: SymbolKind::Function,
        name: "cold_function".to_string(),
        file: "src/utils.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    // Add many references to hot function
    for i in 0..10 {
        graph.add_reference(Reference {
            symbol_id: "hot".to_string(),
            file: format!("src/caller_{}.rs", i),
            line: i as u32,
            role: ReferenceRole::Call,
        });
    }

    // Add few references to cold function
    graph.add_reference(Reference {
        symbol_id: "cold".to_string(),
        file: "src/main.rs".to_string(),
        line: 1,
        role: ReferenceRole::Call,
    });

    // Find hotspots with >= 5 callers
    let hotspots = graph.find_hotspots(5, None);
    assert_eq!(hotspots.len(), 1);
    assert_eq!(hotspots[0].name, "hot_function");
    assert_eq!(hotspots[0].caller_count, 10);

    // Find hotspots with path filter
    let hotspots = graph.find_hotspots(1, Some("core"));
    assert_eq!(hotspots.len(), 1);
    assert_eq!(hotspots[0].file, "src/core.rs");
}

#[test]
fn test_json_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let json_path = temp_dir.path().join("graph.json");

    // Create a graph with data
    let mut graph = Codegraph::new();
    graph.add_symbol(Symbol {
        id: "test_sym".to_string(),
        kind: SymbolKind::Function,
        name: "test_function".to_string(),
        file: "src/test.rs".to_string(),
        range: Range {
            start_line: 1,
            start_column: 0,
            end_line: 5,
            end_column: 1,
        },
        documentation: Some("Test doc".to_string()),
    });

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&graph).unwrap();
    fs::write(&json_path, &json).unwrap();

    // Load from JSON
    let loaded = Codegraph::load_from_json(&json_path).unwrap();

    assert_eq!(loaded.symbols_count(), 1);
    let sym = loaded.get_symbol("test_sym").unwrap();
    assert_eq!(sym.name, "test_function");
    assert_eq!(sym.documentation, Some("Test doc".to_string()));
}

#[test]
fn test_symbol_kind_variants() {
    assert_eq!(SymbolKind::Function, SymbolKind::Function);
    assert_ne!(SymbolKind::Function, SymbolKind::Class);

    // Test all variants compile
    let _kinds = [
        SymbolKind::Class,
        SymbolKind::Function,
        SymbolKind::Method,
        SymbolKind::Variable,
        SymbolKind::Constant,
        SymbolKind::Module,
        SymbolKind::Interface,
        SymbolKind::Type,
        SymbolKind::Property,
        SymbolKind::Unknown,
    ];
}

#[test]
fn test_reference_role_variants() {
    assert_eq!(ReferenceRole::Definition, ReferenceRole::Definition);
    assert_ne!(ReferenceRole::Call, ReferenceRole::Import);

    // Test all variants compile
    let _roles = [
        ReferenceRole::Definition,
        ReferenceRole::Reference,
        ReferenceRole::Call,
        ReferenceRole::Import,
        ReferenceRole::TypeReference,
    ];
}

// ============================================================================
// Server Handler Tests
// ============================================================================

use codegraph_server::server::CodegraphServer;
use rmcp::handler::server::ServerHandler;

#[test]
fn test_server_creation_and_info() {
    let server = CodegraphServer::new();
    let info = server.get_info();

    assert_eq!(info.server_info.name, "codegraph");
    assert_eq!(info.server_info.version, "0.1.0");
    assert!(info.instructions.is_some());
    assert!(info.instructions.unwrap().contains("Codegraph"));
}

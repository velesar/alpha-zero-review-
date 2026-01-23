//! Integration tests for Codegraph Server
//!
//! These tests verify the core functionality of the codegraph
//! including symbol management, reference tracking, and impact analysis.

use codegraph_server::graph::{
    Codegraph, Range, Reference, ReferenceRole,
    Symbol, SymbolKind,
};
use tempfile::TempDir;
use std::fs;

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

#[test]
fn test_module_dependencies() {
    let mut graph = Codegraph::new();

    // Add symbols in module A
    graph.add_symbol(Symbol {
        id: "mod_a_func1".to_string(),
        kind: SymbolKind::Function,
        name: "func1".to_string(),
        file: "src/module_a.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    graph.add_symbol(Symbol {
        id: "mod_a_func2".to_string(),
        kind: SymbolKind::Function,
        name: "func2".to_string(),
        file: "src/module_a.rs".to_string(),
        range: Range::default(),
        documentation: None,
    });

    // Module A references symbols in other modules
    graph.add_reference(Reference {
        symbol_id: "mod_a_func1".to_string(),
        file: "src/module_b.rs".to_string(),
        line: 5,
        role: ReferenceRole::Call,
    });

    graph.add_reference(Reference {
        symbol_id: "mod_a_func1".to_string(),
        file: "src/module_c.rs".to_string(),
        line: 10,
        role: ReferenceRole::Call,
    });

    let deps = graph.get_module_deps("src/module_a.rs");
    assert_eq!(deps.module, "src/module_a.rs");
    assert_eq!(deps.symbols_count, 2);
    assert!(deps.depends_on.contains_key("src/module_b.rs"));
    assert!(deps.depends_on.contains_key("src/module_c.rs"));
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
    let _kinds = vec![
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
    let _roles = vec![
        ReferenceRole::Definition,
        ReferenceRole::Reference,
        ReferenceRole::Call,
        ReferenceRole::Import,
        ReferenceRole::TypeReference,
    ];
}

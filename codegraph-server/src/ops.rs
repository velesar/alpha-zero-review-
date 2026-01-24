//! Business logic operations for codegraph-server
//!
//! This module contains pure functions extracted from MCP handlers
//! to enable unit testing without rmcp infrastructure.

use crate::graph::{Codegraph, SymbolKind};
use std::collections::HashSet;
use std::path::Path;

/// Impact analysis result
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImpactAnalysis {
    pub symbol_id: String,
    pub symbol_name: String,
    pub direct_callers: usize,
    pub affected_files: Vec<String>,
    pub change_risk: String,
}

/// Analyze the impact of changing a symbol
pub fn analyze_impact(graph: &Codegraph, symbol_id: &str) -> Option<ImpactAnalysis> {
    let symbol = graph.get_symbol(symbol_id)?;
    let callers = graph.get_callers(symbol_id);

    let affected_files: Vec<String> = callers
        .iter()
        .map(|r| r.file.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let change_risk = if callers.len() > 20 {
        "HIGH"
    } else if callers.len() > 5 {
        "MEDIUM"
    } else {
        "LOW"
    }
    .to_string();

    Some(ImpactAnalysis {
        symbol_id: symbol_id.to_string(),
        symbol_name: symbol.name.clone(),
        direct_callers: callers.len(),
        affected_files,
        change_risk,
    })
}

/// Hotspot symbol info
#[derive(Debug, Clone, serde::Serialize)]
pub struct HotspotSymbol {
    pub id: String,
    pub name: String,
    pub kind: SymbolKind,
    pub file: String,
    pub caller_count: usize,
}

/// Find symbols with many callers (hotspots)
pub fn find_hotspots(
    graph: &Codegraph,
    min_callers: usize,
    path_filter: Option<&str>,
) -> Vec<HotspotSymbol> {
    let mut hotspots = Vec::new();

    for (id, symbol) in &graph.symbols {
        // Apply path filter if provided
        if let Some(filter) = path_filter {
            if !symbol.file.contains(filter) {
                continue;
            }
        }

        let callers = graph.get_callers(id);
        if callers.len() >= min_callers {
            hotspots.push(HotspotSymbol {
                id: id.clone(),
                name: symbol.name.clone(),
                kind: symbol.kind.clone(),
                file: symbol.file.clone(),
                caller_count: callers.len(),
            });
        }
    }

    // Sort by caller count descending
    hotspots.sort_by(|a, b| b.caller_count.cmp(&a.caller_count));
    hotspots
}

/// Module dependency info
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleDependencies {
    pub module: String,
    pub dependencies: Vec<String>,
    pub dependency_count: usize,
}

/// Get dependencies of a module/file
pub fn get_module_dependencies(graph: &Codegraph, module_path: &str) -> ModuleDependencies {
    let symbols = graph.get_file_symbols(module_path);
    let mut deps = HashSet::new();

    for symbol in &symbols {
        let callees = graph.get_callees(&symbol.id);
        for callee in callees {
            // Only include if from a different file
            if callee.file != module_path {
                deps.insert(callee.file.clone());
            }
        }
    }

    let dependencies: Vec<String> = deps.into_iter().collect();
    let dependency_count = dependencies.len();

    ModuleDependencies {
        module: module_path.to_string(),
        dependencies,
        dependency_count,
    }
}

/// Detect index file format from path
pub fn detect_index_format(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| match ext.to_lowercase().as_str() {
            "scip" => Some("scip"),
            "json" => Some("json"),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_index_format_scip() {
        assert_eq!(
            detect_index_format(Path::new("index.scip")),
            Some("scip")
        );
    }

    #[test]
    fn test_detect_index_format_json() {
        assert_eq!(
            detect_index_format(Path::new("graph.json")),
            Some("json")
        );
    }

    #[test]
    fn test_detect_index_format_unknown() {
        assert_eq!(detect_index_format(Path::new("file.txt")), None);
    }

    #[test]
    fn test_find_hotspots_empty() {
        let graph = Codegraph::new();
        let hotspots = find_hotspots(&graph, 5, None);
        assert!(hotspots.is_empty());
    }

    #[test]
    fn test_get_module_dependencies_empty() {
        let graph = Codegraph::new();
        let deps = get_module_dependencies(&graph, "src/main.rs");
        assert_eq!(deps.dependency_count, 0);
    }
}

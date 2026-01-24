//! Codegraph MCP Server implementation
//!
//! This module implements the MCP server that provides SCIP-based
//! semantic code intelligence.

use crate::graph::Codegraph;
use crate::index_manager::{IndexManager, Language};
use rmcp::{
    handler::server::{
        tool::{Parameters, ToolCallContext, ToolRouter},
        ServerHandler,
    },
    model::{
        CallToolRequestParam, CallToolResult, Content, ErrorData, ListToolsResult,
        PaginatedRequestParam, ServerCapabilities, ServerInfo,
    },
    schemars, tool, tool_router,
    service::{RequestContext, RoleServer},
};
use std::future::Future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Codegraph MCP Server
#[derive(Clone)]
pub struct CodegraphServer {
    /// Single graph for backward compatibility (explicit load_index)
    graph: Arc<RwLock<Option<Codegraph>>>,
    /// Multiple graphs keyed by language (auto-loaded)
    graphs: Arc<RwLock<HashMap<Language, Codegraph>>>,
    /// Project path for index management
    project_path: Arc<RwLock<Option<PathBuf>>>,
    tool_router: ToolRouter<Self>,
}

/// Input for load_project_indexes
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoadProjectIndexesInput {
    /// Path to the project directory (must have .audit/indexes/)
    pub project_path: String,
    /// Build missing indexes on-demand if indexer is available (default: false)
    #[serde(default)]
    pub build_if_missing: bool,
}

/// Input for load_index
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LoadIndexInput {
    /// Path to SCIP index file (.scip) or JSON codegraph file (.json)
    pub scip_path: String,
}

/// Output for load_index
#[derive(Debug, Serialize)]
pub struct LoadIndexOutput {
    pub status: String,
    pub symbols_count: usize,
    pub files_count: usize,
}

/// Input for get_symbol_info
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SymbolInput {
    /// Symbol identifier
    pub symbol_id: String,
}

/// Input for get_file_symbols
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FileInput {
    /// Path to file
    pub file_path: String,
}

/// Input for find_symbol
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindSymbolInput {
    /// Search pattern (case-insensitive substring match)
    pub pattern: String,
}

/// Input for find_hotspot_symbols
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct FindHotspotsInput {
    /// Minimum number of callers to qualify as a hotspot
    pub min_callers: usize,
    /// Optional path filter (only include symbols from matching paths)
    pub path_filter: Option<String>,
}

/// Input for get_module_deps
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ModuleDepsInput {
    /// Path to module/file
    pub module_path: String,
}

/// Callers response
#[derive(Debug, Serialize)]
pub struct CallersResponse {
    pub caller_count: usize,
    pub callers: Vec<CallerInfo>,
}

#[derive(Debug, Serialize)]
pub struct CallerInfo {
    pub file: String,
    pub line: u32,
    pub role: String,
}

#[tool_router]
impl CodegraphServer {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(None)),
            graphs: Arc::new(RwLock::new(HashMap::new())),
            project_path: Arc::new(RwLock::new(None)),
            tool_router: Self::tool_router(),
        }
    }

    /// Helper to get any available graph (prefers multi-graph, falls back to single)
    #[allow(dead_code)] // Helper for future multi-graph support
    fn get_any_graph(&self) -> Result<std::sync::RwLockReadGuard<'_, Option<Codegraph>>, rmcp::ErrorData> {
        // First check if we have graphs loaded via load_project_indexes
        let graphs = self.graphs.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        if !graphs.is_empty() {
            // We have multi-graphs, but this helper returns Option<Codegraph>
            // For now, we'll fall through to single graph logic
            // In a real impl, we'd merge or pick the right one
            drop(graphs);
        }

        self.graph.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })
    }

    /// Validate that a path is safe to access
    ///
    /// Ensures the path:
    /// - Exists
    /// - Is within the project directory or .audit/indexes/ directory
    /// - Does not traverse outside allowed directories
    fn validate_index_path(&self, path: &std::path::Path) -> Result<PathBuf, rmcp::ErrorData> {
        // Canonicalize to resolve symlinks and relative paths
        let canonical = path.canonicalize().map_err(|e| {
            rmcp::ErrorData::invalid_params(
                format!("Cannot resolve path {}: {}", path.display(), e),
                None,
            )
        })?;

        // Check if we have a project path set
        if let Ok(project_guard) = self.project_path.read() {
            if let Some(project_path) = project_guard.as_ref() {
                let project_canonical = project_path.canonicalize().unwrap_or_else(|_| project_path.clone());

                // Allow paths within project directory or its .audit subdirectory
                if !canonical.starts_with(&project_canonical) {
                    return Err(rmcp::ErrorData::invalid_params(
                        format!(
                            "Path {} is outside project directory {}",
                            canonical.display(),
                            project_canonical.display()
                        ),
                        None,
                    ));
                }
            }
        }

        // Ensure it's a file, not a directory
        if canonical.is_dir() {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Path {} is a directory, expected a file", canonical.display()),
                None,
            ));
        }

        // Ensure it has an allowed extension
        let extension = canonical.extension().and_then(|e| e.to_str()).unwrap_or("");
        if extension != "scip" && extension != "json" {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Unsupported file type: .{}. Expected .scip or .json", extension),
                None,
            ));
        }

        Ok(canonical)
    }

    /// Load a SCIP index file
    #[tool(description = "Load a SCIP index file (.scip) or JSON codegraph file for semantic analysis")]
    async fn load_index(&self, input: Parameters<LoadIndexInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let path = PathBuf::from(&input.scip_path);

        // Validate path before loading
        let validated_path = if path.exists() {
            self.validate_index_path(&path)?
        } else {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Index file not found: {}", input.scip_path),
                None,
            ));
        };

        let graph = if validated_path.extension().map(|e| e == "json").unwrap_or(false) {
            Codegraph::load_from_json(&validated_path)
        } else {
            Codegraph::load_from_scip(&validated_path)
        }.map_err(|e| rmcp::ErrorData::internal_error(format!("Failed to load index: {}", e), None))?;

        let output = LoadIndexOutput {
            status: "loaded".to_string(),
            symbols_count: graph.symbols_count(),
            files_count: graph.files_count(),
        };

        *self.graph.write().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })? = Some(graph);

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Load all project indexes from .audit/indexes/
    #[tool(description = "Auto-load all SCIP indexes from .audit/indexes/ directory. Optionally builds missing indexes if indexer is available.")]
    async fn load_project_indexes(
        &self,
        input: Parameters<LoadProjectIndexesInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let project_path = PathBuf::from(&input.project_path);

        if !project_path.exists() {
            return Err(rmcp::ErrorData::invalid_params(
                format!("Project path not found: {}", input.project_path),
                None,
            ));
        }

        // Store project path for future reference
        *self.project_path.write().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })? = Some(project_path.clone());

        let manager = IndexManager::new(project_path);
        let result = manager.auto_load_or_build(input.build_if_missing);

        // Load all discovered indexes into our graphs map
        let mut loaded_count = 0;
        let mut total_symbols = 0;
        let mut total_files = 0;

        for status in &result.loaded {
            if status.exists {
                match Codegraph::load_from_scip(&status.path) {
                    Ok(graph) => {
                        total_symbols += graph.symbols_count();
                        total_files += graph.files_count();
                        self.graphs
                            .write()
                            .map_err(|e| {
                                rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
                            })?
                            .insert(status.language, graph);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        // Add to warnings but continue
                        tracing::warn!("Failed to load {} index: {}", status.language, e);
                    }
                }
            }
        }

        // Also set the first loaded graph as the default single graph for backward compatibility
        if loaded_count > 0 {
            if let Some(status) = result.loaded.first() {
                if let Ok(graph) = Codegraph::load_from_scip(&status.path) {
                    *self.graph.write().map_err(|e| {
                        rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
                    })? = Some(graph);
                }
            }
        }

        let response = serde_json::json!({
            "status": if loaded_count > 0 { "loaded" } else { "no_indexes" },
            "loaded_count": loaded_count,
            "total_symbols": total_symbols,
            "total_files": total_files,
            "languages": result.loaded.iter().map(|s| s.language.to_string()).collect::<Vec<_>>(),
            "missing": result.missing.iter().map(|l| l.to_string()).collect::<Vec<_>>(),
            "warnings": result.warnings,
        });

        let json = serde_json::to_string_pretty(&response)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get information about a symbol
    #[tool(description = "Get detailed information about a symbol by its ID")]
    async fn get_symbol_info(&self, input: Parameters<SymbolInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let graph = self.graph.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let graph = graph.as_ref().ok_or_else(|| {
            rmcp::ErrorData::internal_error("No index loaded. Call load_index or load_project_indexes first.".to_string(), None)
        })?;

        let symbol = graph.get_symbol(&input.symbol_id).ok_or_else(|| {
            rmcp::ErrorData::invalid_params(format!("Symbol not found: {}", input.symbol_id), None)
        })?;

        let json = serde_json::to_string_pretty(&symbol)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get all callers of a symbol
    #[tool(description = "Get all locations where a symbol is called/referenced")]
    async fn get_callers(&self, input: Parameters<SymbolInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let graph = self.graph.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let graph = graph.as_ref().ok_or_else(|| {
            rmcp::ErrorData::internal_error("No index loaded. Call load_index first.".to_string(), None)
        })?;

        let callers = graph.get_callers(&input.symbol_id);

        let response = CallersResponse {
            caller_count: callers.len(),
            callers: callers
                .iter()
                .map(|r| CallerInfo {
                    file: r.file.clone(),
                    line: r.line,
                    role: format!("{:?}", r.role),
                })
                .collect(),
        };

        let json = serde_json::to_string_pretty(&response)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get callees from a symbol
    #[tool(description = "Get all symbols called/referenced from within a symbol's definition")]
    async fn get_callees(&self, input: Parameters<SymbolInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let graph = self.graph.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let graph = graph.as_ref().ok_or_else(|| {
            rmcp::ErrorData::internal_error("No index loaded. Call load_index first.".to_string(), None)
        })?;

        let callees = graph.get_callees(&input.symbol_id);

        let response = CallersResponse {
            caller_count: callees.len(),
            callers: callees
                .iter()
                .map(|r| CallerInfo {
                    file: r.file.clone(),
                    line: r.line,
                    role: format!("{:?}", r.role),
                })
                .collect(),
        };

        let json = serde_json::to_string_pretty(&response)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get impact analysis for a symbol
    #[tool(description = "Analyze the impact of changing a symbol (how many files/references would be affected)")]
    async fn get_impact(&self, input: Parameters<SymbolInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let graph = self.graph.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let graph = graph.as_ref().ok_or_else(|| {
            rmcp::ErrorData::internal_error("No index loaded. Call load_index first.".to_string(), None)
        })?;

        let impact = graph.get_impact(&input.symbol_id);

        let json = serde_json::to_string_pretty(&impact)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get module dependencies
    #[tool(description = "Get dependencies of a module/file (what other files it depends on)")]
    async fn get_module_deps(&self, input: Parameters<ModuleDepsInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let graph = self.graph.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let graph = graph.as_ref().ok_or_else(|| {
            rmcp::ErrorData::internal_error("No index loaded. Call load_index first.".to_string(), None)
        })?;

        let deps = graph.get_module_deps(&input.module_path);

        let json = serde_json::to_string_pretty(&deps)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Get all symbols in a file
    #[tool(description = "Get all symbols defined in a specific file")]
    async fn get_file_symbols(&self, input: Parameters<FileInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let graph = self.graph.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let graph = graph.as_ref().ok_or_else(|| {
            rmcp::ErrorData::internal_error("No index loaded. Call load_index first.".to_string(), None)
        })?;

        let symbols = graph.get_file_symbols(&input.file_path);

        let json = serde_json::to_string_pretty(&symbols)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Found {} symbols in {}\n\n{}",
            symbols.len(),
            input.file_path,
            json
        ))]))
    }

    /// Find symbols by pattern
    #[tool(description = "Search for symbols by name pattern (case-insensitive substring match)")]
    async fn find_symbol(&self, input: Parameters<FindSymbolInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let graph = self.graph.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let graph = graph.as_ref().ok_or_else(|| {
            rmcp::ErrorData::internal_error("No index loaded. Call load_index first.".to_string(), None)
        })?;

        let symbols = graph.find_symbol(&input.pattern);

        let json = serde_json::to_string_pretty(&symbols)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Found {} symbols matching '{}'\n\n{}",
            symbols.len(),
            input.pattern,
            json
        ))]))
    }

    /// Find hotspot symbols
    #[tool(description = "Find symbols with many callers (hotspots that may need careful attention during changes)")]
    async fn find_hotspot_symbols(&self, input: Parameters<FindHotspotsInput>) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        let graph = self.graph.read().map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Lock error: {}", e), None)
        })?;

        let graph = graph.as_ref().ok_or_else(|| {
            rmcp::ErrorData::internal_error("No index loaded. Call load_index first.".to_string(), None)
        })?;

        let hotspots = graph.find_hotspots(input.min_callers, input.path_filter.as_deref());

        let json = serde_json::to_string_pretty(&hotspots)
            .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Found {} hotspot symbols with >= {} callers\n\n{}",
            hotspots.len(),
            input.min_callers,
            json
        ))]))
    }
}

impl Default for CodegraphServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for CodegraphServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: rmcp::model::Implementation {
                name: "codegraph".into(),
                version: "0.1.0".into(),
            },
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some("Codegraph MCP Server for AI Code Audit. Use this server for SCIP-based semantic code intelligence - finding symbol references, callers, impact analysis, and hotspots.".into()),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _pagination: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            next_cursor: None,
        })
    }

    fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        let tool_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = CodegraphServer::new();
        assert!(server.graph.read().unwrap().is_none());
    }
}

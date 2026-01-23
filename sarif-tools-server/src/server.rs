//! SARIF Tools MCP Server implementation
//!
//! This module implements the MCP server that provides tools for
//! running code analysis tools and working with SARIF output.

use crate::sarif::Sarif;
use crate::tools::{ToolInfo, ToolRegistry};
use anyhow::Result;
use std::future::Future;
use rmcp::{
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo, PaginatedRequestParam, ListToolsResult, ErrorData, CallToolRequestParam},
    schemars, tool,
    handler::server::{tool::{ToolRouter, Parameters, ToolCallContext}, ServerHandler},
    tool_router,
    service::{RequestContext, RoleServer},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// SARIF Tools MCP Server
#[derive(Clone)]
pub struct SarifToolsServer {
    registry: ToolRegistry,
    mappings_path: Option<PathBuf>,
    tool_router: ToolRouter<Self>,
}

/// Input for run_tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RunToolInput {
    /// Tool to run (semgrep, bandit, ruff, trivy)
    pub tool: String,
    /// Path to analyze
    pub path: String,
    /// Tool-specific configuration (optional)
    pub config: Option<serde_json::Value>,
}

/// Output for run_tool
#[derive(Debug, Serialize)]
pub struct RunToolOutput {
    pub sarif: Sarif,
    pub exit_code: i32,
    pub stderr: Option<String>,
    pub result_count: usize,
}

/// Input for merge_sarif
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeSarifInput {
    /// SARIF objects to merge
    pub sarif_files: Vec<serde_json::Value>,
}

/// Output for merge_sarif
#[derive(Debug, Serialize)]
pub struct MergeSarifOutput {
    pub combined: Sarif,
    pub total_results: usize,
    pub runs_count: usize,
}

/// Input for normalize_sarif
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NormalizeSarifInput {
    /// SARIF object to normalize
    pub sarif: serde_json::Value,
    /// Path to rule mappings YAML file (optional, uses server default if not specified)
    pub rule_mappings: Option<String>,
}

/// Input for get_tool_config
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetToolConfigInput {
    /// Tool name
    pub tool: String,
}

/// Output for get_tool_config
#[derive(Debug, Serialize)]
pub struct GetToolConfigOutput {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub supported_languages: Vec<String>,
}

/// Output for list_available_tools
#[derive(Debug, Serialize)]
pub struct ListToolsOutput {
    pub tools: Vec<ToolInfo>,
    pub installed_count: usize,
}

/// Enriched SARIF result with category information
#[derive(Debug, Clone, Serialize)]
pub struct EnrichedResult {
    pub rule_id: String,
    pub level: Option<String>,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub cwe: Option<String>,
    pub severity_base: Option<String>,
}

#[tool_router]
impl SarifToolsServer {
    pub fn new(mappings_path: Option<PathBuf>) -> Self {
        Self {
            registry: ToolRegistry::new(),
            mappings_path,
            tool_router: Self::tool_router(),
        }
    }

    /// Run a code analysis tool and get SARIF output
    #[tool(description = "Run a code analysis tool (semgrep, bandit, ruff, trivy) on a path and get SARIF output")]
    async fn run_tool(&self, input: Parameters<RunToolInput>) -> Result<CallToolResult, rmcp::Error> {
        let input = input.0;

        let runner = self.registry.get(&input.tool)
            .ok_or_else(|| rmcp::Error::invalid_params(format!("Unknown tool: {}", input.tool), None))?;

        if !runner.is_available() {
            return Err(rmcp::Error::internal_error(
                format!("Tool not installed: {}. Please install it first.", input.tool),
                None,
            ));
        }

        let path = PathBuf::from(&input.path);
        if !path.exists() {
            return Err(rmcp::Error::invalid_params(
                format!("Path does not exist: {}", input.path),
                None,
            ));
        }

        tracing::info!("Running {} on {}", input.tool, input.path);

        let result = runner.run(&path, input.config.as_ref())
            .map_err(|e| rmcp::Error::internal_error(format!("Tool execution failed: {}", e), None))?;

        let result_count = result.sarif.result_count();
        let output = RunToolOutput {
            sarif: result.sarif,
            exit_code: result.exit_code,
            stderr: result.stderr,
            result_count,
        };

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| rmcp::Error::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// List all available analysis tools
    #[tool(description = "List all available code analysis tools with their installation status")]
    async fn list_available_tools(&self) -> Result<CallToolResult, rmcp::Error> {
        let tools = self.registry.list_available();
        let installed_count = tools.iter().filter(|t| t.installed).count();

        let output = ListToolsOutput {
            tools,
            installed_count,
        };

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| rmcp::Error::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Merge multiple SARIF files into one
    #[tool(description = "Merge multiple SARIF objects into a single combined SARIF object")]
    async fn merge_sarif(&self, input: Parameters<MergeSarifInput>) -> Result<CallToolResult, rmcp::Error> {
        let input = input.0;

        let mut combined = Sarif::new();

        for sarif_value in input.sarif_files {
            let sarif: Sarif = serde_json::from_value(sarif_value)
                .map_err(|e| rmcp::Error::invalid_params(format!("Invalid SARIF: {}", e), None))?;
            combined.merge(sarif);
        }

        let total_results = combined.result_count();
        let runs_count = combined.runs.len();

        let output = MergeSarifOutput {
            combined,
            total_results,
            runs_count,
        };

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| rmcp::Error::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Enrich SARIF with category and severity mappings
    #[tool(description = "Normalize and enrich SARIF results with categories, CWE IDs, and base severity from rule mappings")]
    async fn normalize_sarif(&self, input: Parameters<NormalizeSarifInput>) -> Result<CallToolResult, rmcp::Error> {
        let input = input.0;

        let sarif: Sarif = serde_json::from_value(input.sarif)
            .map_err(|e| rmcp::Error::invalid_params(format!("Invalid SARIF: {}", e), None))?;

        // Load rule mappings
        let mappings_path = input.rule_mappings
            .map(PathBuf::from)
            .or_else(|| self.mappings_path.clone());

        let mappings = if let Some(path) = mappings_path {
            load_rule_mappings(&path)
                .map_err(|e| rmcp::Error::internal_error(format!("Failed to load mappings: {}", e), None))?
        } else {
            HashMap::new()
        };

        // Enrich results
        let mut enriched_results = Vec::new();

        for run in &sarif.runs {
            let tool_name = &run.tool.driver.name;

            for result in &run.results {
                let mapping_key = format!("{}:{}", tool_name.to_lowercase(), result.rule_id);
                let fallback_key = result.rule_id.clone();

                let mapping = mappings.get(&mapping_key)
                    .or_else(|| mappings.get(&fallback_key));

                let file = result.locations.first()
                    .map(|l| l.physical_location.artifact_location.uri.clone());

                let line = result.locations.first()
                    .and_then(|l| l.physical_location.region.as_ref())
                    .and_then(|r| r.start_line);

                enriched_results.push(EnrichedResult {
                    rule_id: result.rule_id.clone(),
                    level: result.level.clone(),
                    message: result.message.text.clone(),
                    file,
                    line,
                    category: mapping.and_then(|m| m.get("category").and_then(|v| v.as_str())).map(String::from),
                    subcategory: mapping.and_then(|m| m.get("subcategory").and_then(|v| v.as_str())).map(String::from),
                    cwe: mapping.and_then(|m| m.get("cwe").and_then(|v| v.as_str())).map(String::from),
                    severity_base: mapping.and_then(|m| m.get("base_severity").and_then(|v| v.as_str())).map(String::from),
                });
            }
        }

        let json = serde_json::to_string_pretty(&enriched_results)
            .map_err(|e| rmcp::Error::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Enriched {} results\n\n{}",
            enriched_results.len(),
            json
        ))]))
    }

    /// Get configuration for a specific tool
    #[tool(description = "Get configuration and status for a specific analysis tool")]
    async fn get_tool_config(&self, input: Parameters<GetToolConfigInput>) -> Result<CallToolResult, rmcp::Error> {
        let input = input.0;

        let runner = self.registry.get(&input.tool)
            .ok_or_else(|| rmcp::Error::invalid_params(format!("Unknown tool: {}", input.tool), None))?;

        let output = GetToolConfigOutput {
            name: runner.name().to_string(),
            installed: runner.is_available(),
            version: runner.version(),
            supported_languages: runner.supported_languages(),
        };

        let json = serde_json::to_string_pretty(&output)
            .map_err(|e| rmcp::Error::internal_error(format!("Serialization error: {}", e), None))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }
}

impl ServerHandler for SarifToolsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: rmcp::model::Implementation {
                name: "sarif-tools".into(),
                version: "0.1.0".into(),
            },
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            instructions: Some("SARIF Tools MCP Server for AI Code Audit. Use this server to run code analysis tools (semgrep, bandit, ruff, trivy) and work with SARIF output format.".into()),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _pagination: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        async move {
            Ok(ListToolsResult {
                tools: self.tool_router.list_all(),
                next_cursor: None,
            })
        }
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

/// Load rule mappings from YAML file
fn load_rule_mappings(path: &PathBuf) -> Result<HashMap<String, serde_json::Value>> {
    let content = std::fs::read_to_string(path)?;

    // Parse as YAML array of mappings
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;

    let mut mappings = HashMap::new();

    if let Some(items) = yaml_value.as_sequence() {
        for item in items {
            if let (Some(tool), Some(rule_id)) = (
                item.get("tool").and_then(|v| v.as_str()),
                item.get("rule_id").and_then(|v| v.as_str()),
            ) {
                let key = format!("{}:{}", tool.to_lowercase(), rule_id);
                let json_value = serde_json::to_value(item)?;
                mappings.insert(key, json_value.clone());
                // Also insert with just rule_id for fallback
                mappings.insert(rule_id.to_string(), json_value);
            }
        }
    }

    Ok(mappings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = SarifToolsServer::new(None);
        assert!(server.registry.get("semgrep").is_some());
    }
}

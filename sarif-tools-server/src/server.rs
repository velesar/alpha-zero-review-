//! SARIF Tools MCP Server implementation
//!
//! This module implements the MCP server that provides tools for
//! running code analysis tools and working with SARIF output.
//!
//! This is the adapter layer that bridges MCP protocol to domain operations.

use crate::domain::RuleMappings;
use crate::ops;
use crate::sarif::Sarif;
use crate::tools::{ToolInfo, ToolRegistry};
use crate::utils::{
    format_json_response, format_prefixed_json_response,
    json_to_tool_config, load_rule_mappings, parse_sarif_values, parse_sarif_json,
};
use rmcp::{
    handler::server::{
        tool::{Parameters, ToolCallContext, ToolRouter},
        ServerHandler,
    },
    model::{
        CallToolRequestParam, CallToolResult, ErrorData, ListToolsResult,
        PaginatedRequestParam, ServerCapabilities, ServerInfo,
    },
    schemars,
    service::{RequestContext, RoleServer},
    tool, tool_router,
};
use serde::{Deserialize, Serialize};
use std::future::Future;
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

/// Input for install_tool
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct InstallToolInput {
    /// Tool name to install (semgrep, bandit, ruff, trivy)
    pub tool: String,
}

/// Output for install_tool
#[derive(Debug, Serialize)]
pub struct InstallToolOutput {
    pub success: bool,
    pub tool: String,
    pub message: String,
    /// The command that was attempted (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tried: Option<String>,
    /// Alternative install commands the user can try
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub alternatives: Vec<String>,
}

/// Output for list_available_tools
#[derive(Debug, Serialize)]
pub struct ListToolsOutput {
    pub tools: Vec<ToolInfo>,
    pub installed_count: usize,
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
    #[tool(
        description = "Run a code analysis tool (semgrep, bandit, ruff, trivy) on a path and get SARIF output"
    )]
    async fn run_tool(
        &self,
        input: Parameters<RunToolInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;
        let path = PathBuf::from(&input.path);

        tracing::info!("Running {} on {}", input.tool, input.path);

        // Adapter: Convert JSON config to domain type
        let tool_config = input.config.as_ref().map(json_to_tool_config);

        let result = ops::execute_tool(&self.registry, &input.tool, &path, tool_config.as_ref())?;

        let output = RunToolOutput {
            sarif: result.sarif,
            exit_code: result.exit_code,
            stderr: result.stderr,
            result_count: result.result_count,
        };

        format_json_response(&output)
    }

    /// List all available analysis tools
    #[tool(description = "List all available code analysis tools with their installation status")]
    async fn list_available_tools(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        let (tools, installed_count) = ops::list_available_tools(&self.registry);

        let output = ListToolsOutput {
            tools,
            installed_count,
        };

        format_json_response(&output)
    }

    /// Merge multiple SARIF files into one
    #[tool(description = "Merge multiple SARIF objects into a single combined SARIF object")]
    async fn merge_sarif(
        &self,
        input: Parameters<MergeSarifInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        // Adapter: Parse JSON values to Sarif domain types
        let sarif_files = parse_sarif_values(input.0.sarif_files)?;

        let result = ops::merge_sarif_files(sarif_files);

        let output = MergeSarifOutput {
            combined: result.combined,
            total_results: result.total_results,
            runs_count: result.runs_count,
        };

        format_json_response(&output)
    }

    /// Enrich SARIF with category and severity mappings
    #[tool(
        description = "Normalize and enrich SARIF results with categories, CWE IDs, and base severity from rule mappings"
    )]
    async fn normalize_sarif(
        &self,
        input: Parameters<NormalizeSarifInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let input = input.0;

        // Adapter: Parse JSON to Sarif domain type
        let sarif = parse_sarif_json(input.sarif)?;

        // Adapter: Load rule mappings to domain type
        let mappings_path = input
            .rule_mappings
            .map(PathBuf::from)
            .or_else(|| self.mappings_path.clone());

        let mappings = if let Some(path) = mappings_path {
            load_rule_mappings(&path)?
        } else {
            RuleMappings::new()
        };

        let result = ops::normalize_sarif(sarif, &mappings);

        format_prefixed_json_response(
            &format!("Enriched {} results", result.count),
            &result.enriched_results,
        )
    }

    /// Get configuration for a specific tool
    #[tool(description = "Get configuration and status for a specific analysis tool")]
    async fn get_tool_config(
        &self,
        input: Parameters<GetToolConfigInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let config = ops::get_tool_config(&self.registry, &input.0.tool)?;
        format_json_response(&config)
    }

    /// Install a code analysis tool
    #[tool(description = "Install a code analysis tool (semgrep, bandit, ruff, trivy). Automatically detects available package managers.")]
    async fn install_tool(
        &self,
        input: Parameters<InstallToolInput>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let tool_name = &input.0.tool;

        tracing::info!("Attempting to install tool: {}", tool_name);

        let runner = self.registry.get(tool_name).ok_or_else(|| {
            rmcp::ErrorData::invalid_params(format!("Unknown tool: {}", tool_name), None)
        })?;

        // Check if already installed
        if runner.is_available() {
            let version = runner.version().unwrap_or_else(|| "unknown".to_string());
            return format_json_response(&InstallToolOutput {
                success: true,
                tool: tool_name.clone(),
                message: format!("{} is already installed (version: {})", tool_name, version),
                tried: None,
                alternatives: vec![],
            });
        }

        // Get all install commands for this tool
        let install_commands = runner.install_commands();

        if install_commands.is_empty() {
            return format_json_response(&InstallToolOutput {
                success: false,
                tool: tool_name.clone(),
                message: format!(
                    "{} does not support automatic installation. For clippy: rustup component add clippy",
                    tool_name
                ),
                tried: None,
                alternatives: vec![],
            });
        }

        // Collect all command strings for alternatives list
        let all_commands: Vec<String> = install_commands
            .iter()
            .map(|cmd| cmd.to_command_string())
            .collect();

        // Find first available package manager
        let available_cmd = install_commands.iter().find(|cmd| cmd.is_available());

        let Some(install_cmd) = available_cmd else {
            // No package managers available - return all as instructions
            return format_json_response(&InstallToolOutput {
                success: false,
                tool: tool_name.clone(),
                message: "No supported package manager found. Install one of the following manually:".to_string(),
                tried: None,
                alternatives: all_commands,
            });
        };

        let tried_cmd = install_cmd.to_command_string();

        // Execute installation (hybrid: try first available, return alternatives on failure)
        match install_cmd.execute() {
            Ok(()) => {
                // Verify installation
                if runner.is_available() {
                    let version = runner.version().unwrap_or_else(|| "unknown".to_string());
                    format_json_response(&InstallToolOutput {
                        success: true,
                        tool: tool_name.clone(),
                        message: format!("Successfully installed {} (version: {})", tool_name, version),
                        tried: Some(tried_cmd),
                        alternatives: vec![],
                    })
                } else {
                    format_json_response(&InstallToolOutput {
                        success: false,
                        tool: tool_name.clone(),
                        message: format!(
                            "Installation completed but {} is not available in PATH. You may need to restart your shell or add ~/.local/bin to PATH.",
                            tool_name
                        ),
                        tried: Some(tried_cmd),
                        alternatives: all_commands,
                    })
                }
            }
            Err(e) => {
                // Failed - return error with alternatives
                format_json_response(&InstallToolOutput {
                    success: false,
                    tool: tool_name.clone(),
                    message: format!("Installation failed: {}", e),
                    tried: Some(tried_cmd),
                    alternatives: all_commands,
                })
            }
        }
    }
}

impl ServerHandler for SarifToolsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: rmcp::model::Implementation {
                name: "sarif-tools".into(),
                version: "0.1.0".into(),
            },
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(
                "SARIF Tools MCP Server for AI Code Audit. Use this server to run code analysis tools (semgrep, bandit, ruff, trivy) and work with SARIF output format.".into(),
            ),
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
        let server = SarifToolsServer::new(None);
        assert!(server.registry.get("semgrep").is_some());
    }
}

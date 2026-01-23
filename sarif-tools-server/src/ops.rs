//! Business logic operations for sarif-tools-server
//!
//! This module contains pure functions extracted from MCP handlers
//! to enable unit testing without rmcp infrastructure.
//!
//! **Domain purity:** This module uses only domain types and avoids
//! framework dependencies like serde_json in function signatures.
//! Serialization/deserialization happens at adapter boundaries.

use crate::domain::{RuleMappings, ToolConfig};
use crate::error::ToolError;
use crate::sarif::Sarif;
use crate::tools::{ToolInfo, ToolRegistry};
use std::path::Path;

/// Output from tool execution
#[derive(Debug)]
pub struct ExecuteToolOutput {
    pub sarif: Sarif,
    pub exit_code: i32,
    pub stderr: Option<String>,
    pub result_count: usize,
}

/// Output from merging SARIF files
#[derive(Debug)]
pub struct MergeSarifOutput {
    pub combined: Sarif,
    pub total_results: usize,
    pub runs_count: usize,
}

/// Enriched SARIF result with category information
///
/// Note: serde::Serialize is needed for MCP response formatting,
/// but the ops layer does not depend on serde_json::Value in APIs.
#[derive(Debug, Clone, serde::Serialize)]
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

/// Output from normalizing SARIF
#[derive(Debug)]
pub struct NormalizeSarifOutput {
    pub enriched_results: Vec<EnrichedResult>,
    pub count: usize,
}

/// Tool configuration output
///
/// Note: serde::Serialize is needed for MCP response formatting.
#[derive(Debug, serde::Serialize)]
pub struct ToolConfigOutput {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub supported_languages: Vec<String>,
}

/// Execute a code analysis tool
///
/// Takes domain types only - JSON conversion happens at adapter layer.
pub fn execute_tool(
    registry: &ToolRegistry,
    tool_name: &str,
    path: &Path,
    config: Option<&ToolConfig>,
) -> Result<ExecuteToolOutput, ToolError> {
    let runner = registry
        .get(tool_name)
        .ok_or_else(|| ToolError::UnknownTool(tool_name.to_string()))?;

    if !runner.is_available() {
        return Err(ToolError::ToolNotInstalled(tool_name.to_string()));
    }

    if !path.exists() {
        return Err(ToolError::PathNotFound(path.display().to_string()));
    }

    let result = runner
        .run(path, config)
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

    let result_count = result.sarif.result_count();

    Ok(ExecuteToolOutput {
        sarif: result.sarif,
        exit_code: result.exit_code,
        stderr: result.stderr,
        result_count,
    })
}

/// Merge multiple SARIF objects into one
///
/// Takes domain types directly - JSON parsing happens at adapter layer.
pub fn merge_sarif_files(sarif_files: Vec<Sarif>) -> MergeSarifOutput {
    let mut combined = Sarif::new();

    for sarif in sarif_files {
        combined.merge(sarif);
    }

    let total_results = combined.result_count();
    let runs_count = combined.runs.len();

    MergeSarifOutput {
        combined,
        total_results,
        runs_count,
    }
}

/// Normalize and enrich SARIF results with category mappings
///
/// Takes domain types only - uses RuleMappings instead of HashMap<String, Value>.
pub fn normalize_sarif(sarif: Sarif, mappings: &RuleMappings) -> NormalizeSarifOutput {
    let mut enriched_results = Vec::new();

    for run in &sarif.runs {
        let tool_name = &run.tool.driver.name;

        for result in &run.results {
            let mapping = mappings.get(tool_name, &result.rule_id);

            let file = result
                .locations
                .first()
                .map(|l| l.physical_location.artifact_location.uri.clone());

            let line = result
                .locations
                .first()
                .and_then(|l| l.physical_location.region.as_ref())
                .and_then(|r| r.start_line);

            enriched_results.push(EnrichedResult {
                rule_id: result.rule_id.clone(),
                level: result.level.clone(),
                message: result.message.text.clone(),
                file,
                line,
                category: mapping.and_then(|m| m.category.clone()),
                subcategory: mapping.and_then(|m| m.subcategory.clone()),
                cwe: mapping.and_then(|m| m.cwe.clone()),
                severity_base: mapping.and_then(|m| m.base_severity.clone()),
            });
        }
    }

    let count = enriched_results.len();
    NormalizeSarifOutput {
        enriched_results,
        count,
    }
}

/// Get configuration for a specific tool
pub fn get_tool_config(registry: &ToolRegistry, tool_name: &str) -> Result<ToolConfigOutput, ToolError> {
    let runner = registry
        .get(tool_name)
        .ok_or_else(|| ToolError::UnknownTool(tool_name.to_string()))?;

    Ok(ToolConfigOutput {
        name: runner.name().to_string(),
        installed: runner.is_available(),
        version: runner.version(),
        supported_languages: runner.supported_languages(),
    })
}

/// List all available tools
pub fn list_available_tools(registry: &ToolRegistry) -> (Vec<ToolInfo>, usize) {
    let tools = registry.list_available();
    let installed_count = tools.iter().filter(|t| t.installed).count();
    (tools, installed_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RuleMapping;

    #[test]
    fn test_execute_tool_unknown() {
        let registry = ToolRegistry::new();
        let result = execute_tool(&registry, "nonexistent", Path::new("."), None);
        assert!(matches!(result, Err(ToolError::UnknownTool(_))));
    }

    #[test]
    fn test_execute_tool_path_not_found() {
        let registry = ToolRegistry::new();
        let result = execute_tool(&registry, "ruff", Path::new("/nonexistent/path/xyz"), None);
        // Either tool not installed or path not found
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_sarif_empty() {
        let output = merge_sarif_files(vec![]);
        assert_eq!(output.total_results, 0);
        assert_eq!(output.runs_count, 0);
    }

    #[test]
    fn test_merge_sarif_multiple() {
        let sarif1 = Sarif::new();
        let sarif2 = Sarif::new();
        let output = merge_sarif_files(vec![sarif1, sarif2]);
        assert_eq!(output.total_results, 0);
        // Empty Sarifs have no runs, so runs_count is 0
        assert_eq!(output.runs_count, 0);
    }

    #[test]
    fn test_normalize_sarif_empty() {
        let sarif = Sarif::new();
        let mappings = RuleMappings::new();
        let output = normalize_sarif(sarif, &mappings);
        assert_eq!(output.count, 0);
        assert!(output.enriched_results.is_empty());
    }

    #[test]
    fn test_normalize_sarif_with_mappings() {
        let sarif = Sarif::new();
        let mut mappings = RuleMappings::new();
        mappings.insert(
            "test",
            "rule1",
            RuleMapping::new()
                .with_category("security")
                .with_severity("HIGH"),
        );

        let output = normalize_sarif(sarif, &mappings);
        // Empty sarif has no results to enrich
        assert_eq!(output.count, 0);
    }

    #[test]
    fn test_get_tool_config_unknown() {
        let registry = ToolRegistry::new();
        let result = get_tool_config(&registry, "nonexistent");
        assert!(matches!(result, Err(ToolError::UnknownTool(_))));
    }

    #[test]
    fn test_get_tool_config_known() {
        let registry = ToolRegistry::new();
        let result = get_tool_config(&registry, "ruff");
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.name, "ruff");
    }

    #[test]
    fn test_list_available_tools() {
        let registry = ToolRegistry::new();
        let (tools, _installed_count) = list_available_tools(&registry);
        assert!(!tools.is_empty());
    }
}

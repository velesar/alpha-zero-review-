//! Business logic operations for sarif-tools-server
//!
//! This module contains pure functions extracted from MCP handlers
//! to enable unit testing without rmcp infrastructure.

use crate::error::{NormalizeError, SarifError, ToolError};
use crate::runner::ToolRunner;
use crate::sarif::Sarif;
use crate::tools::{ToolInfo, ToolRegistry};
use std::collections::HashMap;
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
#[derive(Debug, serde::Serialize)]
pub struct ToolConfigOutput {
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub supported_languages: Vec<String>,
}

/// Execute a code analysis tool
pub fn execute_tool(
    registry: &ToolRegistry,
    tool_name: &str,
    path: &Path,
    config: Option<&serde_json::Value>,
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
pub fn merge_sarif_files(sarif_values: Vec<serde_json::Value>) -> Result<MergeSarifOutput, SarifError> {
    let mut combined = Sarif::new();

    for sarif_value in sarif_values {
        let sarif: Sarif = serde_json::from_value(sarif_value)
            .map_err(|e| SarifError::InvalidSarif(e.to_string()))?;
        combined.merge(sarif);
    }

    let total_results = combined.result_count();
    let runs_count = combined.runs.len();

    Ok(MergeSarifOutput {
        combined,
        total_results,
        runs_count,
    })
}

/// Normalize and enrich SARIF results with category mappings
pub fn normalize_sarif(
    sarif: Sarif,
    mappings: &HashMap<String, serde_json::Value>,
) -> NormalizeSarifOutput {
    let mut enriched_results = Vec::new();

    for run in &sarif.runs {
        let tool_name = &run.tool.driver.name;

        for result in &run.results {
            let mapping_key = format!("{}:{}", tool_name.to_lowercase(), result.rule_id);
            let fallback_key = result.rule_id.clone();

            let mapping = mappings
                .get(&mapping_key)
                .or_else(|| mappings.get(&fallback_key));

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
                category: mapping
                    .and_then(|m| m.get("category").and_then(|v| v.as_str()))
                    .map(String::from),
                subcategory: mapping
                    .and_then(|m| m.get("subcategory").and_then(|v| v.as_str()))
                    .map(String::from),
                cwe: mapping
                    .and_then(|m| m.get("cwe").and_then(|v| v.as_str()))
                    .map(String::from),
                severity_base: mapping
                    .and_then(|m| m.get("base_severity").and_then(|v| v.as_str()))
                    .map(String::from),
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

/// Load rule mappings from YAML file
pub fn load_rule_mappings(path: &Path) -> Result<HashMap<String, serde_json::Value>, NormalizeError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| NormalizeError::MappingLoadFailed(e.to_string()))?;

    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| NormalizeError::MappingLoadFailed(e.to_string()))?;

    let mut mappings = HashMap::new();

    if let Some(items) = yaml_value.as_sequence() {
        for item in items {
            if let (Some(tool), Some(rule_id)) = (
                item.get("tool").and_then(|v| v.as_str()),
                item.get("rule_id").and_then(|v| v.as_str()),
            ) {
                let key = format!("{}:{}", tool.to_lowercase(), rule_id);
                let json_value = serde_json::to_value(item)
                    .map_err(|e| NormalizeError::MappingLoadFailed(e.to_string()))?;
                mappings.insert(key, json_value.clone());
                // Also insert with just rule_id for fallback
                mappings.insert(rule_id.to_string(), json_value);
            }
        }
    }

    Ok(mappings)
}

/// Parse SARIF from JSON value
pub fn parse_sarif(value: serde_json::Value) -> Result<Sarif, SarifError> {
    serde_json::from_value(value).map_err(|e| SarifError::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = merge_sarif_files(vec![]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.total_results, 0);
        assert_eq!(output.runs_count, 0);
    }

    #[test]
    fn test_merge_sarif_invalid() {
        // Use an array instead of object - this will fail to parse as Sarif struct
        let invalid = serde_json::json!(["not", "a", "sarif", "object"]);
        let result = merge_sarif_files(vec![invalid]);
        assert!(matches!(result, Err(SarifError::InvalidSarif(_))));
    }

    #[test]
    fn test_merge_sarif_valid() {
        let sarif1 = serde_json::json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {"driver": {"name": "test", "rules": []}},
                "results": []
            }]
        });
        let sarif2 = serde_json::json!({
            "version": "2.1.0",
            "runs": [{
                "tool": {"driver": {"name": "test2", "rules": []}},
                "results": []
            }]
        });
        let result = merge_sarif_files(vec![sarif1, sarif2]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.runs_count, 2);
    }

    #[test]
    fn test_normalize_sarif_empty() {
        let sarif = Sarif::new();
        let mappings = HashMap::new();
        let output = normalize_sarif(sarif, &mappings);
        assert_eq!(output.count, 0);
        assert!(output.enriched_results.is_empty());
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

    #[test]
    fn test_parse_sarif_valid() {
        let value = serde_json::json!({
            "version": "2.1.0",
            "runs": []
        });
        let result = parse_sarif(value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sarif_invalid() {
        // Use an array instead of object - this will fail to parse as Sarif struct
        let value = serde_json::json!(["not", "a", "sarif"]);
        let result = parse_sarif(value);
        assert!(matches!(result, Err(SarifError::ParseError(_))));
    }
}

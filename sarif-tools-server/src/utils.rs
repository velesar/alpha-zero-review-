//! Response formatting utilities and adapters for MCP handlers
//!
//! This module contains:
//! - Response formatting utilities for MCP tool results
//! - Adapter functions that convert JSON/YAML to domain types
//!
//! Adapters bridge the framework layer (serde_json) to domain types,
//! keeping the domain layer pure.

use crate::domain::{ConfigValue, RuleMapping, RuleMappings, ToolConfig};
use crate::error::NormalizeError;
use crate::sarif::Sarif;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::path::Path;

/// Format a serializable value as a pretty-printed JSON response
pub fn format_json_response<T: Serialize>(data: &T) -> Result<CallToolResult, rmcp::ErrorData> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| rmcp::ErrorData::internal_error(format!("JSON serialization error: {}", e), None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Format a plain text response
pub fn format_text_response(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text.into())])
}

/// Format a response with a prefix message followed by JSON data
pub fn format_prefixed_json_response<T: Serialize>(
    prefix: &str,
    data: &T,
) -> Result<CallToolResult, rmcp::ErrorData> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| rmcp::ErrorData::internal_error(format!("JSON serialization error: {}", e), None))?;
    Ok(CallToolResult::success(vec![Content::text(format!(
        "{}\n\n{}",
        prefix, json
    ))]))
}

// ============================================================================
// Adapter Functions: JSON/YAML to Domain Types
// ============================================================================

/// Convert JSON Value to ToolConfig domain type
///
/// This is an adapter function that bridges serde_json to the domain layer.
pub fn json_to_tool_config(value: &serde_json::Value) -> ToolConfig {
    let mut config = ToolConfig::new();

    // Extract config source (rules/config)
    if let Some(rules) = value.get("rules").and_then(|v| v.as_str()) {
        config.config_source = Some(rules.to_string());
    } else if let Some(cfg_file) = value.get("config").and_then(|v| v.as_str()) {
        config.config_source = Some(cfg_file.to_string());
    }

    // Extract severity
    if let Some(severity) = value.get("severity").and_then(|v| v.as_str()) {
        config.severity = Some(severity.to_string());
    }

    // Extract exclude patterns
    if let Some(exclude) = value.get("exclude").and_then(|v| v.as_array()) {
        for pattern in exclude {
            if let Some(p) = pattern.as_str() {
                config.exclude_patterns.push(p.to_string());
            }
        }
    } else if let Some(exclude) = value.get("exclude").and_then(|v| v.as_str()) {
        // Single string exclude
        config.exclude_patterns.push(exclude.to_string());
    }

    // Extract common options
    for (key, val) in value.as_object().iter().flat_map(|o| o.iter()) {
        // Skip already processed keys
        if matches!(key.as_str(), "rules" | "config" | "severity" | "exclude") {
            continue;
        }

        let config_val = match val {
            serde_json::Value::String(s) => Some(ConfigValue::String(s.clone())),
            serde_json::Value::Bool(b) => Some(ConfigValue::Bool(*b)),
            serde_json::Value::Number(n) => n.as_f64().map(ConfigValue::Number),
            serde_json::Value::Array(arr) => {
                let strings: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                if !strings.is_empty() {
                    Some(ConfigValue::Array(strings))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(cv) = config_val {
            config.options.insert(key.clone(), cv);
        }
    }

    config
}

/// Parse SARIF from JSON Value (adapter function)
///
/// This is an adapter that bridges serde_json to the Sarif domain type.
pub fn parse_sarif_json(value: serde_json::Value) -> Result<Sarif, crate::error::SarifError> {
    serde_json::from_value(value)
        .map_err(|e| crate::error::SarifError::ParseError(e.to_string()))
}

/// Parse multiple SARIF JSON values into domain types
pub fn parse_sarif_values(values: Vec<serde_json::Value>) -> Result<Vec<Sarif>, crate::error::SarifError> {
    values.into_iter().map(parse_sarif_json).collect()
}

/// Load rule mappings from YAML file (adapter function)
///
/// This is an infrastructure adapter that reads from filesystem
/// and converts to domain types.
pub fn load_rule_mappings(path: &Path) -> Result<RuleMappings, NormalizeError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| NormalizeError::MappingLoadFailed(e.to_string()))?;

    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| NormalizeError::MappingLoadFailed(e.to_string()))?;

    yaml_to_rule_mappings(&yaml_value)
}

/// Convert YAML value to RuleMappings domain type
fn yaml_to_rule_mappings(yaml_value: &serde_yaml::Value) -> Result<RuleMappings, NormalizeError> {
    let mut mappings = RuleMappings::new();

    if let Some(items) = yaml_value.as_sequence() {
        for item in items {
            if let (Some(tool), Some(rule_id)) = (
                item.get("tool").and_then(|v| v.as_str()),
                item.get("rule_id").and_then(|v| v.as_str()),
            ) {
                let mapping = RuleMapping {
                    category: item.get("category").and_then(|v| v.as_str()).map(String::from),
                    subcategory: item.get("subcategory").and_then(|v| v.as_str()).map(String::from),
                    cwe: item.get("cwe").and_then(|v| v.as_str()).map(String::from),
                    base_severity: item.get("base_severity").and_then(|v| v.as_str()).map(String::from),
                    description: item.get("description").and_then(|v| v.as_str()).map(String::from),
                };
                mappings.insert(tool, rule_id, mapping);
            }
        }
    }

    Ok(mappings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_format_json_response() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        let result = format_json_response(&data).unwrap();
        assert!(!result.content.is_empty());
    }

    #[test]
    fn test_format_text_response() {
        let result = format_text_response("Hello, World!");
        assert!(!result.content.is_empty());
    }

    #[test]
    fn test_format_prefixed_json_response() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        let result = format_prefixed_json_response("Status:", &data).unwrap();
        assert!(!result.content.is_empty());
    }

    // Adapter function tests
    #[test]
    fn test_json_to_tool_config_empty() {
        let json = serde_json::json!({});
        let config = json_to_tool_config(&json);
        assert!(config.config_source.is_none());
        assert!(config.severity.is_none());
        assert!(config.exclude_patterns.is_empty());
    }

    #[test]
    fn test_json_to_tool_config_with_values() {
        let json = serde_json::json!({
            "config": "auto",
            "severity": "error",
            "exclude": ["*.test.rs", "tests/"],
            "all_targets": true
        });
        let config = json_to_tool_config(&json);
        assert_eq!(config.config_source, Some("auto".to_string()));
        assert_eq!(config.severity, Some("error".to_string()));
        assert_eq!(config.exclude_patterns, vec!["*.test.rs", "tests/"]);
        assert!(config.options.contains_key("all_targets"));
    }

    #[test]
    fn test_parse_sarif_json_valid() {
        let json = serde_json::json!({
            "version": "2.1.0",
            "runs": []
        });
        let result = parse_sarif_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_sarif_json_invalid() {
        let json = serde_json::json!(["not", "a", "sarif"]);
        let result = parse_sarif_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_yaml_to_rule_mappings() {
        let yaml_content = r#"
- tool: semgrep
  rule_id: python.security.eval
  category: security
  base_severity: HIGH
"#;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content).unwrap();
        let mappings = yaml_to_rule_mappings(&yaml_value).unwrap();

        let found = mappings.get("semgrep", "python.security.eval");
        assert!(found.is_some());
        assert_eq!(found.unwrap().category, Some("security".to_string()));
        assert_eq!(found.unwrap().base_severity, Some("HIGH".to_string()));
    }
}

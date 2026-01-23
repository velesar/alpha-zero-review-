//! Response formatting utilities for MCP handlers

use rmcp::model::{CallToolResult, Content};
use serde::Serialize;

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

/// Format a response with metric not found message
pub fn format_not_found_response(item_type: &str, name: &str) -> CallToolResult {
    format_text_response(format!("{} '{}' not found", item_type, name))
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

    #[test]
    fn test_format_not_found_response() {
        let result = format_not_found_response("Metric", "test_metric");
        assert!(!result.content.is_empty());
    }
}

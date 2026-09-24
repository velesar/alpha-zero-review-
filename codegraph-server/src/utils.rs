//! Response formatting utilities for MCP handlers

use rmcp::model::{CallToolResult, Content};
use serde::Serialize;

/// Format a serializable value as a pretty-printed JSON response
pub fn format_json_response<T: Serialize>(data: &T) -> Result<CallToolResult, rmcp::ErrorData> {
    let json = serde_json::to_string_pretty(data).map_err(|e| {
        rmcp::ErrorData::internal_error(format!("JSON serialization error: {}", e), None)
    })?;
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
    let json = serde_json::to_string_pretty(data).map_err(|e| {
        rmcp::ErrorData::internal_error(format!("JSON serialization error: {}", e), None)
    })?;
    Ok(CallToolResult::success(vec![Content::text(format!(
        "{}\n\n{}",
        prefix, json
    ))]))
}

/// Format a symbol not found response
pub fn format_symbol_not_found(symbol_id: &str) -> CallToolResult {
    format_text_response(format!("Symbol not found: {}", symbol_id))
}

/// Default allowed roots: the server's working directory (the audited
/// project when launched by an MCP client from there).
pub fn default_allowed_roots() -> Vec<std::path::PathBuf> {
    std::env::current_dir().into_iter().collect()
}

/// Resolve a caller-supplied path and require it to lie inside one of the
/// allowed roots (after resolving symlinks and `..`). MCP arguments come
/// from an LLM that reads untrusted code, so they must not reach arbitrary
/// locations on disk.
pub fn resolve_within(
    roots: &[std::path::PathBuf],
    path: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let resolved = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve path {}: {}", path.display(), e))?;
    let inside = roots
        .iter()
        .filter_map(|root| root.canonicalize().ok())
        .any(|root| resolved.starts_with(root));
    if inside {
        Ok(resolved)
    } else {
        Err(format!(
            "Path {} is outside the allowed roots ({}); start the server with --allowed-root to permit it",
            resolved.display(),
            roots
                .iter()
                .map(|r| r.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn resolve_within_accepts_paths_under_a_root() {
        let root = tempfile::TempDir::new().unwrap();
        let sub = root.path().join("src");
        std::fs::create_dir_all(&sub).unwrap();
        let roots = vec![root.path().to_path_buf()];
        let resolved = resolve_within(&roots, &sub).unwrap();
        assert_eq!(resolved, sub.canonicalize().unwrap());
        // `..` that stays inside the root is fine
        assert!(resolve_within(&roots, &sub.join("..")).is_ok());
    }

    #[test]
    fn resolve_within_rejects_paths_outside_roots() {
        let root = tempfile::TempDir::new().unwrap();
        let other = tempfile::TempDir::new().unwrap();
        let roots = vec![root.path().to_path_buf()];
        let err = resolve_within(&roots, other.path()).unwrap_err();
        assert!(err.contains("outside the allowed roots"));
        assert!(resolve_within(&roots, &root.path().join("../")).is_err());
        assert!(resolve_within(&roots, &root.path().join("missing")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn resolve_within_follows_symlinks_before_checking() {
        let root = tempfile::TempDir::new().unwrap();
        let other = tempfile::TempDir::new().unwrap();
        let link = root.path().join("link");
        std::os::unix::fs::symlink(other.path(), &link).unwrap();
        assert!(resolve_within(&[root.path().to_path_buf()], &link).is_err());
    }

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
    fn test_format_symbol_not_found() {
        let result = format_symbol_not_found("test_symbol");
        assert!(!result.content.is_empty());
    }
}

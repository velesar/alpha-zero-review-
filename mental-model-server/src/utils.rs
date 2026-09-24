//! Response formatting utilities for MCP handlers

use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::io::{Error, ErrorKind};
use std::path::{Component, Path, PathBuf};

/// Resolve a caller-supplied output file inside `root` (the audit directory).
///
/// Relative paths are taken relative to `root`; absolute paths must already
/// point inside it. `..` components, symlinked parents that leave `root`,
/// writing through a symlink and (unless `overwrite`) replacing an existing
/// file are rejected. Missing parent directories under `root` are created.
pub fn resolve_output_path(
    root: &Path,
    requested: &str,
    overwrite: bool,
) -> Result<PathBuf, Error> {
    let invalid = |msg: String| Error::new(ErrorKind::InvalidInput, msg);

    std::fs::create_dir_all(root)?;
    let root = root.canonicalize()?;

    let requested_path = Path::new(requested);
    let relative = if requested_path.is_absolute() {
        requested_path
            .strip_prefix(&root)
            .map_err(|_| {
                invalid(format!(
                    "{} is outside the audit directory {}",
                    requested,
                    root.display()
                ))
            })?
            .to_path_buf()
    } else {
        requested_path.to_path_buf()
    };

    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(invalid(format!(
            "Invalid output path '{}': use a file path inside the audit directory without '..'",
            requested
        )));
    }

    let target = root.join(&relative);
    let parent = target
        .parent()
        .ok_or_else(|| invalid(format!("Invalid output path '{}'", requested)))?;
    std::fs::create_dir_all(parent)?;
    if !parent.canonicalize()?.starts_with(&root) {
        return Err(invalid(format!(
            "{} resolves outside the audit directory",
            requested
        )));
    }

    if target.is_symlink() {
        return Err(invalid(format!(
            "{} is a symlink; refusing to write through it",
            requested
        )));
    }
    if target.exists() && !overwrite {
        return Err(invalid(format!(
            "{} already exists; pass overwrite: true to replace it",
            target.display()
        )));
    }
    Ok(target)
}

/// Format a serializable value as a pretty-printed JSON response
pub fn format_json_response<T: Serialize>(data: &T) -> Result<CallToolResult, rmcp::ErrorData> {
    let json = serde_json::to_string_pretty(data).map_err(|e| {
        rmcp::ErrorData::internal_error(format!("JSON serialization error: {}", e), None)
    })?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Format a serializable value as a YAML response
pub fn format_yaml_response<T: Serialize>(data: &T) -> Result<CallToolResult, rmcp::ErrorData> {
    let yaml = serde_yaml::to_string(data).map_err(|e| {
        rmcp::ErrorData::internal_error(format!("YAML serialization error: {}", e), None)
    })?;
    Ok(CallToolResult::success(vec![Content::text(yaml)]))
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
    fn test_format_yaml_response() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        let result = format_yaml_response(&data).unwrap();
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
}

#[cfg(test)]
mod path_tests {
    use super::resolve_output_path;
    use std::io::ErrorKind;
    use tempfile::TempDir;

    #[test]
    fn resolves_relative_and_absolute_paths_inside_root() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join(".audit");

        let target = resolve_output_path(&root, "reports/findings.json", false).unwrap();
        assert!(target.ends_with(".audit/reports/findings.json"));
        assert!(target.parent().unwrap().is_dir());

        let absolute = root.canonicalize().unwrap().join("export.json");
        let target = resolve_output_path(&root, absolute.to_str().unwrap(), false).unwrap();
        assert_eq!(target, absolute);
    }

    #[test]
    fn rejects_paths_outside_root() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join(".audit");
        let outside = dir.path().join("victim.txt");

        for requested in ["../victim.txt", "a/../../victim.txt", "", "/etc/passwd"] {
            let err = resolve_output_path(&root, requested, true).unwrap_err();
            assert_eq!(err.kind(), ErrorKind::InvalidInput, "{:?}", requested);
        }
        let err = resolve_output_path(&root, outside.to_str().unwrap(), true).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
    }

    #[test]
    fn refuses_to_overwrite_unless_asked() {
        let dir = TempDir::new().unwrap();
        let root = dir.path().join(".audit");
        let target = resolve_output_path(&root, "f.json", false).unwrap();
        std::fs::write(&target, "{}").unwrap();

        let err = resolve_output_path(&root, "f.json", false).unwrap_err();
        assert!(err.to_string().contains("overwrite"));
        assert!(resolve_output_path(&root, "f.json", true).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_parent_or_target() {
        let dir = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let root = dir.path().join(".audit");
        std::fs::create_dir_all(&root).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.join("escape")).unwrap();
        std::os::unix::fs::symlink(outside.path().join("x"), root.join("link.json")).unwrap();

        assert!(resolve_output_path(&root, "escape/f.json", true).is_err());
        assert!(resolve_output_path(&root, "link.json", true).is_err());
    }
}

//! Domain error types for sarif-tools-server

use thiserror::Error;

/// Errors that can occur during tool execution
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Unknown tool: {0}")]
    UnknownTool(String),

    #[error("Tool not installed: {0}")]
    ToolNotInstalled(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),
}

/// Errors that can occur during SARIF operations
#[derive(Error, Debug)]
pub enum SarifError {
    #[error("Invalid SARIF: {0}")]
    InvalidSarif(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Merge error: {0}")]
    MergeError(String),
}

/// Errors that can occur during normalization
#[derive(Error, Debug)]
pub enum NormalizeError {
    #[error("Invalid SARIF input: {0}")]
    InvalidInput(String),

    #[error("Mapping load failed: {0}")]
    MappingLoadFailed(String),
}

impl From<ToolError> for rmcp::ErrorData {
    fn from(e: ToolError) -> Self {
        match e {
            ToolError::UnknownTool(_) | ToolError::PathNotFound(_) => {
                rmcp::ErrorData::invalid_params(e.to_string(), None)
            }
            ToolError::ToolNotInstalled(_) | ToolError::ExecutionFailed(_) => {
                rmcp::ErrorData::internal_error(e.to_string(), None)
            }
        }
    }
}

impl From<SarifError> for rmcp::ErrorData {
    fn from(e: SarifError) -> Self {
        match e {
            SarifError::InvalidSarif(_) | SarifError::ParseError(_) => {
                rmcp::ErrorData::invalid_params(e.to_string(), None)
            }
            SarifError::MergeError(_) => rmcp::ErrorData::internal_error(e.to_string(), None),
        }
    }
}

impl From<NormalizeError> for rmcp::ErrorData {
    fn from(e: NormalizeError) -> Self {
        match e {
            NormalizeError::InvalidInput(_) => rmcp::ErrorData::invalid_params(e.to_string(), None),
            NormalizeError::MappingLoadFailed(_) => {
                rmcp::ErrorData::internal_error(e.to_string(), None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_error_display() {
        let err = ToolError::UnknownTool("foo".to_string());
        assert_eq!(err.to_string(), "Unknown tool: foo");
    }

    #[test]
    fn test_sarif_error_display() {
        let err = SarifError::InvalidSarif("bad format".to_string());
        assert_eq!(err.to_string(), "Invalid SARIF: bad format");
    }

    #[test]
    fn test_normalize_error_display() {
        let err = NormalizeError::MappingLoadFailed("file not found".to_string());
        assert_eq!(err.to_string(), "Mapping load failed: file not found");
    }

    #[test]
    fn test_tool_error_to_rmcp() {
        let err = ToolError::UnknownTool("foo".to_string());
        let rmcp_err: rmcp::ErrorData = err.into();
        assert!(rmcp_err.message.contains("Unknown tool"));
    }
}

//! Domain error types for codegraph-server

use thiserror::Error;

/// Errors that can occur during index operations
#[derive(Error, Debug)]
pub enum IndexError {
    #[error("No index loaded")]
    NotLoaded,

    #[error("Failed to load index: {0}")]
    LoadFailed(String),

    #[error("Invalid index format: {0}")]
    InvalidFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(String),
}

/// Errors that can occur during symbol operations
#[derive(Error, Debug)]
pub enum SymbolError {
    #[error("Symbol not found: {0}")]
    NotFound(String),

    #[error("Invalid symbol ID: {0}")]
    InvalidId(String),
}

/// Errors that can occur during analysis operations
#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("No index loaded for analysis")]
    NoIndex,

    #[error("Analysis failed: {0}")]
    Failed(String),
}

impl From<IndexError> for rmcp::ErrorData {
    fn from(e: IndexError) -> Self {
        match e {
            IndexError::NotLoaded => rmcp::ErrorData::invalid_request(e.to_string(), None),
            IndexError::FileNotFound(_) | IndexError::InvalidFormat(_) => {
                rmcp::ErrorData::invalid_params(e.to_string(), None)
            }
            IndexError::LoadFailed(_) => rmcp::ErrorData::internal_error(e.to_string(), None),
        }
    }
}

impl From<SymbolError> for rmcp::ErrorData {
    fn from(e: SymbolError) -> Self {
        rmcp::ErrorData::invalid_params(e.to_string(), None)
    }
}

impl From<AnalysisError> for rmcp::ErrorData {
    fn from(e: AnalysisError) -> Self {
        match e {
            AnalysisError::NoIndex => rmcp::ErrorData::invalid_request(e.to_string(), None),
            AnalysisError::Failed(_) => rmcp::ErrorData::internal_error(e.to_string(), None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_error_display() {
        let err = IndexError::NotLoaded;
        assert_eq!(err.to_string(), "No index loaded");
    }

    #[test]
    fn test_symbol_error_display() {
        let err = SymbolError::NotFound("test_symbol".to_string());
        assert_eq!(err.to_string(), "Symbol not found: test_symbol");
    }

    #[test]
    fn test_analysis_error_display() {
        let err = AnalysisError::NoIndex;
        assert_eq!(err.to_string(), "No index loaded for analysis");
    }

    #[test]
    fn test_index_error_to_rmcp() {
        let err = IndexError::NotLoaded;
        let rmcp_err: rmcp::ErrorData = err.into();
        assert!(rmcp_err.message.contains("No index loaded"));
    }

    #[test]
    fn test_symbol_error_to_rmcp() {
        let err = SymbolError::NotFound("test".to_string());
        let rmcp_err: rmcp::ErrorData = err.into();
        assert!(rmcp_err.message.contains("not found"));
    }
}

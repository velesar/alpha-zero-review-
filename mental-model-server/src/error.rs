//! Domain error types for mental-model-server

use thiserror::Error;

/// Errors that can occur during model operations
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Model not initialized")]
    NotInitialized,

    #[error("Model already initialized at: {0}")]
    AlreadyInitialized(String),

    #[error("Failed to save model: {0}")]
    SaveFailed(String),

    #[error("Failed to load model: {0}")]
    LoadFailed(String),

    #[error("Invalid viewpoint data: {0}")]
    InvalidViewpointData(String),
}

/// Errors that can occur during finding operations
#[derive(Error, Debug)]
pub enum FindingError {
    #[error("Invalid severity: {0}")]
    InvalidSeverity(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Errors that can occur during artifact operations
#[derive(Error, Debug)]
pub enum ArtifactError {
    #[error("Artifact not found: {0}")]
    NotFound(String),

    #[error("Failed to store artifact: {0}")]
    StoreFailed(String),

    #[error("Invalid artifact data: {0}")]
    InvalidData(String),

    #[error("Git error: {0}")]
    GitError(String),
}

/// Errors that can occur during synthesis
#[derive(Error, Debug)]
pub enum SynthesisError {
    #[error("No findings to synthesize")]
    NoFindings,

    #[error("Unknown algorithm: {0}")]
    UnknownAlgorithm(String),
}

impl From<ModelError> for rmcp::ErrorData {
    fn from(e: ModelError) -> Self {
        match e {
            ModelError::NotInitialized => {
                rmcp::ErrorData::invalid_request(e.to_string(), None)
            }
            ModelError::AlreadyInitialized(_) => {
                rmcp::ErrorData::invalid_request(e.to_string(), None)
            }
            ModelError::InvalidViewpointData(_) => {
                rmcp::ErrorData::invalid_params(e.to_string(), None)
            }
            ModelError::SaveFailed(_) | ModelError::LoadFailed(_) => {
                rmcp::ErrorData::internal_error(e.to_string(), None)
            }
        }
    }
}

impl From<FindingError> for rmcp::ErrorData {
    fn from(e: FindingError) -> Self {
        rmcp::ErrorData::invalid_params(e.to_string(), None)
    }
}

impl From<ArtifactError> for rmcp::ErrorData {
    fn from(e: ArtifactError) -> Self {
        match e {
            ArtifactError::NotFound(_) => rmcp::ErrorData::invalid_params(e.to_string(), None),
            ArtifactError::InvalidData(_) => rmcp::ErrorData::invalid_params(e.to_string(), None),
            ArtifactError::StoreFailed(_) | ArtifactError::GitError(_) => {
                rmcp::ErrorData::internal_error(e.to_string(), None)
            }
        }
    }
}

impl From<SynthesisError> for rmcp::ErrorData {
    fn from(e: SynthesisError) -> Self {
        match e {
            SynthesisError::NoFindings => rmcp::ErrorData::invalid_request(e.to_string(), None),
            SynthesisError::UnknownAlgorithm(_) => {
                rmcp::ErrorData::invalid_params(e.to_string(), None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_error_display() {
        let err = ModelError::NotInitialized;
        assert_eq!(err.to_string(), "Model not initialized");
    }

    #[test]
    fn test_finding_error_display() {
        let err = FindingError::InvalidSeverity("INVALID".to_string());
        assert_eq!(err.to_string(), "Invalid severity: INVALID");
    }

    #[test]
    fn test_artifact_error_display() {
        let err = ArtifactError::NotFound("test.sarif".to_string());
        assert_eq!(err.to_string(), "Artifact not found: test.sarif");
    }

    #[test]
    fn test_synthesis_error_display() {
        let err = SynthesisError::NoFindings;
        assert_eq!(err.to_string(), "No findings to synthesize");
    }

    #[test]
    fn test_model_error_to_rmcp() {
        let err = ModelError::NotInitialized;
        let rmcp_err: rmcp::ErrorData = err.into();
        assert!(rmcp_err.message.contains("not initialized"));
    }
}

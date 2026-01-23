//! Domain error types for methodology-kb-server

use thiserror::Error;

/// Errors that can occur during metric operations
#[derive(Error, Debug)]
pub enum MetricError {
    #[error("Metric not found: {0}")]
    NotFound(String),

    #[error("Invalid metric value: {0}")]
    InvalidValue(String),
}

/// Errors that can occur during classification
#[derive(Error, Debug)]
pub enum ClassificationError {
    #[error("Invalid severity: {0}")]
    InvalidSeverity(String),

    #[error("Unknown category: {0}")]
    UnknownCategory(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

/// Errors that can occur during compliance checking
#[derive(Error, Debug)]
pub enum ComplianceError {
    #[error("Unknown standard: {0}")]
    UnknownStandard(String),

    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
}

/// Errors that can occur during threshold operations
#[derive(Error, Debug)]
pub enum ThresholdError {
    #[error("Unknown project type: {0}")]
    UnknownProjectType(String),

    #[error("No thresholds defined for: {0}")]
    NoThresholds(String),
}

/// Errors that can occur during template operations
#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    NotFound(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

impl From<MetricError> for rmcp::ErrorData {
    fn from(e: MetricError) -> Self {
        match e {
            MetricError::NotFound(_) => rmcp::ErrorData::invalid_params(e.to_string(), None),
            MetricError::InvalidValue(_) => rmcp::ErrorData::invalid_params(e.to_string(), None),
        }
    }
}

impl From<ClassificationError> for rmcp::ErrorData {
    fn from(e: ClassificationError) -> Self {
        rmcp::ErrorData::invalid_params(e.to_string(), None)
    }
}

impl From<ComplianceError> for rmcp::ErrorData {
    fn from(e: ComplianceError) -> Self {
        rmcp::ErrorData::invalid_params(e.to_string(), None)
    }
}

impl From<ThresholdError> for rmcp::ErrorData {
    fn from(e: ThresholdError) -> Self {
        rmcp::ErrorData::invalid_params(e.to_string(), None)
    }
}

impl From<TemplateError> for rmcp::ErrorData {
    fn from(e: TemplateError) -> Self {
        match e {
            TemplateError::NotFound(_) => rmcp::ErrorData::invalid_params(e.to_string(), None),
            TemplateError::InvalidFormat(_) => rmcp::ErrorData::invalid_params(e.to_string(), None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_error_display() {
        let err = MetricError::NotFound("cyclomatic_complexity".to_string());
        assert_eq!(err.to_string(), "Metric not found: cyclomatic_complexity");
    }

    #[test]
    fn test_classification_error_display() {
        let err = ClassificationError::InvalidSeverity("INVALID".to_string());
        assert_eq!(err.to_string(), "Invalid severity: INVALID");
    }

    #[test]
    fn test_compliance_error_display() {
        let err = ComplianceError::UnknownStandard("unknown".to_string());
        assert_eq!(err.to_string(), "Unknown standard: unknown");
    }

    #[test]
    fn test_threshold_error_display() {
        let err = ThresholdError::UnknownProjectType("unknown".to_string());
        assert_eq!(err.to_string(), "Unknown project type: unknown");
    }

    #[test]
    fn test_template_error_display() {
        let err = TemplateError::NotFound("executive_summary".to_string());
        assert_eq!(err.to_string(), "Template not found: executive_summary");
    }

    #[test]
    fn test_metric_error_to_rmcp() {
        let err = MetricError::NotFound("test".to_string());
        let rmcp_err: rmcp::ErrorData = err.into();
        assert!(rmcp_err.message.contains("not found"));
    }
}

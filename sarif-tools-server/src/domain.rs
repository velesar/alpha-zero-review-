//! Domain types for sarif-tools-server
//!
//! This module contains pure domain types with no framework dependencies.
//! These types represent the core business concepts without serialization
//! or external library dependencies (except thiserror for errors).

use std::collections::HashMap;

/// Tool configuration options
///
/// This is a domain type that represents configuration for analysis tools
/// without depending on serde_json.
#[derive(Debug, Clone, Default)]
pub struct ToolConfig {
    /// Rules or config file to use
    pub config_source: Option<String>,
    /// Severity filter
    pub severity: Option<String>,
    /// Patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Additional tool-specific options
    pub options: HashMap<String, ConfigValue>,
}

/// Configuration value types
#[derive(Debug, Clone)]
pub enum ConfigValue {
    String(String),
    Bool(bool),
    Number(f64),
    Array(Vec<String>),
}

impl ToolConfig {
    /// Create an empty config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set config source (rules file, config file, or preset like "auto")
    pub fn with_config_source(mut self, source: impl Into<String>) -> Self {
        self.config_source = Some(source.into());
        self
    }

    /// Set severity filter
    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        self.severity = Some(severity.into());
        self
    }

    /// Add exclude pattern
    pub fn with_exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Add a custom option
    pub fn with_option(mut self, key: impl Into<String>, value: ConfigValue) -> Self {
        self.options.insert(key.into(), value);
        self
    }
}

/// Rule mapping for enriching SARIF results
///
/// Maps tool rule IDs to categories and severity information.
#[derive(Debug, Clone, Default)]
pub struct RuleMapping {
    /// Quality category (security, reliability, maintainability, etc.)
    pub category: Option<String>,
    /// Subcategory for more specific classification
    pub subcategory: Option<String>,
    /// Common Weakness Enumeration ID
    pub cwe: Option<String>,
    /// Base severity before context adjustment
    pub base_severity: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
}

impl RuleMapping {
    /// Create an empty rule mapping
    pub fn new() -> Self {
        Self::default()
    }

    /// Set category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set base severity
    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        self.base_severity = Some(severity.into());
        self
    }
}

/// Collection of rule mappings indexed by tool:rule_id
#[derive(Debug, Clone, Default)]
pub struct RuleMappings {
    mappings: HashMap<String, RuleMapping>,
}

impl RuleMappings {
    /// Create empty mappings
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mapping for a tool/rule combination
    pub fn insert(&mut self, tool: &str, rule_id: &str, mapping: RuleMapping) {
        let key = format!("{}:{}", tool.to_lowercase(), rule_id);
        self.mappings.insert(key.clone(), mapping.clone());
        // Also insert with just rule_id for fallback
        self.mappings.insert(rule_id.to_string(), mapping);
    }

    /// Get mapping for a rule (tries tool:rule_id first, then just rule_id)
    pub fn get(&self, tool: &str, rule_id: &str) -> Option<&RuleMapping> {
        let key = format!("{}:{}", tool.to_lowercase(), rule_id);
        self.mappings.get(&key).or_else(|| self.mappings.get(rule_id))
    }

    /// Get by exact key
    pub fn get_by_key(&self, key: &str) -> Option<&RuleMapping> {
        self.mappings.get(key)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    /// Get number of mappings
    pub fn len(&self) -> usize {
        self.mappings.len()
    }
}

/// Detected violation for compliance checking
#[derive(Debug, Clone)]
pub struct DetectedViolation {
    /// Rule that was violated
    pub rule: String,
    /// Description of the violation
    pub description: String,
    /// Severity of the violation
    pub severity: String,
    /// Location where the violation was detected
    pub location: Option<String>,
}

impl DetectedViolation {
    /// Create a new violation
    pub fn new(rule: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            rule: rule.into(),
            description: description.into(),
            severity: "MEDIUM".to_string(),
            location: None,
        }
    }

    /// Set severity
    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        self.severity = severity.into();
        self
    }

    /// Set location
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_config_builder() {
        let config = ToolConfig::new()
            .with_config_source("auto")
            .with_severity("error")
            .with_exclude("*.test.rs");

        assert_eq!(config.config_source, Some("auto".to_string()));
        assert_eq!(config.severity, Some("error".to_string()));
        assert_eq!(config.exclude_patterns, vec!["*.test.rs"]);
    }

    #[test]
    fn test_rule_mapping_builder() {
        let mapping = RuleMapping::new()
            .with_category("security")
            .with_severity("HIGH");

        assert_eq!(mapping.category, Some("security".to_string()));
        assert_eq!(mapping.base_severity, Some("HIGH".to_string()));
    }

    #[test]
    fn test_rule_mappings_lookup() {
        let mut mappings = RuleMappings::new();
        mappings.insert("semgrep", "python.lang.security.audit.eval-injection",
            RuleMapping::new().with_category("security"));

        // Lookup by tool:rule
        let found = mappings.get("semgrep", "python.lang.security.audit.eval-injection");
        assert!(found.is_some());
        assert_eq!(found.unwrap().category, Some("security".to_string()));

        // Lookup by just rule_id (fallback)
        let found = mappings.get("unknown", "python.lang.security.audit.eval-injection");
        assert!(found.is_some());
    }

    #[test]
    fn test_detected_violation() {
        let violation = DetectedViolation::new("domain_import", "Domain imports infrastructure")
            .with_severity("CRITICAL")
            .with_location("src/domain/mod.rs:15");

        assert_eq!(violation.rule, "domain_import");
        assert_eq!(violation.severity, "CRITICAL");
        assert!(violation.location.is_some());
    }
}

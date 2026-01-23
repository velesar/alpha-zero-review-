//! Domain types and helpers for methodology-kb-server
//!
//! This module provides domain types that avoid framework dependencies
//! like serde_json in function signatures.

use crate::types::RuleMapping;
use std::collections::HashMap;

/// A collection of rule mappings indexed for efficient lookup
#[derive(Debug, Clone, Default)]
pub struct RuleMappingsIndex {
    by_key: HashMap<String, RuleMapping>,
}

impl RuleMappingsIndex {
    /// Create an empty index
    pub fn new() -> Self {
        Self::default()
    }

    /// Build index from a vector of rule mappings
    pub fn from_mappings(mappings: &[RuleMapping]) -> Self {
        let mut index = Self::new();
        for mapping in mappings {
            index.insert(mapping.clone());
        }
        index
    }

    /// Insert a mapping
    pub fn insert(&mut self, mapping: RuleMapping) {
        let key = format!("{}:{}", mapping.tool.to_lowercase(), mapping.rule_id);
        self.by_key.insert(key.clone(), mapping.clone());
        // Also insert with just rule_id for fallback
        self.by_key.insert(mapping.rule_id.clone(), mapping);
    }

    /// Get mapping for a rule (tries tool:rule_id first, then just rule_id)
    pub fn get(&self, tool: &str, rule_id: &str) -> Option<&RuleMapping> {
        let key = format!("{}:{}", tool.to_lowercase(), rule_id);
        self.by_key.get(&key).or_else(|| self.by_key.get(rule_id))
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.by_key.is_empty()
    }
}

/// Detected violation for compliance checking (domain type)
///
/// This is a pure domain type with no framework dependencies.
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
    fn test_rule_mappings_index_lookup() {
        let mapping = RuleMapping {
            tool: "semgrep".to_string(),
            rule_id: "python.security.eval".to_string(),
            category: "security".to_string(),
            base_severity: Some("HIGH".to_string()),
            description: None,
        };

        let mut index = RuleMappingsIndex::new();
        index.insert(mapping);

        // Lookup by tool:rule_id
        let found = index.get("semgrep", "python.security.eval");
        assert!(found.is_some());
        assert_eq!(found.unwrap().category, "security");

        // Lookup by just rule_id (fallback)
        let found = index.get("unknown_tool", "python.security.eval");
        assert!(found.is_some());
    }

    #[test]
    fn test_detected_violation_builder() {
        let violation = DetectedViolation::new("domain_import", "Domain imports infrastructure")
            .with_severity("CRITICAL")
            .with_location("src/domain/mod.rs:15");

        assert_eq!(violation.rule, "domain_import");
        assert_eq!(violation.severity, "CRITICAL");
        assert!(violation.location.is_some());
    }

    #[test]
    fn test_from_mappings() {
        let mappings = vec![
            RuleMapping {
                tool: "ruff".to_string(),
                rule_id: "E501".to_string(),
                category: "style".to_string(),
                base_severity: Some("LOW".to_string()),
                description: None,
            },
        ];

        let index = RuleMappingsIndex::from_mappings(&mappings);
        assert!(!index.is_empty());
        assert!(index.get("ruff", "E501").is_some());
    }
}

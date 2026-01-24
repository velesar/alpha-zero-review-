//! Business logic operations for methodology-kb-server
//!
//! This module contains pure functions extracted from MCP handlers
//! to enable unit testing without rmcp infrastructure.
//!
//! **Domain purity:** This module uses only domain types and avoids
//! framework dependencies like serde_json in function signatures.
//! Serialization/deserialization happens at adapter boundaries.

use crate::domain::{DetectedViolation, RuleMappingsIndex};
use crate::types::ArchitectureStandard;

/// Severity score mapping
pub fn severity_to_score(severity: &str) -> f64 {
    match severity.to_uppercase().as_str() {
        "CRITICAL" => 4.0,
        "HIGH" => 3.0,
        "MEDIUM" => 2.0,
        "LOW" => 1.0,
        _ => 0.5,
    }
}

/// Convert score to severity string
pub fn score_to_severity(score: f64) -> String {
    if score >= 3.5 {
        "CRITICAL"
    } else if score >= 2.5 {
        "HIGH"
    } else if score >= 1.5 {
        "MEDIUM"
    } else if score >= 0.75 {
        "LOW"
    } else {
        "INFO"
    }
    .to_string()
}

/// Classification result
///
/// Note: serde::Serialize is needed for MCP response formatting.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClassificationResult {
    pub category: String,
    pub base_severity: String,
    pub adjusted_severity: String,
    pub adjustment_factors: Vec<String>,
    pub total_multiplier: f64,
}

/// Finding context for classification
#[derive(Debug, Clone, Default)]
pub struct ClassificationContext {
    pub bounded_context_type: Option<String>,
    pub layer: Option<String>,
    pub is_hotspot: bool,
}

/// Classify a finding based on context
///
/// Uses domain types instead of serde_json::Value.
pub fn classify_finding(
    tool: Option<&str>,
    rule_id: Option<&str>,
    category: Option<&str>,
    base_severity: Option<&str>,
    context: Option<&ClassificationContext>,
    rule_mappings: &RuleMappingsIndex,
) -> ClassificationResult {
    let mut result_category = category.map(String::from);
    let mut result_severity = base_severity.map(String::from);

    // Try to get category from rule mapping
    if result_category.is_none() || result_severity.is_none() {
        if let (Some(t), Some(r)) = (tool, rule_id) {
            if let Some(mapping) = rule_mappings.get(t, r) {
                if result_category.is_none() {
                    result_category = Some(mapping.category.clone());
                }
                if result_severity.is_none() {
                    result_severity = mapping.base_severity.clone();
                }
            }
        }
    }

    let category = result_category.unwrap_or_else(|| "unknown".to_string());
    let base_severity = result_severity.unwrap_or_else(|| "MEDIUM".to_string());

    // Calculate adjustments based on context
    let (total_multiplier, adjustment_factors) = if let Some(ctx) = context {
        calculate_adjustments(&category, ctx)
    } else {
        (1.0, vec![])
    };

    let base_score = severity_to_score(&base_severity);
    let adjusted_score = base_score * total_multiplier;
    let adjusted_severity = score_to_severity(adjusted_score);

    ClassificationResult {
        category,
        base_severity,
        adjusted_severity,
        adjustment_factors,
        total_multiplier,
    }
}

/// Calculate severity adjustments based on context
fn calculate_adjustments(category: &str, context: &ClassificationContext) -> (f64, Vec<String>) {
    let mut multiplier = 1.0;
    let mut factors = Vec::new();

    // Bounded context type adjustment
    if let Some(ref bc_type) = context.bounded_context_type {
        let adj = match bc_type.to_lowercase().as_str() {
            "core" => 1.5,
            "supporting" => 1.0,
            "generic" => 0.7,
            _ => 1.0,
        };
        if (adj - 1.0_f64).abs() > 0.01 {
            multiplier *= adj;
            factors.push(format!("bounded_context:{} ({}x)", bc_type, adj));
        }
    }

    // Layer adjustment
    if let Some(ref layer) = context.layer {
        let adj = match layer.to_lowercase().as_str() {
            "domain" => 1.3,
            "application" => 1.1,
            "adapters" | "adapter" => 1.0,
            "infrastructure" => 0.9,
            _ => 1.0,
        };
        if (adj - 1.0_f64).abs() > 0.01 {
            multiplier *= adj;
            factors.push(format!("layer:{} ({}x)", layer, adj));
        }
    }

    // Hotspot adjustment
    if context.is_hotspot {
        multiplier *= 1.4;
        factors.push("hotspot (1.4x)".to_string());
    }

    // Security category boost
    if category.to_lowercase() == "security" {
        multiplier *= 1.2;
        factors.push("security_category (1.2x)".to_string());
    }

    (multiplier, factors)
}

/// Compliance check result
///
/// Note: serde::Serialize is needed for MCP response formatting.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComplianceResult {
    pub standard: String,
    pub compliant: bool,
    pub score: f64,
    pub violations: Vec<ComplianceViolation>,
    pub recommendations: Vec<String>,
}

/// A compliance violation
///
/// Note: serde::Serialize is needed for MCP response formatting.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComplianceViolation {
    pub rule: String,
    pub description: String,
    pub severity: String,
    pub location: Option<String>,
}

/// Check compliance against a standard
///
/// Uses domain types instead of serde_json::Value.
pub fn check_compliance(
    standard: &ArchitectureStandard,
    detected_layers: &[String],
    detected_violations: &[DetectedViolation],
) -> ComplianceResult {
    let mut violations = Vec::new();
    let mut score: f64 = 100.0;

    // Check required layers
    for required_layer in &standard.layers {
        let layer_exists = detected_layers
            .iter()
            .any(|l| l.to_lowercase() == required_layer.name.to_lowercase());

        if !layer_exists {
            violations.push(ComplianceViolation {
                rule: format!("Required layer: {}", required_layer.name),
                description: format!("Missing {} layer", required_layer.name),
                severity: "MEDIUM".to_string(),
                location: None,
            });
            score -= 15.0;
        }
    }

    // Process detected violations
    for violation in detected_violations {
        violations.push(ComplianceViolation {
            rule: violation.rule.clone(),
            description: violation.description.clone(),
            severity: violation.severity.clone(),
            location: violation.location.clone(),
        });
        score -= 10.0;
    }

    let recommendations = generate_compliance_recommendations(score);

    ComplianceResult {
        standard: standard.name.clone(),
        compliant: violations.is_empty(),
        score: score.max(0.0),
        violations,
        recommendations,
    }
}

fn generate_compliance_recommendations(score: f64) -> Vec<String> {
    if score >= 90.0 {
        vec!["Architecture compliance is excellent".to_string()]
    } else if score >= 70.0 {
        vec![
            "Address minor compliance issues".to_string(),
            "Review layer boundaries".to_string(),
        ]
    } else if score >= 50.0 {
        vec![
            "Significant refactoring recommended".to_string(),
            "Establish clear layer boundaries".to_string(),
            "Review dependency directions".to_string(),
        ]
    } else {
        vec![
            "Major architectural review required".to_string(),
            "Consider architecture redesign".to_string(),
            "Implement strict layer separation".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RuleMapping;

    #[test]
    fn test_severity_to_score() {
        assert_eq!(severity_to_score("CRITICAL"), 4.0);
        assert_eq!(severity_to_score("HIGH"), 3.0);
        assert_eq!(severity_to_score("MEDIUM"), 2.0);
        assert_eq!(severity_to_score("LOW"), 1.0);
        assert_eq!(severity_to_score("unknown"), 0.5);
    }

    #[test]
    fn test_score_to_severity() {
        assert_eq!(score_to_severity(4.0), "CRITICAL");
        assert_eq!(score_to_severity(3.0), "HIGH");
        assert_eq!(score_to_severity(2.0), "MEDIUM");
        assert_eq!(score_to_severity(1.0), "LOW");
        assert_eq!(score_to_severity(0.5), "INFO");
    }

    #[test]
    fn test_classify_finding_defaults() {
        let result = classify_finding(None, None, None, None, None, &RuleMappingsIndex::new());
        assert_eq!(result.category, "unknown");
        assert_eq!(result.base_severity, "MEDIUM");
        assert_eq!(result.total_multiplier, 1.0);
    }

    #[test]
    fn test_classify_finding_with_context() {
        let context = ClassificationContext {
            bounded_context_type: Some("core".to_string()),
            layer: Some("domain".to_string()),
            is_hotspot: true,
        };

        let result = classify_finding(
            None,
            None,
            Some("security"),
            Some("MEDIUM"),
            Some(&context),
            &RuleMappingsIndex::new(),
        );

        // 1.5 (core) * 1.3 (domain) * 1.4 (hotspot) * 1.2 (security) = 3.276
        assert!(result.total_multiplier > 3.0);
        assert_eq!(result.adjusted_severity, "CRITICAL");
    }

    #[test]
    fn test_classify_finding_with_rule_mappings() {
        let mapping = RuleMapping {
            tool: "semgrep".to_string(),
            rule_id: "python.security.eval".to_string(),
            category: "security".to_string(),
            base_severity: Some("HIGH".to_string()),
            description: None,
        };

        let mut mappings = RuleMappingsIndex::new();
        mappings.insert(mapping);

        let result = classify_finding(
            Some("semgrep"),
            Some("python.security.eval"),
            None,
            None,
            None,
            &mappings,
        );

        assert_eq!(result.category, "security");
        assert_eq!(result.base_severity, "HIGH");
    }

    #[test]
    fn test_calculate_adjustments_no_context() {
        let context = ClassificationContext::default();
        let (multiplier, factors) = calculate_adjustments("maintainability", &context);
        assert_eq!(multiplier, 1.0);
        assert!(factors.is_empty());
    }

    #[test]
    fn test_generate_compliance_recommendations() {
        assert_eq!(generate_compliance_recommendations(95.0).len(), 1);
        assert_eq!(generate_compliance_recommendations(75.0).len(), 2);
        assert_eq!(generate_compliance_recommendations(60.0).len(), 3);
        assert_eq!(generate_compliance_recommendations(30.0).len(), 3);
    }

    #[test]
    fn test_check_compliance_with_violations() {
        use crate::types::{ArchitectureStandard, StandardLayer};

        let standard = ArchitectureStandard {
            id: "test".to_string(),
            name: "Test Standard".to_string(),
            description: "A test standard".to_string(),
            layers: vec![StandardLayer {
                name: "domain".to_string(),
                aliases: vec![],
                typical_paths: vec![],
                purpose: "Business logic".to_string(),
                allowed_dependencies: vec![],
            }],
            dependency_rules: vec![],
        };

        let violations = vec![
            DetectedViolation::new("test_rule", "Test violation").with_severity("HIGH"),
        ];

        let result = check_compliance(&standard, &["domain".to_string()], &violations);

        assert!(!result.compliant);
        assert!(result.score < 100.0);
        assert_eq!(result.violations.len(), 1);
    }
}

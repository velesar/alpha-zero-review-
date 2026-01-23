//! Business logic operations for mental-model-server
//!
//! This module contains pure functions extracted from MCP handlers
//! to enable unit testing without rmcp infrastructure.

use crate::model::{BoundedContextType, Finding, FindingContext, RootCause, Severity};
use std::collections::HashMap;
use uuid::Uuid;

/// Adjust severity based on context factors
pub fn adjust_severity(base: &Severity, context: &FindingContext) -> Severity {
    let mut multiplier = 1.0;

    // Bounded context type adjustment
    if let Some(ref bc_type) = context.bounded_context_type {
        multiplier *= match bc_type {
            BoundedContextType::Core => 1.5,
            BoundedContextType::Supporting => 1.0,
            BoundedContextType::Generic => 0.7,
        };
    }

    // Architecture layer adjustment
    if let Some(ref layer) = context.layer {
        multiplier *= match layer.to_lowercase().as_str() {
            "domain" => 1.3,
            "application" => 1.1,
            "adapters" | "adapter" => 1.0,
            "infrastructure" => 0.9,
            _ => 1.0,
        };
    }

    // Hotspot adjustment
    if context.is_hotspot {
        multiplier *= 1.4;
    }

    // Calculate new severity
    let new_score = base.score() * multiplier;
    Severity::from_score(new_score)
}

/// Parse severity string to Severity enum
pub fn parse_severity(severity_str: &str) -> Severity {
    match severity_str.to_uppercase().as_str() {
        "CRITICAL" => Severity::Critical,
        "HIGH" => Severity::High,
        "MEDIUM" => Severity::Medium,
        "LOW" => Severity::Low,
        _ => Severity::Info,
    }
}

/// Synthesize findings by category
pub fn synthesize_by_category(findings: &[Finding]) -> Vec<RootCause> {
    let mut category_groups: HashMap<String, Vec<&Finding>> = HashMap::new();

    for finding in findings {
        category_groups
            .entry(finding.category.clone())
            .or_default()
            .push(finding);
    }

    let mut root_causes = Vec::new();

    for (category, findings) in category_groups {
        if findings.is_empty() {
            continue;
        }

        // Calculate aggregate impact
        let max_severity = findings
            .iter()
            .map(|f| &f.adjusted_severity)
            .max()
            .cloned()
            .unwrap_or(Severity::Info);

        // Collect affected areas
        let affected_areas: Vec<String> = findings
            .iter()
            .filter_map(|f| {
                f.context
                    .as_ref()
                    .and_then(|c| c.bounded_context.clone())
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Generate recommendations based on category
        let recommendations = generate_recommendations(&category, &findings);

        root_causes.push(RootCause {
            id: format!("RC-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            title: format!("{} Issues", capitalize_first(&category)),
            description: format!(
                "Multiple {} issues detected across the codebase affecting code quality and maintainability.",
                category
            ),
            category: category.clone(),
            impact: max_severity,
            finding_ids: findings.iter().map(|f| f.id.clone()).collect(),
            finding_count: findings.len() as u32,
            affected_areas,
            recommendations,
        });
    }

    // Sort by impact
    root_causes.sort_by(|a, b| b.impact.cmp(&a.impact));

    // Limit to top 5
    root_causes.truncate(5);

    root_causes
}

/// Synthesize findings by location
pub fn synthesize_by_location(findings: &[Finding]) -> Vec<RootCause> {
    let mut location_groups: HashMap<String, Vec<&Finding>> = HashMap::new();

    for finding in findings {
        // Group by directory
        let dir = std::path::Path::new(&finding.file_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| finding.file_path.clone());

        location_groups.entry(dir).or_default().push(finding);
    }

    let mut root_causes = Vec::new();

    for (location, findings) in location_groups {
        if findings.len() < 2 {
            continue; // Need at least 2 findings to form a root cause
        }

        let max_severity = findings
            .iter()
            .map(|f| &f.adjusted_severity)
            .max()
            .cloned()
            .unwrap_or(Severity::Info);

        let categories: Vec<String> = findings
            .iter()
            .map(|f| f.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        root_causes.push(RootCause {
            id: format!("RC-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            title: format!("Quality Issues in {}", location),
            description: format!(
                "Multiple quality issues concentrated in {}. Categories: {}",
                location,
                categories.join(", ")
            ),
            category: "location_cluster".to_string(),
            impact: max_severity,
            finding_ids: findings.iter().map(|f| f.id.clone()).collect(),
            finding_count: findings.len() as u32,
            affected_areas: vec![location],
            recommendations: vec![
                "Review and refactor this area for improved quality".to_string(),
                "Consider adding tests before refactoring".to_string(),
            ],
        });
    }

    root_causes.sort_by(|a, b| b.finding_count.cmp(&a.finding_count));
    root_causes.truncate(5);

    root_causes
}

/// Generate recommendations based on category
pub fn generate_recommendations(category: &str, _findings: &[&Finding]) -> Vec<String> {
    match category.to_lowercase().as_str() {
        "security" => vec![
            "Conduct a focused security review of affected components".to_string(),
            "Implement input validation and sanitization".to_string(),
            "Review authentication and authorization patterns".to_string(),
        ],
        "reliability" => vec![
            "Add error handling and recovery mechanisms".to_string(),
            "Implement retry logic for external dependencies".to_string(),
            "Add monitoring and alerting for critical paths".to_string(),
        ],
        "maintainability" => vec![
            "Refactor complex code into smaller, focused functions".to_string(),
            "Improve code documentation and naming".to_string(),
            "Consider extracting reusable components".to_string(),
        ],
        "performance" => vec![
            "Profile and optimize critical paths".to_string(),
            "Review database queries for N+1 issues".to_string(),
            "Consider caching frequently accessed data".to_string(),
        ],
        "testability" => vec![
            "Increase test coverage for critical paths".to_string(),
            "Add integration tests for key workflows".to_string(),
            "Refactor tightly coupled code for better testability".to_string(),
        ],
        _ => vec![
            "Review affected code and apply best practices".to_string(),
            "Consider architectural improvements".to_string(),
        ],
    }
}

/// Capitalize first letter of a string
pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("CRITICAL"), Severity::Critical);
        assert_eq!(parse_severity("high"), Severity::High);
        assert_eq!(parse_severity("Medium"), Severity::Medium);
        assert_eq!(parse_severity("LOW"), Severity::Low);
        assert_eq!(parse_severity("unknown"), Severity::Info);
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("HELLO"), "HELLO");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("a"), "A");
    }

    #[test]
    fn test_adjust_severity_core_domain_hotspot() {
        let context = FindingContext {
            bounded_context: Some("Orders".to_string()),
            bounded_context_type: Some(BoundedContextType::Core),
            layer: Some("domain".to_string()),
            is_hotspot: true,
            hotspot_score: Some(80.0),
        };

        let adjusted = adjust_severity(&Severity::Medium, &context);
        // Medium (2.0) * Core (1.5) * Domain (1.3) * Hotspot (1.4) = 5.46 → Critical
        assert_eq!(adjusted, Severity::Critical);
    }

    #[test]
    fn test_adjust_severity_generic_infrastructure() {
        let context = FindingContext {
            bounded_context: Some("Utilities".to_string()),
            bounded_context_type: Some(BoundedContextType::Generic),
            layer: Some("infrastructure".to_string()),
            is_hotspot: false,
            hotspot_score: None,
        };

        let adjusted = adjust_severity(&Severity::Medium, &context);
        // Medium (2.0) * Generic (0.7) * Infrastructure (0.9) = 1.26 → Low
        assert_eq!(adjusted, Severity::Low);
    }

    #[test]
    fn test_adjust_severity_no_context() {
        let context = FindingContext {
            bounded_context: None,
            bounded_context_type: None,
            layer: None,
            is_hotspot: false,
            hotspot_score: None,
        };

        let adjusted = adjust_severity(&Severity::High, &context);
        // No adjustments - stays High
        assert_eq!(adjusted, Severity::High);
    }

    #[test]
    fn test_synthesize_by_category_empty() {
        let findings: Vec<Finding> = vec![];
        let root_causes = synthesize_by_category(&findings);
        assert!(root_causes.is_empty());
    }

    #[test]
    fn test_generate_recommendations_security() {
        let findings: Vec<&Finding> = vec![];
        let recs = generate_recommendations("security", &findings);
        assert_eq!(recs.len(), 3);
        assert!(recs[0].contains("security"));
    }

    #[test]
    fn test_generate_recommendations_unknown() {
        let findings: Vec<&Finding> = vec![];
        let recs = generate_recommendations("unknown_category", &findings);
        assert_eq!(recs.len(), 2);
    }
}

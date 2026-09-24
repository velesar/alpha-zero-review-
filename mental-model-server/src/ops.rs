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

/// Parse a severity name (case-insensitive). Unknown names are `None`
/// rather than silently becoming INFO, which would downgrade a finding.
pub fn parse_severity(severity_str: &str) -> Option<Severity> {
    match severity_str.to_uppercase().as_str() {
        "CRITICAL" => Some(Severity::Critical),
        "HIGH" => Some(Severity::High),
        "MEDIUM" => Some(Severity::Medium),
        "LOW" => Some(Severity::Low),
        "INFO" => Some(Severity::Info),
        _ => None,
    }
}

/// Maximum number of root causes reported by synthesis
pub const MAX_ROOT_CAUSES: usize = 5;

/// Result of clustering findings into root causes
#[derive(Debug, Clone, Default)]
pub struct Synthesis {
    /// Highest-impact root causes (at most `MAX_ROOT_CAUSES`)
    pub root_causes: Vec<RootCause>,
    /// Lower-ranked clusters that did not fit, so nothing is dropped silently
    pub omitted: Vec<RootCause>,
    /// Findings that did not join any cluster (e.g. alone in their directory)
    pub ungrouped_findings: usize,
}

fn highest_severity(findings: &[&Finding]) -> Severity {
    findings
        .iter()
        .map(|f| f.adjusted_severity.clone())
        .max()
        .unwrap_or(Severity::Info)
}

fn split_top(mut root_causes: Vec<RootCause>, ungrouped_findings: usize) -> Synthesis {
    let omitted = if root_causes.len() > MAX_ROOT_CAUSES {
        root_causes.split_off(MAX_ROOT_CAUSES)
    } else {
        Vec::new()
    };
    Synthesis {
        root_causes,
        omitted,
        ungrouped_findings,
    }
}

fn finding_word(count: usize) -> &'static str {
    if count == 1 {
        "finding"
    } else {
        "findings"
    }
}

/// Synthesize findings by category, most severe clusters first
pub fn synthesize_by_category(findings: &[Finding]) -> Synthesis {
    let mut category_groups: HashMap<String, Vec<&Finding>> = HashMap::new();

    for finding in findings {
        category_groups
            .entry(finding.category.clone())
            .or_default()
            .push(finding);
    }

    let mut root_causes = Vec::new();

    for (category, findings) in category_groups {
        let impact = highest_severity(&findings);

        let mut affected_areas: Vec<String> = findings
            .iter()
            .filter_map(|f| f.context.as_ref().and_then(|c| c.bounded_context.clone()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        affected_areas.sort();

        let recommendations = generate_recommendations(&category, &findings);

        let description = format!(
            "{} {} in category '{}' (highest severity {:?}){}.",
            findings.len(),
            finding_word(findings.len()),
            category,
            impact,
            if affected_areas.is_empty() {
                String::new()
            } else {
                format!(", affecting {}", affected_areas.join(", "))
            }
        );

        root_causes.push(RootCause {
            id: format!("RC-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            title: format!("{} Issues", capitalize_first(&category)),
            description,
            category: category.clone(),
            impact,
            finding_ids: findings.iter().map(|f| f.id.clone()).collect(),
            finding_count: findings.len() as u32,
            affected_areas,
            recommendations,
        });
    }

    // Most severe first, then larger clusters, then name for determinism
    root_causes.sort_by(|a, b| {
        b.impact
            .cmp(&a.impact)
            .then(b.finding_count.cmp(&a.finding_count))
            .then(a.category.cmp(&b.category))
    });

    split_top(root_causes, 0)
}

/// Synthesize findings by location (directory), largest clusters first
pub fn synthesize_by_location(findings: &[Finding]) -> Synthesis {
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
    let mut ungrouped_findings = 0;

    for (location, findings) in location_groups {
        if findings.len() < 2 {
            // A single finding does not make a location cluster
            ungrouped_findings += findings.len();
            continue;
        }

        let impact = highest_severity(&findings);

        let mut categories: Vec<String> = findings
            .iter()
            .map(|f| f.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();

        root_causes.push(RootCause {
            id: format!("RC-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            title: format!("Quality Issues in {}", location),
            description: format!(
                "{} findings concentrated in {} (highest severity {:?}). Categories: {}",
                findings.len(),
                location,
                impact,
                categories.join(", ")
            ),
            category: "location_cluster".to_string(),
            impact,
            finding_ids: findings.iter().map(|f| f.id.clone()).collect(),
            finding_count: findings.len() as u32,
            affected_areas: vec![location],
            recommendations: vec![
                "Review and refactor this area for improved quality".to_string(),
                "Consider adding tests before refactoring".to_string(),
            ],
        });
    }

    root_causes.sort_by(|a, b| {
        b.finding_count
            .cmp(&a.finding_count)
            .then(b.impact.cmp(&a.impact))
            .then(a.title.cmp(&b.title))
    });

    split_top(root_causes, ungrouped_findings)
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
        assert_eq!(parse_severity("CRITICAL"), Some(Severity::Critical));
        assert_eq!(parse_severity("high"), Some(Severity::High));
        assert_eq!(parse_severity("Medium"), Some(Severity::Medium));
        assert_eq!(parse_severity("LOW"), Some(Severity::Low));
        assert_eq!(parse_severity("info"), Some(Severity::Info));
        assert_eq!(parse_severity("unknown"), None);
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
        let synthesis = synthesize_by_category(&findings);
        assert!(synthesis.root_causes.is_empty());
        assert!(synthesis.omitted.is_empty());
    }

    fn finding(id: &str, category: &str, file: &str, severity: Severity) -> Finding {
        Finding {
            id: id.to_string(),
            viewpoint: "VP-Q01".to_string(),
            category: category.to_string(),
            title: id.to_string(),
            description: String::new(),
            file_path: file.to_string(),
            line_number: None,
            base_severity: severity.clone(),
            adjusted_severity: severity,
            rule_id: None,
            context: None,
            recommendation: None,
        }
    }

    #[test]
    fn test_severity_orders_by_seriousness() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Info);
        let max = [Severity::Info, Severity::Critical, Severity::Low]
            .into_iter()
            .max();
        assert_eq!(max, Some(Severity::Critical));
    }

    #[test]
    fn test_synthesis_keeps_most_severe_clusters() {
        // Six categories: five low-severity clusters with several findings
        // each, and one category with a single critical finding.
        let mut findings = Vec::new();
        for (i, cat) in ["docs", "style", "naming", "perf", "dup"]
            .iter()
            .enumerate()
        {
            for j in 0..3 {
                findings.push(finding(
                    &format!("F-{}{}", i, j),
                    cat,
                    "src/a.rs",
                    Severity::Low,
                ));
            }
        }
        findings.push(finding("F-SEC", "security", "src/b.rs", Severity::Medium));
        findings.push(finding(
            "F-SEC2",
            "security",
            "src/b.rs",
            Severity::Critical,
        ));

        let synthesis = synthesize_by_category(&findings);

        assert_eq!(synthesis.root_causes.len(), MAX_ROOT_CAUSES);
        assert_eq!(synthesis.root_causes[0].category, "security");
        assert_eq!(synthesis.root_causes[0].impact, Severity::Critical);
        assert!(synthesis.root_causes[0]
            .description
            .starts_with("2 findings"));
        assert_eq!(synthesis.omitted.len(), 1);
        let reported: u32 = synthesis
            .root_causes
            .iter()
            .chain(&synthesis.omitted)
            .map(|rc| rc.finding_count)
            .sum();
        assert_eq!(reported as usize, findings.len());
    }

    #[test]
    fn test_synthesis_by_location_reports_ungrouped() {
        let findings = vec![
            finding("F-1", "security", "src/a/x.rs", Severity::Critical),
            finding("F-2", "style", "src/a/y.rs", Severity::Low),
            finding("F-3", "style", "src/b/z.rs", Severity::Low),
        ];
        let synthesis = synthesize_by_location(&findings);
        assert_eq!(synthesis.root_causes.len(), 1);
        assert_eq!(synthesis.root_causes[0].impact, Severity::Critical);
        assert_eq!(synthesis.ungrouped_findings, 1);
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

//! SARIF 2.1.0 type definitions
//!
//! Static Analysis Results Interchange Format (SARIF) types
//! for parsing and generating SARIF output.

use serde::{Deserialize, Serialize};

/// SARIF 2.1.0 root object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sarif {
    #[serde(rename = "$schema", default = "default_schema")]
    pub schema: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub runs: Vec<Run>,
}

fn default_schema() -> String {
    "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string()
}

fn default_version() -> String {
    "2.1.0".to_string()
}

impl Default for Sarif {
    fn default() -> Self {
        Self {
            schema: default_schema(),
            version: default_version(),
            runs: Vec::new(),
        }
    }
}

/// A single run of an analysis tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Run {
    pub tool: Tool,
    #[serde(default)]
    pub results: Vec<Result>,
    #[serde(default)]
    pub invocations: Vec<Invocation>,
}

/// Information about the analysis tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tool {
    pub driver: ToolDriver,
}

/// The primary tool component
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolDriver {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "informationUri", skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

/// A rule (or check) defined by the tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Rule {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "shortDescription", skip_serializing_if = "Option::is_none")]
    pub short_description: Option<Message>,
    #[serde(rename = "fullDescription", skip_serializing_if = "Option::is_none")]
    pub full_description: Option<Message>,
    #[serde(rename = "helpUri", skip_serializing_if = "Option::is_none")]
    pub help_uri: Option<String>,
    #[serde(rename = "defaultConfiguration", skip_serializing_if = "Option::is_none")]
    pub default_configuration: Option<RuleConfiguration>,
    #[serde(default)]
    pub properties: serde_json::Value,
}

/// Default configuration for a rule
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

/// A single result from the analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Result {
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    pub message: Message,
    #[serde(default)]
    pub locations: Vec<Location>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprints: Option<serde_json::Value>,
    #[serde(default)]
    pub properties: serde_json::Value,
}

/// A message with text
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Message {
    #[serde(default)]
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
}

/// A location in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "physicalLocation")]
    pub physical_location: PhysicalLocation,
}

/// A physical location in a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalLocation {
    #[serde(rename = "artifactLocation")]
    pub artifact_location: ArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
}

/// Reference to a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactLocation {
    pub uri: String,
    #[serde(rename = "uriBaseId", skip_serializing_if = "Option::is_none")]
    pub uri_base_id: Option<String>,
}

/// A region within a file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Region {
    #[serde(rename = "startLine", skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(rename = "startColumn", skip_serializing_if = "Option::is_none")]
    pub start_column: Option<u32>,
    #[serde(rename = "endLine", skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
    #[serde(rename = "endColumn", skip_serializing_if = "Option::is_none")]
    pub end_column: Option<u32>,
}

/// Information about a tool invocation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Invocation {
    #[serde(rename = "executionSuccessful")]
    pub execution_successful: bool,
    #[serde(rename = "exitCode", skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(rename = "commandLine", skip_serializing_if = "Option::is_none")]
    pub command_line: Option<String>,
    #[serde(rename = "workingDirectory", skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<ArtifactLocation>,
}

impl Sarif {
    /// Create a new empty SARIF object
    pub fn new() -> Self {
        Self {
            schema: default_schema(),
            version: default_version(),
            runs: Vec::new(),
        }
    }

    /// Merge another SARIF object into this one
    pub fn merge(&mut self, other: Sarif) {
        self.runs.extend(other.runs);
    }

    /// Get total count of results across all runs
    pub fn result_count(&self) -> usize {
        self.runs.iter().map(|r| r.results.len()).sum()
    }

    /// Get results grouped by rule ID
    #[allow(dead_code)]
    pub fn results_by_rule(&self) -> std::collections::HashMap<String, Vec<&Result>> {
        let mut map = std::collections::HashMap::new();
        for run in &self.runs {
            for result in &run.results {
                map.entry(result.rule_id.clone())
                    .or_insert_with(Vec::new)
                    .push(result);
            }
        }
        map
    }

    /// Get unique file paths from all results
    #[allow(dead_code)]
    pub fn affected_files(&self) -> std::collections::HashSet<String> {
        let mut files = std::collections::HashSet::new();
        for run in &self.runs {
            for result in &run.results {
                for location in &result.locations {
                    files.insert(location.physical_location.artifact_location.uri.clone());
                }
            }
        }
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sarif_new() {
        let sarif = Sarif::new();
        assert_eq!(sarif.version, "2.1.0");
        assert!(sarif.runs.is_empty());
        assert!(sarif.schema.contains("sarif-schema-2.1.0.json"));
    }

    #[test]
    fn test_sarif_default() {
        let sarif = Sarif::default();
        assert_eq!(sarif.version, "2.1.0");
        assert!(sarif.runs.is_empty());
    }

    #[test]
    fn test_sarif_merge() {
        let mut sarif1 = Sarif::new();
        sarif1.runs.push(Run::default());

        let mut sarif2 = Sarif::new();
        sarif2.runs.push(Run::default());

        sarif1.merge(sarif2);
        assert_eq!(sarif1.runs.len(), 2);
    }

    #[test]
    fn test_sarif_result_count_empty() {
        let sarif = Sarif::new();
        assert_eq!(sarif.result_count(), 0);
    }

    #[test]
    fn test_sarif_result_count_with_results() {
        let mut sarif = Sarif::new();
        let mut run = Run::default();
        run.results.push(Result {
            rule_id: "rule1".to_string(),
            level: Some("warning".to_string()),
            message: Message { text: "test".to_string(), markdown: None },
            locations: vec![],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        run.results.push(Result {
            rule_id: "rule2".to_string(),
            level: Some("error".to_string()),
            message: Message { text: "test2".to_string(), markdown: None },
            locations: vec![],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        sarif.runs.push(run);
        assert_eq!(sarif.result_count(), 2);
    }

    #[test]
    fn test_sarif_result_count_multiple_runs() {
        let mut sarif = Sarif::new();

        let mut run1 = Run::default();
        run1.results.push(Result {
            rule_id: "rule1".to_string(),
            level: None,
            message: Message::default(),
            locations: vec![],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        sarif.runs.push(run1);

        let mut run2 = Run::default();
        run2.results.push(Result {
            rule_id: "rule2".to_string(),
            level: None,
            message: Message::default(),
            locations: vec![],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        run2.results.push(Result {
            rule_id: "rule3".to_string(),
            level: None,
            message: Message::default(),
            locations: vec![],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        sarif.runs.push(run2);

        assert_eq!(sarif.result_count(), 3);
    }

    #[test]
    fn test_sarif_results_by_rule() {
        let mut sarif = Sarif::new();
        let mut run = Run::default();
        run.results.push(Result {
            rule_id: "rule1".to_string(),
            level: None,
            message: Message::default(),
            locations: vec![],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        run.results.push(Result {
            rule_id: "rule1".to_string(),
            level: None,
            message: Message::default(),
            locations: vec![],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        run.results.push(Result {
            rule_id: "rule2".to_string(),
            level: None,
            message: Message::default(),
            locations: vec![],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        sarif.runs.push(run);

        let by_rule = sarif.results_by_rule();
        assert_eq!(by_rule.len(), 2);
        assert_eq!(by_rule.get("rule1").unwrap().len(), 2);
        assert_eq!(by_rule.get("rule2").unwrap().len(), 1);
    }

    #[test]
    fn test_sarif_affected_files() {
        let mut sarif = Sarif::new();
        let mut run = Run::default();
        run.results.push(Result {
            rule_id: "rule1".to_string(),
            level: None,
            message: Message::default(),
            locations: vec![Location {
                physical_location: PhysicalLocation {
                    artifact_location: ArtifactLocation {
                        uri: "src/main.rs".to_string(),
                        uri_base_id: None,
                    },
                    region: None,
                },
            }],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        run.results.push(Result {
            rule_id: "rule2".to_string(),
            level: None,
            message: Message::default(),
            locations: vec![Location {
                physical_location: PhysicalLocation {
                    artifact_location: ArtifactLocation {
                        uri: "src/lib.rs".to_string(),
                        uri_base_id: None,
                    },
                    region: None,
                },
            }],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        run.results.push(Result {
            rule_id: "rule3".to_string(),
            level: None,
            message: Message::default(),
            locations: vec![Location {
                physical_location: PhysicalLocation {
                    artifact_location: ArtifactLocation {
                        uri: "src/main.rs".to_string(), // duplicate
                        uri_base_id: None,
                    },
                    region: None,
                },
            }],
            fingerprints: None,
            properties: serde_json::Value::Null,
        });
        sarif.runs.push(run);

        let files = sarif.affected_files();
        assert_eq!(files.len(), 2);
        assert!(files.contains("src/main.rs"));
        assert!(files.contains("src/lib.rs"));
    }

    #[test]
    fn test_parse_sarif() {
        let json = r#"{
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "TestTool",
                        "version": "1.0.0",
                        "rules": []
                    }
                },
                "results": []
            }]
        }"#;

        let sarif: Sarif = serde_json::from_str(json).unwrap();
        assert_eq!(sarif.runs.len(), 1);
        assert_eq!(sarif.runs[0].tool.driver.name, "TestTool");
    }

    #[test]
    fn test_parse_sarif_with_results() {
        let json = r#"{
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "TestTool",
                        "version": "1.0.0",
                        "rules": [{
                            "id": "test-rule",
                            "shortDescription": {"text": "Test rule description"}
                        }]
                    }
                },
                "results": [{
                    "ruleId": "test-rule",
                    "level": "warning",
                    "message": {"text": "Found an issue"},
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {"uri": "src/test.rs"},
                            "region": {
                                "startLine": 10,
                                "startColumn": 5,
                                "endLine": 10,
                                "endColumn": 20
                            }
                        }
                    }]
                }]
            }]
        }"#;

        let sarif: Sarif = serde_json::from_str(json).unwrap();
        assert_eq!(sarif.result_count(), 1);
        let result = &sarif.runs[0].results[0];
        assert_eq!(result.rule_id, "test-rule");
        assert_eq!(result.level, Some("warning".to_string()));
        assert_eq!(result.message.text, "Found an issue");

        let location = &result.locations[0];
        assert_eq!(location.physical_location.artifact_location.uri, "src/test.rs");
        let region = location.physical_location.region.as_ref().unwrap();
        assert_eq!(region.start_line, Some(10));
        assert_eq!(region.start_column, Some(5));
    }

    #[test]
    fn test_sarif_serialization_roundtrip() {
        let mut sarif = Sarif::new();
        let run = Run {
            tool: Tool {
                driver: ToolDriver {
                    name: "test-tool".to_string(),
                    version: Some("1.0.0".to_string()),
                    information_uri: Some("https://example.com".to_string()),
                    rules: vec![Rule {
                        id: "rule1".to_string(),
                        name: Some("Test Rule".to_string()),
                        short_description: Some(Message { text: "Short".to_string(), markdown: None }),
                        full_description: Some(Message { text: "Full description".to_string(), markdown: None }),
                        help_uri: Some("https://example.com/rule1".to_string()),
                        default_configuration: Some(RuleConfiguration { level: Some("warning".to_string()) }),
                        properties: serde_json::json!({"category": "security"}),
                    }],
                },
            },
            results: vec![Result {
                rule_id: "rule1".to_string(),
                level: Some("warning".to_string()),
                message: Message { text: "Test message".to_string(), markdown: Some("**Test**".to_string()) },
                locations: vec![Location {
                    physical_location: PhysicalLocation {
                        artifact_location: ArtifactLocation {
                            uri: "src/main.rs".to_string(),
                            uri_base_id: Some("%SRCROOT%".to_string()),
                        },
                        region: Some(Region {
                            start_line: Some(10),
                            start_column: Some(5),
                            end_line: Some(10),
                            end_column: Some(20),
                        }),
                    },
                }],
                fingerprints: Some(serde_json::json!({"v1": "abc123"})),
                properties: serde_json::json!({"tags": ["security"]}),
            }],
            invocations: vec![Invocation {
                execution_successful: true,
                exit_code: Some(0),
                command_line: Some("test-tool scan".to_string()),
                working_directory: Some(ArtifactLocation {
                    uri: "/home/user/project".to_string(),
                    uri_base_id: None,
                }),
            }],
        };
        sarif.runs.push(run);

        // Serialize to JSON
        let json = serde_json::to_string(&sarif).unwrap();

        // Deserialize back
        let parsed: Sarif = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.version, "2.1.0");
        assert_eq!(parsed.runs.len(), 1);
        assert_eq!(parsed.runs[0].tool.driver.name, "test-tool");
        assert_eq!(parsed.runs[0].results.len(), 1);
        assert_eq!(parsed.runs[0].invocations.len(), 1);
        assert!(parsed.runs[0].invocations[0].execution_successful);
    }

    #[test]
    fn test_rule_configuration() {
        let config = RuleConfiguration { level: Some("error".to_string()) };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("error"));

        let parsed: RuleConfiguration = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.level, Some("error".to_string()));
    }

    #[test]
    fn test_invocation_default() {
        let inv = Invocation::default();
        assert!(!inv.execution_successful);
        assert!(inv.exit_code.is_none());
        assert!(inv.command_line.is_none());
    }

    #[test]
    fn test_region_default() {
        let region = Region::default();
        assert!(region.start_line.is_none());
        assert!(region.start_column.is_none());
        assert!(region.end_line.is_none());
        assert!(region.end_column.is_none());
    }

    #[test]
    fn test_message_default() {
        let msg = Message::default();
        assert_eq!(msg.text, "");
        assert!(msg.markdown.is_none());
    }
}

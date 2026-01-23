//! SARIF 2.1.0 type definitions
//!
//! Static Analysis Results Interchange Format (SARIF) types
//! for parsing and generating SARIF output.

use serde::{Deserialize, Serialize};

/// SARIF 2.1.0 root object
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
}

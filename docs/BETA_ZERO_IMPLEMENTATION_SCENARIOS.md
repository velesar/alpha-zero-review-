# Beta-Zero Implementation Scenarios

**Detailed Technical Implementation Guide**

**Version:** 1.0
**Date:** January 2026
**Status:** Planning
**Author:** Light IT Global

---

## Overview

This document provides detailed implementation scenarios for each Beta-Zero component. Each scenario includes:
- Prerequisites
- Step-by-step implementation
- Code examples
- Test cases
- Integration points

---

## Table of Contents

1. [Scenario 1: sarif-tools-server Implementation](#scenario-1-sarif-tools-server-implementation)
2. [Scenario 2: codegraph-server Implementation](#scenario-2-codegraph-server-implementation)
3. [Scenario 3: mental-model-server Extensions](#scenario-3-mental-model-server-extensions)
4. [Scenario 4: methodology-kb-server Extensions](#scenario-4-methodology-kb-server-extensions)
5. [Scenario 5: KB Content Population](#scenario-5-kb-content-population)
6. [Scenario 6: End-to-End Integration](#scenario-6-end-to-end-integration)
7. [Scenario 7: CI/CD Integration](#scenario-7-cicd-integration)

---

## Scenario 1: sarif-tools-server Implementation

### 1.1 Prerequisites

```bash
# Required tools installed on target system
semgrep --version    # >= 1.0
bandit --version     # >= 1.7
ruff --version       # >= 0.1
trivy --version      # >= 0.45
```

### 1.2 Project Setup

```bash
# Create new MCP server
cd /path/to/alpha-zero-review-
cargo new sarif-tools-server
cd sarif-tools-server
```

**Cargo.toml:**
```toml
[package]
name = "sarif-tools-server"
version = "0.1.0"
edition = "2021"

[dependencies]
rmcp = "0.3"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
which = "6"              # Tool detection
tempfile = "3"           # Temp file handling

[dev-dependencies]
tokio-test = "0.4"
```

### 1.3 Implementation Steps

#### Step 1: Define SARIF Types

```rust
// src/sarif.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sarif {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub version: String,
    pub runs: Vec<Run>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub tool: Tool,
    pub results: Vec<Result>,
    #[serde(default)]
    pub invocations: Vec<Invocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub driver: ToolDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDriver {
    pub name: String,
    pub version: Option<String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "shortDescription")]
    pub short_description: Option<Message>,
    #[serde(rename = "defaultConfiguration")]
    pub default_configuration: Option<RuleConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfiguration {
    pub level: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Result {
    #[serde(rename = "ruleId")]
    pub rule_id: String,
    pub level: Option<String>,
    pub message: Message,
    pub locations: Vec<Location>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "physicalLocation")]
    pub physical_location: PhysicalLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalLocation {
    #[serde(rename = "artifactLocation")]
    pub artifact_location: ArtifactLocation,
    pub region: Option<Region>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactLocation {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    #[serde(rename = "startLine")]
    pub start_line: u32,
    #[serde(rename = "startColumn")]
    pub start_column: Option<u32>,
    #[serde(rename = "endLine")]
    pub end_line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invocation {
    #[serde(rename = "executionSuccessful")]
    pub execution_successful: bool,
    #[serde(rename = "exitCode")]
    pub exit_code: Option<i32>,
}

impl Sarif {
    pub fn new() -> Self {
        Self {
            schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: Vec::new(),
        }
    }

    pub fn merge(&mut self, other: Sarif) {
        self.runs.extend(other.runs);
    }

    pub fn result_count(&self) -> usize {
        self.runs.iter().map(|r| r.results.len()).sum()
    }
}
```

#### Step 2: Tool Runner Abstraction

```rust
// src/runner.rs
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct ToolResult {
    pub sarif: Sarif,
    pub exit_code: i32,
    pub stderr: Option<String>,
}

pub trait ToolRunner: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn version(&self) -> Option<String>;
    fn supported_languages(&self) -> Vec<String>;
    fn run(&self, path: &Path, config: Option<&serde_json::Value>) -> Result<ToolResult, RunnerError>;
}

// Tool detection helper
pub fn detect_tool(name: &str) -> bool {
    which::which(name).is_ok()
}

pub fn get_tool_version(name: &str) -> Option<String> {
    let output = Command::new(name)
        .arg("--version")
        .output()
        .ok()?;

    String::from_utf8(output.stdout)
        .ok()
        .map(|s| s.lines().next().unwrap_or("").to_string())
}
```

#### Step 3: Implement Tool Runners

```rust
// src/tools/semgrep.rs
use crate::runner::{RunnerError, ToolResult, ToolRunner, detect_tool, get_tool_version};
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

pub struct SemgrepRunner;

impl ToolRunner for SemgrepRunner {
    fn name(&self) -> &str {
        "semgrep"
    }

    fn is_available(&self) -> bool {
        detect_tool("semgrep")
    }

    fn version(&self) -> Option<String> {
        get_tool_version("semgrep")
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "python", "javascript", "typescript", "java", "go",
            "ruby", "rust", "c", "cpp", "csharp", "kotlin", "scala"
        ].into_iter().map(String::from).collect()
    }

    fn run(&self, path: &Path, config: Option<&serde_json::Value>) -> Result<ToolResult, RunnerError> {
        let output_file = NamedTempFile::new()?;

        let mut cmd = Command::new("semgrep");
        cmd.arg("scan")
           .arg("--sarif")
           .arg("-o")
           .arg(output_file.path())
           .arg(path);

        // Apply custom config if provided
        if let Some(cfg) = config {
            if let Some(rules) = cfg.get("rules").and_then(|v| v.as_str()) {
                cmd.arg("--config").arg(rules);
            } else {
                cmd.arg("--config").arg("auto");
            }
        } else {
            cmd.arg("--config").arg("auto");
        }

        let output = cmd.output()?;
        let exit_code = output.status.code().unwrap_or(-1);

        let sarif_content = std::fs::read_to_string(output_file.path())?;
        let sarif: Sarif = serde_json::from_str(&sarif_content)
            .map_err(|e| RunnerError::ParseError(e.to_string()))?;

        let stderr = if output.stderr.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        };

        Ok(ToolResult {
            sarif,
            exit_code,
            stderr,
        })
    }
}
```

```rust
// src/tools/bandit.rs
use crate::runner::{RunnerError, ToolResult, ToolRunner, detect_tool, get_tool_version};
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

pub struct BanditRunner;

impl ToolRunner for BanditRunner {
    fn name(&self) -> &str {
        "bandit"
    }

    fn is_available(&self) -> bool {
        detect_tool("bandit")
    }

    fn version(&self) -> Option<String> {
        get_tool_version("bandit")
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["python".to_string()]
    }

    fn run(&self, path: &Path, config: Option<&serde_json::Value>) -> Result<ToolResult, RunnerError> {
        let output_file = NamedTempFile::new()?;

        let mut cmd = Command::new("bandit");
        cmd.arg("-r")
           .arg(path)
           .arg("-f")
           .arg("sarif")
           .arg("-o")
           .arg(output_file.path());

        // Apply severity filter if provided
        if let Some(cfg) = config {
            if let Some(severity) = cfg.get("severity").and_then(|v| v.as_str()) {
                cmd.arg("-l").arg(severity);
            }
        }

        let output = cmd.output()?;
        let exit_code = output.status.code().unwrap_or(-1);

        let sarif_content = std::fs::read_to_string(output_file.path())?;
        let sarif: Sarif = serde_json::from_str(&sarif_content)
            .map_err(|e| RunnerError::ParseError(e.to_string()))?;

        let stderr = if output.stderr.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        };

        Ok(ToolResult {
            sarif,
            exit_code,
            stderr,
        })
    }
}
```

```rust
// src/tools/ruff.rs
use crate::runner::{RunnerError, ToolResult, ToolRunner, detect_tool, get_tool_version};
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;

pub struct RuffRunner;

impl ToolRunner for RuffRunner {
    fn name(&self) -> &str {
        "ruff"
    }

    fn is_available(&self) -> bool {
        detect_tool("ruff")
    }

    fn version(&self) -> Option<String> {
        get_tool_version("ruff")
    }

    fn supported_languages(&self) -> Vec<String> {
        vec!["python".to_string()]
    }

    fn run(&self, path: &Path, _config: Option<&serde_json::Value>) -> Result<ToolResult, RunnerError> {
        let output = Command::new("ruff")
            .arg("check")
            .arg("--output-format")
            .arg("sarif")
            .arg(path)
            .output()?;

        let exit_code = output.status.code().unwrap_or(-1);

        let sarif: Sarif = serde_json::from_slice(&output.stdout)
            .map_err(|e| RunnerError::ParseError(e.to_string()))?;

        let stderr = if output.stderr.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        };

        Ok(ToolResult {
            sarif,
            exit_code,
            stderr,
        })
    }
}
```

```rust
// src/tools/trivy.rs
use crate::runner::{RunnerError, ToolResult, ToolRunner, detect_tool, get_tool_version};
use crate::sarif::Sarif;
use std::path::Path;
use std::process::Command;

pub struct TrivyRunner;

impl ToolRunner for TrivyRunner {
    fn name(&self) -> &str {
        "trivy"
    }

    fn is_available(&self) -> bool {
        detect_tool("trivy")
    }

    fn version(&self) -> Option<String> {
        get_tool_version("trivy")
    }

    fn supported_languages(&self) -> Vec<String> {
        vec![
            "python", "javascript", "java", "go", "ruby", "rust",
            "dockerfile", "terraform", "kubernetes"
        ].into_iter().map(String::from).collect()
    }

    fn run(&self, path: &Path, config: Option<&serde_json::Value>) -> Result<ToolResult, RunnerError> {
        let mut cmd = Command::new("trivy");
        cmd.arg("fs")
           .arg("--format")
           .arg("sarif")
           .arg(path);

        // Apply scanners filter if provided
        if let Some(cfg) = config {
            if let Some(scanners) = cfg.get("scanners").and_then(|v| v.as_str()) {
                cmd.arg("--scanners").arg(scanners);
            }
        }

        let output = cmd.output()?;
        let exit_code = output.status.code().unwrap_or(-1);

        let sarif: Sarif = serde_json::from_slice(&output.stdout)
            .map_err(|e| RunnerError::ParseError(e.to_string()))?;

        let stderr = if output.stderr.is_empty() {
            None
        } else {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        };

        Ok(ToolResult {
            sarif,
            exit_code,
            stderr,
        })
    }
}
```

```rust
// src/tools/mod.rs
mod semgrep;
mod bandit;
mod ruff;
mod trivy;

pub use semgrep::SemgrepRunner;
pub use bandit::BanditRunner;
pub use ruff::RuffRunner;
pub use trivy::TrivyRunner;

use crate::runner::ToolRunner;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn ToolRunner>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut tools: HashMap<String, Arc<dyn ToolRunner>> = HashMap::new();

        tools.insert("semgrep".to_string(), Arc::new(SemgrepRunner));
        tools.insert("bandit".to_string(), Arc::new(BanditRunner));
        tools.insert("ruff".to_string(), Arc::new(RuffRunner));
        tools.insert("trivy".to_string(), Arc::new(TrivyRunner));

        Self { tools }
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolRunner>> {
        self.tools.get(name).cloned()
    }

    pub fn list_available(&self) -> Vec<ToolInfo> {
        self.tools
            .values()
            .map(|runner| ToolInfo {
                name: runner.name().to_string(),
                languages: runner.supported_languages(),
                installed: runner.is_available(),
                version: runner.version(),
            })
            .collect()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub languages: Vec<String>,
    pub installed: bool,
    pub version: Option<String>,
}
```

#### Step 4: SARIF Normalization

```rust
// src/normalize.rs
use crate::sarif::{Result as SarifResult, Sarif};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMapping {
    pub category: String,
    pub subcategory: Option<String>,
    pub cwe: Option<String>,
    pub severity_base: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMappings {
    pub mappings: HashMap<String, RuleMapping>,
}

impl RuleMappings {
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        let mappings: RuleMappings = serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(mappings)
    }

    pub fn get(&self, rule_id: &str) -> Option<&RuleMapping> {
        self.mappings.get(rule_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedResult {
    #[serde(flatten)]
    pub original: SarifResult,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub cwe: Option<String>,
    pub severity_base: Option<String>,
}

pub fn normalize_sarif(sarif: &Sarif, mappings: &RuleMappings) -> Vec<EnrichedResult> {
    let mut enriched = Vec::new();

    for run in &sarif.runs {
        for result in &run.results {
            let mapping = mappings.get(&result.rule_id);

            enriched.push(EnrichedResult {
                original: result.clone(),
                category: mapping.map(|m| m.category.clone()),
                subcategory: mapping.and_then(|m| m.subcategory.clone()),
                cwe: mapping.and_then(|m| m.cwe.clone()),
                severity_base: mapping.map(|m| m.severity_base.clone()),
            });
        }
    }

    enriched
}
```

#### Step 5: MCP Server Implementation

```rust
// src/server.rs
use crate::normalize::{normalize_sarif, RuleMappings};
use crate::runner::ToolRunner;
use crate::sarif::Sarif;
use crate::tools::{ToolInfo, ToolRegistry};
use rmcp::{ServerHandler, Tool, ToolError, model::*};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;

pub struct SarifToolsServer {
    registry: ToolRegistry,
    mappings_path: Option<PathBuf>,
}

impl SarifToolsServer {
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
            mappings_path: None,
        }
    }

    pub fn with_mappings(mut self, path: PathBuf) -> Self {
        self.mappings_path = Some(path);
        self
    }
}

// Tool input/output types
#[derive(Debug, Deserialize)]
struct RunToolInput {
    tool: String,
    path: String,
    config: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RunToolOutput {
    sarif: Sarif,
    exit_code: i32,
    stderr: Option<String>,
}

#[derive(Debug, Serialize)]
struct ListToolsOutput {
    tools: Vec<ToolInfo>,
}

#[derive(Debug, Deserialize)]
struct MergeSarifInput {
    sarif_files: Vec<Sarif>,
}

#[derive(Debug, Serialize)]
struct MergeSarifOutput {
    combined: Sarif,
    total_results: usize,
}

#[derive(Debug, Deserialize)]
struct NormalizeSarifInput {
    sarif: Sarif,
    rule_mappings: String,
}

#[derive(Debug, Deserialize)]
struct GetToolConfigInput {
    tool: String,
}

#[derive(Debug, Serialize)]
struct GetToolConfigOutput {
    name: String,
    command: String,
    supported_languages: Vec<String>,
    installed: bool,
    version: Option<String>,
}

#[rmcp::async_trait]
impl ServerHandler for SarifToolsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: "sarif-tools-server".to_string(),
            version: "0.1.0".to_string(),
            instructions: Some("SARIF-based code analysis tool server".to_string()),
            ..Default::default()
        }
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "run_tool".to_string(),
                description: "Run a code analysis tool and get SARIF output".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "tool": {
                            "type": "string",
                            "enum": ["semgrep", "bandit", "ruff", "trivy"],
                            "description": "Tool to run"
                        },
                        "path": {
                            "type": "string",
                            "description": "Path to analyze"
                        },
                        "config": {
                            "type": "object",
                            "description": "Tool-specific configuration"
                        }
                    },
                    "required": ["tool", "path"]
                }),
            },
            Tool {
                name: "list_available_tools".to_string(),
                description: "List all available analysis tools".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "merge_sarif".to_string(),
                description: "Merge multiple SARIF files into one".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "sarif_files": {
                            "type": "array",
                            "items": { "type": "object" },
                            "description": "SARIF objects to merge"
                        }
                    },
                    "required": ["sarif_files"]
                }),
            },
            Tool {
                name: "normalize_sarif".to_string(),
                description: "Enrich SARIF with category and severity mappings".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "sarif": {
                            "type": "object",
                            "description": "SARIF object to normalize"
                        },
                        "rule_mappings": {
                            "type": "string",
                            "description": "Path to rule mappings YAML file"
                        }
                    },
                    "required": ["sarif", "rule_mappings"]
                }),
            },
            Tool {
                name: "get_tool_config".to_string(),
                description: "Get configuration for a specific tool".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "tool": {
                            "type": "string",
                            "description": "Tool name"
                        }
                    },
                    "required": ["tool"]
                }),
            },
        ]
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Vec<Content>, ToolError> {
        match name {
            "run_tool" => {
                let input: RunToolInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let runner = self.registry.get(&input.tool)
                    .ok_or_else(|| ToolError::ExecutionError(format!("Unknown tool: {}", input.tool)))?;

                if !runner.is_available() {
                    return Err(ToolError::ExecutionError(format!("Tool not installed: {}", input.tool)));
                }

                let path = PathBuf::from(&input.path);
                let result = runner.run(&path, input.config.as_ref())
                    .map_err(|e| ToolError::ExecutionError(e.to_string()))?;

                let output = RunToolOutput {
                    sarif: result.sarif,
                    exit_code: result.exit_code,
                    stderr: result.stderr,
                };

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&output).unwrap(),
                }])
            }

            "list_available_tools" => {
                let output = ListToolsOutput {
                    tools: self.registry.list_available(),
                };

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&output).unwrap(),
                }])
            }

            "merge_sarif" => {
                let input: MergeSarifInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let mut combined = Sarif::new();
                for sarif in input.sarif_files {
                    combined.merge(sarif);
                }

                let total_results = combined.result_count();
                let output = MergeSarifOutput { combined, total_results };

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&output).unwrap(),
                }])
            }

            "normalize_sarif" => {
                let input: NormalizeSarifInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let mappings_path = PathBuf::from(&input.rule_mappings);
                let mappings = RuleMappings::load(&mappings_path)
                    .map_err(|e| ToolError::ExecutionError(format!("Failed to load mappings: {}", e)))?;

                let enriched = normalize_sarif(&input.sarif, &mappings);

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&enriched).unwrap(),
                }])
            }

            "get_tool_config" => {
                let input: GetToolConfigInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let runner = self.registry.get(&input.tool)
                    .ok_or_else(|| ToolError::ExecutionError(format!("Unknown tool: {}", input.tool)))?;

                let output = GetToolConfigOutput {
                    name: runner.name().to_string(),
                    command: runner.name().to_string(),
                    supported_languages: runner.supported_languages(),
                    installed: runner.is_available(),
                    version: runner.version(),
                };

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&output).unwrap(),
                }])
            }

            _ => Err(ToolError::NotFound(name.to_string())),
        }
    }
}
```

#### Step 6: Main Entry Point

```rust
// src/main.rs
mod normalize;
mod runner;
mod sarif;
mod server;
mod tools;

use rmcp::ServiceExt;
use server::SarifToolsServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting sarif-tools-server");

    let server = SarifToolsServer::new();
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
```

### 1.4 Test Cases

```rust
// tests/integration_tests.rs
use sarif_tools_server::tools::ToolRegistry;
use sarif_tools_server::sarif::Sarif;
use std::path::PathBuf;

#[test]
fn test_tool_registry_creation() {
    let registry = ToolRegistry::new();
    let tools = registry.list_available();

    assert!(tools.iter().any(|t| t.name == "semgrep"));
    assert!(tools.iter().any(|t| t.name == "bandit"));
    assert!(tools.iter().any(|t| t.name == "ruff"));
    assert!(tools.iter().any(|t| t.name == "trivy"));
}

#[test]
fn test_sarif_merge() {
    let mut sarif1 = Sarif::new();
    let mut sarif2 = Sarif::new();

    // Add mock runs...

    sarif1.merge(sarif2);
    // Assert merged state
}

#[tokio::test]
async fn test_semgrep_runner() {
    let registry = ToolRegistry::new();
    let runner = registry.get("semgrep").unwrap();

    if runner.is_available() {
        let test_path = PathBuf::from("./test_fixtures/python_sample");
        let result = runner.run(&test_path, None);
        assert!(result.is_ok());
    }
}
```

### 1.5 MCP Configuration

```json
// Add to .mcp.json
{
  "mcpServers": {
    "sarif-tools": {
      "command": "./target/release/sarif-tools-server",
      "args": [],
      "env": {}
    }
  }
}
```

---

## Scenario 2: codegraph-server Implementation

### 2.1 Prerequisites

```bash
# SCIP proto file
curl -o proto/scip.proto https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto

# SCIP indexers (optional, for testing)
pip install scip-python
npm install -g @sourcegraph/scip-typescript
```

### 2.2 Project Setup

```bash
cargo new codegraph-server
cd codegraph-server
```

**Cargo.toml:**
```toml
[package]
name = "codegraph-server"
version = "0.1.0"
edition = "2021"

[dependencies]
rmcp = "0.3"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
prost = "0.12"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"

[build-dependencies]
prost-build = "0.12"
```

**build.rs:**
```rust
fn main() {
    prost_build::compile_protos(&["proto/scip.proto"], &["proto/"])
        .expect("Failed to compile SCIP proto");
}
```

### 2.3 Implementation Steps

#### Step 1: SCIP Types (Generated)

```rust
// src/scip.rs
// Include generated proto types
include!(concat!(env!("OUT_DIR"), "/scip.rs"));
```

#### Step 2: Codegraph Data Structure

```rust
// src/graph.rs
use crate::scip::{Index, Document, Occurrence, SymbolInformation, SymbolRole};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Symbol {
    pub id: String,
    pub kind: SymbolKind,
    pub name: String,
    pub file: String,
    pub range: Range,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum SymbolKind {
    Class,
    Function,
    Method,
    Variable,
    Constant,
    Module,
    Interface,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct Range {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Reference {
    pub symbol_id: String,
    pub file: String,
    pub line: u32,
    pub role: ReferenceRole,
}

#[derive(Debug, Clone, Serialize)]
pub enum ReferenceRole {
    Definition,
    Reference,
    Call,
    Import,
}

#[derive(Debug)]
pub struct Codegraph {
    pub symbols: HashMap<String, Symbol>,
    pub definitions: HashMap<String, Vec<Reference>>,  // symbol_id -> definitions
    pub references: HashMap<String, Vec<Reference>>,   // symbol_id -> references
    pub file_symbols: HashMap<String, Vec<String>>,    // file -> symbol_ids
}

impl Codegraph {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            definitions: HashMap::new(),
            references: HashMap::new(),
            file_symbols: HashMap::new(),
        }
    }

    pub fn load_from_scip(path: &Path) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        let index: Index = prost::Message::decode(&data[..])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut graph = Self::new();

        for doc in &index.documents {
            graph.process_document(doc);
        }

        Ok(graph)
    }

    fn process_document(&mut self, doc: &Document) {
        let file = &doc.relative_path;

        // Process symbol information
        for sym_info in &doc.symbols {
            let symbol = Symbol {
                id: sym_info.symbol.clone(),
                kind: Self::parse_kind(&sym_info.symbol),
                name: Self::extract_name(&sym_info.symbol),
                file: file.clone(),
                range: Range {
                    start_line: 0, // Will be updated from occurrences
                    start_column: 0,
                    end_line: 0,
                    end_column: 0,
                },
                documentation: sym_info.documentation.first().cloned(),
            };

            self.symbols.insert(sym_info.symbol.clone(), symbol);
            self.file_symbols
                .entry(file.clone())
                .or_default()
                .push(sym_info.symbol.clone());
        }

        // Process occurrences
        for occ in &doc.occurrences {
            let reference = Reference {
                symbol_id: occ.symbol.clone(),
                file: file.clone(),
                line: occ.range.first().copied().unwrap_or(0) as u32,
                role: Self::parse_role(occ.symbol_roles),
            };

            if occ.symbol_roles & (SymbolRole::Definition as i32) != 0 {
                self.definitions
                    .entry(occ.symbol.clone())
                    .or_default()
                    .push(reference);
            } else {
                self.references
                    .entry(occ.symbol.clone())
                    .or_default()
                    .push(reference);
            }
        }
    }

    fn parse_kind(symbol: &str) -> SymbolKind {
        if symbol.contains("#") {
            SymbolKind::Method
        } else if symbol.contains("()") {
            SymbolKind::Function
        } else if symbol.starts_with(|c: char| c.is_uppercase()) {
            SymbolKind::Class
        } else {
            SymbolKind::Variable
        }
    }

    fn extract_name(symbol: &str) -> String {
        symbol.split('/').last()
            .unwrap_or(symbol)
            .split('#').last()
            .unwrap_or(symbol)
            .trim_end_matches("().")
            .to_string()
    }

    fn parse_role(roles: i32) -> ReferenceRole {
        if roles & (SymbolRole::Definition as i32) != 0 {
            ReferenceRole::Definition
        } else if roles & (SymbolRole::Import as i32) != 0 {
            ReferenceRole::Import
        } else {
            ReferenceRole::Reference
        }
    }

    // Query methods
    pub fn get_symbol(&self, id: &str) -> Option<&Symbol> {
        self.symbols.get(id)
    }

    pub fn get_callers(&self, symbol_id: &str) -> Vec<&Reference> {
        self.references.get(symbol_id)
            .map(|refs| refs.iter().collect())
            .unwrap_or_default()
    }

    pub fn get_callees(&self, symbol_id: &str) -> Vec<&Reference> {
        // Find all references made FROM the definition of this symbol
        // This requires tracking which symbols are referenced within a symbol's scope
        // Simplified: return empty for now, full implementation needs scope analysis
        Vec::new()
    }

    pub fn get_impact(&self, symbol_id: &str) -> Impact {
        let refs = self.get_callers(symbol_id);
        let affected_files: std::collections::HashSet<_> = refs.iter()
            .map(|r| r.file.as_str())
            .collect();

        Impact {
            direct_references: refs.len(),
            affected_files: affected_files.len(),
            files: affected_files.into_iter().map(String::from).collect(),
        }
    }

    pub fn get_module_deps(&self, module_path: &str) -> ModuleDeps {
        let symbols: Vec<_> = self.file_symbols.get(module_path)
            .map(|ids| ids.iter().filter_map(|id| self.symbols.get(id)).collect())
            .unwrap_or_default();

        let mut deps: HashMap<String, usize> = HashMap::new();

        for sym in &symbols {
            for refs in self.references.get(&sym.id).into_iter().flatten() {
                *deps.entry(refs.file.clone()).or_default() += 1;
            }
        }

        ModuleDeps {
            module: module_path.to_string(),
            symbols_count: symbols.len(),
            depends_on: deps,
        }
    }

    pub fn get_file_symbols(&self, file_path: &str) -> Vec<&Symbol> {
        self.file_symbols.get(file_path)
            .map(|ids| ids.iter().filter_map(|id| self.symbols.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_symbol(&self, pattern: &str) -> Vec<&Symbol> {
        self.symbols.values()
            .filter(|s| s.name.contains(pattern) || s.id.contains(pattern))
            .collect()
    }

    pub fn find_hotspots(&self, min_callers: usize, path_filter: Option<&str>) -> Vec<Hotspot> {
        self.symbols.values()
            .filter(|s| {
                if let Some(filter) = path_filter {
                    s.file.contains(filter)
                } else {
                    true
                }
            })
            .filter_map(|s| {
                let caller_count = self.get_callers(&s.id).len();
                if caller_count >= min_callers {
                    Some(Hotspot {
                        symbol_id: s.id.clone(),
                        file: s.file.clone(),
                        caller_count,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn symbols_count(&self) -> usize {
        self.symbols.len()
    }

    pub fn files_count(&self) -> usize {
        self.file_symbols.len()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Impact {
    pub direct_references: usize,
    pub affected_files: usize,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleDeps {
    pub module: String,
    pub symbols_count: usize,
    pub depends_on: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Hotspot {
    pub symbol_id: String,
    pub file: String,
    pub caller_count: usize,
}
```

#### Step 3: MCP Server Implementation

```rust
// src/server.rs
use crate::graph::{Codegraph, Hotspot, Impact, ModuleDeps, Symbol};
use rmcp::{ServerHandler, Tool, ToolError, model::*};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub struct CodegraphServer {
    graph: Arc<RwLock<Option<Codegraph>>>,
}

impl CodegraphServer {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(None)),
        }
    }
}

// Input/Output types
#[derive(Debug, Deserialize)]
struct LoadIndexInput {
    scip_path: String,
}

#[derive(Debug, Serialize)]
struct LoadIndexOutput {
    status: String,
    symbols_count: usize,
    files_count: usize,
}

#[derive(Debug, Deserialize)]
struct SymbolInput {
    symbol_id: String,
}

#[derive(Debug, Deserialize)]
struct FileInput {
    file_path: String,
}

#[derive(Debug, Deserialize)]
struct FindSymbolInput {
    pattern: String,
}

#[derive(Debug, Deserialize)]
struct FindHotspotsInput {
    min_callers: usize,
    path_filter: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModuleDepsInput {
    module_path: String,
}

#[rmcp::async_trait]
impl ServerHandler for CodegraphServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: "codegraph-server".to_string(),
            version: "0.1.0".to_string(),
            instructions: Some("SCIP-based semantic code intelligence server".to_string()),
            ..Default::default()
        }
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "load_index".to_string(),
                description: "Load a SCIP index file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "scip_path": {
                            "type": "string",
                            "description": "Path to SCIP index file"
                        }
                    },
                    "required": ["scip_path"]
                }),
            },
            Tool {
                name: "get_symbol_info".to_string(),
                description: "Get information about a symbol".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "symbol_id": {
                            "type": "string",
                            "description": "Symbol identifier"
                        }
                    },
                    "required": ["symbol_id"]
                }),
            },
            Tool {
                name: "get_callers".to_string(),
                description: "Get all callers of a symbol".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "symbol_id": {
                            "type": "string",
                            "description": "Symbol identifier"
                        }
                    },
                    "required": ["symbol_id"]
                }),
            },
            Tool {
                name: "get_callees".to_string(),
                description: "Get all callees from a symbol".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "symbol_id": {
                            "type": "string",
                            "description": "Symbol identifier"
                        }
                    },
                    "required": ["symbol_id"]
                }),
            },
            Tool {
                name: "get_impact".to_string(),
                description: "Get impact analysis for a symbol".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "symbol_id": {
                            "type": "string",
                            "description": "Symbol identifier"
                        }
                    },
                    "required": ["symbol_id"]
                }),
            },
            Tool {
                name: "get_module_deps".to_string(),
                description: "Get dependencies of a module".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "module_path": {
                            "type": "string",
                            "description": "Path to module"
                        }
                    },
                    "required": ["module_path"]
                }),
            },
            Tool {
                name: "get_file_symbols".to_string(),
                description: "Get all symbols in a file".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to file"
                        }
                    },
                    "required": ["file_path"]
                }),
            },
            Tool {
                name: "find_symbol".to_string(),
                description: "Find symbols by pattern".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "Search pattern"
                        }
                    },
                    "required": ["pattern"]
                }),
            },
            Tool {
                name: "find_hotspot_symbols".to_string(),
                description: "Find symbols with many callers".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "min_callers": {
                            "type": "integer",
                            "description": "Minimum number of callers"
                        },
                        "path_filter": {
                            "type": "string",
                            "description": "Optional path filter"
                        }
                    },
                    "required": ["min_callers"]
                }),
            },
        ]
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Vec<Content>, ToolError> {
        match name {
            "load_index" => {
                let input: LoadIndexInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let path = PathBuf::from(&input.scip_path);
                let graph = Codegraph::load_from_scip(&path)
                    .map_err(|e| ToolError::ExecutionError(format!("Failed to load SCIP: {}", e)))?;

                let output = LoadIndexOutput {
                    status: "loaded".to_string(),
                    symbols_count: graph.symbols_count(),
                    files_count: graph.files_count(),
                };

                *self.graph.write().unwrap() = Some(graph);

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&output).unwrap(),
                }])
            }

            "get_symbol_info" => {
                let input: SymbolInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let graph = self.graph.read().unwrap();
                let graph = graph.as_ref()
                    .ok_or_else(|| ToolError::ExecutionError("No index loaded".to_string()))?;

                let symbol = graph.get_symbol(&input.symbol_id)
                    .ok_or_else(|| ToolError::ExecutionError("Symbol not found".to_string()))?;

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&symbol).unwrap(),
                }])
            }

            "get_callers" => {
                let input: SymbolInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let graph = self.graph.read().unwrap();
                let graph = graph.as_ref()
                    .ok_or_else(|| ToolError::ExecutionError("No index loaded".to_string()))?;

                let callers = graph.get_callers(&input.symbol_id);

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&json!({
                        "caller_count": callers.len(),
                        "callers": callers
                    })).unwrap(),
                }])
            }

            "get_callees" => {
                let input: SymbolInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let graph = self.graph.read().unwrap();
                let graph = graph.as_ref()
                    .ok_or_else(|| ToolError::ExecutionError("No index loaded".to_string()))?;

                let callees = graph.get_callees(&input.symbol_id);

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&json!({
                        "callee_count": callees.len(),
                        "callees": callees
                    })).unwrap(),
                }])
            }

            "get_impact" => {
                let input: SymbolInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let graph = self.graph.read().unwrap();
                let graph = graph.as_ref()
                    .ok_or_else(|| ToolError::ExecutionError("No index loaded".to_string()))?;

                let impact = graph.get_impact(&input.symbol_id);

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&impact).unwrap(),
                }])
            }

            "get_module_deps" => {
                let input: ModuleDepsInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let graph = self.graph.read().unwrap();
                let graph = graph.as_ref()
                    .ok_or_else(|| ToolError::ExecutionError("No index loaded".to_string()))?;

                let deps = graph.get_module_deps(&input.module_path);

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&deps).unwrap(),
                }])
            }

            "get_file_symbols" => {
                let input: FileInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let graph = self.graph.read().unwrap();
                let graph = graph.as_ref()
                    .ok_or_else(|| ToolError::ExecutionError("No index loaded".to_string()))?;

                let symbols = graph.get_file_symbols(&input.file_path);

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&symbols).unwrap(),
                }])
            }

            "find_symbol" => {
                let input: FindSymbolInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let graph = self.graph.read().unwrap();
                let graph = graph.as_ref()
                    .ok_or_else(|| ToolError::ExecutionError("No index loaded".to_string()))?;

                let symbols = graph.find_symbol(&input.pattern);

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&symbols).unwrap(),
                }])
            }

            "find_hotspot_symbols" => {
                let input: FindHotspotsInput = serde_json::from_value(arguments)
                    .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;

                let graph = self.graph.read().unwrap();
                let graph = graph.as_ref()
                    .ok_or_else(|| ToolError::ExecutionError("No index loaded".to_string()))?;

                let hotspots = graph.find_hotspots(
                    input.min_callers,
                    input.path_filter.as_deref(),
                );

                Ok(vec![Content::Text {
                    text: serde_json::to_string_pretty(&hotspots).unwrap(),
                }])
            }

            _ => Err(ToolError::NotFound(name.to_string())),
        }
    }
}
```

#### Step 4: Main Entry Point

```rust
// src/main.rs
mod graph;
mod server;

// Include generated proto
pub mod scip {
    include!(concat!(env!("OUT_DIR"), "/scip.rs"));
}

use rmcp::ServiceExt;
use server::CodegraphServer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting codegraph-server");

    let server = CodegraphServer::new();
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;

    Ok(())
}
```

### 2.4 Test Cases

```rust
// tests/graph_tests.rs
use codegraph_server::graph::Codegraph;
use std::path::PathBuf;

#[test]
fn test_load_scip_index() {
    let path = PathBuf::from("./test_fixtures/sample.scip");
    if path.exists() {
        let graph = Codegraph::load_from_scip(&path).unwrap();
        assert!(graph.symbols_count() > 0);
    }
}

#[test]
fn test_find_hotspots() {
    let mut graph = Codegraph::new();
    // Add test data...

    let hotspots = graph.find_hotspots(5, None);
    // Assert results
}
```

---

## Scenario 3: mental-model-server Extensions

### 3.1 New Tools to Add

Extend existing `mental-model-server` with artifact store functionality.

### 3.2 Implementation

```rust
// Add to existing mental-model-server/src/artifacts.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub commit: String,
    pub branch: Option<String>,
    pub timestamp: String,
    pub artifacts: HashMap<String, ArtifactInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub produced_at: String,
    pub producer: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AvailableArtifact {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub path: String,
    pub produced_at: String,
    pub producer: String,
}

pub struct ArtifactStore {
    base_path: PathBuf,
}

impl ArtifactStore {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn artifacts_dir(&self) -> PathBuf {
        self.base_path.join(".audit").join("artifacts")
    }

    fn commit_dir(&self, commit: &str) -> PathBuf {
        self.artifacts_dir().join(commit)
    }

    fn meta_path(&self, commit: &str) -> PathBuf {
        self.commit_dir(commit).join("_meta.yaml")
    }

    pub fn resolve_commit(&self, commit: &str) -> Result<String, std::io::Error> {
        if commit == "HEAD" || commit == "latest" {
            let latest = self.artifacts_dir().join("latest");
            if latest.is_symlink() {
                let target = fs::read_link(&latest)?;
                return Ok(target.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(commit)
                    .to_string());
            }
        }
        Ok(commit.to_string())
    }

    pub fn get_commit_artifacts(&self, commit: &str) -> Result<(Vec<AvailableArtifact>, Vec<String>), std::io::Error> {
        let commit = self.resolve_commit(commit)?;
        let meta_path = self.meta_path(&commit);

        if !meta_path.exists() {
            return Ok((vec![], vec!["semgrep", "bandit", "scip", "coverage"]
                .into_iter().map(String::from).collect()));
        }

        let content = fs::read_to_string(&meta_path)?;
        let meta: ArtifactMetadata = serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut available = Vec::new();
        let all_types = vec!["semgrep", "bandit", "ruff", "trivy", "scip", "coverage"];
        let mut missing = Vec::new();

        for artifact_type in all_types {
            if let Some(info) = meta.artifacts.get(artifact_type) {
                available.push(AvailableArtifact {
                    artifact_type: artifact_type.to_string(),
                    path: self.commit_dir(&commit)
                        .join(format!("{}.sarif", artifact_type))
                        .to_string_lossy()
                        .to_string(),
                    produced_at: info.produced_at.clone(),
                    producer: info.producer.clone(),
                });
            } else {
                missing.push(artifact_type.to_string());
            }
        }

        Ok((available, missing))
    }

    pub fn store_artifact(
        &self,
        commit: &str,
        artifact_type: &str,
        data: &[u8],
        producer: &str,
    ) -> Result<String, std::io::Error> {
        let commit_dir = self.commit_dir(commit);
        fs::create_dir_all(&commit_dir)?;

        // Determine file extension
        let ext = match artifact_type {
            "scip" => "scip",
            _ => "sarif",
        };
        let artifact_path = commit_dir.join(format!("{}.{}", artifact_type, ext));
        fs::write(&artifact_path, data)?;

        // Update metadata
        let meta_path = self.meta_path(commit);
        let mut meta = if meta_path.exists() {
            let content = fs::read_to_string(&meta_path)?;
            serde_yaml::from_str(&content)
                .unwrap_or_else(|_| ArtifactMetadata {
                    commit: commit.to_string(),
                    branch: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    artifacts: HashMap::new(),
                })
        } else {
            ArtifactMetadata {
                commit: commit.to_string(),
                branch: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
                artifacts: HashMap::new(),
            }
        };

        meta.artifacts.insert(artifact_type.to_string(), ArtifactInfo {
            produced_at: chrono::Utc::now().to_rfc3339(),
            producer: producer.to_string(),
        });

        let meta_yaml = serde_yaml::to_string(&meta)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&meta_path, meta_yaml)?;

        // Update latest symlink
        let latest = self.artifacts_dir().join("latest");
        let _ = fs::remove_file(&latest);
        #[cfg(unix)]
        std::os::unix::fs::symlink(&commit_dir, &latest)?;

        Ok(artifact_path.to_string_lossy().to_string())
    }

    pub fn get_artifact(&self, commit: &str, artifact_type: &str) -> Result<(Vec<u8>, ArtifactInfo), std::io::Error> {
        let commit = self.resolve_commit(commit)?;
        let meta_path = self.meta_path(&commit);

        let content = fs::read_to_string(&meta_path)?;
        let meta: ArtifactMetadata = serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let info = meta.artifacts.get(artifact_type)
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Artifact not found: {}", artifact_type)
            ))?
            .clone();

        let ext = match artifact_type {
            "scip" => "scip",
            _ => "sarif",
        };
        let artifact_path = self.commit_dir(&commit).join(format!("{}.{}", artifact_type, ext));
        let data = fs::read(&artifact_path)?;

        Ok((data, info))
    }
}
```

### 3.3 Add to Server Handler

```rust
// Add to existing server.rs tool handlers

"get_commit_artifacts" => {
    let input: GetCommitArtifactsInput = serde_json::from_value(arguments)?;
    let commit = input.commit.unwrap_or("HEAD".to_string());

    let (available, missing) = self.artifact_store.get_commit_artifacts(&commit)?;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&json!({
            "commit": commit,
            "available": available,
            "missing": missing
        })).unwrap(),
    }])
}

"store_artifact" => {
    let input: StoreArtifactInput = serde_json::from_value(arguments)?;

    let stored_at = self.artifact_store.store_artifact(
        &input.commit,
        &input.artifact_type,
        input.data.as_bytes(),
        &input.metadata.producer,
    )?;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&json!({
            "stored_at": stored_at
        })).unwrap(),
    }])
}

"get_artifact" => {
    let input: GetArtifactInput = serde_json::from_value(arguments)?;

    let (data, metadata) = self.artifact_store.get_artifact(
        &input.commit,
        &input.artifact_type,
    )?;

    Ok(vec![Content::Text {
        text: serde_json::to_string_pretty(&json!({
            "data": String::from_utf8_lossy(&data),
            "metadata": metadata
        })).unwrap(),
    }])
}
```

---

## Scenario 4: methodology-kb-server Extensions

### 4.1 Data Acquisition Implementation

```rust
// Add to methodology-kb-server/src/acquisition.rs
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub project: String,
    pub store: StoreConfig,
    pub artifact_types: std::collections::HashMap<String, ArtifactTypeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreConfig {
    #[serde(rename = "type")]
    pub store_type: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactTypeConfig {
    pub schema: String,
    pub producer: String,
    #[serde(default)]
    pub tools: Vec<String>,
    pub fallback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CIConfig {
    pub ci_system: Option<String>,
    pub artifact_locations: Vec<ArtifactLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactLocation {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AcquisitionSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub source: String,
    pub acquisition_path: String,
}

pub struct DataAcquisition {
    project_path: PathBuf,
}

impl DataAcquisition {
    pub fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    pub fn check_manifest(&self) -> Result<Option<Manifest>, std::io::Error> {
        let manifest_path = self.project_path.join(".audit").join("manifest.yaml");

        if !manifest_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: Manifest = serde_yaml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Some(manifest))
    }

    pub fn parse_ci_config(&self) -> Result<CIConfig, std::io::Error> {
        // Check GitHub Actions
        let github_workflows = self.project_path.join(".github").join("workflows");
        if github_workflows.exists() {
            return self.parse_github_actions(&github_workflows);
        }

        // Check GitLab CI
        let gitlab_ci = self.project_path.join(".gitlab-ci.yml");
        if gitlab_ci.exists() {
            return self.parse_gitlab_ci(&gitlab_ci);
        }

        Ok(CIConfig {
            ci_system: None,
            artifact_locations: vec![],
        })
    }

    fn parse_github_actions(&self, workflows_dir: &Path) -> Result<CIConfig, std::io::Error> {
        let mut locations = Vec::new();

        for entry in std::fs::read_dir(workflows_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "yml" || e == "yaml").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)?;

                // Simple pattern matching for artifact uploads
                if content.contains("upload-artifact") {
                    if content.contains("sarif") || content.contains("security") {
                        locations.push(ArtifactLocation {
                            artifact_type: "security".to_string(),
                            location: format!("github:artifacts:{}",
                                path.file_stem().unwrap_or_default().to_string_lossy()),
                        });
                    }
                    if content.contains("coverage") {
                        locations.push(ArtifactLocation {
                            artifact_type: "coverage".to_string(),
                            location: format!("github:artifacts:{}",
                                path.file_stem().unwrap_or_default().to_string_lossy()),
                        });
                    }
                }
            }
        }

        Ok(CIConfig {
            ci_system: Some("github".to_string()),
            artifact_locations: locations,
        })
    }

    fn parse_gitlab_ci(&self, ci_file: &Path) -> Result<CIConfig, std::io::Error> {
        let content = std::fs::read_to_string(ci_file)?;
        let mut locations = Vec::new();

        // Simple pattern matching
        if content.contains("artifacts:") {
            if content.contains("sast") || content.contains("security") {
                locations.push(ArtifactLocation {
                    artifact_type: "security".to_string(),
                    location: "gitlab:artifacts:security".to_string(),
                });
            }
        }

        Ok(CIConfig {
            ci_system: Some("gitlab".to_string()),
            artifact_locations: locations,
        })
    }
}
```

### 4.2 Cascade Logic

```rust
// Add to methodology-kb-server/src/cascade.rs
use crate::acquisition::{AcquisitionSource, DataAcquisition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AcquireOptions {
    #[serde(default)]
    pub allow_stale: bool,
    pub max_age: Option<String>,
    #[serde(default = "default_generate")]
    pub generate_if_missing: bool,
}

fn default_generate() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub struct AcquireResult {
    pub sarif: serde_json::Value,  // Combined SARIF
    pub sources: Vec<AcquisitionSource>,
}

pub struct AcquisitionCascade {
    acquisition: DataAcquisition,
    // References to other servers would be MCP client calls in practice
}

impl AcquisitionCascade {
    pub fn new(project_path: std::path::PathBuf) -> Self {
        Self {
            acquisition: DataAcquisition::new(project_path),
        }
    }

    pub async fn acquire_findings(
        &self,
        commit: &str,
        types: &[String],
        options: &AcquireOptions,
    ) -> Result<AcquireResult, Box<dyn std::error::Error>> {
        let mut sources = Vec::new();
        let mut combined_sarif = serde_json::json!({
            "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
            "version": "2.1.0",
            "runs": []
        });

        for finding_type in types {
            // Step 1: Check local artifacts
            if let Some(sarif) = self.check_local_artifacts(commit, finding_type).await? {
                sources.push(AcquisitionSource {
                    source_type: finding_type.clone(),
                    source: "local".to_string(),
                    acquisition_path: format!(".audit/artifacts/{}/{}.sarif", commit, finding_type),
                });
                self.merge_sarif(&mut combined_sarif, sarif);
                continue;
            }

            // Step 2: Check manifest
            if let Some(manifest) = self.acquisition.check_manifest()? {
                if let Some(config) = manifest.artifact_types.get(finding_type) {
                    if let Some(sarif) = self.fetch_from_manifest(commit, finding_type, config).await? {
                        sources.push(AcquisitionSource {
                            source_type: finding_type.clone(),
                            source: "manifest".to_string(),
                            acquisition_path: config.producer.clone(),
                        });
                        self.merge_sarif(&mut combined_sarif, sarif);
                        continue;
                    }
                }
            }

            // Step 3: Check CI config
            let ci_config = self.acquisition.parse_ci_config()?;
            if let Some(location) = ci_config.artifact_locations.iter()
                .find(|l| l.artifact_type == *finding_type)
            {
                if let Some(sarif) = self.fetch_from_ci(&ci_config.ci_system, &location.location).await? {
                    sources.push(AcquisitionSource {
                        source_type: finding_type.clone(),
                        source: "ci".to_string(),
                        acquisition_path: location.location.clone(),
                    });
                    self.merge_sarif(&mut combined_sarif, sarif);
                    continue;
                }
            }

            // Step 4: Generate on-demand if allowed
            if options.generate_if_missing {
                if let Some(sarif) = self.generate_findings(finding_type).await? {
                    sources.push(AcquisitionSource {
                        source_type: finding_type.clone(),
                        source: "generated".to_string(),
                        acquisition_path: format!("sarif-tools-server:{}", finding_type),
                    });
                    self.merge_sarif(&mut combined_sarif, sarif);
                }
            }
        }

        Ok(AcquireResult {
            sarif: combined_sarif,
            sources,
        })
    }

    async fn check_local_artifacts(
        &self,
        commit: &str,
        finding_type: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        // In practice, this would call mental-model-server.get_artifact via MCP
        // For now, return None to demonstrate cascade
        Ok(None)
    }

    async fn fetch_from_manifest(
        &self,
        commit: &str,
        finding_type: &str,
        config: &crate::acquisition::ArtifactTypeConfig,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        // Fetch from configured store location
        Ok(None)
    }

    async fn fetch_from_ci(
        &self,
        ci_system: &Option<String>,
        location: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        // Fetch from CI artifact storage
        Ok(None)
    }

    async fn generate_findings(
        &self,
        finding_type: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
        // In practice, this would call sarif-tools-server.run_tool via MCP
        // Map finding_type to tool
        let tool = match finding_type {
            "security" => "semgrep",
            "python_security" => "bandit",
            "python_style" => "ruff",
            "vulnerabilities" => "trivy",
            _ => return Ok(None),
        };

        // Would call: sarif-tools-server.run_tool(tool, project_path)
        Ok(None)
    }

    fn merge_sarif(&self, combined: &mut serde_json::Value, new: serde_json::Value) {
        if let (Some(combined_runs), Some(new_runs)) = (
            combined.get_mut("runs").and_then(|r| r.as_array_mut()),
            new.get("runs").and_then(|r| r.as_array()),
        ) {
            combined_runs.extend(new_runs.iter().cloned());
        }
    }
}
```

---

## Scenario 5: KB Content Population

### 5.1 Glossary Files

```yaml
# methodology_kb/glossary/cognitive_complexity.yaml
id: cognitive_complexity
name: Cognitive Complexity
source: sonarqube
description: |
  Measures how difficult code is to understand.
  Unlike cyclomatic complexity, accounts for nesting depth,
  break in linear flow, and structural complexity.

  Increments for:
  - if, else if, else, switch
  - for, while, do while
  - catch
  - goto, break, continue (to label)
  - Sequences of binary logical operators
  - Recursion
  - Nesting (increments penalty)

thresholds:
  low: 0-10
  medium: 11-20
  high: 21-50
  critical: 51+

project_type_adjustments:
  legacy_system:
    warning: 30
    error: 60
  python_backend:
    warning: 15
    error: 30
  typescript_frontend:
    warning: 20
    error: 40

tools:
  - name: sonarqube
    metric_key: cognitive_complexity
  - name: radon
    command: "radon cc -s"
    languages: [python]

references:
  - https://www.sonarsource.com/docs/CognitiveComplexity.pdf
  - https://blog.sonarsource.com/cognitive-complexity-because-testability-understandability

related_metrics:
  - cyclomatic_complexity
  - maintainability_index
```

```yaml
# methodology_kb/glossary/cyclomatic_complexity.yaml
id: cyclomatic_complexity
name: Cyclomatic Complexity
source: mccabe
description: |
  Measures the number of linearly independent paths through code.
  Calculated as: E - N + 2P
  Where E = edges, N = nodes, P = connected components

  Simpler calculation: count decision points + 1
  Decision points: if, while, for, case, catch, &&, ||, ?:

thresholds:
  low: 1-10
  medium: 11-20
  high: 21-50
  critical: 51+

interpretation:
  1-10: Simple, low risk
  11-20: Moderate complexity, moderate risk
  21-50: Complex, high risk, consider refactoring
  51+: Untestable, very high risk, must refactor

tools:
  - name: radon
    command: "radon cc -a"
    languages: [python]
  - name: eslint
    rule: complexity
    languages: [javascript, typescript]
  - name: sonarqube
    metric_key: complexity

references:
  - https://en.wikipedia.org/wiki/Cyclomatic_complexity
```

```yaml
# methodology_kb/glossary/test_coverage.yaml
id: test_coverage
name: Test Coverage
source: industry
description: |
  Percentage of code executed during test runs.

  Types:
  - Line coverage: % of lines executed
  - Branch coverage: % of branches taken
  - Function coverage: % of functions called
  - Statement coverage: % of statements executed

thresholds:
  minimum: 60
  target: 80
  excellent: 90

critical_path_requirements:
  payment_processing: 95
  authentication: 90
  data_validation: 85
  api_endpoints: 80

project_type_adjustments:
  python_backend:
    minimum: 70
    target: 85
  typescript_frontend:
    minimum: 60
    target: 75
  legacy_system:
    minimum: 40
    target: 60

tools:
  - name: pytest-cov
    command: "pytest --cov"
    languages: [python]
  - name: istanbul/nyc
    command: "nyc npm test"
    languages: [javascript, typescript]
  - name: coverage.py
    command: "coverage run -m pytest"
    languages: [python]

references:
  - https://martinfowler.com/bliki/TestCoverage.html
```

### 5.2 Taxonomies Files

```yaml
# methodology_kb/taxonomies/sarif_rule_mappings.yaml
version: "1.0"
description: "Rule ID to category/severity mappings for SARIF normalization"

mappings:
  # ============================================
  # SEMGREP RULES
  # ============================================

  # Security - Injection
  "python.lang.security.audit.dangerous-exec-use":
    category: security
    subcategory: injection
    cwe: CWE-78
    owasp: A03:2021
    severity_base: critical
    description: "Use of exec() with user input"

  "python.lang.security.audit.dangerous-system-call":
    category: security
    subcategory: injection
    cwe: CWE-78
    owasp: A03:2021
    severity_base: critical

  "python.lang.security.audit.subprocess-shell-true":
    category: security
    subcategory: injection
    cwe: CWE-78
    severity_base: high

  "python.django.security.audit.xss.template-autoescape-off":
    category: security
    subcategory: xss
    cwe: CWE-79
    owasp: A03:2021
    severity_base: high

  "python.lang.security.audit.eval-detected":
    category: security
    subcategory: injection
    cwe: CWE-95
    severity_base: critical

  # Security - Authentication
  "python.django.security.audit.hardcoded-password":
    category: security
    subcategory: authentication
    cwe: CWE-798
    owasp: A07:2021
    severity_base: critical

  "python.lang.security.audit.insecure-hash-algorithms":
    category: security
    subcategory: cryptography
    cwe: CWE-327
    severity_base: medium

  # ============================================
  # BANDIT RULES
  # ============================================

  "B101":
    category: security
    subcategory: assertion
    cwe: CWE-703
    severity_base: low
    description: "Use of assert detected"

  "B102":
    category: security
    subcategory: injection
    cwe: CWE-78
    severity_base: high
    description: "Use of exec detected"

  "B103":
    category: security
    subcategory: permissions
    cwe: CWE-732
    severity_base: medium
    description: "Permissive file permissions"

  "B104":
    category: security
    subcategory: binding
    cwe: CWE-200
    severity_base: medium
    description: "Possible binding to all interfaces"

  "B105":
    category: security
    subcategory: authentication
    cwe: CWE-259
    severity_base: high
    description: "Hardcoded password string"

  "B106":
    category: security
    subcategory: authentication
    cwe: CWE-259
    severity_base: high
    description: "Hardcoded password as function argument"

  "B107":
    category: security
    subcategory: authentication
    cwe: CWE-259
    severity_base: high
    description: "Hardcoded password as default value"

  "B108":
    category: security
    subcategory: path_traversal
    cwe: CWE-377
    severity_base: medium
    description: "Insecure temp file creation"

  "B110":
    category: security
    subcategory: error_handling
    cwe: CWE-703
    severity_base: low
    description: "Try-except-pass detected"

  "B201":
    category: security
    subcategory: debug
    cwe: CWE-489
    severity_base: medium
    description: "Flask debug mode enabled"

  "B301":
    category: security
    subcategory: deserialization
    cwe: CWE-502
    severity_base: high
    description: "Pickle usage detected"

  "B303":
    category: security
    subcategory: cryptography
    cwe: CWE-327
    severity_base: high
    description: "Use of insecure MD2/MD4/MD5/SHA1"

  "B307":
    category: security
    subcategory: injection
    cwe: CWE-95
    severity_base: critical
    description: "Use of eval() detected"

  "B324":
    category: security
    subcategory: cryptography
    cwe: CWE-327
    severity_base: high
    description: "Use of insecure hash function"

  "B501":
    category: security
    subcategory: ssl
    cwe: CWE-295
    severity_base: high
    description: "SSL certificate verification disabled"

  "B506":
    category: security
    subcategory: yaml
    cwe: CWE-502
    severity_base: high
    description: "Use of unsafe yaml load"

  # ============================================
  # RUFF RULES
  # ============================================

  "F401":
    category: maintainability
    subcategory: imports
    severity_base: low
    description: "Unused import"

  "F841":
    category: maintainability
    subcategory: dead_code
    severity_base: low
    description: "Local variable assigned but never used"

  "E501":
    category: style
    subcategory: formatting
    severity_base: info
    description: "Line too long"

  "E711":
    category: bugs
    subcategory: comparison
    severity_base: low
    description: "Comparison to None"

  "E712":
    category: bugs
    subcategory: comparison
    severity_base: low
    description: "Comparison to True/False"

  "W291":
    category: style
    subcategory: whitespace
    severity_base: info
    description: "Trailing whitespace"

  "W292":
    category: style
    subcategory: whitespace
    severity_base: info
    description: "No newline at end of file"

  "C901":
    category: maintainability
    subcategory: complexity
    severity_base: medium
    description: "Function is too complex"

  "S101":
    category: security
    subcategory: assertion
    severity_base: low
    description: "Use of assert detected"

  "S105":
    category: security
    subcategory: authentication
    cwe: CWE-259
    severity_base: high
    description: "Hardcoded password string"

  "S106":
    category: security
    subcategory: authentication
    cwe: CWE-259
    severity_base: high
    description: "Hardcoded password as argument"

  "S107":
    category: security
    subcategory: authentication
    cwe: CWE-259
    severity_base: high
    description: "Hardcoded password as default"

  # ============================================
  # ESLINT RULES
  # ============================================

  "no-unused-vars":
    category: maintainability
    subcategory: dead_code
    severity_base: low

  "no-undef":
    category: bugs
    subcategory: reference
    severity_base: high

  "no-console":
    category: maintainability
    subcategory: debug
    severity_base: low

  "eqeqeq":
    category: bugs
    subcategory: comparison
    severity_base: medium

  "no-eval":
    category: security
    subcategory: injection
    cwe: CWE-95
    severity_base: critical

  "no-implied-eval":
    category: security
    subcategory: injection
    cwe: CWE-95
    severity_base: high

  "complexity":
    category: maintainability
    subcategory: complexity
    severity_base: medium

  # ============================================
  # TRIVY RULES
  # ============================================

  "CVE-*":
    category: security
    subcategory: vulnerability
    severity_base: varies  # Derived from CVE severity

  "GHSA-*":
    category: security
    subcategory: vulnerability
    severity_base: varies

# Severity adjustment contexts
severity_adjustments:
  # Adjust based on file path patterns
  path_patterns:
    "test/*":
      multiplier: 0.5
      reason: "Test code has lower impact"
    "tests/*":
      multiplier: 0.5
      reason: "Test code has lower impact"
    "*_test.py":
      multiplier: 0.5
    "*.test.ts":
      multiplier: 0.5
    "migrations/*":
      multiplier: 0.3
      reason: "Migration code is often auto-generated"
    "vendor/*":
      multiplier: 0.2
      reason: "Vendor code is third-party"
    "node_modules/*":
      multiplier: 0.1
      reason: "Node modules are dependencies"

  # Adjust based on business context
  business_context:
    payment:
      multiplier: 2.0
      reason: "Payment code is critical"
    auth:
      multiplier: 1.5
      reason: "Authentication code is sensitive"
    admin:
      multiplier: 1.3
      reason: "Admin code has elevated privileges"
    public_api:
      multiplier: 1.5
      reason: "Public API has external exposure"
```

### 5.3 Thresholds Files

```yaml
# methodology_kb/thresholds/python_backend.yaml
project_type: python_backend
description: "Thresholds for Python backend services (Django, FastAPI, Flask)"

coverage:
  line:
    minimum: 70
    target: 85
    excellent: 95
  branch:
    minimum: 60
    target: 75
    excellent: 90
  critical_paths:
    minimum: 90
    target: 95

complexity:
  cyclomatic:
    low: 10
    warning: 15
    error: 25
  cognitive:
    low: 10
    warning: 20
    error: 35
  max_function_length:
    warning: 50
    error: 100
  max_file_length:
    warning: 300
    error: 500

duplication:
  min_tokens: 100
  warning_percent: 3
  error_percent: 5

maintainability:
  maintainability_index:
    minimum: 20
    target: 40
  technical_debt_ratio:
    warning: 5
    error: 10

security:
  vulnerabilities:
    critical: 0
    high: 0
    medium: 5
    low: 20
  security_hotspots:
    to_review: 10

dependencies:
  outdated:
    warning: 5
    error: 10
  vulnerable:
    critical: 0
    high: 0

api:
  undocumented_endpoints:
    warning: 10
    error: 20
  missing_validation:
    warning: 5
    error: 10
```

```yaml
# methodology_kb/thresholds/typescript_frontend.yaml
project_type: typescript_frontend
description: "Thresholds for TypeScript/React frontend applications"

coverage:
  line:
    minimum: 60
    target: 75
    excellent: 90
  branch:
    minimum: 50
    target: 65
    excellent: 85
  component_coverage:
    minimum: 70
    target: 85

complexity:
  cyclomatic:
    low: 10
    warning: 15
    error: 20
  cognitive:
    low: 15
    warning: 25
    error: 40
  max_function_length:
    warning: 40
    error: 80
  max_file_length:
    warning: 250
    error: 400
  max_component_props:
    warning: 7
    error: 12

duplication:
  min_tokens: 75
  warning_percent: 4
  error_percent: 7

bundle:
  main_chunk_kb:
    warning: 250
    error: 500
  total_size_kb:
    warning: 1000
    error: 2000

accessibility:
  violations:
    critical: 0
    serious: 3
    moderate: 10

type_coverage:
  minimum: 80
  target: 95
```

```yaml
# methodology_kb/thresholds/legacy_system.yaml
project_type: legacy_system
description: "Relaxed thresholds for legacy systems under maintenance"

coverage:
  line:
    minimum: 40
    target: 60
    excellent: 80
  new_code:
    minimum: 70
    target: 85

complexity:
  cyclomatic:
    low: 20
    warning: 40
    error: 60
  cognitive:
    low: 25
    warning: 50
    error: 80
  max_function_length:
    warning: 100
    error: 200
  max_file_length:
    warning: 500
    error: 1000

duplication:
  min_tokens: 150
  warning_percent: 10
  error_percent: 15

security:
  vulnerabilities:
    critical: 0
    high: 3
    medium: 20
    low: unlimited

# Legacy-specific metrics
legacy:
  dead_code_percent:
    warning: 10
    error: 20
  deprecated_api_usage:
    warning: 20
    error: 50
  missing_types_percent:
    warning: 30
    error: 50
```

---

## Scenario 6: End-to-End Integration

### 6.1 Integration Test Workflow

```yaml
# test_scenarios/e2e_audit_test.yaml
name: "End-to-End Audit Test"
description: "Full audit workflow using all MCP servers"

setup:
  - name: "Create test project"
    action: create_test_fixture
    fixture: python_backend_sample

  - name: "Generate SCIP index"
    action: run_command
    command: "scip-python index --output index.scip ."

steps:
  - name: "Initialize mental model"
    server: mental-model
    tool: init_model
    input:
      name: "test-project"
      path: "./test_fixtures/python_backend_sample"
    expected:
      status: initialized

  - name: "Run security scan"
    server: sarif-tools
    tool: run_tool
    input:
      tool: semgrep
      path: "./test_fixtures/python_backend_sample"
    expected:
      sarif.version: "2.1.0"

  - name: "Store security artifacts"
    server: mental-model
    tool: store_artifact
    input:
      commit: "HEAD"
      type: "semgrep"
      data: "${previous.sarif}"
      metadata:
        producer: "test:sarif-tools"

  - name: "Load codegraph"
    server: codegraph
    tool: load_index
    input:
      scip_path: "./test_fixtures/python_backend_sample/index.scip"
    expected:
      status: loaded

  - name: "Find hotspots"
    server: codegraph
    tool: find_hotspot_symbols
    input:
      min_callers: 3
    validate:
      - hotspots is list

  - name: "Acquire findings with cascade"
    server: methodology-kb
    tool: acquire_findings
    input:
      commit: "HEAD"
      types: ["security", "complexity"]
      options:
        generate_if_missing: true
    expected:
      sources.length: "> 0"

  - name: "Classify and enrich findings"
    server: methodology-kb
    tool: classify_finding
    input:
      tool: semgrep
      rule_id: "python.lang.security.audit.dangerous-exec"
      base_severity: high
      context:
        file_path: "src/api/handlers.py"
        business_context: payment
    expected:
      adjusted_severity: critical

  - name: "Synthesize root causes"
    server: mental-model
    tool: synthesize
    input:
      algorithm: affinity
    expected:
      clusters: ">= 1"

validation:
  - name: "Verify findings count"
    query: mental-model.get_findings
    assert: "length > 0"

  - name: "Verify artifacts stored"
    query: mental-model.get_commit_artifacts
    input:
      commit: "HEAD"
    assert: "available.length > 0"
```

### 6.2 Integration Test Runner

```rust
// tests/integration/e2e_test.rs
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_full_audit_workflow() {
    // Start all MCP servers
    let servers = start_mcp_servers().await;

    // Run test workflow
    let result = timeout(
        Duration::from_secs(300),
        run_audit_workflow(&servers)
    ).await;

    assert!(result.is_ok(), "Workflow timed out");
    let workflow_result = result.unwrap();
    assert!(workflow_result.is_ok(), "Workflow failed: {:?}", workflow_result.err());

    // Cleanup
    stop_mcp_servers(servers).await;
}

async fn start_mcp_servers() -> ServerHandles {
    // Start each server and wait for ready
    let mental_model = start_server("mental-model-server").await;
    let methodology_kb = start_server("methodology-kb-server").await;
    let sarif_tools = start_server("sarif-tools-server").await;
    let codegraph = start_server("codegraph-server").await;

    ServerHandles {
        mental_model,
        methodology_kb,
        sarif_tools,
        codegraph,
    }
}

async fn run_audit_workflow(servers: &ServerHandles) -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Initialize mental model
    let init_result = servers.mental_model.call("init_model", json!({
        "name": "test-project",
        "path": "./test_fixtures/python_backend_sample"
    })).await?;
    assert_eq!(init_result["status"], "initialized");

    // Step 2: Run security scan
    let scan_result = servers.sarif_tools.call("run_tool", json!({
        "tool": "semgrep",
        "path": "./test_fixtures/python_backend_sample"
    })).await?;
    assert!(scan_result["sarif"]["runs"].is_array());

    // Step 3: Store artifacts
    servers.mental_model.call("store_artifact", json!({
        "commit": "HEAD",
        "type": "semgrep",
        "data": scan_result["sarif"],
        "metadata": {"producer": "test:sarif-tools"}
    })).await?;

    // Step 4: Load codegraph
    let load_result = servers.codegraph.call("load_index", json!({
        "scip_path": "./test_fixtures/python_backend_sample/index.scip"
    })).await?;
    assert_eq!(load_result["status"], "loaded");

    // Step 5: Find hotspots
    let hotspots = servers.codegraph.call("find_hotspot_symbols", json!({
        "min_callers": 3
    })).await?;

    // Step 6: Synthesize
    let synthesis = servers.mental_model.call("synthesize", json!({
        "algorithm": "affinity"
    })).await?;

    Ok(())
}
```

---

## Scenario 7: CI/CD Integration

### 7.1 GitHub Actions Workflow

```yaml
# .github/workflows/code-audit.yml
name: Code Audit Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  generate-artifacts:
    name: Generate Audit Artifacts
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Set up Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Install analysis tools
        run: |
          pip install semgrep bandit ruff scip-python

      - name: Generate SCIP index
        run: |
          scip-python index --output index.scip .

      - name: Run Semgrep
        run: |
          semgrep scan --config auto --sarif -o semgrep.sarif .
        continue-on-error: true

      - name: Run Bandit
        run: |
          bandit -r src/ -f sarif -o bandit.sarif || true

      - name: Run Ruff
        run: |
          ruff check --output-format sarif -o ruff.sarif . || true

      - name: Merge SARIF files
        run: |
          # Use sarif-multitool or custom script
          pip install sarif-tools
          sarif copy semgrep.sarif bandit.sarif ruff.sarif --output combined.sarif

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: audit-artifacts-${{ github.sha }}
          path: |
            index.scip
            semgrep.sarif
            bandit.sarif
            ruff.sarif
            combined.sarif
          retention-days: 30

      - name: Upload SARIF to GitHub Security
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: combined.sarif
        continue-on-error: true

  store-artifacts:
    name: Store in Audit Repository
    runs-on: ubuntu-latest
    needs: generate-artifacts
    if: github.event_name == 'push'

    steps:
      - uses: actions/checkout@v4

      - name: Download artifacts
        uses: actions/download-artifact@v4
        with:
          name: audit-artifacts-${{ github.sha }}
          path: ./artifacts

      - name: Store artifacts locally
        run: |
          mkdir -p .audit/artifacts/${{ github.sha }}
          cp ./artifacts/* .audit/artifacts/${{ github.sha }}/

          # Create metadata
          cat > .audit/artifacts/${{ github.sha }}/_meta.yaml << EOF
          commit: "${{ github.sha }}"
          branch: "${{ github.ref_name }}"
          timestamp: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
          artifacts:
            scip:
              produced_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
              producer: "ci:github-actions"
            semgrep:
              produced_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
              producer: "ci:github-actions"
            bandit:
              produced_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
              producer: "ci:github-actions"
            ruff:
              produced_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
              producer: "ci:github-actions"
          EOF

          # Update latest symlink
          ln -sfn ${{ github.sha }} .audit/artifacts/latest

      - name: Commit artifacts
        run: |
          git config user.name "GitHub Actions"
          git config user.email "actions@github.com"
          git add .audit/artifacts/
          git commit -m "ci: store audit artifacts for ${{ github.sha }}" || echo "No changes"
          git push
```

### 7.2 Manifest File Template

```yaml
# .audit/manifest.yaml
version: "1.0"
project: "{{PROJECT_NAME}}"

store:
  type: "local"
  path: ".audit/artifacts/"

# Alternative: S3 storage
# store:
#   type: "s3"
#   bucket: "audit-artifacts"
#   prefix: "{{PROJECT_NAME}}/"
#   region: "us-east-1"

artifact_types:
  security:
    schema: "sarif/2.1.0"
    producer: "ci:github-actions"
    tools: ["semgrep", "bandit"]

  style:
    schema: "sarif/2.1.0"
    producer: "ci:github-actions"
    tools: ["ruff"]

  scip:
    schema: "scip/1.0"
    producer: "ci:github-actions"

  coverage:
    schema: "coverage/cobertura"
    producer: "ci:github-actions"
    fallback: "audit:generate"

ci_integration:
  system: "github"
  artifact_name_pattern: "audit-artifacts-{commit}"
```

---

## Appendix: File Structure Summary

```
alpha-zero-review-/
├── .mcp.json                          # MCP server configuration
├── mental-model-server/               # [EXTEND]
│   └── src/
│       ├── artifacts.rs               # NEW: Artifact store
│       └── ...
├── methodology-kb-server/             # [EXTEND]
│   └── src/
│       ├── acquisition.rs             # NEW: Data acquisition
│       ├── cascade.rs                 # NEW: Cascade logic
│       └── ...
├── sarif-tools-server/                # [NEW]
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── server.rs
│       ├── sarif.rs
│       ├── runner.rs
│       ├── normalize.rs
│       └── tools/
│           ├── mod.rs
│           ├── semgrep.rs
│           ├── bandit.rs
│           ├── ruff.rs
│           └── trivy.rs
├── codegraph-server/                  # [NEW]
│   ├── Cargo.toml
│   ├── build.rs
│   ├── proto/
│   │   └── scip.proto
│   └── src/
│       ├── main.rs
│       ├── server.rs
│       ├── graph.rs
│       └── scip.rs
├── methodology_kb/                    # [FILL]
│   ├── glossary/
│   │   ├── cognitive_complexity.yaml
│   │   ├── cyclomatic_complexity.yaml
│   │   └── test_coverage.yaml
│   ├── taxonomies/
│   │   └── sarif_rule_mappings.yaml
│   └── thresholds/
│       ├── python_backend.yaml
│       ├── typescript_frontend.yaml
│       └── legacy_system.yaml
├── .github/workflows/                 # [NEW]
│   └── code-audit.yml
└── .audit/
    ├── manifest.yaml
    └── artifacts/
        └── {commit}/
            ├── _meta.yaml
            ├── index.scip
            └── *.sarif
```

---

*Beta-Zero Implementation Scenarios — Light IT Global*

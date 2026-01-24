//! Embedded templates for CLI tool configurations

use serde_json::json;
use std::path::Path;

/// Generate .mcp.json for Claude CLI
pub fn mcp_json(agent_dir: &Path, target_dir: &Path) -> String {
    let config = json!({
        "mcpServers": {
            "mental-model": {
                "command": agent_dir.join("target/release/mental-model-server").to_string_lossy(),
                "args": [
                    "--model-path", target_dir.join(".audit/mental_model.yaml").to_string_lossy()
                ]
            },
            "methodology-kb": {
                "command": agent_dir.join("target/release/methodology-kb-server").to_string_lossy(),
                "args": [
                    "--kb-path", agent_dir.join("methodology_kb/").to_string_lossy()
                ]
            },
            "sarif-tools": {
                "command": agent_dir.join("target/release/sarif-tools-server").to_string_lossy(),
                "args": []
            },
            "codegraph": {
                "command": agent_dir.join("target/release/codegraph-server").to_string_lossy(),
                "args": []
            }
        }
    });

    serde_json::to_string_pretty(&config).unwrap()
}

/// MCP server configuration for Codex CLI
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CodexMcpServer {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

/// Generate MCP server configs for Codex CLI (to be merged into ~/.codex/config.toml)
pub fn codex_mcp_servers(agent_dir: &Path, target_dir: &Path) -> std::collections::HashMap<String, CodexMcpServer> {
    let mut servers = std::collections::HashMap::new();

    servers.insert(
        "mental-model".to_string(),
        CodexMcpServer {
            command: agent_dir.join("target/release/mental-model-server").to_string_lossy().to_string(),
            args: vec![
                "--model-path".to_string(),
                target_dir.join(".audit/mental_model.yaml").to_string_lossy().to_string(),
            ],
        },
    );

    servers.insert(
        "methodology-kb".to_string(),
        CodexMcpServer {
            command: agent_dir.join("target/release/methodology-kb-server").to_string_lossy().to_string(),
            args: vec![
                "--kb-path".to_string(),
                agent_dir.join("methodology_kb/").to_string_lossy().to_string(),
            ],
        },
    );

    servers.insert(
        "sarif-tools".to_string(),
        CodexMcpServer {
            command: agent_dir.join("target/release/sarif-tools-server").to_string_lossy().to_string(),
            args: vec![],
        },
    );

    servers.insert(
        "codegraph".to_string(),
        CodexMcpServer {
            command: agent_dir.join("target/release/codegraph-server").to_string_lossy().to_string(),
            args: vec![],
        },
    );

    servers
}

/// Generate .cline/mcp_settings.json
pub fn cline_mcp_settings(agent_dir: &Path, target_dir: &Path) -> String {
    let config = json!({
        "mcpServers": {
            "mental-model": {
                "command": agent_dir.join("target/release/mental-model-server").to_string_lossy(),
                "args": [
                    "--model-path", target_dir.join(".audit/mental_model.yaml").to_string_lossy()
                ],
                "disabled": false
            },
            "methodology-kb": {
                "command": agent_dir.join("target/release/methodology-kb-server").to_string_lossy(),
                "args": [
                    "--kb-path", agent_dir.join("methodology_kb/").to_string_lossy()
                ],
                "disabled": false
            },
            "sarif-tools": {
                "command": agent_dir.join("target/release/sarif-tools-server").to_string_lossy(),
                "args": [],
                "disabled": false
            },
            "codegraph": {
                "command": agent_dir.join("target/release/codegraph-server").to_string_lossy(),
                "args": [],
                "disabled": false
            }
        }
    });

    serde_json::to_string_pretty(&config).unwrap()
}

/// Generate AGENTS.md for Codex CLI
pub fn agents_md() -> &'static str {
    r#"# AI Code Audit Agent Instructions

You are conducting a structured code audit. You MUST use the MCP tools below - they are essential for quality audits.

## CRITICAL: Always Use These MCP Tools

### 1. mental-model (REQUIRED - Start Here)
**Purpose:** Central audit state management. All findings go here.

**MUST call first:**
```
mental-model/init_model { "project_name": "project-name", "repo_path": "." }
```

**After analyzing code, ALWAYS add findings:**
```
mental-model/add_findings {
  "findings": [
    { "title": "SQL Injection", "severity": "high", "file_path": "src/db.py", "line": 42, "description": "..." }
  ]
}
```

**At the end, ALWAYS synthesize:**
```
mental-model/synthesize {}
```

### 2. methodology-kb (REQUIRED - For Severity Classification)
**Purpose:** Provides metric thresholds and adjusts finding severity based on context.

**Before adding findings, classify severity:**
```
methodology-kb/classify_finding {
  "tool": "semgrep",
  "rule_id": "python.lang.security.audit.dangerous-exec",
  "context": { "bounded_context_type": "core", "layer": "domain", "is_hotspot": true }
}
```
This returns adjusted severity (a finding in "core/domain" is more severe than in "adapter/infra").

**Look up metric thresholds:**
```
methodology-kb/get_thresholds { "project_type": "web_service", "language": "python" }
```

### 3. codegraph (REQUIRED - For Impact Analysis)
**Purpose:** Semantic code intelligence. Find callers, dependencies, hotspots.

**First, load indexes (if .audit/indexes/ exists):**
```
codegraph/load_project_indexes { "project_path": "." }
```

**Find high-impact code (hotspots):**
```
codegraph/find_hotspot_symbols { "min_callers": 5 }
```

**Analyze impact before recommending changes:**
```
codegraph/get_impact { "symbol_id": "src/auth.py#authenticate" }
```

**Find all callers of a function:**
```
codegraph/get_callers { "symbol_id": "src/db.py#execute_query" }
```

### 4. sarif-tools (For Running Scanners)
**Purpose:** Run code analysis tools and get SARIF output.

```
sarif-tools/run_tool { "tool": "semgrep", "path": "src/" }
sarif-tools/run_tool { "tool": "bandit", "path": "src/" }
```

## Audit Workflow

### Phase 1: Foundation
1. Call `mental-model/init_model`
2. Analyze: tech stack, project structure, build system
3. Call `mental-model/update_viewpoint` with VP-F01, VP-F02, VP-F03 data

### Phase 2: Structure
1. If `.audit/indexes/` exists: `codegraph/load_project_indexes`
2. Use `codegraph/find_hotspot_symbols` to identify critical code
3. Analyze: modules, layers, domain model, interfaces
4. Call `mental-model/update_viewpoint` with VP-S01 through VP-S07 data

### Phase 3: Quality
1. Run scanners: `sarif-tools/run_tool`
2. For each finding:
   - Get context: `mental-model/get_context`
   - Classify severity: `methodology-kb/classify_finding`
   - Check impact: `codegraph/get_impact` (for critical findings)
3. Batch add: `mental-model/add_findings`

### Phase 4: Synthesis
1. Call `mental-model/synthesize` - clusters findings into root causes
2. NEVER dump 100+ raw findings. Present 3-5 root causes.
3. Classify debt using Fowler Quadrant (Prudent/Reckless × Deliberate/Inadvertent)

## Key Rules
- **ALWAYS initialize mental-model first**
- **ALWAYS use methodology-kb to classify finding severity** - raw tool output is not context-aware
- **ALWAYS use codegraph for impact analysis** - understand what code is critical before making recommendations
- **NEVER present raw findings** - always synthesize into root causes
"#
}

/// Generate .clinerules for Cline
pub fn clinerules() -> &'static str {
    r#"# AI Code Audit Agent - Cline Rules

## Project Context
This project is configured for code auditing using the AI Code Audit Agent methodology with MCP servers.

## MCP Servers Available
- **mental-model**: Central audit artifact (22 tools) - init_model, get_model, add_finding, synthesize, etc.
- **methodology-kb**: Metrics & thresholds (11 tools) - lookup_metric, classify_finding, check_compliance
- **sarif-tools**: Code analysis (3 tools) - list_available_tools, merge_sarif
- **codegraph**: Code intelligence (11 tools) - load_index, load_project_indexes, find_symbol, get_impact

## Audit Instructions

When asked to audit this codebase:
1. Initialize mental model with `mental-model/init_model`
2. Execute viewpoints in order: Foundation (F01-F03) → Structure (S01-S07) → Quality (Q01-Q05) → Synthesis (Q06)
3. Use batch operations (add_findings, get_contexts) for efficiency
4. Synthesize findings into 3-5 root causes, don't dump raw findings
5. Classify debt using Fowler Quadrant (Prudent/Reckless × Deliberate/Inadvertent)

## Key Tools
- `add_findings` - Batch add multiple findings (preferred over single add_finding)
- `get_findings_summary` - Get counts by severity/category/viewpoint
- `synthesize` - Cluster findings into root causes
- `export_findings` - Export to JSON for external tools
- `load_project_indexes` - Auto-load SCIP indexes from .audit/indexes/

## SCIP Indexes
If `.audit/indexes/` contains SCIP index files, use `codegraph/load_project_indexes` to enable code intelligence features like symbol lookup, callers, and impact analysis.
"#
}

/// Generate minimal CLAUDE.md when source not available
pub fn claude_md() -> &'static str {
    r#"# AI Code Audit Agent - Claude CLI Instructions

This project is configured for code auditing using the AI Code Audit Agent methodology.

## Quick Start

```
Run a full code audit using the viewpoints framework
```

## Available MCP Servers

- **mental-model**: Central audit artifact management (22 tools)
- **methodology-kb**: Metrics, thresholds, classification (11 tools)
- **sarif-tools**: Code analysis tools (3 tools)
- **codegraph**: SCIP-based code intelligence (11 tools)

## Audit Workflow

1. **Foundation** (VP-F01-F03): Technology stack, structure, build
2. **Structure** (VP-S01-S07): Modules, layers, domain model
3. **Quality** (VP-Q01-Q05): Security, performance, testability
4. **Synthesis** (VP-Q06): Root causes with Fowler Quadrant

## Key Rules

1. **Initialize First**: Call `mental-model/init_model` before starting
2. **Context-Aware Findings**: Always enrich findings with business context
3. **No Raw Dumps**: Synthesize into 3-5 root causes, not 800+ findings
4. **Fowler Quadrant**: Classify debt as Prudent/Reckless × Deliberate/Inadvertent
"#
}

/// Generate .audit-viewpoints.md reference
pub fn viewpoints_reference(agent_dir: &Path) -> String {
    format!(
        r#"# Audit Viewpoints Reference

Agent installation: {}
Skills directory: {}/skills/

## Viewpoint Execution

| Phase | Viewpoints | Description |
|-------|------------|-------------|
| Foundation | VP-F01, VP-F02, VP-F03 | Tech stack, structure, build |
| Structure | VP-S01 - VP-S07 | Modules, layers, domain, entities |
| Quality | VP-Q01 - VP-Q05 | Security, performance, testability |
| Synthesis | VP-Q06 | Root causes, Fowler Quadrant |

## Skill Files Location

```
{}/skills/
├── vp-f01-tech-stack/SKILL.md
├── vp-f02-structure/SKILL.md
├── vp-f03-build-deploy/SKILL.md
├── vp-s01-module-hierarchy/SKILL.md
├── vp-s02-layer-architecture/SKILL.md
├── vp-s03-domain-model/SKILL.md
├── vp-s04-entity-model/SKILL.md
├── vp-s05-interface-surface/SKILL.md
├── vp-s06-dependency-graph/SKILL.md
├── vp-s07-architecture-decisions/SKILL.md
├── vp-q01-security/SKILL.md
├── vp-q02-performance/SKILL.md
├── vp-q03-testability/SKILL.md
├── vp-q04-code-style/SKILL.md
├── vp-q05-documentation/SKILL.md
└── vp-q06-synthesis/SKILL.md
```

## MCP Server Summary

| Server | Tools | Purpose |
|--------|-------|---------|
| mental-model | 22 | Central audit artifact, findings, synthesis |
| methodology-kb | 11 | Metrics, thresholds, classification |
| sarif-tools | 3 | Code analysis tools |
| codegraph | 11 | SCIP code intelligence, auto-indexing |
| **Total** | **47** | |
"#,
        agent_dir.display(),
        agent_dir.display(),
        agent_dir.display()
    )
}

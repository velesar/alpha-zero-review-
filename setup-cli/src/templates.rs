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
                    "--model-path", target_dir.join(".audit/mental_model.yaml").to_string_lossy(),
                    "--audit-path", target_dir.join(".audit").to_string_lossy()
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

/// Generate codex.json for Codex CLI
pub fn codex_json(agent_dir: &Path, target_dir: &Path) -> String {
    let config = json!({
        "name": "AI Code Audit Agent",
        "version": "2.0",
        "mcp_servers": {
            "mental-model": {
                "command": agent_dir.join("target/release/mental-model-server").to_string_lossy(),
                "args": [
                    "--model-path", target_dir.join(".audit/mental_model.yaml").to_string_lossy(),
                    "--audit-path", target_dir.join(".audit").to_string_lossy()
                ],
                "transport": "stdio"
            },
            "methodology-kb": {
                "command": agent_dir.join("target/release/methodology-kb-server").to_string_lossy(),
                "args": [
                    "--kb-path", agent_dir.join("methodology_kb/").to_string_lossy()
                ],
                "transport": "stdio"
            },
            "sarif-tools": {
                "command": agent_dir.join("target/release/sarif-tools-server").to_string_lossy(),
                "args": [],
                "transport": "stdio"
            },
            "codegraph": {
                "command": agent_dir.join("target/release/codegraph-server").to_string_lossy(),
                "args": [],
                "transport": "stdio"
            }
        }
    });

    serde_json::to_string_pretty(&config).unwrap()
}

/// Generate .cline/mcp_settings.json
pub fn cline_mcp_settings(agent_dir: &Path, target_dir: &Path) -> String {
    let config = json!({
        "mcpServers": {
            "mental-model": {
                "command": agent_dir.join("target/release/mental-model-server").to_string_lossy(),
                "args": [
                    "--model-path", target_dir.join(".audit/mental_model.yaml").to_string_lossy(),
                    "--audit-path", target_dir.join(".audit").to_string_lossy()
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

This project is configured for code auditing using the AI Code Audit Agent methodology.

## Quick Start

Run a full code audit:
```
Run a full code audit using the viewpoints framework
```

## Available MCP Tools

### mental-model (22 tools)
Core audit artifact management: init_model, get_model, update_viewpoint, add_finding, add_findings (batch), get_findings, get_findings_by_file, get_findings_by_severity, synthesize, export_findings, flush

### methodology-kb (11 tools)
Metrics and classification: lookup_metric, classify_finding, get_thresholds, check_compliance, get_template, list_metrics

### sarif-tools (3 tools)
Code analysis: list_available_tools, merge_sarif, get_tool_config

### codegraph (11 tools)
Code intelligence: load_index, load_project_indexes, find_symbol, get_callers, get_impact, find_hotspot_symbols

## Audit Workflow

1. **Foundation**: VP-F01 (Tech Stack), VP-F02 (Structure), VP-F03 (Build)
2. **Structure**: VP-S01-S07 (Modules, Layers, Domain, Entities, Interfaces)
3. **Quality**: VP-Q01-Q05 (Security, Performance, Testability, Style, Docs)
4. **Synthesis**: VP-Q06 (Root causes with Fowler Quadrant)

## Key Rules
- **NEVER dump raw findings.** Always synthesize into 3-5 root causes.
- **Use `load_project_indexes`** if `.audit/indexes/` exists for code intelligence.
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

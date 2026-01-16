# AI Code Audit Agent

**Alpha-Zero Test Implementation - Framework v2.0**

A Mental Model-first approach to code quality audit using Cline as the execution engine and custom MCP servers built in Rust. Now with **Fowler Quadrant** technical debt classification.

## Overview

This system provides a unique approach to code audits that goes beyond traditional static analysis tools. Instead of producing 800+ raw findings, it:

1. **Builds Understanding First**: Creates a mental model of the codebase through structured viewpoints
2. **Adds Business Context**: Enriches findings with bounded context type, architecture layer, and hotspot status
3. **Adjusts Severity**: Uses context to adjust finding severity (issues in core domain are more critical)
4. **Classifies Technical Debt**: Uses Fowler Quadrant (Prudent/Reckless × Deliberate/Inadvertent)
5. **Synthesizes Root Causes**: Clusters findings into 3-5 actionable root causes

See [VIEWPOINTS_FRAMEWORK.md](docs/VIEWPOINTS_FRAMEWORK.md) for the complete v2.0 framework specification.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           YOUR IP LAYER                                 │
│                                                                         │
│  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────┐   │
│  │    Viewpoints     │  │   Mental Model    │  │   Methodology     │   │
│  │    Framework      │  │   MCP Server      │  │   KB Server       │   │
│  │  (SKILL.md)       │  │   (Rust/rmcp)     │  │   (Rust/rmcp)     │   │
│  └───────────────────┘  └───────────────────┘  └───────────────────┘   │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                           CLINE ENGINE                                  │
│  • Reads SKILL.md files as instructions                                 │
│  • Executes viewpoints in sequence                                      │
│  • Calls MCP servers for context                                        │
│  • Generates reports                                                    │
└─────────────────────────────────────────────────────────────────────────┘
```

## Components

### MCP Servers (Rust)

- **mental-model-server**: Manages the central mental model artifact
  - Tools: `init_model`, `get_model`, `update_viewpoint`, `get_context`, `get_constraints`, `add_finding`, `get_findings`, `synthesize`, `get_completed_viewpoints`

- **methodology-kb-server**: Knowledge base for metrics, thresholds, and standards
  - Tools: `lookup_metric`, `classify_finding`, `get_thresholds`, `check_compliance`, `get_template`, `list_metrics`, `list_standards`, `get_category`

### Viewpoints Framework (16 required + 3 optional SKILL.md files)

**Foundation Phase:**
- VP-F01: Technology Stack Analysis
- VP-F02: Project Structure Analysis
- VP-F03: Build & Deployment Analysis

**Structure Phase:**
- VP-S01: Module Hierarchy Analysis
- VP-S02: Layer Architecture Analysis
- VP-S03: Domain Model Analysis
- VP-S04: Entity Model Analysis
- VP-S05: Interface Surface Analysis
- VP-S06: Dependency Graph Analysis
- VP-S07: Architecture Decisions Analysis

**Optional Structure Viewpoints:**
- VP-S08: Team Topologies Mapping *(enterprise projects)*
- VP-S09: Building Block Compliance *(TOGAF environments)*
- VP-S10: Standards Compliance *(compliance audits)*

**Quality Phase:**
- VP-Q01: Security Analysis
- VP-Q02: Performance Analysis
- VP-Q03: Testability Analysis
- VP-Q04: Code Style Analysis
- VP-Q05: Documentation Analysis

**Synthesis Phase:**
- VP-Q06: Technical Debt Synthesis (with Fowler Quadrant classification)

### Methodology Knowledge Base

- **glossary/**: Metric definitions (cognitive_complexity, coverage, etc.)
- **taxonomies/**: Finding categories, severity adjustments, rule mappings
- **standards/**: Architecture standards (clean, layered, hexagonal)
- **thresholds/**: Project-type specific thresholds
- **templates/**: Report templates

## Building

```bash
# Build MCP servers
cargo build --release

# Servers will be at:
# ./target/release/mental-model-server
# ./target/release/methodology-kb-server
```

## Configuration

Add to your Cline MCP settings (see `mcp_settings.json`):

```json
{
  "mcpServers": {
    "mental-model": {
      "command": "./target/release/mental-model-server",
      "args": ["--model-path", "./mental_model.yaml"]
    },
    "methodology-kb": {
      "command": "./target/release/methodology-kb-server",
      "args": ["--kb-path", "./methodology_kb/"]
    }
  }
}
```

## Usage

1. Start an audit by asking Cline to "audit this codebase"
2. Cline will load `.clinerules` and execute viewpoints in sequence
3. Mental model accumulates understanding through each viewpoint
4. Findings are enriched with context and adjusted severity
5. Final synthesis produces 3-5 root causes and actionable recommendations

## Audit Outputs

- `executive_summary.md`: Stakeholder-friendly overview
- `root_cause_analysis.md`: Technical root causes with Fowler Quadrant classification
- `detailed_findings.md`: All findings with context
- `mental_model.yaml`: Complete audit data
- `technical_debt_inventory.yaml`: Debt items classified by quadrant

## Key Differentiators

| Traditional Tools | AI Code Audit Agent |
|-------------------|---------------------|
| 800+ raw findings | 3-5 root causes |
| Rule-based severity | Context-aware severity |
| File-level analysis | Business domain context |
| Lists problems | Explains why and how to fix |
| One-size-fits-all | Project-type aware thresholds |
| Generic debt tracking | Fowler Quadrant classification |

## Fowler Quadrant Classification

Technical debt is classified along two dimensions:

```
                DELIBERATE              INADVERTENT
         ┌────────────────────┬────────────────────┐
 PRUDENT │ "Must ship now"    │ "Now we know       │
         │ → Schedule payback │  better"           │
         │                    │ → Refactor         │
         ├────────────────────┼────────────────────┤
RECKLESS │ "No time for       │ "What's layering?" │
         │  design"           │ → Training needed  │
         │ → Urgent fix       │                    │
         └────────────────────┴────────────────────┘
```

## Success Criteria

- Full audit execution: All 16 required viewpoints complete
- Root cause synthesis: 3-5 causes (not 800+ findings)
- Quadrant distribution analyzed for team health indicators
- Total audit time: < 8 hours
- Client satisfaction: ≥ 4/5

## Project Structure

```
.
├── Cargo.toml                    # Workspace manifest
├── docs/
│   └── VIEWPOINTS_FRAMEWORK.md   # v2.0 Framework specification
├── mental-model-server/          # Mental Model MCP Server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── server.rs
│       └── model.rs
├── methodology-kb-server/        # Methodology KB MCP Server
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── server.rs
│   │   └── types.rs
│   └── templates/
├── methodology_kb/               # Knowledge Base Content
│   ├── glossary/
│   ├── taxonomies/
│   ├── standards/
│   ├── thresholds/
│   └── templates/
├── skills/                       # Viewpoints Framework (v2.0)
│   ├── .clinerules              # Audit execution rules
│   ├── vp-f01-tech-stack/       # Foundation Phase
│   ├── vp-f02-structure/
│   ├── vp-f03-build-deploy/
│   ├── vp-s01-module-hierarchy/ # Structure Phase
│   ├── vp-s02-layer-architecture/
│   ├── vp-s03-domain-model/
│   ├── vp-s04-entity-model/
│   ├── vp-s05-interface-surface/
│   ├── vp-s06-dependency-graph/
│   ├── vp-s07-architecture-decisions/
│   ├── vp-s08-team-topologies/  # Optional viewpoints
│   ├── vp-s09-building-blocks/
│   ├── vp-s10-standards-compliance/
│   ├── vp-q01-security/         # Quality Phase
│   ├── vp-q02-performance/
│   ├── vp-q03-testability/
│   ├── vp-q04-code-style/
│   ├── vp-q05-documentation/
│   └── vp-q06-synthesis/        # Synthesis Phase (Fowler Quadrant)
└── mcp_settings.json            # MCP configuration
```

## License

Proprietary - Light IT Global

---

*Alpha-Zero Test - Light IT Global*

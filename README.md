# AI Code Audit Agent

**Beta-Zero Test Implementation - Framework v2.0**

A Mental Model-first approach to code quality audit using Claude CLI as the execution engine and four custom MCP servers built in Rust. Features **Fowler Quadrant** technical debt classification and SARIF-based analysis.

## Overview

This system provides a unique approach to code audits that goes beyond traditional static analysis tools. Instead of producing 800+ raw findings, it:

1. **Builds Understanding First**: Creates a mental model of the codebase through structured viewpoints
2. **Adds Business Context**: Enriches findings with bounded context type, architecture layer, and hotspot status
3. **Adjusts Severity**: Uses context to adjust finding severity (issues in core domain are more critical)
4. **Classifies Technical Debt**: Uses Fowler Quadrant (Prudent/Reckless × Deliberate/Inadvertent)
5. **Synthesizes Root Causes**: Clusters findings into 3-5 actionable root causes

## Documentation

| Document | Description |
|----------|-------------|
| [VIEWPOINTS_FRAMEWORK.md](docs/VIEWPOINTS_FRAMEWORK.md) | Complete v2.0 framework specification |
| [OPERATIONS_GUIDE.md](docs/OPERATIONS_GUIDE.md) | Installation, configuration, and usage |
| [ARCHITECTURE_OVERVIEW.md](docs/ARCHITECTURE_OVERVIEW.md) | System architecture and design |
| [Architecture Decision Records](docs/adr/) | ADRs documenting key decisions |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              MCP SERVERS                                     │
│                                                                              │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐  ┌────────────┐ │
│  │ Mental Model   │  │ Methodology KB │  │ SARIF Tools    │  │ Codegraph  │ │
│  │ Server         │  │ Server         │  │ Server         │  │ Server     │ │
│  │                │  │                │  │                │  │            │ │
│  │ • Model state  │  │ • Metrics      │  │ • Tool runners │  │ • SCIP     │ │
│  │ • Artifacts    │  │ • Thresholds   │  │ • SARIF parse  │  │ • Symbols  │ │
│  │ • Context      │  │ • Standards    │  │ • Normalize    │  │ • Impact   │ │
│  └────────────────┘  └────────────────┘  └────────────────┘  └────────────┘ │
│                                                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                           CLAUDE CLI ENGINE                                  │
│  • Reads SKILL.md files as instructions                                      │
│  • Executes viewpoints in sequence                                           │
│  • Calls MCP servers for context                                             │
│  • Generates reports                                                         │
└──────────────────────────────────────────────────────────────────────────────┘
```

## MCP Servers

### mental-model-server
Manages the central mental model artifact and commit-indexed artifact storage.

| Tool | Description |
|------|-------------|
| `init_model` | Initialize new mental model for project |
| `get_model` | Get current model state as YAML |
| `update_viewpoint` | Update with viewpoint results |
| `get_context` | Get business context for a file |
| `get_constraints` | Get derived analysis constraints |
| `add_finding` | Add finding with context enrichment |
| `get_findings` | Get all findings |
| `synthesize` | Cluster findings into root causes |
| `store_artifact` | Store SARIF/SCIP artifact for commit |
| `get_artifact` | Retrieve stored artifact |
| `get_commit_artifacts` | List available artifacts for commit |

### methodology-kb-server
Knowledge base for metrics, thresholds, and standards.

| Tool | Description |
|------|-------------|
| `lookup_metric` | Get metric definition and thresholds |
| `classify_finding` | Classify finding with severity adjustment |
| `get_thresholds` | Get thresholds for project type |
| `check_compliance` | Check architecture compliance |
| `get_template` | Get report template |
| `list_metrics` | List available metrics |
| `list_standards` | List architecture standards |
| `get_category` | Get category details |

### sarif-tools-server
Runs code analysis tools with SARIF output.

| Tool | Description |
|------|-------------|
| `run_tool` | Run analysis tool (semgrep, bandit, ruff, trivy, clippy) |
| `list_available_tools` | List tools with installation status |
| `get_tool_config` | Get tool configuration |
| `merge_sarif` | Merge multiple SARIF results |
| `normalize_sarif` | Enrich SARIF with categories and severity |

### codegraph-server
SCIP-based semantic code intelligence.

| Tool | Description |
|------|-------------|
| `load_index` | Load SCIP index or JSON codegraph |
| `find_symbol` | Search symbols by name pattern |
| `get_symbol_info` | Get detailed symbol information |
| `get_callers` | Get all references to a symbol |
| `get_callees` | Get symbols called from within a symbol |
| `get_impact` | Analyze change impact |
| `get_file_symbols` | Get all symbols in a file |
| `get_module_deps` | Get module dependencies |
| `find_hotspot_symbols` | Find heavily-referenced symbols |

## Viewpoints Framework

### Foundation Phase
- **VP-F01**: Technology Stack Analysis
- **VP-F02**: Project Structure Analysis
- **VP-F03**: Build & Deployment Analysis

### Structure Phase
- **VP-S01**: Module Hierarchy Analysis
- **VP-S02**: Layer Architecture Analysis
- **VP-S03**: Domain Model Analysis
- **VP-S04**: Entity Model Analysis
- **VP-S05**: Interface Surface Analysis
- **VP-S06**: Dependency Graph Analysis
- **VP-S07**: Architecture Decisions Analysis

### Optional Structure Viewpoints
- **VP-S08**: Team Topologies Mapping *(enterprise)*
- **VP-S09**: Building Block Compliance *(TOGAF)*
- **VP-S10**: Standards Compliance *(compliance audits)*

### Quality Phase
- **VP-Q01**: Security Analysis
- **VP-Q02**: Performance Analysis
- **VP-Q03**: Testability Analysis
- **VP-Q04**: Code Style Analysis
- **VP-Q05**: Documentation Analysis

### Synthesis Phase
- **VP-Q06**: Technical Debt Synthesis (Fowler Quadrant)

## Building

```bash
# Build all MCP servers
cargo build --release

# Run tests
cargo test --all

# Check code quality
cargo clippy --all-targets
```

## Configuration

Add to your MCP settings (`.mcp.json`):

```json
{
  "mcpServers": {
    "mental-model": {
      "command": "./target/release/mental-model-server",
      "args": ["--model-path", "./.audit/mental_model.yaml"]
    },
    "methodology-kb": {
      "command": "./target/release/methodology-kb-server",
      "args": ["--kb-path", "./methodology_kb/"]
    },
    "sarif-tools": {
      "command": "./target/release/sarif-tools-server"
    },
    "codegraph": {
      "command": "./target/release/codegraph-server"
    }
  }
}
```

## Usage

1. Start an audit: `claude "Audit this codebase using the viewpoints framework"`
2. Claude executes viewpoints in sequence
3. Mental model accumulates understanding
4. Findings are enriched with context
5. Synthesis produces root causes and recommendations

## Audit Outputs

| File | Description |
|------|-------------|
| `executive_summary.md` | Stakeholder-friendly overview |
| `root_cause_analysis.md` | Technical root causes with Fowler Quadrant |
| `detailed_findings.md` | All findings with context |
| `mental_model.yaml` | Complete audit data |
| `technical_debt_inventory.yaml` | Debt items by quadrant |

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

## Project Structure

```
.
├── Cargo.toml                    # Workspace manifest
├── CLAUDE.md                     # Claude CLI instructions
├── .mcp.json                     # MCP server configuration
├── docs/
│   ├── VIEWPOINTS_FRAMEWORK.md   # v2.0 Framework specification
│   ├── OPERATIONS_GUIDE.md       # Installation and usage guide
│   ├── ARCHITECTURE_OVERVIEW.md  # System architecture
│   └── adr/                      # Architecture Decision Records
├── mental-model-server/          # Mental Model MCP Server
├── methodology-kb-server/        # Methodology KB MCP Server
├── sarif-tools-server/           # SARIF Tools MCP Server
├── codegraph-server/             # Codegraph MCP Server
├── methodology_kb/               # Knowledge Base Content
├── skills/                       # Viewpoints Framework (v2.0)
├── .audit/                       # Audit artifacts storage
└── .github/workflows/            # CI/CD workflows
```

## CI/CD

- **audit-artifacts.yml**: Generates SARIF artifacts (semgrep, bandit, ruff, trivy, clippy)
- **coverage.yml**: Code coverage with cargo-tarpaulin and Codecov

## License

Proprietary - Light IT Global

---

*Beta-Zero Test Implementation - Light IT Global*

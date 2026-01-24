# AI Code Audit Agent - Claude CLI Instructions

This project provides a Mental Model-first approach to code quality audits.
When conducting audits, Claude uses custom MCP servers to build understanding
and produce actionable insights instead of raw finding dumps.

## MCP Servers

This project uses four MCP servers (configured in `.mcp.json`):

- **mental-model**: Manages the central mental model artifact and commit-indexed artifact storage
- **methodology-kb**: Knowledge base for metrics, thresholds, and standards
- **sarif-tools**: Runs code analysis tools (semgrep, bandit, ruff, trivy, clippy) with SARIF output
- **codegraph**: SCIP-based semantic code intelligence for symbol analysis and impact assessment

## Available Commands

### Start a Full Audit

To audit a target codebase, run Claude CLI in the target project directory with:

```
claude "Run a full code audit using the AI Code Audit Agent methodology"
```

Or start Claude CLI and ask:
```
Audit this codebase using the viewpoints framework
```

### Quick Commands

- `Init audit` - Initialize mental model for current project
- `Show model` - Display current mental model state
- `Show findings` - List all findings with context
- `Synthesize` - Generate root causes from findings

## Audit Workflow

When asked to perform a code audit, execute viewpoints in this sequence:

### PHASE 1: Foundation (understand the project)
1. **VP-F01**: Technology Stack Analysis
2. **VP-F02**: Project Structure Analysis
3. **VP-F03**: Build & Deployment Analysis

### PHASE 2: Structure (map the architecture)
4. **VP-S01**: Module Hierarchy Analysis
5. **VP-S02**: Layer Architecture Analysis
6. **VP-S03**: Domain Model Analysis
7. **VP-S04**: Entity Model Analysis
8. **VP-S05**: Interface Surface Analysis
9. **VP-S06**: Dependency Graph Analysis
10. **VP-S07**: Architecture Decisions Analysis

### PHASE 2b: Optional Structure (if applicable)
- **VP-S08**: Team Topologies Mapping *(enterprise projects)*
- **VP-S09**: Building Block Compliance *(TOGAF environments)*
- **VP-S10**: Standards Compliance *(compliance audits)*

### PHASE 3: Quality (find issues with context)
11. **VP-Q01**: Security Analysis
12. **VP-Q02**: Performance Analysis
13. **VP-Q03**: Testability Analysis
14. **VP-Q04**: Code Style Analysis
15. **VP-Q05**: Documentation Analysis

### PHASE 4: Synthesis (generate insights)
16. **VP-Q06**: Technical Debt Synthesis & Reporting (with Fowler Quadrant)

## Key Rules

### Rule 1: Initialize Mental Model First
Before starting any audit:
1. Call `mental-model/init_model` with project name and path
2. Verify initialization succeeded before proceeding

### Rule 2: Context-Aware Findings
When adding findings, prefer batch operations for efficiency (ADR-0005):

**For multiple findings (preferred):**
1. Collect all file paths from findings
2. Call `mental-model/get_contexts` with all paths (batch)
3. Call `mental-model/add_findings` with all enriched findings (batch)

**For single finding:**
1. Call `mental-model/get_context` with file path
2. Call `methodology-kb/classify_finding` with context for adjusted severity
3. Call `mental-model/add_finding` with enriched data

**NEVER report raw tool output. Always enrich with context.**

### Rule 3: No Raw Tool Dumps
When using analysis tools (linters, security scanners):
1. Filter results to focus on constrained paths (`mental-model/get_constraints`)
2. Enrich each finding with business context
3. Group findings by root cause pattern, not by rule ID

**NEVER dump 800+ raw findings. Synthesize into 3-5 root causes.**

### Rule 4: Update Mental Model After Each Viewpoint
After completing each viewpoint:
1. Call `mental-model/update_viewpoint` with viewpoint results
2. Note that constraints are auto-recalculated

### Rule 5: Fowler Quadrant Classification
Classify technical debt along two dimensions:

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

## Viewpoint Instructions

Each viewpoint has detailed instructions in `skills/vp-*/SKILL.md`.
Read the SKILL.md file for the current viewpoint before executing it.

Example for VP-F01:
```
Read skills/vp-f01-tech-stack/SKILL.md and follow its instructions
```

## MCP Tool Reference

### mental-model server
- `init_model(name, path, description?)` - Initialize new mental model
- `get_model()` - Get current model state as YAML
- `get_model_section(section)` - Get specific section only (ADR-0005) ⚡
- `update_viewpoint(viewpoint, data)` - Update with viewpoint results
- `get_context(file_path)` - Get business context for a file
- `get_contexts(file_paths[])` - Batch get context for multiple files (ADR-0005) ⚡
- `get_constraints()` - Get derived analysis constraints
- `add_finding(viewpoint, category, title, description, file_path, base_severity, ...)` - Add finding
- `add_findings(findings[])` - Batch add multiple findings (ADR-0005) ⚡
- `get_findings()` - Get all findings
- `get_findings_by_file(file_path)` - Get findings for a specific file (ADR-0007) 🔍
- `get_findings_by_severity(severity)` - Filter findings by severity level (ADR-0007) 🔍
- `get_findings_by_viewpoint(viewpoint)` - Get findings from a viewpoint (ADR-0007) 🔍
- `get_findings_by_category(category)` - Filter findings by category (ADR-0007) 🔍
- `get_findings_summary()` - Get finding counts by severity/category/viewpoint (ADR-0007) 🔍
- `export_findings(output_path)` - Export findings to JSON file (ADR-0007) 📤
- `synthesize(algorithm?)` - Cluster findings into root causes
- `get_completed_viewpoints()` - List completed viewpoints
- `get_commit_artifacts(commit?)` - List available/missing artifacts for commit (HEAD/latest supported)
- `store_artifact(commit, type, data, producer)` - Store SARIF/SCIP artifact for commit
- `get_artifact(commit?, type)` - Retrieve stored artifact
- `flush()` - Persist pending changes to disk (ADR-0006) 💾

*⚡ Batch operations (ADR-0005) - prefer these for efficiency*
*💾 Deferred persistence (ADR-0006) - changes auto-flush at phase boundaries*
*🔍 Query tools (ADR-0007) - findings stored in SQLite for fast queries*
*📤 Export (ADR-0007) - export findings to JSON for external tools*

### methodology-kb server
- `lookup_metric(metric, project_type?)` - Get metric definition and thresholds
- `classify_finding(tool?, rule_id?, category?, base_severity?, context?)` - Classify finding
- `get_thresholds(project_type, language?)` - Get thresholds for project type
- `check_compliance(standard, detected_pattern)` - Check architecture compliance
- `get_template(template_type, format?)` - Get report template
- `list_metrics()` - List available metrics
- `list_standards()` - List architecture standards
- `get_category(category)` - Get category details

### sarif-tools server
- `run_tool(tool, path, config?)` - Run analysis tool and get SARIF output
  - Tools: `semgrep`, `bandit`, `ruff`, `trivy`, `clippy`
- `list_available_tools()` - List tools with installation status
- `get_tool_config(tool)` - Get tool configuration
- `merge_sarif(sarif_files)` - Merge multiple SARIF results
- `normalize_sarif(sarif, rule_mappings?)` - Enrich SARIF with categories and severity

### codegraph server
- `load_index(scip_path)` - Load SCIP index or JSON codegraph
- `load_project_indexes(project_path, build_if_missing?)` - Auto-load all indexes from `.audit/indexes/`, optionally build missing 🆕
- `find_symbol(pattern)` - Search symbols by name pattern
- `get_symbol_info(symbol_id)` - Get detailed symbol information
- `get_callers(symbol_id)` - Get all references to a symbol
- `get_callees(symbol_id)` - Get symbols called from within a symbol
- `get_impact(symbol_id)` - Analyze change impact (affected files/references)
- `get_file_symbols(file_path)` - Get all symbols defined in a file
- `get_module_deps(module_path)` - Get module dependencies
- `find_hotspot_symbols(min_callers, path_filter?)` - Find heavily-referenced symbols

*🆕 ADR-0008: Auto-loads from `.audit/indexes/`, checks commit freshness, builds on-demand*

## Output Deliverables

After completing all viewpoints, generate:
1. `executive_summary.md` - Stakeholder-friendly overview
2. `root_cause_analysis.md` - Technical root causes with Fowler Quadrant
3. `detailed_findings.md` - All findings with context
4. `mental_model.yaml` - Complete audit data
5. `technical_debt_inventory.yaml` - Debt items by quadrant

## Success Criteria

- All 16 required viewpoints complete
- 3-5 root causes synthesized (not 800+ findings)
- Fowler Quadrant distribution analyzed
- Actionable recommendations provided

## Self-Audit

To audit this codebase itself:

### Quick Self-Audit
```bash
# Run Clippy for Rust analysis
cargo clippy --all-targets --message-format=json 2>&1 | head -100

# Run all tests
cargo test --all
```

### Full Self-Audit with SARIF
```bash
# Using sarif-tools MCP server
sarif-tools/run_tool clippy .

# Store artifact for current commit
mental-model/store_artifact HEAD clippy <sarif-data> "cargo-clippy"

# Check available artifacts
mental-model/get_commit_artifacts HEAD
```

### Artifacts Location
Audit artifacts are stored in `.audit/artifacts/<commit>/`:
- `*.sarif` - SARIF format analysis results
- `_meta.yaml` - Artifact metadata (producer, timestamp)
- `latest` symlink - Points to most recent commit

## Development

### Running Tests
```bash
# All tests
cargo test --all

# Specific server tests
cargo test -p mental-model-server
cargo test -p sarif-tools-server
cargo test -p codegraph-server
cargo test -p methodology-kb-server
```

### Building
```bash
cargo build --release
```

### Checking Code Quality
```bash
# No warnings should appear
cargo clippy --all-targets
```

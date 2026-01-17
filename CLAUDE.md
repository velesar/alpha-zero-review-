# AI Code Audit Agent - Claude CLI Instructions

This project provides a Mental Model-first approach to code quality audits.
When conducting audits, Claude uses custom MCP servers to build understanding
and produce actionable insights instead of raw finding dumps.

## MCP Servers

This project requires two MCP servers (configured in `.mcp.json`):

- **mental-model**: Manages the central mental model artifact
- **methodology-kb**: Knowledge base for metrics, thresholds, and standards

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
When adding any finding:
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
- `update_viewpoint(viewpoint, data)` - Update with viewpoint results
- `get_context(file_path)` - Get business context for a file
- `get_constraints()` - Get derived analysis constraints
- `add_finding(viewpoint, category, title, description, file_path, base_severity, ...)` - Add finding
- `get_findings()` - Get all findings
- `synthesize(algorithm?)` - Cluster findings into root causes
- `get_completed_viewpoints()` - List completed viewpoints

### methodology-kb server
- `lookup_metric(metric, project_type?)` - Get metric definition and thresholds
- `classify_finding(tool?, rule_id?, category?, base_severity?, context?)` - Classify finding
- `get_thresholds(project_type, language?)` - Get thresholds for project type
- `check_compliance(standard, detected_pattern)` - Check architecture compliance
- `get_template(template_type, format?)` - Get report template
- `list_metrics()` - List available metrics
- `list_standards()` - List architecture standards
- `get_category(category)` - Get category details

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

# AI Code Audit Agent - Operations Guide

> **Version:** 2.0 | **Last Updated:** January 2026

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Running an Audit](#running-an-audit)
5. [Understanding Output](#understanding-output)
6. [Customization](#customization)
7. [Troubleshooting](#troubleshooting)
8. [Best Practices](#best-practices)

---

## Prerequisites

### System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| RAM | 8 GB | 16 GB |
| Disk Space | 500 MB | 1 GB |
| Rust | 1.75+ | Latest stable |
| Git | 2.30+ | Latest |

### Required Tools

```bash
# Verify Rust installation
rustc --version
cargo --version

# Verify Git
git --version
```

### Optional Analysis Tools

Depending on the project language, install these for enhanced analysis:

**Python Projects:**
```bash
pip install radon bandit pip-audit
```

**JavaScript/TypeScript Projects:**
```bash
npm install -g madge eslint
```

**Rust Projects:**
```bash
cargo install cargo-audit cargo-deny
```

---

## Installation

### 1. Clone the Repository

```bash
git clone <repository-url> ai-code-audit-agent
cd ai-code-audit-agent
```

### 2. Build MCP Servers

```bash
# Build in release mode for production use
cargo build --release

# Verify builds
ls -la target/release/mental-model-server
ls -la target/release/methodology-kb-server
```

### 3. Verify Installation

```bash
# Test mental-model-server
./target/release/mental-model-server --help

# Test methodology-kb-server
./target/release/methodology-kb-server --help
```

---

## Configuration

### MCP Settings

Create or update your Cline MCP settings file (`mcp_settings.json`):

```json
{
  "mcpServers": {
    "mental-model": {
      "command": "/path/to/ai-code-audit-agent/target/release/mental-model-server",
      "args": ["--model-path", "./audit_output/mental_model.yaml"]
    },
    "methodology-kb": {
      "command": "/path/to/ai-code-audit-agent/target/release/methodology-kb-server",
      "args": ["--kb-path", "/path/to/ai-code-audit-agent/methodology_kb/"]
    }
  }
}
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AUDIT_OUTPUT_DIR` | Directory for audit outputs | `./audit_output` |
| `MENTAL_MODEL_PATH` | Path to mental model file | `./mental_model.yaml` |
| `KB_PATH` | Path to methodology KB | `./methodology_kb/` |

### Project Type Configuration

The audit adjusts thresholds based on project type. Specify in your project:

```yaml
# .audit-config.yaml (optional)
project:
  type: mature  # greenfield | mature | legacy | startup | enterprise
  language: python
  frameworks:
    - fastapi
    - sqlalchemy
```

**Project Types:**

| Type | Description | Threshold Strictness |
|------|-------------|---------------------|
| `greenfield` | New project, clean slate | Strictest |
| `mature` | Established, stable project | Standard |
| `legacy` | Old codebase, technical debt | Relaxed |
| `startup` | Fast-moving, MVP focus | Moderate |
| `enterprise` | Mission-critical, regulated | Strict |

---

## Running an Audit

### Quick Start

1. Navigate to the target project directory
2. Start Cline with the AI Code Audit Agent
3. Request an audit:

```
Audit this codebase
```

### Full Audit Workflow

#### Step 1: Initialize

```
Start a new code audit for this project
```

The agent will:
- Initialize the mental model
- Detect project type and language
- Load appropriate thresholds

#### Step 2: Foundation Phase

The agent automatically executes:
- **VP-F01**: Technology Stack Analysis
- **VP-F02**: Project Structure Analysis
- **VP-F03**: Build & Deployment Analysis

#### Step 3: Structure Phase

The agent maps the architecture:
- **VP-S01**: Module Hierarchy (C4 model)
- **VP-S02**: Layer Architecture
- **VP-S03**: Domain Model (bounded contexts)
- **VP-S04**: Entity Model
- **VP-S05**: Interface Surface
- **VP-S06**: Dependency Graph
- **VP-S07**: Architecture Decisions

**Optional viewpoints** (request if needed):
```
Also analyze team topologies for this enterprise project
```

#### Step 4: Quality Phase

The agent finds issues with context:
- **VP-Q01**: Security Analysis
- **VP-Q02**: Performance Analysis
- **VP-Q03**: Testability Analysis
- **VP-Q04**: Code Style Analysis
- **VP-Q05**: Documentation Analysis

#### Step 5: Synthesis Phase

The agent generates insights:
- **VP-Q06**: Technical Debt Synthesis with Fowler Quadrant classification

### Audit Commands

| Command | Description |
|---------|-------------|
| `Audit this codebase` | Full audit (all 16 viewpoints) |
| `Quick security scan` | VP-Q01 only with dependencies |
| `Analyze architecture` | Foundation + Structure phases |
| `Check technical debt` | Run synthesis on existing findings |
| `Resume audit from VP-S03` | Continue interrupted audit |

### Specifying Scope

```
Audit only the src/payments module
```

```
Focus the audit on security and performance
```

```
Run an enterprise compliance audit including TOGAF building blocks
```

---

## Understanding Output

### Output Files

After a complete audit, you'll find these files in the output directory:

```
audit_output/
├── executive_summary.md          # Stakeholder report (2 pages)
├── root_cause_analysis.md        # Technical root causes
├── detailed_findings.md          # All findings with context
├── mental_model.yaml             # Complete audit data
└── technical_debt_inventory.yaml # Debt items by quadrant
```

### Executive Summary

The executive summary contains:

1. **Health Score**: Overall project health (0-100)
2. **Key Metrics**: Coverage, complexity, security posture
3. **Top 3-5 Root Causes**: Not individual findings
4. **Quadrant Distribution**: Team health indicators
5. **Recommended Actions**: Prioritized by impact

### Fowler Quadrant Analysis

```
                DELIBERATE              INADVERTENT
         ┌────────────────────┬────────────────────┐
 PRUDENT │ 8 items            │ 12 items           │
         │ → Track in backlog │ → Refactor         │
         ├────────────────────┼────────────────────┤
RECKLESS │ 5 items            │ 20 items           │
         │ → Urgent fix       │ → Training needed  │
         └────────────────────┴────────────────────┘
```

**Interpretation:**
- **High Prudent**: Healthy team making conscious trade-offs
- **High Reckless-Deliberate**: Process problems (rushed delivery)
- **High Reckless-Inadvertent**: Knowledge gaps (training needed)

### Root Cause Format

Each root cause includes:

```markdown
## RC-001: Missing Input Validation Layer

**Category**: Security
**Quadrant**: Reckless-Inadvertent
**Affected Findings**: F-001, F-003, F-007, F-012, F-015

### Why This Exists
The team was unaware of centralized validation patterns.
No middleware for request validation was established early.

### Impact
- 15 injection vulnerabilities across API endpoints
- Inconsistent validation logic duplicated in handlers
- Security review failures in CI/CD

### Recommendation
Implement centralized validation middleware using Pydantic
or similar schema validation at the API boundary.

### Effort Estimate
- Implementation: M (2-3 days)
- Testing: S (1 day)
- Rollout: S (gradual, per endpoint)
```

### Severity Levels

| Level | Description | Action Timeline |
|-------|-------------|-----------------|
| **CRITICAL** | Security vulnerability, data loss risk | Immediate (0-24h) |
| **HIGH** | Significant impact on reliability/security | This sprint |
| **MEDIUM** | Maintainability issues, code smell | Next sprint |
| **LOW** | Minor improvements, nice-to-have | Backlog |

### Context-Adjusted Severity

Severity is adjusted based on:

| Factor | Multiplier | Example |
|--------|------------|---------|
| Core Domain | 1.5x | Payment processing |
| Supporting Domain | 1.0x | User preferences |
| Generic Domain | 0.7x | Utilities, helpers |
| Hotspot File | 1.3x | High churn + complexity |
| Public API | 1.2x | External-facing endpoints |

---

## Customization

### Custom Thresholds

Create `methodology_kb/thresholds/custom.yaml`:

```yaml
custom:
  name: "Custom Thresholds"
  description: "Project-specific thresholds"

  complexity:
    cyclomatic:
      good: 8
      acceptable: 15
      warning: 25
    cognitive:
      good: 10
      acceptable: 20
      warning: 35

  coverage:
    line:
      good: 90
      acceptable: 80
      warning: 60
    branch:
      good: 85
      acceptable: 70
      warning: 50
```

### Custom Standards

Add to `methodology_kb/standards/`:

```yaml
# custom_architecture.yaml
custom_architecture:
  name: "Custom Architecture Standard"
  layers:
    - name: "api"
      allowed_deps: ["service", "domain"]
    - name: "service"
      allowed_deps: ["domain", "repository"]
    - name: "domain"
      allowed_deps: []
    - name: "repository"
      allowed_deps: ["domain"]
```

### Extending Viewpoints

Create custom viewpoint in `skills/vp-custom/SKILL.md`:

```yaml
---
name: vp-custom-compliance
version: 1.0
dependencies: [vp-f01-tech-stack]
mcp_servers: [mental-model, methodology-kb]
---

# VP-Custom: Compliance Check

## Purpose
Check compliance with internal policies...

## Instructions
...
```

### Filtering Findings

In `.audit-config.yaml`:

```yaml
filters:
  exclude_paths:
    - "**/test/**"
    - "**/migrations/**"
    - "**/*.generated.ts"

  exclude_rules:
    - "S101"  # Skip specific rule

  severity_minimum: "medium"  # Ignore LOW findings
```

---

## Troubleshooting

### Common Issues

#### MCP Server Not Starting

```bash
# Check if port is in use
lsof -i :3000

# Verify binary exists
ls -la target/release/mental-model-server

# Run with verbose logging
RUST_LOG=debug ./target/release/mental-model-server
```

#### Audit Stalls at Viewpoint

```
Resume audit from VP-S03
```

Or manually update the mental model:
```
Mark VP-S02 as complete and proceed to VP-S03
```

#### Out of Memory

For large codebases, limit scope:
```
Audit only the core domain (src/domain, src/application)
```

#### Git History Not Available

If churn analysis fails:
```bash
# Verify git repository
git status

# Ensure sufficient history
git log --oneline | head -20
```

### Debug Mode

Enable verbose output:

```yaml
# .audit-config.yaml
debug:
  verbose: true
  save_intermediate: true
  log_tool_calls: true
```

### Logs Location

```
~/.cache/ai-code-audit/
├── audit.log           # Main audit log
├── mcp-mental-model.log
└── mcp-methodology-kb.log
```

---

## Best Practices

### Before the Audit

1. **Clean Working Directory**
   ```bash
   git status  # Should be clean or stashed
   ```

2. **Update Dependencies**
   ```bash
   # Ensure dependency files are current
   pip freeze > requirements.txt  # Python
   npm list --depth=0            # Node.js
   ```

3. **Run Existing Tests**
   ```bash
   # Audit can reference test results
   pytest --cov=src --cov-report=xml
   ```

### During the Audit

1. **Don't Interrupt Viewpoints**: Let each viewpoint complete
2. **Review Intermediate Results**: Check mental model after each phase
3. **Ask for Clarification**: If findings seem wrong, provide context

### After the Audit

1. **Review with Team**: Share executive summary first
2. **Prioritize by Quadrant**:
   - Fix Reckless-Deliberate immediately
   - Schedule training for Reckless-Inadvertent
   - Track Prudent-Deliberate in backlog
   - Refactor Prudent-Inadvertent incrementally

3. **Create Tickets**: One ticket per root cause, not per finding
4. **Schedule Follow-up**: Re-audit in 3-6 months to track improvement

### Audit Frequency

| Project Stage | Recommended Frequency |
|---------------|----------------------|
| Active Development | Monthly |
| Maintenance Mode | Quarterly |
| Pre-Release | Before each major release |
| Post-Incident | After security/reliability issues |

### Integrating with CI/CD

```yaml
# .github/workflows/audit.yml
name: Code Audit
on:
  schedule:
    - cron: '0 0 1 * *'  # Monthly
  workflow_dispatch:

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for churn analysis

      - name: Run Audit
        run: |
          # Run audit and export SARIF
          ./scripts/run-audit.sh --output sarif

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: audit_output/findings.sarif
```

---

## Quick Reference

### Viewpoint Execution Order

```
FOUNDATION (1-3)     STRUCTURE (4-10)       QUALITY (11-15)     SYNTHESIS (16)
─────────────────    ──────────────────     ────────────────    ──────────────
VP-F01 Tech Stack    VP-S01 Modules         VP-Q01 Security     VP-Q06 Debt
VP-F02 Structure     VP-S02 Layers          VP-Q02 Performance  Synthesis
VP-F03 Build/Deploy  VP-S03 Domain          VP-Q03 Testability
                     VP-S04 Entities        VP-Q04 Code Style
                     VP-S05 Interfaces      VP-Q05 Documentation
                     VP-S06 Dependencies
                     VP-S07 ADRs
                     [VP-S08-S10 Optional]
```

### MCP Tool Quick Reference

**mental-model-server:**
| Tool | Purpose |
|------|---------|
| `init_model` | Initialize new audit |
| `get_model` | Retrieve current state |
| `update_viewpoint` | Save viewpoint results |
| `get_context` | Get file context for findings |
| `get_constraints` | Get priority paths |
| `add_finding` | Add individual finding |
| `get_findings` | Retrieve all findings |
| `synthesize` | Generate root causes |
| `get_completed_viewpoints` | List completed viewpoints |

**methodology-kb-server:**
| Tool | Purpose |
|------|---------|
| `lookup_metric` | Get metric definition |
| `classify_finding` | Adjust severity by context |
| `get_thresholds` | Get project-type thresholds |
| `check_compliance` | Verify architecture compliance |
| `get_template` | Get report template |
| `list_metrics` | List available metrics |
| `list_standards` | List available standards |
| `get_category` | Get category hierarchy |

---

## Support

- **Documentation**: See `docs/VIEWPOINTS_FRAMEWORK.md`
- **Issues**: Report at repository issue tracker
- **Questions**: Contact Light IT Global team

---

*AI Code Audit Agent - Operations Guide v2.0*

# AI Code Audit Agent - Quick Start Guide

## Prerequisites

1. **Claude CLI** installed (`claude --version`)
2. **Rust toolchain** for building MCP servers

## Setup (One-time)

```bash
# Clone the repository
cd /home/velesar/projects/alpha-zero-review-

# Build MCP servers
cargo build --release

# Verify build
./target/release/mental-model-server --help
./target/release/methodology-kb-server --help
```

## Running an Audit

### Option 1: Use the Setup Script

```bash
# Run from the agent directory
./scripts/run-audit.sh /path/to/target/project

# Then start Claude CLI
cd /path/to/target/project
claude
```

### Option 2: Manual Setup

1. Copy `.mcp.json` to the target project:
```json
{
  "mcpServers": {
    "mental-model": {
      "command": "/home/velesar/projects/alpha-zero-review-/target/release/mental-model-server",
      "args": ["--model-path", "./mental_model.yaml"]
    },
    "methodology-kb": {
      "command": "/home/velesar/projects/alpha-zero-review-/target/release/methodology-kb-server",
      "args": ["--kb-path", "/home/velesar/projects/alpha-zero-review-/methodology_kb/"]
    }
  }
}
```

2. Start Claude CLI in the target project:
```bash
cd /path/to/target/project
claude
```

## Starting the Audit

Once Claude CLI is running, ask:

```
Run a full code audit using the viewpoints framework.
Start by reading /home/velesar/projects/alpha-zero-review-/skills/vp-f01-tech-stack/SKILL.md
```

Or for step-by-step:

```
Let's audit this codebase. Start with VP-F01: Technology Stack Analysis.
Read the instructions from /home/velesar/projects/alpha-zero-review-/skills/vp-f01-tech-stack/SKILL.md
```

## Audit Workflow

Claude will execute viewpoints in sequence:

### Phase 1: Foundation
1. VP-F01: Technology Stack Analysis
2. VP-F02: Project Structure Analysis
3. VP-F03: Build & Deployment Analysis

### Phase 2: Structure
4. VP-S01: Module Hierarchy
5. VP-S02: Layer Architecture
6. VP-S03: Domain Model
7. VP-S04: Entity Model
8. VP-S05: Interface Surface
9. VP-S06: Dependency Graph
10. VP-S07: Architecture Decisions

### Phase 3: Quality
11. VP-Q01: Security Analysis
12. VP-Q02: Performance Analysis
13. VP-Q03: Testability Analysis
14. VP-Q04: Code Style Analysis
15. VP-Q05: Documentation Analysis

### Phase 4: Synthesis
16. VP-Q06: Technical Debt Synthesis (Fowler Quadrant)

## MCP Tools Quick Reference

### Checking MCP Status

In Claude CLI, type `/mcp` to see connected servers.

### Key Tools

```
# Initialize model (do this first!)
mental-model/init_model(name="ProjectName", path="/path/to/project")

# Get current model state
mental-model/get_model()

# Update after viewpoint
mental-model/update_viewpoint(viewpoint="VP-F01", data={...})

# Add a finding
mental-model/add_finding(
  viewpoint="VP-Q01",
  category="security",
  title="SQL Injection",
  description="...",
  file_path="src/db.py",
  base_severity="HIGH"
)

# Synthesize root causes
mental-model/synthesize()
```

## Expected Outputs

After completing all viewpoints:

- `mental_model.yaml` - Complete audit data
- `executive_summary.md` - Stakeholder report
- `root_cause_analysis.md` - Technical findings with Fowler Quadrant
- `detailed_findings.md` - All findings with context
- `technical_debt_inventory.yaml` - Debt items by quadrant

## Troubleshooting

### MCP servers not connecting

1. Check if servers are built:
   ```bash
   ls -la /home/velesar/projects/alpha-zero-review-/target/release/
   ```

2. Test server manually:
   ```bash
   echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | ./target/release/mental-model-server
   ```

3. Check `.mcp.json` paths are correct

### "Tool not found" errors

Verify MCP servers are listed in `/mcp` command output.

## Test on This Project

To test the agent on itself:

```bash
cd /home/velesar/projects/alpha-zero-review-
claude

# Then:
"Initialize mental model for this project and run VP-F01"
```

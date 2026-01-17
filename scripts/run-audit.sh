#!/bin/bash
# Run AI Code Audit Agent on a target project
#
# Usage: ./scripts/run-audit.sh /path/to/target/project
#
# This script:
# 1. Copies the MCP config to the target project
# 2. Copies CLAUDE.md instructions to the target project
# 3. Starts Claude CLI in the target project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR="${1:-.}"

# Resolve to absolute path
TARGET_DIR="$(cd "$TARGET_DIR" && pwd)"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           AI Code Audit Agent - Claude CLI Setup               ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║ Agent Directory: $AGENT_DIR"
echo "║ Target Project:  $TARGET_DIR"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if servers are built
if [ ! -f "$AGENT_DIR/target/release/mental-model-server" ]; then
    echo "❌ MCP servers not built. Building..."
    cd "$AGENT_DIR" && cargo build --release
fi

if [ -f "$AGENT_DIR/target/release/mental-model-server" ]; then
    echo "✓ mental-model-server built"
else
    echo "❌ Failed to build mental-model-server"
    exit 1
fi

if [ -f "$AGENT_DIR/target/release/methodology-kb-server" ]; then
    echo "✓ methodology-kb-server built"
else
    echo "❌ Failed to build methodology-kb-server"
    exit 1
fi

# Copy MCP config to target
echo ""
echo "Setting up MCP configuration..."

cat > "$TARGET_DIR/.mcp.json" << EOF
{
  "mcpServers": {
    "mental-model": {
      "command": "$AGENT_DIR/target/release/mental-model-server",
      "args": [
        "--model-path",
        "./mental_model.yaml"
      ]
    },
    "methodology-kb": {
      "command": "$AGENT_DIR/target/release/methodology-kb-server",
      "args": [
        "--kb-path",
        "$AGENT_DIR/methodology_kb/"
      ]
    }
  }
}
EOF
echo "✓ Created $TARGET_DIR/.mcp.json"

# Copy CLAUDE.md to target (with skills path updated)
cat > "$TARGET_DIR/CLAUDE.md" << 'HEREDOC'
# AI Code Audit Agent Instructions

This project is being audited using the AI Code Audit Agent methodology.

## Quick Start

To run a full audit, ask:
```
Run a full code audit using the viewpoints framework
```

## MCP Tools Available

### mental-model server
- `init_model(name, path)` - Initialize mental model
- `get_model()` - Get current state
- `update_viewpoint(viewpoint, data)` - Update after analysis
- `get_context(file_path)` - Get business context for a file
- `add_finding(...)` - Add a finding with context
- `get_findings()` - List all findings
- `synthesize()` - Generate root causes

### methodology-kb server
- `lookup_metric(metric)` - Get metric thresholds
- `classify_finding(...)` - Classify with severity adjustment
- `get_thresholds(project_type)` - Get all thresholds
- `check_compliance(standard, pattern)` - Check architecture

## Audit Phases

1. **Foundation**: Tech stack, structure, build system
2. **Structure**: Modules, layers, domain, entities, interfaces
3. **Quality**: Security, performance, testability, style, docs
4. **Synthesis**: Root causes with Fowler Quadrant classification

## Key Rule

**NEVER dump raw findings. Always synthesize into 3-5 root causes.**
HEREDOC

echo "✓ Created $TARGET_DIR/CLAUDE.md"

# Create skills symlink or copy viewpoint reference
echo ""
echo "Creating viewpoint reference..."
cat > "$TARGET_DIR/.audit-viewpoints.md" << EOF
# Audit Viewpoints Reference

Viewpoint instructions are located at:
$AGENT_DIR/skills/

## Viewpoint List

### Foundation Phase
- VP-F01: Technology Stack - $AGENT_DIR/skills/vp-f01-tech-stack/SKILL.md
- VP-F02: Project Structure - $AGENT_DIR/skills/vp-f02-structure/SKILL.md
- VP-F03: Build & Deploy - $AGENT_DIR/skills/vp-f03-build-deploy/SKILL.md

### Structure Phase
- VP-S01: Module Hierarchy - $AGENT_DIR/skills/vp-s01-module-hierarchy/SKILL.md
- VP-S02: Layer Architecture - $AGENT_DIR/skills/vp-s02-layer-architecture/SKILL.md
- VP-S03: Domain Model - $AGENT_DIR/skills/vp-s03-domain-model/SKILL.md
- VP-S04: Entity Model - $AGENT_DIR/skills/vp-s04-entity-model/SKILL.md
- VP-S05: Interface Surface - $AGENT_DIR/skills/vp-s05-interface-surface/SKILL.md
- VP-S06: Dependency Graph - $AGENT_DIR/skills/vp-s06-dependency-graph/SKILL.md
- VP-S07: Architecture Decisions - $AGENT_DIR/skills/vp-s07-architecture-decisions/SKILL.md

### Quality Phase
- VP-Q01: Security - $AGENT_DIR/skills/vp-q01-security/SKILL.md
- VP-Q02: Performance - $AGENT_DIR/skills/vp-q02-performance/SKILL.md
- VP-Q03: Testability - $AGENT_DIR/skills/vp-q03-testability/SKILL.md
- VP-Q04: Code Style - $AGENT_DIR/skills/vp-q04-code-style/SKILL.md
- VP-Q05: Documentation - $AGENT_DIR/skills/vp-q05-documentation/SKILL.md

### Synthesis Phase
- VP-Q06: Technical Debt Synthesis - $AGENT_DIR/skills/vp-q06-synthesis/SKILL.md

To execute a viewpoint, read its SKILL.md file and follow the instructions.
EOF
echo "✓ Created $TARGET_DIR/.audit-viewpoints.md"

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                      Setup Complete!                           ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║                                                                ║"
echo "║  To start the audit:                                           ║"
echo "║                                                                ║"
echo "║    cd $TARGET_DIR"
echo "║    claude                                                      ║"
echo "║                                                                ║"
echo "║  Then ask:                                                     ║"
echo "║    \"Run a full code audit using the viewpoints framework\"     ║"
echo "║                                                                ║"
echo "║  Or for step-by-step:                                          ║"
echo "║    \"Start with VP-F01: Technology Stack Analysis\"             ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"

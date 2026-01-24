#!/bin/bash
# Run AI Code Audit Agent on a target project
#
# Usage: ./scripts/run-audit.sh /path/to/target/project [options]
#
# Options:
#   --skip-build    Skip building MCP servers
#   --clean         Remove existing audit artifacts before starting
#
# This script:
# 1. Builds MCP servers if needed
# 2. Copies MCP config to the target project
# 3. Copies CLAUDE.md instructions to the target project
# 4. Sets up .audit directory for artifacts
# 5. Provides instructions to start Claude CLI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR=""
SKIP_BUILD=false
CLEAN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        *)
            TARGET_DIR="$1"
            shift
            ;;
    esac
done

# Default to current directory if no target specified
TARGET_DIR="${TARGET_DIR:-.}"

# Resolve to absolute path
TARGET_DIR="$(cd "$TARGET_DIR" && pwd)"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           AI Code Audit Agent - Setup v2.0                     ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║ Agent Directory: $AGENT_DIR"
echo "║ Target Project:  $TARGET_DIR"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# List of all MCP servers
SERVERS=(
    "mental-model-server"
    "methodology-kb-server"
    "sarif-tools-server"
    "codegraph-server"
)

# Build servers if needed
if [ "$SKIP_BUILD" = false ]; then
    NEED_BUILD=false
    for server in "${SERVERS[@]}"; do
        if [ ! -f "$AGENT_DIR/target/release/$server" ]; then
            NEED_BUILD=true
            break
        fi
    done

    if [ "$NEED_BUILD" = true ]; then
        echo "Building MCP servers..."
        cd "$AGENT_DIR" && cargo build --release
        echo ""
    fi
fi

# Verify all servers are built
echo "Checking MCP servers..."
ALL_BUILT=true
for server in "${SERVERS[@]}"; do
    if [ -f "$AGENT_DIR/target/release/$server" ]; then
        echo "  ✓ $server"
    else
        echo "  ✗ $server (not found)"
        ALL_BUILT=false
    fi
done

if [ "$ALL_BUILT" = false ]; then
    echo ""
    echo "❌ Some servers are missing. Run: cd $AGENT_DIR && cargo build --release"
    exit 1
fi

# Clean existing audit artifacts if requested
if [ "$CLEAN" = true ]; then
    echo ""
    echo "Cleaning existing audit artifacts..."
    rm -rf "$TARGET_DIR/.audit"
    rm -f "$TARGET_DIR/.mcp.json"
    rm -f "$TARGET_DIR/CLAUDE.md"
    rm -f "$TARGET_DIR/.audit-viewpoints.md"
    echo "  ✓ Cleaned"
fi

# Create .audit directory
echo ""
echo "Setting up audit directory..."
mkdir -p "$TARGET_DIR/.audit/artifacts"
echo "  ✓ Created $TARGET_DIR/.audit/"

# Create MCP configuration with all 4 servers
echo ""
echo "Creating MCP configuration..."
cat > "$TARGET_DIR/.mcp.json" << EOF
{
  "mcpServers": {
    "mental-model": {
      "command": "$AGENT_DIR/target/release/mental-model-server",
      "args": [
        "--model-path", "$TARGET_DIR/.audit/mental_model.yaml",
        "--audit-path", "$TARGET_DIR/.audit"
      ]
    },
    "methodology-kb": {
      "command": "$AGENT_DIR/target/release/methodology-kb-server",
      "args": [
        "--kb-path", "$AGENT_DIR/methodology_kb/"
      ]
    },
    "sarif-tools": {
      "command": "$AGENT_DIR/target/release/sarif-tools-server",
      "args": []
    },
    "codegraph": {
      "command": "$AGENT_DIR/target/release/codegraph-server",
      "args": []
    }
  }
}
EOF
echo "  ✓ Created $TARGET_DIR/.mcp.json"

# Copy the full CLAUDE.md from agent directory
echo ""
echo "Copying CLAUDE.md instructions..."
cp "$AGENT_DIR/CLAUDE.md" "$TARGET_DIR/CLAUDE.md"
echo "  ✓ Copied $TARGET_DIR/CLAUDE.md"

# Create viewpoints reference with full paths
echo ""
echo "Creating viewpoints reference..."
cat > "$TARGET_DIR/.audit-viewpoints.md" << EOF
# Audit Viewpoints Reference

Agent installation: $AGENT_DIR
Skills directory: $AGENT_DIR/skills/

## How to Execute a Viewpoint

1. Read the SKILL.md file for the viewpoint
2. Follow the instructions using the MCP tools
3. Update the mental model with results

Example:
\`\`\`
Read $AGENT_DIR/skills/vp-f01-tech-stack/SKILL.md and follow its instructions
\`\`\`

## Viewpoint List

### Phase 1: Foundation
| ID | Name | Skill File |
|----|------|------------|
| VP-F01 | Technology Stack | $AGENT_DIR/skills/vp-f01-tech-stack/SKILL.md |
| VP-F02 | Project Structure | $AGENT_DIR/skills/vp-f02-structure/SKILL.md |
| VP-F03 | Build & Deploy | $AGENT_DIR/skills/vp-f03-build-deploy/SKILL.md |

### Phase 2: Structure
| ID | Name | Skill File |
|----|------|------------|
| VP-S01 | Module Hierarchy | $AGENT_DIR/skills/vp-s01-module-hierarchy/SKILL.md |
| VP-S02 | Layer Architecture | $AGENT_DIR/skills/vp-s02-layer-architecture/SKILL.md |
| VP-S03 | Domain Model | $AGENT_DIR/skills/vp-s03-domain-model/SKILL.md |
| VP-S04 | Entity Model | $AGENT_DIR/skills/vp-s04-entity-model/SKILL.md |
| VP-S05 | Interface Surface | $AGENT_DIR/skills/vp-s05-interface-surface/SKILL.md |
| VP-S06 | Dependency Graph | $AGENT_DIR/skills/vp-s06-dependency-graph/SKILL.md |
| VP-S07 | Architecture Decisions | $AGENT_DIR/skills/vp-s07-architecture-decisions/SKILL.md |

### Phase 3: Quality
| ID | Name | Skill File |
|----|------|------------|
| VP-Q01 | Security | $AGENT_DIR/skills/vp-q01-security/SKILL.md |
| VP-Q02 | Performance | $AGENT_DIR/skills/vp-q02-performance/SKILL.md |
| VP-Q03 | Testability | $AGENT_DIR/skills/vp-q03-testability/SKILL.md |
| VP-Q04 | Code Style | $AGENT_DIR/skills/vp-q04-code-style/SKILL.md |
| VP-Q05 | Documentation | $AGENT_DIR/skills/vp-q05-documentation/SKILL.md |

### Phase 4: Synthesis
| ID | Name | Skill File |
|----|------|------------|
| VP-Q06 | Technical Debt Synthesis | $AGENT_DIR/skills/vp-q06-synthesis/SKILL.md |

## MCP Servers Available

| Server | Tools | Purpose |
|--------|-------|---------|
| mental-model | 22 | Central audit artifact, findings, synthesis |
| methodology-kb | 11 | Metrics, thresholds, classification |
| sarif-tools | 3 | Code analysis tools (clippy, semgrep, etc.) |
| codegraph | 10 | SCIP code intelligence, impact analysis |

## Audit Outputs

After completing the audit, generate:
1. \`executive_summary.md\` - Stakeholder overview
2. \`root_cause_analysis.md\` - Technical debt with Fowler Quadrant
3. \`detailed_findings.md\` - All findings with context

Files will be stored in: $TARGET_DIR/.audit/
EOF
echo "  ✓ Created $TARGET_DIR/.audit-viewpoints.md"

# Add .audit to .gitignore if not already there
if [ -f "$TARGET_DIR/.gitignore" ]; then
    if ! grep -q "^\.audit/" "$TARGET_DIR/.gitignore"; then
        echo "" >> "$TARGET_DIR/.gitignore"
        echo "# AI Code Audit Agent artifacts" >> "$TARGET_DIR/.gitignore"
        echo ".audit/" >> "$TARGET_DIR/.gitignore"
        echo "  ✓ Added .audit/ to .gitignore"
    fi
fi

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                      Setup Complete!                           ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║                                                                ║"
echo "║  Files created:                                                ║"
echo "║    • .mcp.json           (MCP server configuration)            ║"
echo "║    • CLAUDE.md           (Audit instructions)                  ║"
echo "║    • .audit-viewpoints.md (Viewpoint reference)                ║"
echo "║    • .audit/             (Artifact storage directory)          ║"
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
echo "║    \"Init audit\" then \"Start with VP-F01\"                      ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"

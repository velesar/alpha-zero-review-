#!/bin/bash
# Run AI Code Audit Agent on a target project
#
# Usage: ./scripts/run-audit.sh /path/to/target/project [options]
#
# Options:
#   --cli <tool>    CLI tool to configure: claude, codex, cline (default: claude)
#   --skip-build    Skip building MCP servers
#   --clean         Remove existing audit artifacts before starting
#   --all           Configure for all supported CLI tools
#
# Supported CLI Tools:
#   claude  - Anthropic Claude CLI (uses .mcp.json + CLAUDE.md)
#   codex   - OpenAI Codex CLI (uses codex.json + AGENTS.md)
#   cline   - Cline VS Code Extension (uses .cline/ directory)
#
# Examples:
#   ./scripts/run-audit.sh /path/to/project                    # Default (Claude)
#   ./scripts/run-audit.sh /path/to/project --cli codex        # Codex CLI
#   ./scripts/run-audit.sh /path/to/project --cli cline        # Cline
#   ./scripts/run-audit.sh /path/to/project --all              # All tools

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_DIR="$(dirname "$SCRIPT_DIR")"
TARGET_DIR=""
CLI_TOOL="claude"
SKIP_BUILD=false
CLEAN=false
CONFIGURE_ALL=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --cli)
            CLI_TOOL="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --all)
            CONFIGURE_ALL=true
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

# Validate CLI tool
if [ "$CONFIGURE_ALL" = false ]; then
    case $CLI_TOOL in
        claude|codex|cline)
            ;;
        *)
            echo "❌ Unknown CLI tool: $CLI_TOOL"
            echo "   Supported: claude, codex, cline"
            exit 1
            ;;
    esac
fi

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║           AI Code Audit Agent - Multi-CLI Setup v2.1           ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║ Agent Directory: $AGENT_DIR"
echo "║ Target Project:  $TARGET_DIR"
if [ "$CONFIGURE_ALL" = true ]; then
    echo "║ CLI Tools:       ALL (claude, codex, cline)"
else
    echo "║ CLI Tool:        $CLI_TOOL"
fi
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
    rm -rf "$TARGET_DIR/.cline"
    rm -f "$TARGET_DIR/.mcp.json"
    rm -f "$TARGET_DIR/codex.json"
    rm -f "$TARGET_DIR/CLAUDE.md"
    rm -f "$TARGET_DIR/AGENTS.md"
    rm -f "$TARGET_DIR/.clinerules"
    rm -f "$TARGET_DIR/.audit-viewpoints.md"
    echo "  ✓ Cleaned"
fi

# Create .audit directory
echo ""
echo "Setting up audit directory..."
mkdir -p "$TARGET_DIR/.audit/artifacts"
echo "  ✓ Created $TARGET_DIR/.audit/"

# ============================================================================
# Configuration Functions
# ============================================================================

configure_claude() {
    echo ""
    echo "Configuring for Claude CLI..."

    # Create MCP configuration
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
    echo "  ✓ Created .mcp.json"

    # Copy CLAUDE.md
    cp "$AGENT_DIR/CLAUDE.md" "$TARGET_DIR/CLAUDE.md"
    echo "  ✓ Created CLAUDE.md"
}

configure_codex() {
    echo ""
    echo "Configuring for Codex CLI..."

    # Create Codex configuration (OpenAI format)
    cat > "$TARGET_DIR/codex.json" << EOF
{
  "name": "AI Code Audit Agent",
  "version": "2.0",
  "mcp_servers": {
    "mental-model": {
      "command": "$AGENT_DIR/target/release/mental-model-server",
      "args": ["--model-path", "$TARGET_DIR/.audit/mental_model.yaml", "--audit-path", "$TARGET_DIR/.audit"],
      "transport": "stdio"
    },
    "methodology-kb": {
      "command": "$AGENT_DIR/target/release/methodology-kb-server",
      "args": ["--kb-path", "$AGENT_DIR/methodology_kb/"],
      "transport": "stdio"
    },
    "sarif-tools": {
      "command": "$AGENT_DIR/target/release/sarif-tools-server",
      "args": [],
      "transport": "stdio"
    },
    "codegraph": {
      "command": "$AGENT_DIR/target/release/codegraph-server",
      "args": [],
      "transport": "stdio"
    }
  }
}
EOF
    echo "  ✓ Created codex.json"

    # Create AGENTS.md (Codex instruction format)
    cat > "$TARGET_DIR/AGENTS.md" << 'EOF'
# AI Code Audit Agent Instructions

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

### codegraph (10 tools)
Code intelligence: load_index, find_symbol, get_callers, get_impact, find_hotspot_symbols

## Audit Workflow

1. **Foundation**: VP-F01 (Tech Stack), VP-F02 (Structure), VP-F03 (Build)
2. **Structure**: VP-S01-S07 (Modules, Layers, Domain, Entities, Interfaces)
3. **Quality**: VP-Q01-Q05 (Security, Performance, Testability, Style, Docs)
4. **Synthesis**: VP-Q06 (Root causes with Fowler Quadrant)

## Key Rule
**NEVER dump raw findings. Always synthesize into 3-5 root causes.**
EOF
    echo "  ✓ Created AGENTS.md"
}

configure_cline() {
    echo ""
    echo "Configuring for Cline (VS Code Extension)..."

    # Create .cline directory
    mkdir -p "$TARGET_DIR/.cline"

    # Create Cline MCP settings
    cat > "$TARGET_DIR/.cline/mcp_settings.json" << EOF
{
  "mcpServers": {
    "mental-model": {
      "command": "$AGENT_DIR/target/release/mental-model-server",
      "args": [
        "--model-path", "$TARGET_DIR/.audit/mental_model.yaml",
        "--audit-path", "$TARGET_DIR/.audit"
      ],
      "disabled": false
    },
    "methodology-kb": {
      "command": "$AGENT_DIR/target/release/methodology-kb-server",
      "args": [
        "--kb-path", "$AGENT_DIR/methodology_kb/"
      ],
      "disabled": false
    },
    "sarif-tools": {
      "command": "$AGENT_DIR/target/release/sarif-tools-server",
      "args": [],
      "disabled": false
    },
    "codegraph": {
      "command": "$AGENT_DIR/target/release/codegraph-server",
      "args": [],
      "disabled": false
    }
  }
}
EOF
    echo "  ✓ Created .cline/mcp_settings.json"

    # Create .clinerules (Cline instruction format)
    cat > "$TARGET_DIR/.clinerules" << 'EOF'
# AI Code Audit Agent - Cline Rules

## Project Context
This project is configured for code auditing using the AI Code Audit Agent methodology with MCP servers.

## MCP Servers Available
- **mental-model**: Central audit artifact (22 tools) - init_model, get_model, add_finding, synthesize, etc.
- **methodology-kb**: Metrics & thresholds (11 tools) - lookup_metric, classify_finding, check_compliance
- **sarif-tools**: Code analysis (3 tools) - list_available_tools, merge_sarif
- **codegraph**: Code intelligence (10 tools) - load_index, find_symbol, get_impact

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
EOF
    echo "  ✓ Created .clinerules"
}

# ============================================================================
# Apply Configuration
# ============================================================================

if [ "$CONFIGURE_ALL" = true ]; then
    configure_claude
    configure_codex
    configure_cline
else
    case $CLI_TOOL in
        claude)
            configure_claude
            ;;
        codex)
            configure_codex
            ;;
        cline)
            configure_cline
            ;;
    esac
fi

# Create viewpoints reference (shared across all tools)
echo ""
echo "Creating viewpoints reference..."
cat > "$TARGET_DIR/.audit-viewpoints.md" << EOF
# Audit Viewpoints Reference

Agent installation: $AGENT_DIR
Skills directory: $AGENT_DIR/skills/

## Viewpoint Execution

| Phase | Viewpoints | Description |
|-------|------------|-------------|
| Foundation | VP-F01, VP-F02, VP-F03 | Tech stack, structure, build |
| Structure | VP-S01 - VP-S07 | Modules, layers, domain, entities |
| Quality | VP-Q01 - VP-Q05 | Security, performance, testability |
| Synthesis | VP-Q06 | Root causes, Fowler Quadrant |

## Skill Files Location

\`\`\`
$AGENT_DIR/skills/
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
\`\`\`

## MCP Server Summary

| Server | Tools | Purpose |
|--------|-------|---------|
| mental-model | 22 | Central audit artifact, findings, synthesis |
| methodology-kb | 11 | Metrics, thresholds, classification |
| sarif-tools | 3 | Code analysis tools |
| codegraph | 10 | SCIP code intelligence |
| **Total** | **46** | |
EOF
echo "  ✓ Created .audit-viewpoints.md"

# Add .audit to .gitignore if not already there
if [ -f "$TARGET_DIR/.gitignore" ]; then
    if ! grep -q "^\.audit/" "$TARGET_DIR/.gitignore"; then
        echo "" >> "$TARGET_DIR/.gitignore"
        echo "# AI Code Audit Agent artifacts" >> "$TARGET_DIR/.gitignore"
        echo ".audit/" >> "$TARGET_DIR/.gitignore"
        echo "  ✓ Added .audit/ to .gitignore"
    fi
fi

# Print completion message based on configured tools
echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                      Setup Complete!                           ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║                                                                ║"
echo "║  Files created:                                                ║"

if [ "$CONFIGURE_ALL" = true ] || [ "$CLI_TOOL" = "claude" ]; then
    echo "║    • .mcp.json + CLAUDE.md      (Claude CLI)                  ║"
fi
if [ "$CONFIGURE_ALL" = true ] || [ "$CLI_TOOL" = "codex" ]; then
    echo "║    • codex.json + AGENTS.md     (Codex CLI)                   ║"
fi
if [ "$CONFIGURE_ALL" = true ] || [ "$CLI_TOOL" = "cline" ]; then
    echo "║    • .cline/ + .clinerules      (Cline VS Code)               ║"
fi

echo "║    • .audit/                    (Artifact storage)             ║"
echo "║    • .audit-viewpoints.md       (Viewpoint reference)          ║"
echo "║                                                                ║"
echo "║  To start the audit:                                           ║"
echo "║                                                                ║"

if [ "$CONFIGURE_ALL" = true ]; then
    echo "║    Claude CLI:  cd $TARGET_DIR && claude"
    echo "║    Codex CLI:   cd $TARGET_DIR && codex"
    echo "║    Cline:       Open folder in VS Code with Cline extension   ║"
else
    case $CLI_TOOL in
        claude)
            echo "║    cd $TARGET_DIR"
            echo "║    claude                                                      ║"
            ;;
        codex)
            echo "║    cd $TARGET_DIR"
            echo "║    codex                                                       ║"
            ;;
        cline)
            echo "║    Open $TARGET_DIR in VS Code"
            echo "║    Use Cline extension to start chat                          ║"
            ;;
    esac
fi

echo "║                                                                ║"
echo "║  Then ask:                                                     ║"
echo "║    \"Run a full code audit using the viewpoints framework\"     ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"

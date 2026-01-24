#!/bin/bash
# Run AI Code Audit Agent on a target project
#
# Usage: ./scripts/run-audit.sh /path/to/target/project [options]
#
# This is a thin wrapper around the setup-audit Rust CLI.
# It builds the MCP servers if needed, then calls setup-audit.
#
# Options are passed directly to setup-audit:
#   --cli <tool>    CLI tool to configure: claude, codex, cline (default: claude)
#   --all           Configure for all supported CLI tools
#   --clean         Remove existing audit configurations before setup
#   --skip-check    Skip checking for built MCP servers
#
# Examples:
#   ./scripts/run-audit.sh /path/to/project                    # Default (Claude)
#   ./scripts/run-audit.sh /path/to/project --cli codex        # Codex CLI
#   ./scripts/run-audit.sh /path/to/project --cli cline        # Cline
#   ./scripts/run-audit.sh /path/to/project --all              # All tools

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AGENT_DIR="$(dirname "$SCRIPT_DIR")"
SETUP_CLI="$AGENT_DIR/target/release/setup-audit"

# Check if we need to build
NEED_BUILD=false

# Check for setup-audit binary
if [ ! -f "$SETUP_CLI" ]; then
    NEED_BUILD=true
fi

# Check for MCP server binaries
SERVERS=(
    "mental-model-server"
    "methodology-kb-server"
    "sarif-tools-server"
    "codegraph-server"
)

for server in "${SERVERS[@]}"; do
    if [ ! -f "$AGENT_DIR/target/release/$server" ]; then
        NEED_BUILD=true
        break
    fi
done

# Build if needed
if [ "$NEED_BUILD" = true ]; then
    echo "Building AI Code Audit Agent..."
    cd "$AGENT_DIR" && cargo build --release
    echo ""
fi

# Run setup-audit with all arguments
exec "$SETUP_CLI" "$@"

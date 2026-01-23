# ADR-0002: MCP Protocol for Tool Interface

**Status:** Accepted
**Date:** 2026-01-23
**Deciders:** Project maintainers

## Context

The AI Code Audit Agent needs to expose its functionality (mental model management, code analysis, knowledge base queries) to AI assistants like Claude. The interface must:

- Be discoverable (tools can be listed with schemas)
- Support structured input/output
- Work with existing AI assistant infrastructure
- Allow independent server deployment

Several options exist for exposing tools to AI systems, including REST APIs, gRPC, custom protocols, and the Model Context Protocol (MCP).

## Decision

Use the **Model Context Protocol (MCP)** via the `rmcp 0.3` crate with stdio transport for all server-client communication.

Each functional domain is implemented as a separate MCP server:
- `mental-model-server` - Mental model and artifact management
- `methodology-kb-server` - Metrics and standards knowledge base
- `sarif-tools-server` - Code analysis tool execution
- `codegraph-server` - Semantic code intelligence

## Consequences

### Positive

- **Native Claude CLI integration** - Works out-of-the-box with Claude Code
- **Structured tool definitions** - JSON Schema for all inputs/outputs
- **Discoverable API** - Clients can list tools and understand capabilities
- **Protocol standardization** - Interoperable with any MCP-compatible client
- **Stdio simplicity** - No network configuration, process-based isolation

### Negative

- **Protocol overhead** - JSON serialization vs direct function calls
- **rmcp dependency** - Tied to library's evolution and API changes
- **Debugging complexity** - Stdio transport harder to inspect than HTTP
- **Limited ecosystem** - Fewer tools compared to REST/gRPC

### Neutral

- **Process isolation** - Each server runs independently (good for reliability, adds overhead)
- **Stateful servers** - Servers maintain state between calls (mental model persists)

## Alternatives Considered

| Alternative | Pros | Cons | Why Not |
|-------------|------|------|---------|
| REST API | Familiar, easy debugging | No native Claude integration, manual schema | Would require custom Claude integration |
| gRPC | Efficient binary protocol | Complex setup, no native MCP support | Overkill for this use case |
| Direct library | No serialization overhead | No process isolation, single language | Can't use multiple servers |
| Custom protocol | Full control | Maintenance burden, no ecosystem | Unnecessary complexity |

## Related

- [MCP Specification](https://modelcontextprotocol.io/)
- [ADR-0001](0001-rust-language.md) - Rust language choice
- `.mcp.json` - Server configuration

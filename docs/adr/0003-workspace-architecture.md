# ADR-0003: Workspace-Based Modular Architecture

**Status:** Accepted
**Date:** 2026-01-23
**Deciders:** Project maintainers

## Context

The AI Code Audit Agent encompasses four distinct functional domains:

1. **Mental Model** - Accumulating understanding of a codebase during audit
2. **Methodology KB** - Reference data for metrics, thresholds, and standards
3. **SARIF Tools** - Execution and processing of code analysis tools
4. **Codegraph** - Semantic code intelligence via SCIP indexes

These domains have different responsibilities, change at different rates, and could potentially be deployed independently. The architecture must balance modularity with development convenience.

## Decision

Organize the project as a **Cargo workspace** with four independent crates:

```
alpha-zero-review-/
├── Cargo.toml              # Workspace definition
├── mental-model-server/    # Crate 1
├── methodology-kb-server/  # Crate 2
├── sarif-tools-server/     # Crate 3
└── codegraph-server/       # Crate 4
```

Each crate:
- Has its own `Cargo.toml` with specific dependencies
- Produces an independent binary
- Has no compile-time dependencies on other crates
- Shares workspace-level dependency versions

## Consequences

### Positive

- **Clear domain boundaries** - Each crate owns a single bounded context
- **Independent testing** - `cargo test -p <crate>` tests only one domain
- **Parallel development** - Teams can work on different crates simultaneously
- **Selective deployment** - Deploy only the servers needed for a use case
- **Faster incremental builds** - Changes to one crate don't rebuild others
- **Zero coupling** - No accidental dependencies between domains

### Negative

- **Cross-crate changes require coordination** - Interface changes affect multiple crates
- **No shared code** - Common utilities must be duplicated or extracted to a new crate
- **Multiple binaries to manage** - Four processes instead of one
- **Workspace configuration complexity** - Dependency versions managed at two levels

### Neutral

- **Consistent patterns** - Same structure repeated in each crate (server.rs, lib.rs, types)
- **Independent versioning possible** - Each crate could have its own version (not currently used)

## Alternatives Considered

| Alternative | Pros | Cons | Why Not |
|-------------|------|------|---------|
| Monolithic crate | Simpler, shared code | Tight coupling, slower builds | Domains are distinct enough to separate |
| Shared core crate | Code reuse | Coupling through shared types | Minimal shared code currently needed |
| Microservices | Full independence | Network complexity, deployment overhead | Overkill for local tool usage |

## Related

- [ADR-0002](0002-mcp-protocol.md) - MCP protocol enables independent servers
- Cargo.toml workspace definition
- `.mcp.json` - Configures all four servers

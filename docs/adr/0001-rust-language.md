# ADR-0001: Rust as Primary Language

**Status:** Accepted
**Date:** 2026-01-23
**Deciders:** Project maintainers

## Context

The AI Code Audit Agent needs to implement four MCP servers that:
- Handle concurrent requests efficiently
- Process potentially large codebases and SARIF files
- Integrate with Claude CLI and other MCP clients
- Maintain long-running processes with minimal resource usage

The choice of implementation language significantly impacts performance, safety, maintainability, and the developer experience.

## Decision

Use **Rust** (Edition 2021, version 1.75+) as the sole implementation language for all MCP servers and supporting libraries.

## Consequences

### Positive

- **Memory safety without garbage collection** - No GC pauses during analysis, predictable performance
- **Strong type system** - Catches errors at compile time, reduces runtime bugs
- **Excellent async support** - Tokio runtime handles concurrent MCP requests efficiently
- **Zero-cost abstractions** - High-level patterns without runtime overhead
- **Cargo ecosystem** - Excellent dependency management and build tooling
- **Cross-platform** - Single codebase runs on Linux, macOS, Windows

### Negative

- **Steeper learning curve** - Borrow checker and lifetimes require learning
- **Longer compile times** - Full rebuild takes 30-60 seconds
- **Smaller talent pool** - Fewer developers compared to Python/JavaScript
- **Verbose error handling** - Result types require explicit handling everywhere

### Neutral

- **Binary distribution** - No runtime dependencies, but larger binary sizes
- **Ecosystem maturity** - Excellent for systems programming, fewer high-level libraries

## Alternatives Considered

| Alternative | Pros | Cons | Why Not |
|-------------|------|------|---------|
| Python | Large ecosystem, fast development | GC pauses, type safety, packaging | Performance concerns for large codebases |
| Go | Simple, fast compile, good async | Less expressive types, GC | Weaker type system for domain modeling |
| TypeScript | MCP SDK available, familiar | Node.js overhead, type erasure | Runtime type safety concerns |

## Related

- [rmcp crate](https://crates.io/crates/rmcp) - Rust MCP implementation
- [Tokio runtime](https://tokio.rs/) - Async runtime choice
- Cargo.toml workspace configuration

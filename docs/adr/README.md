# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the AI Code Audit Agent project.

## What is an ADR?

An ADR is a document that captures an important architectural decision made along with its context and consequences. ADRs help:

- Document the "why" behind decisions
- Provide context for future maintainers
- Prevent accidental reversal of deliberate choices
- Onboard new team members faster

## ADR Index

| ID | Title | Status | Date |
|----|-------|--------|------|
| [0001](0001-rust-language.md) | Rust as Primary Language | Accepted | 2026-01-23 |
| [0002](0002-mcp-protocol.md) | MCP Protocol for Tool Interface | Accepted | 2026-01-23 |
| [0003](0003-workspace-architecture.md) | Workspace-Based Modular Architecture | Accepted | 2026-01-23 |
| [0004](0004-sarif-output.md) | SARIF as Standard Output Format | Accepted | 2026-01-23 |
| [0005](0005-batch-operations.md) | Batch Operations for Mental Model Server | Accepted | 2026-01-24 |

## Creating New ADRs

1. Copy `template.md` to a new file: `NNNN-short-title.md`
2. Fill in all sections
3. Update this README index
4. Submit for review

## ADR Lifecycle

- **Proposed**: Under discussion
- **Accepted**: Decision made and implemented
- **Deprecated**: No longer applies (superseded or context changed)
- **Superseded**: Replaced by a newer ADR

## References

- [ADR GitHub Organization](https://adr.github.io/)
- [Michael Nygard's ADR Article](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)

# Architecture Review - Recommendations Implementation Plan

**Based on:** Architecture Overview & System Design (Commit `0eb9fd8`)
**Created:** 2026-01-23
**Status:** Planning

---

## Overview

This plan addresses the 4 findings from the architecture review self-audit. All findings are low severity but represent opportunities to improve maintainability and developer experience.

---

## Finding 1: Large server.rs Files

**Current State:** Server files are 650-775 LOC
**Recommendation:** Consider splitting into modules as they grow

### Analysis

| File | LOC | Complexity |
|------|-----|------------|
| `mental-model-server/src/server.rs` | 775 | 12 MCP tools |
| `methodology-kb-server/src/server.rs` | 657 | 8 MCP tools |
| `sarif-tools-server/src/server.rs` | 485 | 5 MCP tools |
| `codegraph-server/src/server.rs` | 620 | 9 MCP tools |

### Proposed Module Structure

```
src/
├── server.rs           # Main ServerHandler trait impl (dispatch only)
├── lib.rs              # Public API
├── tools/
│   ├── mod.rs          # Tool trait definition
│   ├── model.rs        # init_model, get_model, update_viewpoint
│   ├── context.rs      # get_context, get_constraints
│   ├── findings.rs     # add_finding, get_findings, synthesize
│   └── artifacts.rs    # store_artifact, get_artifact, get_commit_artifacts
└── types.rs            # Input/output structs
```

### Implementation Steps

1. **Create tools/ module structure**
   - Define `Tool` trait with `name()`, `schema()`, `call()` methods
   - Each tool gets its own handler function

2. **Extract tool implementations**
   - Move each `async fn` tool handler to appropriate module
   - Keep `call_tool` in server.rs as dispatcher

3. **Update server.rs**
   - Import from tools/ modules
   - Simplify to just trait impl + dispatch

### Effort Estimate

| Task | Time |
|------|------|
| mental-model-server refactor | 2-3 hours |
| methodology-kb-server refactor | 1-2 hours |
| sarif-tools-server refactor | 1 hour |
| codegraph-server refactor | 1-2 hours |
| **Total** | **5-8 hours** |

### Priority: Low (Defer)

**Rationale:** Current structure works well. Refactor when files exceed 1000 LOC or when adding significant new functionality.

---

## Finding 2: No Code Coverage Metrics

**Current State:** No coverage reporting in CI
**Recommendation:** Add coverage reporting to CI

### Proposed Solution

Use `cargo-tarpaulin` for Rust code coverage with GitHub Actions integration.

### Implementation Steps

1. **Add coverage workflow** (`.github/workflows/coverage.yml`)

```yaml
name: Code Coverage

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Run coverage
        run: |
          cargo tarpaulin --all --out Xml --out Html \
            --output-dir coverage \
            --skip-clean

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: coverage/cobertura.xml
          fail_ci_if_error: false

      - name: Upload coverage report
        uses: actions/upload-artifact@v4
        with:
          name: coverage-report
          path: coverage/
```

2. **Add coverage badge to README.md**

```markdown
[![codecov](https://codecov.io/gh/ORG/REPO/branch/main/graph/badge.svg)](https://codecov.io/gh/ORG/REPO)
```

3. **Create `.codecov.yml` configuration**

```yaml
coverage:
  status:
    project:
      default:
        target: 70%
        threshold: 5%
    patch:
      default:
        target: 80%
```

### Effort Estimate

| Task | Time |
|------|------|
| Create coverage workflow | 30 min |
| Test locally with tarpaulin | 30 min |
| Configure Codecov | 30 min |
| Update README with badge | 10 min |
| **Total** | **~2 hours** |

### Priority: Medium

**Rationale:** Coverage metrics provide visibility into test quality and help maintain test discipline.

---

## Finding 3: Clippy Running Externally

**Current State:** Clippy SARIF runner implemented but not integrated into workflow
**Recommendation:** Rebuild servers to include new clippy runner

### Current Implementation

The `ClippyRunner` in `sarif-tools-server/src/tools/clippy.rs` can:
- Run `cargo clippy --message-format=json`
- Convert output to SARIF format
- Support workspace and package-level analysis

### Implementation Steps

1. **Verify clippy runner works**
   ```bash
   cargo build --release -p sarif-tools-server
   # Test via MCP tool
   ```

2. **Update audit-artifacts workflow**

Add to `.github/workflows/audit-artifacts.yml`:

```yaml
  clippy:
    needs: detect-language
    runs-on: ubuntu-latest
    if: needs.detect-language.outputs.has_rust == 'true'
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: clippy

      - name: Create artifacts directory
        run: mkdir -p ${{ env.AUDIT_ARTIFACTS_DIR }}/${{ github.sha }}

      - name: Run Clippy with JSON output
        run: |
          cargo clippy --all-targets --message-format=json 2>&1 \
            | tee clippy-output.json || true

      - name: Convert to SARIF (via sarif-tools-server)
        run: |
          # Build sarif-tools-server
          cargo build --release -p sarif-tools-server

          # Use clippy runner to generate SARIF
          # (requires MCP client or direct invocation)
          # For now, use simple JSON-to-SARIF conversion

      - name: Upload Clippy SARIF
        uses: actions/upload-artifact@v4
        with:
          name: clippy-sarif
          path: ${{ env.AUDIT_ARTIFACTS_DIR }}/${{ github.sha }}/clippy.sarif
```

3. **Add clippy to combine-artifacts job**

Update the `needs` array:
```yaml
combine-artifacts:
  needs: [semgrep, bandit, ruff, trivy, scip-index, clippy]
```

### Effort Estimate

| Task | Time |
|------|------|
| Test clippy runner locally | 30 min |
| Update workflow file | 1 hour |
| Test in CI | 30 min |
| **Total** | **~2 hours** |

### Priority: Medium

**Rationale:** Enables automated Rust-specific quality checks aligned with existing SARIF infrastructure.

---

## Finding 4: Missing Explicit ADR Documentation

**Current State:** Architecture decisions exist implicitly in code
**Recommendation:** Create `docs/adr/` with formal ADRs

### Reconstructed ADRs (from Architecture Overview)

The self-audit identified 4 implicit ADRs:
1. ADR-001: Rust as Primary Language
2. ADR-002: MCP Protocol for Tool Interface
3. ADR-003: Workspace-Based Modular Architecture
4. ADR-004: SARIF as Standard Output Format

### Implementation Steps

1. **Create ADR directory structure**
   ```
   docs/adr/
   ├── README.md           # ADR process documentation
   ├── template.md         # ADR template
   ├── 0001-rust-language.md
   ├── 0002-mcp-protocol.md
   ├── 0003-workspace-architecture.md
   └── 0004-sarif-output.md
   ```

2. **Create ADR Template** (`docs/adr/template.md`)

```markdown
# ADR-NNNN: Title

**Status:** Proposed | Accepted | Deprecated | Superseded
**Date:** YYYY-MM-DD
**Deciders:** [list of stakeholders]

## Context

What is the issue that we're seeing that is motivating this decision?

## Decision

What is the change that we're proposing and/or doing?

## Consequences

### Positive
- [benefit 1]

### Negative
- [drawback 1]

### Neutral
- [side effect 1]

## Alternatives Considered

| Alternative | Pros | Cons | Why Not |
|-------------|------|------|---------|
| Option A | ... | ... | ... |

## Related

- Links to related ADRs, issues, or documentation
```

3. **Document existing decisions**

Create formal ADRs for the 4 identified decisions.

### Effort Estimate

| Task | Time |
|------|------|
| Create directory and template | 30 min |
| Write ADR-0001 (Rust) | 30 min |
| Write ADR-0002 (MCP) | 30 min |
| Write ADR-0003 (Workspace) | 30 min |
| Write ADR-0004 (SARIF) | 30 min |
| Update README references | 15 min |
| **Total** | **~3 hours** |

### Priority: Medium

**Rationale:** Explicit ADRs improve onboarding, prevent decision reversal, and document tradeoffs.

---

## Implementation Schedule

| Phase | Finding | Priority | Effort | Dependencies |
|-------|---------|----------|--------|--------------|
| 1 | ADR Documentation (#4) | Medium | 3h | None |
| 2 | Code Coverage (#2) | Medium | 2h | None |
| 3 | Clippy CI Integration (#3) | Medium | 2h | None |
| 4 | Server Refactoring (#1) | Low | 5-8h | Defer |

### Recommended Order

1. **ADR Documentation** - Zero risk, improves project documentation
2. **Code Coverage** - Independent, adds CI visibility
3. **Clippy CI** - Builds on existing clippy runner
4. **Server Refactoring** - Defer until files exceed 1000 LOC

### Total Effort: ~7-15 hours

---

## Success Criteria

| Finding | Success Metric |
|---------|---------------|
| Server Refactoring | Files < 500 LOC each |
| Code Coverage | Coverage badge visible, >70% target |
| Clippy CI | Clippy SARIF in artifact bundle |
| ADR Documentation | 4 ADRs in `docs/adr/` |

---

## Tracking

### Status

| Finding | Status | Started | Completed |
|---------|--------|---------|-----------|
| ADR Documentation | Not Started | - | - |
| Code Coverage | Not Started | - | - |
| Clippy CI Integration | Not Started | - | - |
| Server Refactoring | Deferred | - | - |

---

## Decision Log

| Date | Decision |
|------|----------|
| 2026-01-23 | Plan created based on architecture review findings |
| 2026-01-23 | Server refactoring deferred (low priority, working well) |

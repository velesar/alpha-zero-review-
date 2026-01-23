# AI Code Audit Agent - Improvement Plan

**Based on:** Self-Audit Report (Commit `0133109`)
**Created:** 2026-01-23
**Status:** Active

---

## Overview

This improvement plan addresses findings from the self-audit and prioritizes them by impact and effort. The plan is organized into three phases with estimated effort levels.

---

## Phase 1: Quick Wins (1-2 hours)

### 1.1 Remove Unused Imports

**Priority:** High | **Effort:** Low | **Impact:** Code cleanliness

Remove all unused imports identified by clippy:

| File | Unused Imports |
|------|----------------|
| `codegraph-server/src/server.rs` | `Hotspot`, `Impact`, `ModuleDeps`, `Symbol` |
| `codegraph-server/src/graph.rs` | (verify usage of exports) |
| `methodology-kb-server/src/server.rs` | `AcquisitionStatus`, `MetricData`, `Serialize` |
| `methodology-kb-server/src/acquisition.rs` | `HashMap` |
| `mental-model-server/src/server.rs` | `ArtifactInfo`, `Constraints` |
| `mental-model-server/src/artifacts.rs` | `Path` |

**Action Items:**
- [ ] Remove unused imports from each file
- [ ] Run `cargo clippy` to verify no new warnings
- [ ] Run `cargo test` to ensure nothing broke

### 1.2 Remove/Document Unused Structs

**Priority:** Medium | **Effort:** Low | **Impact:** Code cleanliness

| Item | Location | Action |
|------|----------|--------|
| `ListAcquirableMetricsInput` | methodology-kb-server/src/server.rs | Remove (empty struct, not needed) |

### 1.3 Remove/Document Unused Methods

**Priority:** Medium | **Effort:** Low | **Impact:** Code cleanliness

| Method | Location | Action |
|--------|----------|--------|
| `add_symbol` | codegraph-server/src/graph.rs | Keep with `#[allow(dead_code)]` - future API |
| `add_reference` | codegraph-server/src/graph.rs | Keep with `#[allow(dead_code)]` - future API |
| `tool_names` | sarif-tools-server/src/tools/mod.rs | Keep with `#[allow(dead_code)]` - future API |
| `results_by_rule` | sarif-tools-server/src/sarif.rs | Keep with `#[allow(dead_code)]` - future API |
| `affected_files` | sarif-tools-server/src/sarif.rs | Keep with `#[allow(dead_code)]` - future API |

---

## Phase 2: API Updates (2-4 hours)

### 2.1 Migrate from `rmcp::Error` to `rmcp::ErrorData`

**Priority:** High | **Effort:** Medium | **Impact:** Future compatibility

The `rmcp::Error` type alias is deprecated. Update all usages to `rmcp::ErrorData`.

**Files to Update:**

1. **mental-model-server/src/server.rs** (~46 occurrences)
2. **methodology-kb-server/src/server.rs** (~26 occurrences)
3. **sarif-tools-server/src/server.rs** (~16 occurrences)
4. **codegraph-server/src/server.rs** (~40 occurrences)

**Migration Pattern:**
```rust
// Before
async fn my_tool(&self, input: Parameters<Input>) -> Result<CallToolResult, rmcp::Error> {
    // ...
    .map_err(|e| rmcp::Error::internal_error(format!("Error: {}", e), None))?;
}

// After
async fn my_tool(&self, input: Parameters<Input>) -> Result<CallToolResult, rmcp::ErrorData> {
    // ...
    .map_err(|e| rmcp::ErrorData::internal_error(format!("Error: {}", e), None))?;
}
```

**Action Items:**
- [ ] Update mental-model-server/src/server.rs
- [ ] Update methodology-kb-server/src/server.rs
- [ ] Update sarif-tools-server/src/server.rs
- [ ] Update codegraph-server/src/server.rs
- [ ] Run `cargo build --release` to verify
- [ ] Run `cargo clippy` to verify no deprecation warnings

### 2.2 Simplify Async Functions

**Priority:** Low | **Effort:** Low | **Impact:** Code readability

Four functions can be simplified using `async fn` syntax instead of `impl Future`:

| File | Function |
|------|----------|
| mental-model-server/src/server.rs | `list_tools`, `call_tool` |
| methodology-kb-server/src/server.rs | `list_tools`, `call_tool` |
| sarif-tools-server/src/server.rs | `list_tools`, `call_tool` |
| codegraph-server/src/server.rs | `list_tools`, `call_tool` |

**Note:** These are trait implementations - verify if rmcp trait allows async fn before changing.

---

## Phase 3: Architecture Improvements (4-8 hours)

### 3.1 Add Integration Tests

**Priority:** Medium | **Effort:** High | **Impact:** Reliability

Create integration tests for MCP tool interactions.

**Test Scenarios:**
```
tests/
├── mental_model_integration.rs
│   ├── test_init_and_get_model
│   ├── test_update_viewpoint_recalculates_constraints
│   ├── test_add_finding_with_context_enrichment
│   └── test_artifact_store_roundtrip
├── methodology_kb_integration.rs
│   ├── test_lookup_metric_with_thresholds
│   ├── test_classify_finding_with_adjustments
│   └── test_data_acquisition_cascade
├── sarif_tools_integration.rs
│   ├── test_tool_registry
│   └── test_sarif_merge
└── codegraph_integration.rs
    ├── test_load_and_query_index
    └── test_hotspot_detection
```

### 3.2 Add Error Helper Module

**Priority:** Low | **Effort:** Medium | **Impact:** Code consistency

Create a shared error handling module to reduce duplication:

```rust
// shared/src/errors.rs
pub fn internal_error<E: std::fmt::Display>(msg: &str, e: E) -> rmcp::ErrorData {
    rmcp::ErrorData::internal_error(format!("{}: {}", msg, e), None)
}

pub fn invalid_params<E: std::fmt::Display>(msg: &str, e: E) -> rmcp::ErrorData {
    rmcp::ErrorData::invalid_params(format!("{}: {}", msg, e), None)
}
```

### 3.3 Add Rust-Specific SARIF Generation

**Priority:** Medium | **Effort:** High | **Impact:** Self-audit capability

Extend sarif-tools-server to generate SARIF from cargo clippy output:

```rust
// sarif-tools-server/src/tools/clippy.rs
pub struct ClippyRunner;

impl ToolRunner for ClippyRunner {
    fn name(&self) -> &str { "clippy" }
    fn supported_languages(&self) -> Vec<String> { vec!["rust".into()] }
    // Convert clippy JSON output to SARIF
}
```

---

## Phase 4: Documentation & CI (2-4 hours)

### 4.1 Update CLAUDE.md

Add self-audit instructions:
```markdown
## Self-Audit
To audit this codebase:
1. Run `cargo clippy --all-targets`
2. Check `.audit/artifacts/` for reports
```

### 4.2 Add Pre-commit Hook

Create `.github/hooks/pre-commit`:
```bash
#!/bin/bash
cargo clippy --all-targets -- -D warnings
```

### 4.3 Enhance CI Workflow

Update `.github/workflows/audit-artifacts.yml` to include:
- Clippy SARIF generation for Rust projects
- Automatic artifact storage

---

## Implementation Schedule

| Week | Phase | Tasks |
|------|-------|-------|
| 1 | Phase 1 | Remove unused code, clean imports |
| 1 | Phase 2.1 | Migrate rmcp::Error → ErrorData |
| 2 | Phase 3.1 | Add integration tests |
| 2 | Phase 4 | Documentation updates |
| 3 | Phase 3.2-3.3 | Error helpers, Clippy SARIF |

---

## Success Criteria

- [ ] `cargo clippy --all-targets` produces 0 warnings
- [ ] All tests pass: `cargo test --all`
- [ ] No deprecated API usage
- [ ] Integration tests cover main workflows
- [ ] Self-audit can run without external network

---

## Tracking

### Completed
- [x] Initial self-audit (2026-01-23)
- [x] Phase 1: Quick Wins (2026-01-23)
  - Removed 12 unused imports
  - Removed 1 unused struct
  - Marked 6 future API methods with `#[allow(dead_code)]`
- [x] Phase 2: API Updates (2026-01-23)
  - Migrated ~100 `rmcp::Error` → `rmcp::ErrorData` usages
  - Zero deprecation warnings remaining

### In Progress
- [ ] Phase 3: Architecture Improvements

### Pending
- [ ] Phase 4: Documentation & CI

### Warning Summary
| Category | Before | After |
|----------|--------|-------|
| Deprecation warnings | ~100 | 0 |
| Unused code warnings | 12 | 0 |
| Style suggestions | 8 | 8 |

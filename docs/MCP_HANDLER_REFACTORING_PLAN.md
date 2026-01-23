# MCP Handler Refactoring Plan

## Overview

Refactor MCP server handlers to improve testability by extracting business logic into pure functions, making handlers thin wrappers that only handle input parsing and response formatting.

## Current State

| Server | Handler Coverage | Business Logic Location | Issue |
|--------|------------------|------------------------|-------|
| sarif-tools | 6% (7/113 lines) | Mixed in handlers | Tightly coupled to rmcp |
| mental-model | ~15% | Partially extracted | Some pure functions exist |
| methodology-kb | ~10% | Mixed in handlers | Complex classification logic |
| codegraph | ~20% | Mostly in graph.rs | Already well-structured |

## Refactoring Phases

### Phase 1: Extract Response Formatting (Low Risk)

Create `utils.rs` in each server with common response utilities:

```rust
// utils.rs
pub fn format_json_response<T: Serialize>(data: &T) -> Result<CallToolResult, rmcp::ErrorData> {
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| rmcp::ErrorData::internal_error(format!("Serialization error: {}", e), None))?;
    Ok(CallToolResult::success(vec![Content::text(json)]))
}

pub fn format_text_response(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![Content::text(text.into())])
}
```

**Files to create:**
- [ ] `sarif-tools-server/src/utils.rs`
- [ ] `mental-model-server/src/utils.rs`
- [ ] `methodology-kb-server/src/utils.rs`
- [ ] `codegraph-server/src/utils.rs`

### Phase 2: Define Domain Error Types (Low Risk)

Create `error.rs` with domain-specific error enums and `From<Error> for rmcp::ErrorData`:

```rust
// error.rs (example for sarif-tools)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SarifToolsError {
    #[error("Unknown tool: {0}")]
    UnknownTool(String),
    #[error("Tool not installed: {0}")]
    ToolNotInstalled(String),
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

impl From<SarifToolsError> for rmcp::ErrorData {
    fn from(e: SarifToolsError) -> Self {
        match e {
            SarifToolsError::UnknownTool(_) | SarifToolsError::PathNotFound(_) =>
                rmcp::ErrorData::invalid_params(e.to_string(), None),
            _ => rmcp::ErrorData::internal_error(e.to_string(), None),
        }
    }
}
```

**Files to create:**
- [ ] `sarif-tools-server/src/error.rs`
- [ ] `mental-model-server/src/error.rs`
- [ ] `methodology-kb-server/src/error.rs`
- [ ] `codegraph-server/src/error.rs`

### Phase 3: Extract Business Logic (Medium Risk)

#### 3a. sarif-tools-server

Create `ops.rs` with extracted functions:

| Function | Description | Source Lines |
|----------|-------------|--------------|
| `execute_tool()` | Tool lookup, validation, execution | 122-160 |
| `merge_sarif_files()` | Parse and merge SARIF inputs | 181-205 |
| `enrich_sarif_results()` | Apply rule mappings | 209-269 |

#### 3b. mental-model-server

Create `ops.rs`, move existing pure functions:

| Function | Description | Status |
|----------|-------------|--------|
| `adjust_severity()` | Severity calculation | Already pure |
| `synthesize_by_category()` | Category clustering | Already pure |
| `parse_severity()` | String to Severity | Extract |
| `create_finding()` | Finding construction | Extract |

#### 3c. methodology-kb-server

Create `classification.rs` and `compliance.rs`:

| Function | Description | Target Module |
|----------|-------------|---------------|
| `classify_finding()` | Rule lookup + severity calc | classification.rs |
| `check_compliance()` | Layer checking + violations | compliance.rs |
| `lookup_metric_with_thresholds()` | Threshold merging | classification.rs |

#### 3d. codegraph-server

Minimal changes needed - most logic already in `graph.rs`.

### Phase 4: Refactor Handlers to Thin Wrappers

Transform handlers to:
1. Extract input from `Parameters<T>`
2. Call pure function from `ops.rs`
3. Map errors using `From` impl
4. Format response using `utils.rs`

**Before:**
```rust
#[tool(description = "...")]
async fn run_tool(&self, input: Parameters<RunToolInput>) -> Result<CallToolResult, rmcp::ErrorData> {
    let input = input.0;
    let runner = self.registry.get(&input.tool).ok_or_else(||
        rmcp::ErrorData::invalid_params(...))?;
    if !runner.is_available() { return Err(...); }
    // ... 40 more lines of business logic
    Ok(CallToolResult::success(vec![Content::text(json)]))
}
```

**After:**
```rust
#[tool(description = "...")]
async fn run_tool(&self, input: Parameters<RunToolInput>) -> Result<CallToolResult, rmcp::ErrorData> {
    let output = ops::execute_tool(&self.registry, &input.0.tool, &input.0.path, input.0.config.as_ref())
        .map_err(SarifToolsError::into)?;
    utils::format_json_response(&output)
}
```

### Phase 5: Add Unit Tests

For each extracted function, add tests covering:
- Happy path
- Error cases
- Edge cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_tool_unknown_tool() {
        let registry = ToolRegistry::new();
        let result = execute_tool(&registry, "nonexistent", Path::new("."), None);
        assert!(matches!(result, Err(RunToolError::UnknownTool(_))));
    }

    #[test]
    fn test_merge_sarif_empty() {
        let result = merge_sarif_files(vec![]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().total_results, 0);
    }
}
```

## Expected Outcomes

| Metric | Before | After |
|--------|--------|-------|
| Business logic in handlers | ~500 lines | ~100 lines |
| Testable pure functions | ~10 | ~30 |
| Handler unit test coverage | 6-20% | 80%+ |
| Handler complexity | High | Low (thin wrappers) |

## Implementation Order

1. **Phase 1** - Response formatting (all servers in parallel)
2. **Phase 2** - Error types (all servers in parallel)
3. **Phase 3a** - sarif-tools-server ops extraction
4. **Phase 3b** - mental-model-server ops extraction
5. **Phase 3c** - methodology-kb-server ops extraction
6. **Phase 3d** - codegraph-server (minimal)
7. **Phase 4** - Handler refactoring (per server)
8. **Phase 5** - Unit tests (per extracted function)

## Risk Mitigation

- Run full test suite after each phase
- Keep handlers backward compatible with MCP protocol
- Incremental changes - one server at a time for Phase 3-4
- Integration tests verify end-to-end behavior unchanged

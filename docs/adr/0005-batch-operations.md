# ADR-0005: Batch Operations for Mental Model Server

**Status:** Accepted
**Date:** 2026-01-24
**Deciders:** Architecture Review

## Context

The current mental-model-server MCP interface was designed with single-operation tools that work well for interactive use but create significant overhead during automated audits.

### Problem Analysis

During a typical audit with 50 findings:

| Operation | Current Calls | I/O Operations |
|-----------|---------------|----------------|
| `update_viewpoint` | 16 | 16 full model writes |
| `get_context` | 50 | 0 (read-only) |
| `add_finding` | 50 | 50 full model writes |
| `synthesize` | 1 | 1 full model write |
| **Total** | **117** | **67 disk writes** |

Each `save_model()` call serializes the entire mental model (potentially 60KB YAML) and writes to disk.

### Identified Inefficiencies

1. **No batch operations** - Each finding requires separate `get_context` + `add_finding` calls
2. **Full model retrieval** - `get_model` returns entire YAML (60KB) when often only one section is needed
3. **Per-operation persistence** - Model saved after every mutation, not batched

## Decision

Add three new MCP tools as **non-breaking additions** to the existing interface:

### 1. `add_findings` - Batch Finding Addition

```yaml
add_findings:
  input:
    findings: Vec<{
      viewpoint: string,
      category: string,
      title: string,
      description: string,
      file_path: string,
      line_number: Option<u32>,
      base_severity: string,
      rule_id: Option<string>,
      recommendation: Option<string>
    }>
  output:
    added_count: u32
    findings: Vec<Finding>  # With enriched context and adjusted severity
```

- Enriches all findings with context in a single pass
- Saves model once at the end
- Reduces 50 `add_finding` calls to 1 `add_findings` call

### 2. `get_contexts` - Batch Context Retrieval

```yaml
get_contexts:
  input:
    file_paths: Vec<string>
  output:
    contexts: HashMap<string, FindingContext>
```

- Returns context for multiple files in one call
- Useful when pre-computing context before adding findings
- Reduces N `get_context` calls to 1 `get_contexts` call

### 3. `get_model_section` - Selective Model Retrieval

```yaml
get_model_section:
  input:
    section: "project" | "tech_stack" | "structure" | "build_deploy" |
             "module_hierarchy" | "architecture" | "domain_model" |
             "entity_model" | "interface_surface" | "hotspots" |
             "constraints" | "findings" | "root_causes" | "completed_viewpoints"
  output:
    data: JSON  # Only the requested section
```

- Returns only the requested section as JSON
- Reduces context window usage for LLM
- Enables targeted queries without loading full model

### Backward Compatibility

All existing tools remain unchanged:
- `init_model`, `get_model`, `update_viewpoint` - unchanged
- `get_context`, `add_finding` - unchanged (still useful for single operations)
- `get_constraints`, `get_findings`, `synthesize` - unchanged
- `get_completed_viewpoints` - unchanged
- Artifact tools - unchanged

## Consequences

### Positive

- **~95% reduction in tool calls** for finding-heavy audits (50 findings: 117 → ~20 calls)
- **~90% reduction in I/O** operations (67 writes → ~7 writes)
- **Smaller LLM context** when using `get_model_section` instead of `get_model`
- **Non-breaking** - existing integrations continue to work
- **Incremental adoption** - can use new tools as needed

### Negative

- **Increased API surface** - 3 more tools to document and maintain
- **Slight code duplication** - batch tools share logic with single-operation tools
- **Learning curve** - users need to know when to use batch vs single operations

### Neutral

- Memory usage unchanged (model still fully loaded in memory)
- File format unchanged (still YAML)
- Existing tests remain valid

## Alternatives Considered

| Alternative | Pros | Cons | Why Not |
|-------------|------|------|---------|
| Replace single ops with batch only | Simpler API | Breaking change | Existing integrations would break |
| Dirty flag + explicit flush | Fewer writes | Complex state management | Risk of data loss on crash |
| Streaming responses | Lower memory | Complex implementation | MCP doesn't support streaming well |
| Separate cache layer | Fast reads | Added complexity | Over-engineering for current scale |

## Implementation Notes

### Recommended Usage Pattern

```
# Before (117 calls for 50 findings)
for finding in raw_findings:
    context = get_context(finding.file_path)
    add_finding(finding + context)

# After (2 calls for 50 findings)
contexts = get_contexts([f.file_path for f in raw_findings])
add_findings(enrich(raw_findings, contexts))
```

### When to Use Each Tool

| Scenario | Tool |
|----------|------|
| Single finding from manual review | `add_finding` |
| Bulk import from SARIF | `add_findings` |
| Check context for one file | `get_context` |
| Pre-compute context for batch | `get_contexts` |
| Display full audit state | `get_model` |
| Query specific section | `get_model_section` |

## Related

- [ADR-0002: MCP Protocol for Tool Interface](0002-mcp-protocol.md)
- [BETA_ZERO_TEST_PLAN.md](../BETA_ZERO_TEST_PLAN.md) - Documents original architecture
- mental-model-server/src/server.rs - Implementation location

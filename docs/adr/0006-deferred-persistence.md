# ADR-0006: Deferred Persistence with Dirty Flag

**Status:** Accepted
**Date:** 2026-01-24
**Deciders:** Architecture Review

## Context

Even with batch operations (ADR-0005), the mental-model-server still writes to disk after every mutation. For a typical audit:

| Operation | Mutations | Disk Writes |
|-----------|-----------|-------------|
| `update_viewpoint` x 16 | 16 | 16 |
| `add_findings` x 1 | 1 | 1 |
| `synthesize` x 1 | 1 | 1 |
| **Total** | 18 | 18 |

While much better than the original 67 writes, we can reduce this further by deferring writes to natural phase boundaries.

### Use Case Characteristics

- Single-user audit sessions (one Claude client)
- Bounded duration (typically < 1 hour)
- Acceptable crash recovery: re-run current viewpoint
- Not a database - audit artifact, not source of truth

## Decision

Implement **deferred persistence** using a dirty flag with automatic flush at phase boundaries:

### 1. Dirty Flag Tracking

```rust
pub struct MentalModelServer {
    model: Arc<RwLock<MentalModel>>,
    dirty: Arc<AtomicBool>,  // NEW: tracks unsaved changes
    // ...
}
```

### 2. Phase Boundary Auto-Flush

Mutations that mark natural checkpoints automatically flush:

| Method | Behavior | Rationale |
|--------|----------|-----------|
| `init_model` | Immediate flush | New audit starts clean |
| `update_viewpoint` | Mark dirty + flush | Phase boundary |
| `add_finding(s)` | Mark dirty only | Batched within phase |
| `synthesize` | Mark dirty + flush | End of audit |
| `store_artifact` | Immediate flush | External artifact |

### 3. Explicit Flush Tool

New MCP tool for manual control:

```yaml
flush:
  description: "Persist pending changes to disk. Called automatically at
                phase boundaries (update_viewpoint, synthesize), but can
                be called explicitly for additional safety."
  output:
    flushed: boolean  # true if there were pending changes
    message: string
```

### 4. Flush on Server Shutdown

Implement `Drop` or graceful shutdown to flush pending changes.

## Implementation

### Core Methods

```rust
impl MentalModelServer {
    /// Mark model as having unsaved changes
    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Flush if dirty, return whether flush occurred
    fn flush_if_dirty(&self) -> Result<bool> {
        if self.dirty.swap(false, Ordering::SeqCst) {
            self.save_model()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Force flush regardless of dirty state
    fn force_flush(&self) -> Result<()> {
        self.dirty.store(false, Ordering::SeqCst);
        self.save_model()
    }
}
```

### Method Changes

```rust
// Before: always saves
async fn add_findings(&self, ...) {
    // ... add findings ...
    self.save_model()?;  // REMOVED
}

// After: marks dirty, defers save
async fn add_findings(&self, ...) {
    // ... add findings ...
    self.mark_dirty();  // NEW
    // No save - will flush at next update_viewpoint
}

// Phase boundary - auto flush
async fn update_viewpoint(&self, ...) {
    // ... apply viewpoint ...
    self.mark_dirty();
    self.flush_if_dirty()?;  // Always flushes at phase boundary
}
```

## Consequences

### Positive

- **~75% reduction in disk writes** (18 → ~5 for typical audit)
- **Faster audit execution** - less I/O blocking
- **Natural checkpoints** - data saved at meaningful boundaries
- **Explicit control** - `flush` tool for manual persistence
- **Backward compatible** - existing workflows unchanged

### Negative

- **Potential data loss** - crash between phases loses that phase's findings
- **Slightly more complex** - dirty flag state to manage
- **Testing complexity** - need to verify flush behavior

### Neutral

- Memory usage unchanged
- File format unchanged
- API unchanged (new tool is additive)

## Risk Mitigation

### Data Loss Risk

**Scenario**: Claude crashes after `add_findings` but before `update_viewpoint`

**Impact**: Lose findings from current viewpoint only (not entire audit)

**Mitigation**:
1. Phase boundaries are natural checkpoints - losing one viewpoint is acceptable
2. Explicit `flush` tool available if client wants extra safety
3. Could add optional timer-based flush in future (not implemented now)

### Crash Recovery

If server crashes with dirty data:
1. Model file contains last flushed state
2. Client can check `get_completed_viewpoints` to see progress
3. Resume from last completed viewpoint

## Alternatives Considered

| Alternative | Pros | Cons | Why Not |
|-------------|------|------|---------|
| Timer-based flush (30s) | More safety | Complexity, background task | Over-engineering for use case |
| Write-ahead log | Very safe | Complex, overkill | Not a database |
| Explicit flush only | Maximum control | High data loss risk | Too risky for users who forget |
| Flush on every read | Simple | Still many writes | Doesn't solve problem |

## Expected Results

```
Typical 50-Finding Audit:

Before (ADR-0005):               After (ADR-0006):
─────────────────────────────    ─────────────────────────────
update_viewpoint × 16 → 16       update_viewpoint × 16 → 16 (flush)
add_findings × 1 → 1             add_findings × 1 → 0 (dirty only)
synthesize × 1 → 1               synthesize × 1 → 1 (flush)
─────────────────────────────    ─────────────────────────────
Total: 18 writes                 Total: 17 writes
```

Note: The main benefit is when multiple `add_finding` calls are made within a phase (not using batch). In that case:

```
Non-batch scenario (50 individual add_finding calls):

Before:                          After:
add_finding × 50 → 50 writes     add_finding × 50 → 0 writes (all deferred)
update_viewpoint → 1 write       update_viewpoint → 1 write (flushes all)
─────────────────────────────    ─────────────────────────────
Total: 51 writes                 Total: 1 write
```

## Related

- [ADR-0005: Batch Operations](0005-batch-operations.md) - Reduces tool calls
- This ADR complements ADR-0005 by reducing I/O for non-batch usage

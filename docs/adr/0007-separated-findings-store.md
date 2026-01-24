# ADR-0007: Separated Findings Store with SQLite Backend

**Status:** Accepted
**Date:** 2026-01-24
**Deciders:** Architecture Review

## Context

The mental model currently stores findings inline as `Vec<Finding>`. Even with deferred persistence (ADR-0006), the entire model including all findings must be serialized on every flush. For audits with 50+ findings, this creates unnecessary overhead.

### Current Architecture

```
mental_model.yaml (monolithic)
├── project, tech_stack, structure...  (~15KB, stable)
└── findings: Vec<Finding>             (~40KB+, grows continuously)
```

### Problems

1. **Serialization overhead**: Entire model serialized even when only findings change
2. **No query capability**: Must load all findings to filter by file_path or severity
3. **Memory pressure**: Large finding sets held entirely in memory
4. **Export complexity**: No standard way to export findings separately

## Decision

Separate findings into a dedicated SQLite database with:

1. **Binary storage**: SQLite for efficient storage and queries
2. **JSON export**: Ability to export findings to JSON file
3. **Query support**: Index-based filtering by file_path, severity, viewpoint
4. **Unchanged MCP interface**: All existing tools work transparently

### Design Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Backward compatibility? | No | Clean break, simpler implementation |
| Error handling? | Reflect in response | Clear feedback to client |
| Query needs? | Yes (file_path, severity) | Enable filtering and reporting |
| Size threshold? | 50 findings | Below this, overhead not justified |

## Architecture

### File Layout

```
.audit/
├── mental_model.yaml      (~15KB - structure only)
│   ├── project
│   ├── tech_stack
│   ├── structure
│   ├── architecture
│   ├── domain_model
│   ├── hotspots
│   ├── constraints
│   └── root_causes
│
├── findings.db            (SQLite - findings only)
│   ├── findings table
│   └── indexes
│
└── artifacts/
    └── <commit>/
```

### SQLite Schema

```sql
-- Core findings table
CREATE TABLE findings (
    id TEXT PRIMARY KEY,
    viewpoint TEXT NOT NULL,
    category TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    file_path TEXT NOT NULL,
    line_number INTEGER,
    base_severity TEXT NOT NULL,
    adjusted_severity TEXT NOT NULL,
    rule_id TEXT,
    recommendation TEXT,

    -- Context stored as JSON for flexibility
    context_json TEXT,

    -- Metadata
    created_at TEXT DEFAULT (datetime('now')),

    -- Constraints
    CHECK (base_severity IN ('CRITICAL', 'HIGH', 'MEDIUM', 'LOW', 'INFO')),
    CHECK (adjusted_severity IN ('CRITICAL', 'HIGH', 'MEDIUM', 'LOW', 'INFO'))
);

-- Indexes for common queries
CREATE INDEX idx_findings_viewpoint ON findings(viewpoint);
CREATE INDEX idx_findings_file_path ON findings(file_path);
CREATE INDEX idx_findings_severity ON findings(adjusted_severity);
CREATE INDEX idx_findings_category ON findings(category);

-- Metadata table for store info
CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT
);

-- Insert version info
INSERT INTO metadata (key, value) VALUES ('schema_version', '1');
INSERT INTO metadata (key, value) VALUES ('created_at', datetime('now'));
```

### Rust Implementation

```rust
use rusqlite::{Connection, params};

pub struct FindingsStore {
    conn: Connection,
    dirty: bool,
}

impl FindingsStore {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(Self { conn, dirty: false })
    }

    /// Add a single finding - O(1) insert
    pub fn add(&mut self, finding: &Finding) -> Result<()> {
        self.conn.execute(
            "INSERT INTO findings (id, viewpoint, category, title, description,
             file_path, line_number, base_severity, adjusted_severity,
             rule_id, recommendation, context_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                finding.id,
                finding.viewpoint,
                finding.category,
                finding.title,
                finding.description,
                finding.file_path,
                finding.line_number,
                finding.base_severity.to_string(),
                finding.adjusted_severity.to_string(),
                finding.rule_id,
                finding.recommendation,
                serde_json::to_string(&finding.context)?,
            ],
        )?;
        self.dirty = true;
        Ok(())
    }

    /// Add multiple findings in a transaction - O(n) but single commit
    pub fn add_batch(&mut self, findings: &[Finding]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut count = 0;

        for finding in findings {
            tx.execute(/* same INSERT */, /* params */)?;
            count += 1;
        }

        tx.commit()?;
        self.dirty = true;
        Ok(count)
    }

    /// Get all findings
    pub fn get_all(&self) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM findings ORDER BY created_at"
        )?;
        self.query_to_findings(&mut stmt, [])
    }

    /// Query findings by file path (indexed)
    pub fn get_by_file(&self, file_path: &str) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM findings WHERE file_path = ?1"
        )?;
        self.query_to_findings(&mut stmt, [file_path])
    }

    /// Query findings by severity (indexed)
    pub fn get_by_severity(&self, severity: &Severity) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM findings WHERE adjusted_severity = ?1"
        )?;
        self.query_to_findings(&mut stmt, [severity.to_string()])
    }

    /// Query findings by viewpoint (indexed)
    pub fn get_by_viewpoint(&self, viewpoint: &str) -> Result<Vec<Finding>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM findings WHERE viewpoint = ?1"
        )?;
        self.query_to_findings(&mut stmt, [viewpoint])
    }

    /// Get finding counts by category
    pub fn get_counts_by_category(&self) -> Result<HashMap<String, usize>> {
        let mut stmt = self.conn.prepare(
            "SELECT category, COUNT(*) FROM findings GROUP BY category"
        )?;
        // ...
    }

    /// Get finding counts by severity
    pub fn get_counts_by_severity(&self) -> Result<HashMap<Severity, usize>> {
        let mut stmt = self.conn.prepare(
            "SELECT adjusted_severity, COUNT(*) FROM findings GROUP BY adjusted_severity"
        )?;
        // ...
    }

    /// Export all findings to JSON file
    pub fn export_json(&self, output_path: &Path) -> Result<()> {
        let findings = self.get_all()?;
        let json = serde_json::to_string_pretty(&findings)?;
        fs::write(output_path, json)?;
        Ok(())
    }

    /// Count total findings
    pub fn count(&self) -> Result<usize> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM findings",
            [],
            |row| row.get(0)
        )
    }
}
```

### Updated MentalModel Structure

```rust
// Before
pub struct MentalModel {
    // ... structure fields ...
    pub findings: Vec<Finding>,      // REMOVED
    pub root_causes: Vec<RootCause>,
}

// After
pub struct MentalModel {
    // ... structure fields ...
    // findings moved to FindingsStore
    pub root_causes: Vec<RootCause>, // Stays (small, written once at synthesis)
}

pub struct MentalModelServer {
    model_path: PathBuf,
    model: Arc<RwLock<MentalModel>>,
    findings: Arc<RwLock<FindingsStore>>,  // NEW
    artifact_store: Arc<ArtifactStore>,
    dirty: Arc<AtomicBool>,
    tool_router: ToolRouter<Self>,
}
```

### New MCP Tools

```yaml
# Existing tools - interface unchanged
add_finding:      # → findings.add()
add_findings:     # → findings.add_batch()
get_findings:     # → findings.get_all()

# New query tools
get_findings_by_file:
  input:
    file_path: string
  output:
    findings: Vec<Finding>

get_findings_by_severity:
  input:
    severity: "CRITICAL" | "HIGH" | "MEDIUM" | "LOW" | "INFO"
  output:
    findings: Vec<Finding>

get_findings_by_viewpoint:
  input:
    viewpoint: string
  output:
    findings: Vec<Finding>

get_findings_summary:
  output:
    total: number
    by_severity: { CRITICAL: n, HIGH: n, ... }
    by_category: { security: n, performance: n, ... }
    by_viewpoint: { "VP-Q01": n, "VP-Q02": n, ... }

export_findings:
  input:
    format: "json"  # Future: "csv", "sarif"
    output_path: string
  output:
    exported_count: number
    path: string
```

## Consequences

### Positive

- **~90% reduction in model serialization size** (60KB → 15KB)
- **O(1) finding inserts** instead of O(n) array append + serialize
- **Query capabilities** without loading all findings
- **Standard export** to JSON for external tools
- **Better memory usage** for large audits
- **Foundation for pagination** if needed

### Negative

- **New dependency**: rusqlite (~500KB binary size increase)
- **Two storage locations**: Model YAML + findings DB
- **Migration required**: Existing audits need migration or restart
- **Complexity**: Two stores to manage instead of one

### Neutral

- MCP interface largely unchanged (new query tools are additive)
- Synthesis still works (reads all findings, writes root_causes to model)
- Export replaces manual findings extraction

## Performance Comparison

| Operation | Current (Vec in YAML) | Proposed (SQLite) |
|-----------|----------------------|-------------------|
| Add 1 finding | O(n) serialize | O(1) INSERT |
| Add 50 findings | O(n) × 50 | O(50) single tx |
| Get all findings | O(n) deserialize | O(n) SELECT |
| Get by file_path | O(n) scan | O(log n) index |
| Get by severity | O(n) scan | O(log n) index |
| Flush model | 60KB write | 15KB write |
| Flush findings | (included above) | (already persisted) |

## Implementation Plan

### Phase 1: Core Store
1. Add rusqlite dependency
2. Implement FindingsStore with schema
3. Add unit tests for store operations

### Phase 2: Server Integration
1. Update MentalModelServer to use FindingsStore
2. Remove findings from MentalModel struct
3. Update existing MCP tools to use store

### Phase 3: New Tools
1. Add query tools (get_findings_by_*)
2. Add summary tool (get_findings_summary)
3. Add export tool (export_findings)

### Phase 4: Documentation
1. Update CLAUDE.md with new tools
2. Update skill files
3. Update OPERATIONS_GUIDE.md

## Alternatives Considered

| Alternative | Pros | Cons | Why Not |
|-------------|------|------|---------|
| JSONL append-only | Simple, human-readable | No queries, no random access | Need query capability |
| Keep in YAML | No changes | Serialization overhead | Current problem |
| PostgreSQL | Full RDBMS | External dependency, overkill | Too heavy |
| RocksDB | Fast KV store | No SQL queries | Need relational queries |

## Dependencies

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
```

The `bundled` feature includes SQLite, no external dependency needed.

## Related

- [ADR-0005: Batch Operations](0005-batch-operations.md) - Batch tools use store
- [ADR-0006: Deferred Persistence](0006-deferred-persistence.md) - Complementary optimization

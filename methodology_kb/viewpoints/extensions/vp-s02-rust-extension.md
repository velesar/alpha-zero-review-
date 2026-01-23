---
name: vp-s02-rust-extension
version: 1.0
extends: VP-S02
language: rust
dependencies:
  - vp-f01-tech-stack
  - vp-f02-structure
---

# VP-S02 Extension: Rust Layer Architecture Analysis

## Purpose

Specialized layer architecture analysis for Rust projects. Extends standard VP-S02 by leveraging Rust's compile-time guarantees and trait-based architecture patterns.

## Rust-Specific Context

Unlike dynamic languages, Rust provides **compile-time architecture enforcement** through:
- Cargo workspace dependencies (crate can't import what it doesn't declare)
- Trait visibility (pub(crate) limits exposure)
- Module system hierarchy

This means architectural violations in Rust are often **impossible to compile** when properly structured, making the audit focus shift to:
1. Detecting whether proper structure exists
2. Finding violations that compile but violate intent
3. Identifying missing abstractions

## Detection Strategy

### Step 1: Determine Project Structure Type

```bash
# Check if workspace
if [ -f "Cargo.toml" ]; then
  grep -q "\[workspace\]" Cargo.toml && echo "WORKSPACE" || echo "SINGLE_CRATE"
fi
```

**Decision Tree:**
- Workspace → Analyze crate dependencies for layer enforcement
- Single Crate → Analyze module imports for layer compliance

### Step 2: Workspace Analysis (if applicable)

```bash
# List workspace members
grep -A 100 '\[workspace\]' Cargo.toml | grep 'members' | head -5

# For each crate, extract dependencies
for crate_dir in crates/*/; do
  echo "=== $crate_dir ==="
  grep -A 50 '\[dependencies\]' "$crate_dir/Cargo.toml" | grep -v '^\[' | head -20
done
```

**Expected Pattern (Healthy):**
```
domain/       → [thiserror] only
application/  → [domain]
infrastructure/ → [domain, application, sqlx, ...]
server/       → [all crates, framework]
```

**Violation Pattern:**
```
domain/ → [sqlx]           # CRITICAL: Infrastructure in domain
domain/ → [axum]           # CRITICAL: Framework in domain
```

### Step 3: Module Import Analysis

For single-crate projects or within workspace crates:

```bash
# Find all use statements
grep -rn "^use " src/ --include="*.rs"

# Find domain → infrastructure violations
grep -rn "use crate::infrastructure" src/domain/ --include="*.rs"
grep -rn "use crate::outbound" src/domain/ --include="*.rs"

# Find handler business logic indicators
grep -rn "impl.*Service" src/inbound/ --include="*.rs" | grep -v "State"
```

### Step 4: Trait Port Analysis

```bash
# Find trait definitions (potential ports)
grep -rn "pub trait" src/ --include="*.rs"

# Find trait implementations (adapters)
grep -rn "impl.*for.*" src/ --include="*.rs" | grep -v "impl.*<" | head -30

# Find where traits are defined vs implemented
# Healthy: Traits in domain, impls in infrastructure
```

### Step 5: Handler Thickness Check

```bash
# Count lines in handler files
wc -l src/inbound/**/*.rs src/handlers/**/*.rs 2>/dev/null

# Look for business logic indicators in handlers
grep -rn "if.*{" src/inbound/ --include="*.rs" | wc -l
grep -rn "match.*{" src/inbound/ --include="*.rs" | wc -l
grep -rn "for.*in.*{" src/inbound/ --include="*.rs" | wc -l
```

**Threshold:**
- Handler functions > 30 lines → WARNING
- Handler functions > 50 lines → CRITICAL
- Business logic patterns in handlers → HIGH

## Hexagonal Architecture Mapping

### Expected Structure Detection

```yaml
hexagonal_pattern:
  domain_core:
    indicators:
      - "src/domain/" or "crates/domain/"
      - Contains: "mod models", "mod repository", "pub trait"
      - Does NOT contain: framework imports, database drivers
    
  inbound_ports:
    indicators:
      - Trait definitions accepting domain commands
      - Located in domain or separate "ports" module
    
  outbound_ports:
    indicators:
      - Repository traits, external service traits
      - Located in domain (NOT infrastructure)
      - Example: "pub trait UserRepository"
    
  inbound_adapters:
    indicators:
      - HTTP handlers, CLI parsers, gRPC services
      - Located in: "src/inbound/", "src/handlers/", "src/api/"
      - Implements inbound port traits or uses framework patterns
    
  outbound_adapters:
    indicators:
      - Database implementations, API clients
      - Located in: "src/outbound/", "src/infrastructure/", "crates/infrastructure/"
      - Implements: Repository traits from domain
```

## Output Schema

```yaml
rust_layer_architecture:
  structure_type: "workspace" | "single_crate"
  
  # Only for workspaces
  workspace_analysis:
    crates:
      - name: string
        declared_dependencies: list[string]
        layer_role: "domain" | "application" | "infrastructure" | "binary" | "common"
        violations: list[WorkspaceViolation]
    dependency_graph_valid: boolean
  
  # For all projects
  module_analysis:
    detected_layers:
      - name: string
        paths: list[string]
        purpose: string
    import_violations:
      - from_module: string
        imports: string
        severity: "critical" | "high" | "medium"
        reason: string
  
  hexagonal_compliance:
    has_domain_core: boolean
    has_port_traits: boolean
    ports_in_domain: boolean
    adapters_separated: boolean
    domain_purity_score: 0-100  # % of domain files with no external deps
  
  handler_analysis:
    total_handlers: int
    avg_lines: float
    max_lines: int
    business_logic_indicators: int
    thickness_assessment: "thin" | "moderate" | "thick"
  
  pattern_detected: "Hexagonal" | "Clean Architecture" | "Layered" | "Unknown"
  confidence: "high" | "medium" | "low"
  
  violations:
    - type: string
      location: string
      severity: "critical" | "high" | "medium" | "low"
      description: string
      fix_suggestion: string
```

## Severity Adjustment Rules

### Rust-Specific Severity Multipliers

| Finding | Base Severity | Rust Adjustment | Rationale |
|---------|---------------|-----------------|-----------|
| Domain imports infrastructure | HIGH | → CRITICAL (×1.5) | Defeats compile-time guarantees |
| Framework types in domain | MEDIUM | → HIGH (×1.3) | Prevents framework migration |
| Missing trait abstraction | LOW | → MEDIUM (×1.5) | Rust makes this easy, no excuse |
| Fat handlers | MEDIUM | → MEDIUM (×1.0) | Same as other languages |
| No workspace for large project | LOW | → MEDIUM (×1.3) | Misses compile-time enforcement |

### Context Multipliers from Mental Model

```yaml
# Apply after base severity calculation
severity_multipliers:
  core_bounded_context: 1.5
  hotspot_file: 1.4
  domain_layer: 1.3
  low_test_coverage: 1.2
```

## AI Prompt Template

```markdown
Analyze this Rust project for layer architecture compliance:

## Project Context
- Structure: {{structure_type}}
- Primary crates/modules: {{module_list}}
- Framework: {{framework}}

## Collected Data

### Cargo.toml Dependencies (workspace)
```
{{cargo_dependencies}}
```

### Import Statements (sampled)
```
{{import_sample}}
```

### Trait Definitions
```
{{trait_definitions}}
```

### Handler Files (with line counts)
```
{{handler_files}}
```

## Analysis Required

1. **Pattern Detection**: What architectural pattern is this project using?
   - Evidence for Hexagonal/Ports-and-Adapters?
   - Evidence for Clean Architecture?
   - Evidence for simple Layered?

2. **Layer Boundary Analysis**:
   - Are domain/application/infrastructure properly separated?
   - Do dependencies flow in the correct direction?
   - Any violations of layer boundaries?

3. **Port/Adapter Assessment** (Hexagonal focus):
   - Are repository traits defined in domain layer?
   - Are trait implementations in infrastructure?
   - Is the domain free of framework dependencies?

4. **Handler Thickness**:
   - Are handlers thin orchestration layers?
   - Is business logic leaking into handlers?
   - Evidence of framework-specific business logic?

5. **Workspace Utilization** (if applicable):
   - Does the workspace structure enforce architecture?
   - Could violations compile under current structure?
   - Should this project be a workspace (if it isn't)?

## Output

Provide findings in the schema format above. Focus on actionable violations
with specific file/line references where possible.
```

## Quality Indicators

### Healthy Rust Architecture ✅

- Workspace dependencies enforce layer rules
- Domain crate has only `thiserror` (and maybe `serde`)
- All repository traits defined in domain
- Implementations in infrastructure implement domain traits
- Handlers under 30 lines, no conditional business logic
- `cargo test` works without database (mock implementations)

### Warning Signs ⚠️

- Single large crate over 15,000 LOC without workspace
- `Arc<dyn Trait>` everywhere without clear port concept
- Handler files over 200 lines
- Domain module imports `tokio` or `async-trait` excessively
- No trait abstraction for database access

### Critical Issues 🚨

- Domain imports from infrastructure module/crate
- `sqlx` or database driver in domain Cargo.toml
- Framework types (e.g., `axum::Json`) in domain signatures
- Business logic scattered across handlers and services
- Circular dependencies between crates

## Constraints Produced

After VP-S02 Rust analysis:

```yaml
constraints:
  security_focus_paths:
    - "crates/domain/src/**"    # Domain boundaries critical
    - "src/domain/**"
    - "**/handlers/**"          # Input validation points
  
  refactoring_candidates:
    - path: string
      reason: "fat_handler" | "missing_abstraction" | "layer_violation"
      effort: "S" | "M" | "L"
  
  architecture_pattern: "Hexagonal" | "Clean" | "Layered" | "Unknown"
  
  rust_specific:
    workspace_recommended: boolean
    trait_abstraction_gaps: list[string]
    domain_purity_violations: list[string]
```

---

## Examples

### Example 1: Clean Workspace Structure

```
Cargo.toml: [workspace] members = ["crates/*"]

crates/domain/Cargo.toml:
  [dependencies]
  thiserror = "1.0"

crates/infrastructure/Cargo.toml:
  [dependencies]
  domain = { path = "../domain" }
  sqlx = { version = "0.7", features = ["postgres"] }
```

**Assessment:**
- ✅ Domain has no infrastructure dependencies
- ✅ Infrastructure depends on domain
- ✅ Compile-time enforcement active

### Example 2: Problematic Structure

```
crates/domain/Cargo.toml:
  [dependencies]
  thiserror = "1.0"
  sqlx = "0.7"           # VIOLATION!
  
src/domain/user.rs:
  use sqlx::FromRow;     # VIOLATION!
  
  #[derive(FromRow)]
  pub struct User { ... }
```

**Assessment:**
- 🚨 CRITICAL: Database driver in domain
- 🚨 CRITICAL: ORM derive in domain entity
- Domain is coupled to specific database
- Cannot test domain without database

**Recommendation:**
```rust
// domain/user.rs - Pure domain entity
pub struct User {
    pub id: UserId,
    pub email: Email,
    pub name: String,
}

// infrastructure/db/user.rs - DB-specific
use sqlx::FromRow;

#[derive(FromRow)]
struct UserRow {
    id: i64,
    email: String,
    name: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: UserId(row.id),
            email: Email::new(row.email).unwrap(),
            name: row.name,
        }
    }
}
```

---

*Extension for VP-S02 Layer Architecture Viewpoint*
*Part of AI Code Audit Agent Methodology KB*

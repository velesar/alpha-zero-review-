# Rust Server Architecture - KB Intake Package

> **Version:** 1.0 | **Date:** January 2026 | **Language:** Rust

---

## Package Contents

This intake package adds Rust-specific architecture knowledge to the AI Code Audit Agent methodology KB. It consists of four interconnected files:

| File | Location | Purpose |
|------|----------|---------|
| `rust-hexagonal-architecture.yaml` | `standards/` | Machine-readable standards specification |
| `rust-server-architecture.md` | `knowledge/` | Comprehensive human-readable knowledge base |
| `vp-s02-rust-extension.md` | `viewpoints/extensions/` | VP-S02 Layer Architecture specialization |
| `rust-architecture.md` | `glossary/` | Rust-specific terminology |

---

## Integration Points

### 1. Standards Integration

The `rust-hexagonal-architecture.yaml` file follows the same structure as:
- `clean-architecture.yaml`
- `solid-principles.yaml`

It defines:
- Core principles (traits as ports, compile-time enforcement)
- Structure patterns (single crate vs workspace)
- Quality indicators for audit findings
- Anti-pattern detection rules
- Severity adjustments specific to Rust

### 2. Viewpoint Integration

The `vp-s02-rust-extension.md` extends VP-S02 (Layer Architecture) with:
- Rust-specific detection strategies
- Cargo.toml dependency analysis
- Trait/port identification
- Handler thickness evaluation
- AI prompt templates for Rust projects

**Usage:** When auditing a Rust project, load this extension alongside base VP-S02.

### 3. Mental Model Mapping

| Viewpoint | Rust-Specific Additions |
|-----------|------------------------|
| VP-F01 Tech Stack | Rust version, async runtime, workspace detection |
| VP-F02 Structure | Flat workspace pattern, module organization |
| VP-S02 Layers | Trait-as-port analysis, compile-time enforcement |
| VP-S03 Domain | Bounded contexts → crates, aggregates → structs |
| VP-Q01 Security | unwrap() usage, unsafe blocks |
| VP-Q03 Testability | Trait-based mocking, zero-I/O domain tests |

---

## Key Concepts Summary

### Why Hexagonal for Rust

| Hexagonal Concept | Rust Implementation |
|-------------------|---------------------|
| Port | `trait` |
| Adapter | `struct impl Trait` |
| Domain Core | Pure structs/enums |
| Dependency Injection | Generic params or `Arc<dyn Trait>` |
| Compile-time Enforcement | Cargo workspace dependencies |

### Critical Findings (Rust-Specific)

| Finding | Severity | Rationale |
|---------|----------|-----------|
| Domain imports infrastructure | CRITICAL | Defeats compile-time guarantees |
| Framework types in domain | HIGH | Prevents framework migration |
| Business logic in handlers | HIGH | Violates separation of concerns |
| No trait for DB access | MEDIUM | Rust makes this easy |
| Large crate without workspace | LOW→MEDIUM | Misses enforcement opportunity |

### Error Handling Pattern

```
Domain Layer:     thiserror (typed enums for matching)
Application:      anyhow (context attachment, aggregation)
Boundary:         From<DomainError> for HttpError
```

---

## Usage Examples

### During VP-S02 Analysis (Rust Project)

1. Load base VP-S02 SKILL.md
2. Load `vp-s02-rust-extension.md`
3. Detect project structure:
   ```bash
   grep -q "\[workspace\]" Cargo.toml
   ```
4. Apply Rust-specific checks:
   - Workspace dependency analysis
   - Domain purity score calculation
   - Handler thickness assessment

### Severity Calculation

```python
base_severity = finding.severity
rust_adjustment = RUST_MULTIPLIERS.get(finding.type, 1.0)
context_adjustment = get_context_multiplier(mental_model, finding.location)

final_severity = base_severity * rust_adjustment * context_adjustment
```

### AI Prompt Context

When analyzing a Rust project, include:
```markdown
## Rust-Specific Context
- Structure: {{workspace|single_crate}}
- Async Runtime: {{tokio|async-std|etc}}
- Framework: {{axum|actix|etc}}
- Apply standards: rust-hexagonal-architecture.yaml
```

---

## Relationship to Existing Documents

### Extends

- `viewpoints_framework.md` - Adds Rust-specific viewpoint analysis
- `software_quality_metrics_framework.docx` - Adds Rust metrics

### Complements

- `ai_code_audit_agent_vision.docx` - Provides language-specific implementation
- `infrastructure_implementation_vision.md` - Guides MCP server implementation in Rust

### Informs

- MCP server development (using `rmcp` crate)
- Codegraph implementation (using `tree-sitter`)
- Beta-Zero SARIF integration

---

## File Destinations

When integrating into methodology_kb:

```
methodology_kb/
├── glossary/
│   ├── metrics.yaml
│   ├── terms.yaml
│   └── rust-architecture.md          ← NEW
├── knowledge/
│   └── rust-server-architecture.md   ← NEW
├── standards/
│   ├── clean-architecture.yaml
│   ├── solid-principles.yaml
│   └── rust-hexagonal-architecture.yaml  ← NEW
└── viewpoints/
    └── extensions/
        └── vp-s02-rust-extension.md  ← NEW
```

---

## Validation Checklist

- [ ] Standards YAML validates against existing schema
- [ ] Glossary terms don't conflict with existing definitions
- [ ] VP-S02 extension produces compatible output format
- [ ] Severity multipliers align with existing mental model system
- [ ] AI prompts produce consistent output format

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01 | Initial release |

---

*Part of AI Code Audit Agent Methodology Knowledge Base*

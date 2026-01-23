# Angular Application Architecture - KB Intake Package

> **Version:** 1.0 | **Date:** January 2026 | **Framework:** Angular

---

## Package Contents

This intake package adds Angular-specific architecture knowledge to the AI Code Audit Agent methodology KB. It consists of four interconnected files:

| File | Location | Purpose |
|------|----------|---------|
| `angular-clean-architecture.yaml` | `standards/` | Machine-readable standards specification |
| `angular-application-architecture.md` | `knowledge/` | Comprehensive human-readable knowledge base |
| `vp-s02-angular-extension.md` | `viewpoints/extensions/` | VP-S02 Layer Architecture specialization |
| `angular-architecture.md` | `glossary/` | Angular-specific terminology |

---

## Integration Points

### 1. Standards Integration

The `angular-clean-architecture.yaml` file follows the same structure as:
- `clean-architecture.yaml`
- `solid-principles.yaml`

It defines:
- Core principles (Smart/Dumb split, Facade abstraction, Domain purity)
- Structure patterns (Standalone app vs Nx monorepo)
- Component classification rules
- State management patterns
- Severity adjustments specific to Angular

### 2. Viewpoint Integration

The `vp-s02-angular-extension.md` extends VP-S02 (Layer Architecture) with:
- NgModule boundary analysis
- Smart/Dumb component detection
- Domain layer purity checking
- Subscription leak detection
- Nx library boundary validation

**Usage:** When auditing an Angular project, load this extension alongside base VP-S02.

### 3. Mental Model Mapping

| Viewpoint | Angular-Specific Additions |
|-----------|---------------------------|
| VP-F01 Tech Stack | Angular version, CLI, state management library |
| VP-F02 Structure | Nx monorepo detection, module organization |
| VP-S02 Layers | NgModule imports, DI hierarchy, smart/dumb split |
| VP-S03 Domain | Feature modules as bounded contexts |
| VP-Q02 Performance | OnPush coverage, lazy loading, bundle analysis |
| VP-Q03 Testability | TestBed patterns, mock strategies |

---

## Key Concepts Summary

### Why Clean Architecture for Angular

| Clean Architecture Concept | Angular Implementation |
|---------------------------|------------------------|
| Use Cases | Facade Services |
| Entities | Pure TypeScript in `/domain` |
| Interface Adapters | Smart Components, Services |
| Frameworks | Angular itself, HTTP, Storage |
| Dependency Inversion | InjectionToken + useClass |
| Bounded Contexts | Feature Modules (lazy-loaded) |

### Critical Findings (Angular-Specific)

| Finding | Severity | Rationale |
|---------|----------|-----------|
| Feature imports another feature | HIGH | Breaks module isolation |
| Angular imports in domain | HIGH | Framework coupling |
| Subscription without cleanup | HIGH | Memory leak risk |
| Missing OnPush on dumb component | MEDIUM | Performance impact |
| HTTP calls in component | MEDIUM | Architecture violation |
| God component (>300 lines) | HIGH | Maintainability risk |

### Component Architecture Pattern

```
Smart (Container)              Dumb (Presentational)
─────────────────────         ─────────────────────
• Injects facades             • Zero injections
• Handles events              • @Input/@Output only
• Orchestrates data           • OnPush required
• Template: delegates         • Highly reusable
```

---

## Usage Examples

### During VP-S02 Analysis (Angular Project)

1. Load base VP-S02 SKILL.md
2. Load `vp-s02-angular-extension.md`
3. Detect project structure:
   ```bash
   [ -f "nx.json" ] && echo "NX_WORKSPACE" || echo "STANDALONE_APP"
   ```
4. Apply Angular-specific checks:
   - Module import analysis
   - Domain purity verification
   - Smart/Dumb classification
   - Subscription leak detection

### Severity Calculation

```python
base_severity = finding.severity
angular_adjustment = ANGULAR_MULTIPLIERS.get(finding.type, 1.0)
context_adjustment = get_context_multiplier(mental_model, finding.location)

final_severity = base_severity * angular_adjustment * context_adjustment
```

### AI Prompt Context

When analyzing an Angular project, include:
```markdown
## Angular-Specific Context
- Structure: {{standalone_app|nx_workspace}}
- Angular Version: {{version}}
- State Management: {{ngrx|signals|facade}}
- Apply standards: angular-clean-architecture.yaml
```

---

## Relationship to Existing Documents

### Extends

- `viewpoints_framework.md` - Adds Angular-specific viewpoint analysis
- `software_quality_metrics_framework.docx` - Adds Angular metrics

### Complements

- `ai_code_audit_agent_vision.docx` - Provides framework-specific implementation
- `infrastructure_implementation_vision.md` - Frontend audit methodology

### Compares With

- `rust-hexagonal-architecture.yaml` - Backend/server counterpart
- Shows how same architectural principles apply differently per tech stack

---

## Angular Version Considerations

| Version | Key Features | Patterns to Check |
|---------|--------------|-------------------|
| 14-15 | Standalone components, Typed forms | `standalone: true` adoption |
| 16+ | Signals, `takeUntilDestroyed()` | Signal-based state |
| 17+ | `@if`, `@for`, Deferrable views | Control flow migration |

**Version Detection:**
```bash
cat package.json | jq '.dependencies["@angular/core"]'
```

---

## File Destinations

When integrating into methodology_kb:

```
methodology_kb/
├── glossary/
│   ├── metrics.yaml
│   ├── terms.yaml
│   ├── rust-architecture.md
│   └── angular-architecture.md           ← NEW
├── knowledge/
│   ├── rust-server-architecture.md
│   └── angular-application-architecture.md  ← NEW
├── standards/
│   ├── clean-architecture.yaml
│   ├── solid-principles.yaml
│   ├── rust-hexagonal-architecture.yaml
│   └── angular-clean-architecture.yaml   ← NEW
└── viewpoints/
    └── extensions/
        ├── vp-s02-rust-extension.md
        └── vp-s02-angular-extension.md   ← NEW
```

---

## Validation Checklist

- [ ] Standards YAML validates against existing schema
- [ ] Glossary terms don't conflict with existing definitions
- [ ] VP-S02 extension produces compatible output format
- [ ] Severity multipliers align with existing mental model system
- [ ] AI prompts produce consistent output format
- [ ] Angular version considerations documented

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01 | Initial release |

---

*Part of AI Code Audit Agent Methodology Knowledge Base*

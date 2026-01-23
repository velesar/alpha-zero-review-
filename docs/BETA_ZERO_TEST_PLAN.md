# Beta-Zero Test Plan

**AI-Driven Code Quality Practice**

**Version:** 1.0
**Date:** January 2026
**Status:** Planning
**Author:** Light IT Global

---

## Executive Summary

Beta-Zero — еволюція Alpha-Zero в рамках Infrastructure Implementation Vision. Не переписування з нуля, а цільове розширення існуючої системи для закриття виявлених gaps.

**Принцип:** Мінімальні зміни для максимального ефекту.

---

## 1. Контекст: Alpha-Zero Results

### Що працює

| Компонент | Статус | Notes |
|-----------|--------|-------|
| MCP Servers (Rust/rmcp) | ✅ | mental-model-server, methodology-kb-server |
| Viewpoints Framework | ✅ | 16 SKILL.md файлів |
| Mental Model Schema | ✅ | YAML-based |
| Cline Orchestration | ✅ | .clinerules конфігурація |
| Severity Adjustments | ✅ | Context-aware multipliers |
| Fowler Quadrant | ✅ | Tech debt classification |

### Виявлені gaps

| Gap | Impact |
|-----|--------|
| **Methodology KB порожня** | Запити без відповідей, severity adjustments не працюють |
| **Tools не викликаються** | Findings не збираються автоматично |
| **Немає SARIF support** | Кожен tool — свій формат |
| **Немає artifact storage** | Кожен аудит з нуля |
| **Немає commit-indexed cache** | Повторна робота |

---

## 2. Beta-Zero Scope

### Формула

```
Beta-Zero = Alpha-Zero + 2 нових MCP servers + розширення існуючих + KB content
```

### Архітектура

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           BETA-ZERO                                     │
│                                                                         │
│   MCP Servers:                                                          │
│   ├── mental-model-server (Rust) ......... [Alpha-Zero] + extensions   │
│   ├── methodology-kb-server (Rust) ....... [Alpha-Zero] + extensions   │
│   ├── sarif-tools-server (Rust) .......... [NEW]                       │
│   └── codegraph-server (Rust) ............ [NEW]                       │
│                                                                         │
│   Unchanged:                                                            │
│   ├── skills/ (16 viewpoints SKILL.md)                                 │
│   └── Cline orchestration                                              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### MCP Servers: Повна специфікація

```
┌─────────────────────────────────────────────────────────────────────────┐
│  mental-model-server                                                    │
│  [Alpha-Zero: exists] + [Beta-Zero: artifact store extensions]         │
│                                                                         │
│  Existing tools:                                                        │
│  ├── init_model                                                        │
│  ├── get_model                                                         │
│  ├── update_viewpoint                                                  │
│  ├── get_context                                                       │
│  ├── get_constraints                                                   │
│  ├── add_finding                                                       │
│  ├── get_findings                                                      │
│  ├── synthesize                                                        │
│  └── get_completed_viewpoints                                          │
│                                                                         │
│  NEW tools (Artifact Store):                                           │
│  ├── get_commit_artifacts(commit) → available[], missing[]             │
│  ├── store_artifact(commit, type, data)                                │
│  └── get_artifact(commit, type) → data, metadata                       │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  methodology-kb-server                                                  │
│  [Alpha-Zero: exists] + [Beta-Zero: KB content + acquisition]          │
│                                                                         │
│  Existing tools:                                                        │
│  ├── lookup_metric                                                     │
│  ├── classify_finding                                                  │
│  ├── get_thresholds                                                    │
│  ├── check_compliance                                                  │
│  ├── get_template                                                      │
│  ├── list_metrics                                                      │
│  ├── list_standards                                                    │
│  └── get_category                                                      │
│                                                                         │
│  NEW tools (Data Acquisition):                                         │
│  ├── acquire_findings(commit, types[], options) → SARIF, sources[]     │
│  ├── check_manifest(project_path) → manifest                           │
│  └── parse_ci_config(project_path) → artifact_locations[]              │
│                                                                         │
│  REQUIRED: KB Content (glossary/, taxonomies/, thresholds/)            │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  sarif-tools-server [NEW]                                              │
│  Tool execution + SARIF output                                         │
│                                                                         │
│  Tools:                                                                 │
│  ├── run_tool(tool, path, options) → SARIF                             │
│  ├── list_available_tools() → tool[]                                   │
│  ├── merge_sarif(sarif_files[]) → combined SARIF                       │
│  ├── normalize_sarif(sarif, mappings) → enriched SARIF                 │
│  └── get_tool_config(tool) → config, command, schema                   │
│                                                                         │
│  Supported tools:                                                       │
│  ├── semgrep    → semgrep --sarif                                      │
│  ├── bandit     → bandit -f sarif                                      │
│  ├── ruff       → ruff check --output-format sarif                     │
│  ├── eslint     → eslint -f @microsoft/eslint-formatter-sarif          │
│  ├── trivy      → trivy fs --format sarif                              │
│  └── (extensible via config)                                           │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│  codegraph-server [NEW]                                                │
│  SCIP-based semantic code intelligence                                 │
│                                                                         │
│  Tools:                                                                 │
│  ├── load_index(scip_path) → status                                    │
│  ├── get_symbol_info(symbol_id) → symbol                               │
│  ├── get_callers(symbol_id) → references[]                             │
│  ├── get_callees(symbol_id) → references[]                             │
│  ├── get_impact(symbol_id) → affected_files, reference_count           │
│  ├── get_module_deps(module_path) → dependencies                       │
│  ├── get_file_symbols(file_path) → symbols[]                           │
│  ├── find_symbol(pattern) → symbols[]                                  │
│  └── find_hotspot_symbols(min_refs, path_filter) → hotspots[]          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Components Summary

| Component | Type | Status |
|-----------|------|--------|
| mental-model-server | MCP Server (Rust) | Extend |
| methodology-kb-server | MCP Server (Rust) | Extend |
| **sarif-tools-server** | MCP Server (Rust) | **NEW** |
| **codegraph-server** | MCP Server (Rust) | **NEW** |
| methodology_kb/ content | YAML files | Fill |
| skills/ | SKILL.md | Unchanged |
| Cline orchestration | .clinerules | Unchanged |

---

## 3. sarif-tools-server [NEW]

Окремий MCP server для роботи з SARIF та запуску інструментів аналізу.

### Чому окремий server

- **Single Responsibility:** tool execution ≠ methodology knowledge
- **Isolation:** tool crashes не впливають на інші servers
- **Extensibility:** легко додавати нові tools
- **Reusability:** може використовуватись поза audit системою

### ADR-002: SARIF як Unified Format

**SARIF 2.1.0** — єдиний формат для обміну findings.

- Industry standard (OASIS)
- Підтримується: SonarQube (import), GitHub, GitLab, VS Code
- Native export: semgrep, bandit, eslint, trivy, checkov
- Merge tools: `sarif-multitool`

### Tool Matrix

| Tool | Languages | SARIF Export | Command |
|------|-----------|--------------|---------|
| semgrep | Multi | ✅ Native | `semgrep --sarif -o results.sarif` |
| bandit | Python | ✅ Native | `bandit -r src/ -f sarif -o results.sarif` |
| ruff | Python | ✅ Native | `ruff check --output-format sarif` |
| eslint | JS/TS | ✅ Formatter | `eslint -f @microsoft/eslint-formatter-sarif` |
| trivy | Multi | ✅ Native | `trivy fs --format sarif` |

### MCP Tools

```yaml
run_tool:
  input:
    tool: "semgrep" | "bandit" | "ruff" | "eslint" | "trivy"
    path: string
    config: object?          # tool-specific config
  output:
    sarif: SARIF
    exit_code: int
    stderr: string?

list_available_tools:
  output:
    tools: [{name, languages[], installed, version}]

merge_sarif:
  input:
    sarif_files: list[SARIF]
  output:
    combined: SARIF

normalize_sarif:
  input:
    sarif: SARIF
    rule_mappings: string    # path to mappings file
  output:
    enriched: SARIF          # with categories, CWE, severity_base

get_tool_config:
  input:
    tool: string
  output:
    command: string
    args_template: string
    output_format: string
    supported_languages: list[string]
```

### Implementation

```rust
// sarif-tools-server structure
sarif-tools-server/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── server.rs           # MCP server
    ├── runner.rs           # Tool execution
    ├── sarif.rs            # SARIF parsing/merging
    ├── normalize.rs        # Rule mappings
    └── tools/
        ├── mod.rs
        ├── semgrep.rs
        ├── bandit.rs
        ├── ruff.rs
        ├── eslint.rs
        └── trivy.rs
```

**Estimated:** ~600-800 LOC Rust

### SonarQube Note

SonarQube — client-server архітектура. Без сервера не працює.

**Стратегія:**
- Якщо SonarQube API доступний → окремий tool для fetch
- Якщо ні → standalone tools покривають той самий scope
- SARIF як bridge для імпорту в SonarQube

---

## 4. codegraph-server [NEW]

SCIP-based semantic code intelligence.

### Чому SCIP

| Без SCIP | З SCIP |
|----------|--------|
| grep-based пошук | Semantic navigation |
| File-level analysis | Symbol-level precision |
| Приблизні залежності | Точний call graph |
| "Де викликається?" — повільно | Instant lookup |

### SCIP Indexers

| Language | Indexer | Command |
|----------|---------|---------|
| Python | scip-python | `scip-python index --output index.scip .` |
| TypeScript | scip-typescript | `scip-typescript index --output index.scip` |
| Java | scip-java | gradle/maven plugin |
| Rust | rust-analyzer | native SCIP export |

### MCP Tools

```yaml
load_index:
  input:
    scip_path: string
  output:
    status: "loaded" | "error"
    symbols_count: int
    files_count: int

get_symbol_info:
  input:
    symbol_id: string
  output:
    id: string
    kind: "class" | "function" | "method" | "variable"
    file: string
    range: {start_line, end_line}
    documentation: string?

get_callers:
  input:
    symbol_id: string
  output:
    caller_count: int
    callers: [{file, line, role}]

get_callees:
  input:
    symbol_id: string
  output:
    callee_count: int
    callees: [{symbol_id, file, line}]

get_impact:
  input:
    symbol_id: string
  output:
    direct_references: int
    affected_files: int
    files: list[string]

get_module_deps:
  input:
    module_path: string
  output:
    module: string
    symbols_count: int
    depends_on: {file: reference_count}

get_file_symbols:
  input:
    file_path: string
  output:
    symbols: [{id, kind, name, range}]

find_symbol:
  input:
    pattern: string
  output:
    symbols: [{id, kind, file}]

find_hotspot_symbols:
  input:
    min_callers: int
    path_filter: string?
  output:
    hotspots: [{symbol_id, file, caller_count}]
```

### Integration with Viewpoints

```yaml
# VP-S01 Module Hierarchy — symbol-level deps
codegraph.get_module_deps("src/auth/")
→ actual symbol references, not just imports

# VP-S03 Domain Model — entities as symbols
codegraph.find_hotspot_symbols(min_callers=5, path_filter="domain/")
→ identify aggregate roots by usage

# VP-S06 Hotspots — function-level precision
codegraph.get_callers("domain/transactions/process().")
→ 12 callers, impact radius = HIGH

# Finding enrichment
finding.location → symbol_id
codegraph.get_impact(symbol_id) → severity adjustment
```

### Implementation

```rust
// codegraph-server structure
codegraph-server/
├── Cargo.toml
├── build.rs                 # prost compilation of scip.proto
├── proto/
│   └── scip.proto          # from github.com/sourcegraph/scip
└── src/
    ├── main.rs
    ├── server.rs           # MCP server
    ├── scip.rs             # SCIP parser
    ├── graph.rs            # Codegraph struct
    └── queries.rs          # Query methods
```

**Estimated:** ~500-800 LOC Rust

---

## 5. mental-model-server Extensions

Існуючий server + Artifact Store functionality.

### New Tools: Artifact Store

```yaml
get_commit_artifacts:
  input:
    commit: string           # "abc123" або "HEAD"
  output:
    commit: string           # resolved
    available: [{type, path, produced_at, producer}]
    missing: list[string]

store_artifact:
  input:
    commit: string
    type: string             # "semgrep", "coverage", "scip"
    data: bytes | JSON
    metadata:
      producer: string
      produced_at: datetime?
  output:
    stored_at: string

get_artifact:
  input:
    commit: string
    type: string
  output:
    data: bytes | JSON
    metadata:
      commit: string
      produced_at: datetime
      producer: string
```

### Storage Structure

```
.audit/artifacts/
├── abc123def/                    # commit hash
│   ├── _meta.yaml                # metadata
│   ├── index.scip                # SCIP index
│   ├── semgrep.sarif
│   ├── bandit.sarif
│   ├── combined.sarif            # merged
│   └── codegraph.json            # optional cache
├── def456abc/                    # previous commit
│   └── ...
└── latest -> abc123def/          # symlink
```

### _meta.yaml Format

```yaml
commit: "abc123def789"
branch: "main"
timestamp: "2026-01-21T10:30:00Z"
artifacts:
  scip:
    produced_at: "2026-01-21T10:30:00Z"
    producer: "ci:scip-python"
  semgrep:
    produced_at: "2026-01-21T10:32:15Z"
    producer: "ci:security-scan"
  coverage:
    produced_at: "2026-01-21T10:31:00Z"
    producer: "ci:test"
```

---

## 6. methodology-kb-server Extensions

Існуючий server + Data Acquisition + KB Content.

### New Tools: Data Acquisition Cascade

```yaml
acquire_findings:
  input:
    commit: string
    types: list[string]      # ["security", "complexity"]
    options:
      allow_stale: boolean
      max_age: duration
      generate_if_missing: boolean
  output:
    sarif: SARIF             # merged
    sources: [{type, source, acquisition_path}]

check_manifest:
  input:
    project_path: string
  output:
    exists: boolean
    manifest: Manifest?

parse_ci_config:
  input:
    project_path: string
  output:
    ci_system: "github" | "gitlab" | "jenkins" | null
    artifact_locations: [{type, location}]
```

### Cascade Logic

```
Request: acquire_findings(commit="abc123", types=["security"])

1. CHECK LOCAL ARTIFACTS
   └─ mental-model-server.get_commit_artifacts(abc123)
   └─ Found? → return

2. CHECK MANIFEST
   └─ .audit/manifest.yaml → artifact locations
   └─ Found? → fetch → return

3. PARSE CI/CD CONFIGS
   └─ .github/workflows/*.yml → де зберігаються?
   └─ Found? → fetch from CI → return

4. COMPANY KB LOOKUP
   └─ methodology-kb → стандарти компанії
   └─ Found? → fetch from registry → return

5. GENERATE ON-DEMAND
   └─ sarif-tools-server.run_tool("semgrep", path)
   └─ mental-model-server.store_artifact(commit, sarif)
   └─ return

6. LLM REASONING (final fallback)
   └─ "як ще отримати security findings?"
```

### Manifest Format

```yaml
# .audit/manifest.yaml
version: "1.0"
project: "payment-service"

store:
  type: "local"              # або "s3", "gcs"
  path: ".audit/artifacts/"

artifact_types:
  security:
    schema: "sarif/2.1.0"
    producer: "ci:security-scan"
    tools: ["semgrep", "bandit"]

  coverage:
    schema: "coverage/istanbul"
    producer: "ci:test"

  scip:
    schema: "scip/1.0"
    producer: "ci:scip-index"

  complexity:
    schema: "radon/v1"
    producer: "ci:metrics"
    fallback: "audit:generate"
```

### Required: KB Content

```
methodology_kb/
├── glossary/              # Metric definitions — FILL
├── taxonomies/            # Rule mappings — FILL
├── thresholds/            # Project-type thresholds — FILL
├── standards/             # Architecture patterns — exists
└── templates/             # Report templates — exists
```

See Section 7 for KB Content details.

---

## 7. Methodology KB Content

Контент для methodology-kb-server — без нього severity adjustments не працюють.

### Поточний стан

```
methodology_kb/
├── glossary/          # ~порожньо
├── taxonomies/        # частково
├── thresholds/        # частково
└── templates/         # є
```

### glossary/ (Metric Definitions)

```yaml
# glossary/cognitive_complexity.yaml
id: cognitive_complexity
name: Cognitive Complexity
source: sonarqube
description: |
  Measures how difficult code is to understand.
  Unlike cyclomatic complexity, accounts for nesting.
thresholds:
  low: 0-10
  medium: 11-20
  high: 21-50
  critical: 51+
tools:
  - sonarqube
  - radon (Python)
references:
  - https://www.sonarsource.com/docs/CognitiveComplexity.pdf
```

**Required files:**
- cognitive_complexity.yaml
- cyclomatic_complexity.yaml
- test_coverage.yaml
- code_duplication.yaml
- maintainability_index.yaml
- security_hotspot.yaml
- vulnerability.yaml
- code_smell.yaml

### taxonomies/ (Rule Mappings)

```yaml
# taxonomies/sarif_rule_mappings.yaml
mappings:
  # Semgrep rules
  "python.lang.security.audit.dangerous-exec":
    category: security
    subcategory: injection
    cwe: CWE-78
    severity_base: high

  "python.lang.correctness.useless-comparison":
    category: bugs
    subcategory: logic
    severity_base: medium

  # ESLint rules
  "no-unused-vars":
    category: maintainability
    subcategory: dead_code
    severity_base: low
```

**Required files:**
- sarif_rule_mappings.yaml (semgrep, bandit, eslint, ruff rules)
- severity_adjustments.yaml (context multipliers)
- finding_categories.yaml

### thresholds/ (Project-Type Profiles)

```yaml
# thresholds/python_backend.yaml
project_type: python_backend
description: Thresholds for Python backend services

coverage:
  target: 80
  minimum: 60
  critical_paths: 90

complexity:
  cyclomatic:
    warning: 10
    error: 20
  cognitive:
    warning: 15
    error: 30

duplication:
  warning: 3%
  error: 5%

security:
  vulnerabilities:
    critical: 0
    high: 0
```

**Required files:**
- python_backend.yaml
- typescript_frontend.yaml
- python_library.yaml
- legacy_system.yaml

---

## 8. Implementation Priority

### MCP Servers

| Server | Type | Effort | Priority |
|--------|------|--------|----------|
| **sarif-tools-server** | NEW | M | High |
| **codegraph-server** | NEW | M | High |
| mental-model-server | Extend | S | Medium |
| methodology-kb-server | Extend | S | Medium |

### Content & Config

| Component | Type | Effort | Priority |
|-----------|------|--------|----------|
| KB glossary/ | Content | M | High |
| KB taxonomies/ | Content | M | High |
| KB thresholds/ | Content | S | Medium |
| CI templates | Config | S | Low |

### Recommended Order

```
Week 1-2: Foundation
├── KB Content (glossary, taxonomies) — enables severity adjustments
└── sarif-tools-server — enables tool execution

Week 3-4: Storage & Retrieval
├── mental-model-server extensions (artifact store)
├── methodology-kb-server extensions (acquisition cascade)
└── KB thresholds

Week 5-6: Code Intelligence
├── codegraph-server (SCIP)
└── CI integration templates

Week 7-8: Integration & Testing
├── End-to-end testing
└── Refinement
```

### Dependencies

```
KB Content ──────────────────────────────────────┐
                                                 │
sarif-tools-server ──┬──► mental-model-server ───┼──► methodology-kb-server
                     │    (artifact store)       │    (acquisition cascade)
                     │                           │
codegraph-server ────┴───────────────────────────┘
```

---

## 9. Success Criteria

### Beta-Zero Validation

| Metric | Target |
|--------|--------|
| KB queries with responses | > 90% |
| Tools execute via MCP | All supported tools |
| SARIF as unified format | Yes |
| Commit-indexed artifacts | Yes |
| Symbol-level analysis | Via SCIP |
| Repeat audit time | < 50% of first |
| Root causes | 3-5 (not 800+ findings) |

### MCP Servers Operational

| Server | Tools Working |
|--------|---------------|
| sarif-tools-server | run_tool, merge_sarif, normalize_sarif |
| codegraph-server | get_callers, get_impact, get_module_deps |
| mental-model-server | get_commit_artifacts, store_artifact |
| methodology-kb-server | acquire_findings, lookup_metric |

### Alignment with Infrastructure Vision

| Infrastructure Vision | Beta-Zero | Status |
|-----------------------|-----------|--------|
| MCP as universal interface | 4 MCP servers | Yes |
| Cline orchestration | Unchanged | Yes |
| SKILL.md viewpoints | Unchanged | Yes |
| Methodology KB | Filled | Yes |
| SARIF format | Implemented | Yes |
| Commit-centric | Implemented | Yes |
| Codegraph (from Vision doc) | SCIP-based | Yes |

---

## 10. Timeline (Tentative)

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1 | KB Content | glossary/*.yaml, taxonomies/sarif_rule_mappings.yaml |
| 2 | sarif-tools-server | MCP server, tool runners |
| 3 | Artifact Store | mental-model-server extensions |
| 4 | Data Acquisition | methodology-kb-server extensions |
| 5 | codegraph-server | SCIP parser, query API |
| 6 | CI Integration | GitHub Actions templates |
| 7-8 | Testing & Refinement | End-to-end validation |

---

## 11. Open Questions

1. **Artifact storage backend:** Local filesystem для MVP, S3 для production?
2. **CI integration:** GitHub Actions template для автоматичної публікації artifacts?
3. **KB maintenance:** Як оновлювати rule mappings коли виходять нові версії tools?
4. **Multi-language:** Один threshold profile per project чи per directory?
5. **Codegraph caching:** Зберігати pre-computed graph чи будувати on-demand з SCIP?
6. **SCIP incremental:** Чи підтримують indexers incremental builds?

---

## Appendix A: Related Documents

- [Infrastructure Implementation Vision](./infrastructure_implementation_vision.md)
- [ADR-001: Tool Orchestration via mcp-cli](./adr-001-tool-orchestration-mcp-cli.md)
- [Viewpoints Framework](./VIEWPOINTS_FRAMEWORK.md)
- [Alpha-Zero Repository](https://github.com/velesar/alpha-zero-review-)

---

*Beta-Zero Test Plan — Light IT Global*

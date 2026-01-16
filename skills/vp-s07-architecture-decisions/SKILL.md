---
name: vp-s07-architecture-decisions
version: 1.0
dependencies: [vp-s01-module-hierarchy, vp-s02-layer-architecture, vp-s06-hotspots]
mcp_servers: [mental-model, methodology-kb]
---

# VP-S07: Architecture Decisions Analysis

## Purpose
Discover implicit Architecture Decision Records (ADRs) through patterns in code — reconstructing "why" from "what" by analyzing patterns, trade-offs, and consistency of decisions.

## Prerequisites
- VP-S01, VP-S02, VP-S06 completed
- Module hierarchy and layer architecture analyzed

## Rationale

Most architectural decisions are not explicitly documented. This viewpoint reconstructs the rationale behind architectural choices by analyzing:
- Code patterns and their consistency
- Technology choices and their trade-offs
- Integration patterns
- Security patterns

## Instructions

### Step 1: Get Current Context
Call `mental-model/get_model` and extract:
- `tech_stack` - technology choices
- `architecture.pattern` - detected pattern
- `module_hierarchy` - module structure

### Step 2: Find Explicit ADRs

Look for documented decisions:
```bash
# Find ADR documents
find . -path "*/docs/*" -name "*.md" | xargs grep -l "Decision\|ADR"

# Common ADR locations
ls docs/adr/ docs/decisions/ docs/architecture/decisions/ 2>/dev/null
```

Document any explicit ADRs found with:
- ID, Title, Status
- Location
- Decision date (if available)

### Step 3: Discover Implicit Technology Decisions

Analyze configuration for technology choices:

```bash
# Database choices
grep -r "DATABASE_URL\|REDIS_URL\|KAFKA\|MONGODB" --include="*.py" --include="*.env*"

# Framework choices
grep -r "@router\|@app.route\|GraphQL\|gRPC" --include="*.py" --include="*.ts"

# Authentication choices
grep -r "JWT\|OAuth\|Session\|Passport" --include="*.py" --include="*.ts"
```

For each technology decision, identify:
- What was chosen
- Evidence (files, patterns)
- Alternatives that could have been chosen
- Trade-offs (benefits/drawbacks)

### Step 4: Discover Implicit Architectural Decisions

Look for architectural patterns:

```bash
# Clean/Hexagonal Architecture indicators
find . -type d -name "domain" -o -name "adapters" -o -name "infrastructure" -o -name "ports"

# Event-driven patterns
grep -r "EventBus\|publish\|subscribe\|@event" --include="*.py" --include="*.ts"

# CQRS patterns
find . -name "*command*" -o -name "*query*" -o -name "*handler*"
```

### Step 5: Assess Decision Consistency

For each detected pattern, evaluate:
- Is it consistently applied across the codebase?
- Are there deviations/exceptions?
- Calculate consistency score (0-100%)

### Step 6: Identify Decision Conflicts

Look for conflicting patterns:
- REST + GraphQL without clear separation
- Multiple ORMs in same codebase
- Inconsistent error handling patterns
- Mixed authentication mechanisms

### Step 7: Generate Rationale Hypotheses

For implicit decisions, generate hypotheses about WHY:

**Decision Types:**

| Category | Examples |
|----------|----------|
| Technology Choices | "Why PostgreSQL over MongoDB?" |
| Architectural Patterns | "Why Clean Architecture?" |
| Integration Patterns | "Why REST over GraphQL?" |
| Data Patterns | "Why Event Sourcing?" |
| Security Patterns | "Why OAuth2 over custom auth?" |
| Trade-offs | "Why consistency over availability?" |

### Step 8: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S07"
data:
  explicit_adrs:
    - id: "ADR-001"
      title: "Use PostgreSQL for primary database"
      status: "accepted"
      location: "docs/adr/001-database.md"

  implicit_decisions:
    - category: "technology"
      decision: "PostgreSQL as primary database"
      evidence:
        - file: "src/infrastructure/database.py"
          pattern: "asyncpg connection"
          confidence: "high"
      rationale_hypothesis: "Chosen for ACID compliance, JSON support, and team familiarity"
      alternatives_considered:
        - "MongoDB"
        - "MySQL"
      trade_offs:
        benefits:
          - "Strong consistency"
          - "Rich querying"
          - "JSON support"
        drawbacks:
          - "Harder to scale horizontally"
      consistency_score: 0.95

    - category: "pattern"
      decision: "Clean Architecture"
      evidence:
        - file: "src/"
          pattern: "domain/application/infrastructure directories"
          confidence: "high"
      rationale_hypothesis: "Separation of concerns, testability, framework independence"
      consistency_score: 0.85

  decision_conflicts:
    - decisions: ["REST API", "GraphQL endpoint"]
      conflict_type: "mixed_api_styles"
      locations:
        - "src/api/rest/"
        - "src/api/graphql/"
      recommendation: "Document clear boundaries or consolidate to single style"
```

## Output Schema

```yaml
architecture_decisions:
  explicit_adrs:
    - id: string
      title: string
      status: accepted | deprecated | superseded
      location: string

  implicit_decisions:
    - category: technology | pattern | integration | security | trade-off
      decision: string
      evidence:
        - file: string
          pattern: string
          confidence: high | medium | low
      rationale_hypothesis: string
      alternatives_considered: [string]
      trade_offs:
        benefits: [string]
        drawbacks: [string]
      consistency_score: number

  decision_conflicts:
    - decisions: [string]
      conflict_type: string
      locations: [string]
      recommendation: string
```

## Quality Indicators

- ✅ Explicit ADRs for key decisions
- ✅ Consistent patterns across codebase
- ✅ Clear rationale for technology choices
- 🚩 Conflicting patterns (REST + GraphQL without clear separation)
- 🚩 Inconsistent application of chosen patterns
- 🚩 No documentation for non-obvious choices
- 🚩 "Accidental architecture" — patterns without intent

## Common Findings

| Finding | Severity | Recommendation |
|---------|----------|----------------|
| Mixed architectural styles | Medium | Document boundaries, plan migration |
| Undocumented technology choice | Low | Create ADR retroactively |
| Conflicting patterns | High | Resolve or document rationale |
| Inconsistent error handling | Medium | Establish and document standard |

## AI Prompt Template

```
Analyze these code patterns and configurations:
[FILES AND PATTERNS]

For each significant decision found:
1. What decision was made?
2. What evidence supports this (files, patterns)?
3. Why might this decision have been made? (hypothesis)
4. What alternatives existed?
5. What are the trade-offs?
6. Is it consistently applied?
7. Are there any conflicting decisions?
```

## Fitness Functions

```python
def check_architectural_consistency(project_path: str) -> bool:
    """Architectural patterns must be consistently applied."""
    expected_structure = {"domain", "application", "infrastructure"}

    modules = find_top_level_modules(project_path)
    inconsistent = []

    for module in modules:
        module_dirs = set(os.listdir(f"{project_path}/{module}"))
        if not expected_structure.issubset(module_dirs):
            missing = expected_structure - module_dirs
            inconsistent.append((module, missing))

    assert len(inconsistent) == 0, \
        f"Modules with inconsistent structure: {inconsistent}"
    return True


def check_explicit_adrs_exist(project_path: str) -> bool:
    """Key decisions must have explicit ADRs."""
    required_decisions = [
        "database_choice",
        "authentication_method",
        "api_style",
    ]

    adr_path = f"{project_path}/docs/adr"
    if not os.path.exists(adr_path):
        raise AssertionError("No ADR directory found at docs/adr")

    adrs = [f.lower() for f in os.listdir(adr_path)]
    missing = []

    for decision in required_decisions:
        if not any(decision in adr for adr in adrs):
            missing.append(decision)

    assert len(missing) == 0, f"Missing ADRs for: {missing}"
    return True
```

## Next Viewpoint

After completing VP-S07:
- For enterprise projects: proceed to **VP-S08: Team Topologies Mapping** (optional)
- For compliance audits: proceed to **VP-S09: Building Block Compliance** (optional)
- Otherwise: proceed to **VP-Q01: Security Analysis**

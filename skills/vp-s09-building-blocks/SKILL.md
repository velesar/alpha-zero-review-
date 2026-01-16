---
name: vp-s09-building-blocks
version: 1.0
status: optional
dependencies: [vp-s02-layer-architecture, vp-s07-architecture-decisions]
mcp_servers: [mental-model, methodology-kb]
---

# VP-S09: Building Block Compliance `[OPTIONAL]`

## Purpose
Compare the project architecture against a reference architecture using TOGAF Building Blocks (ABB/SBB) to identify gaps and compliance issues.

## When to Use

- Compliance audit for enterprise standards
- Migration planning to target architecture
- Assessing technical debt scope
- Onboarding new architects

## Concepts

| Concept | Description |
|---------|-------------|
| ABB (Architecture Building Block) | Abstract capability ("Authentication") |
| SBB (Solution Building Block) | Concrete implementation ("Keycloak OAuth2") |
| Reference Architecture | Target set of ABBs for project type |
| Gap | Missing or non-compliant building block |

## Reference Architectures

Available reference architectures:

### Clean Architecture (Python)

```yaml
domain_layer:
  - abb: "Entity Base"
    expected_sbbs: ["Pydantic BaseModel", "dataclass"]
  - abb: "Repository Interface"
    expected_sbbs: ["Protocol/ABC definition"]
  - abb: "Domain Event"
    expected_sbbs: ["Event class with publish()"]

application_layer:
  - abb: "Use Case"
    expected_sbbs: ["Command Handler", "Query Handler"]
  - abb: "Unit of Work"
    expected_sbbs: ["Context manager pattern"]

infrastructure_layer:
  - abb: "Repository Implementation"
    expected_sbbs: ["SQLAlchemy Repository"]
  - abb: "HTTP API"
    expected_sbbs: ["FastAPI Router"]
```

### Hexagonal Architecture (TypeScript)

```yaml
domain_layer:
  - abb: "Domain Entity"
    expected_sbbs: ["Class with business logic"]
  - abb: "Port Interface"
    expected_sbbs: ["TypeScript interface"]

adapters_layer:
  - abb: "Inbound Adapter"
    expected_sbbs: ["NestJS Controller", "GraphQL Resolver"]
  - abb: "Outbound Adapter"
    expected_sbbs: ["TypeORM Repository", "HTTP Client"]
```

## Instructions

### Step 1: Select Reference Architecture

Based on VP-F01 (tech stack) and VP-S02 (detected architecture), select appropriate reference:
- `clean_architecture_python`
- `hexagonal_typescript`
- `modular_monolith`
- `microservices`

### Step 2: Inventory Existing Building Blocks

For each layer, identify what SBBs exist:

```bash
# Find entity bases (Python)
grep -r "class.*BaseModel\|@dataclass" --include="*.py" src/domain/

# Find repository interfaces
grep -r "class.*Repository.*ABC\|Protocol" --include="*.py" src/domain/

# Find use cases/handlers
find . -name "*handler*.py" -o -name "*use_case*.py"
```

### Step 3: Map SBBs to ABBs

For each found SBB:
1. Determine which ABB it implements
2. Assess compliance (full/partial/non-compliant)
3. Note any issues

### Step 4: Identify Gaps

Compare against reference:
- Which ABBs are missing entirely?
- Which ABBs have partial implementations?
- Which SBBs deviate from expected patterns?

### Step 5: Assess Impact

For each gap:
- What is the impact on the system?
- What effort is needed to remediate?
- What is the priority?

### Step 6: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S09"
data:
  reference_architecture: "clean_architecture_python"
  overall_compliance: 78

  by_layer:
    domain:
      expected: 5
      found: 4
      compliance: 80
      gaps:
        - abb: "Domain Event"
          impact: "No event-driven communication possible"
          recommendation: "Implement domain events for aggregate notifications"
      concerns:
        - sbb: "User entity"
          issue: "Anemic model - no business methods"
          severity: "medium"

    application:
      expected: 3
      found: 3
      compliance: 100
      gaps: []
      concerns: []

    infrastructure:
      expected: 4
      found: 3
      compliance: 75
      gaps:
        - abb: "Caching Layer"
          impact: "Performance issues under load"
          recommendation: "Implement Redis caching for hot paths"
      concerns: []

  sbb_inventory:
    - abb: "Entity Base"
      sbb: "Pydantic BaseModel"
      location: "src/domain/base.py"
      compliance: "full"
      issues: []

    - abb: "Repository Interface"
      sbb: "ABC Repository"
      location: "src/domain/repositories.py"
      compliance: "partial"
      issues:
        - "Missing unit of work integration"

  recommendations:
    priority_1:
      - "Implement domain events for OrderAggregate"
      - "Add caching layer for user lookups"
    priority_2:
      - "Enrich domain models with business logic"
    priority_3:
      - "Add specification pattern for complex queries"
```

## Output Schema

```yaml
building_block_compliance:
  reference_architecture: string
  overall_compliance: number  # percentage

  by_layer:
    <layer_name>:
      expected: number
      found: number
      compliance: number
      gaps:
        - abb: string
          impact: string
          recommendation: string
      concerns:
        - sbb: string
          issue: string
          severity: string

  sbb_inventory:
    - abb: string
      sbb: string
      location: string
      compliance: full | partial | non-compliant
      issues: [string]

  recommendations:
    priority_1: [string]
    priority_2: [string]
    priority_3: [string]
```

## Quality Indicators

- ✅ All ABBs from reference present
- ✅ SBBs follow reference patterns
- ✅ Consistent implementation across modules
- 🚩 Missing critical building blocks
- 🚩 Custom implementations where standard exists
- 🚩 Inconsistent SBBs for same ABB

## AI Prompt Template

```
Compare this codebase against reference architecture:
[REFERENCE: Clean Architecture Python]
[CODEBASE STRUCTURE]
[KEY FILES]

For each expected building block:
1. Is it present? Where?
2. Is implementation compliant with reference?
3. What issues exist?
4. What is impact of gaps?
5. Prioritized recommendations
```

## Next Viewpoint

After VP-S09:
- For standards audit: proceed to **VP-S10: Standards Compliance**
- Otherwise: proceed to **VP-Q01: Security Analysis**

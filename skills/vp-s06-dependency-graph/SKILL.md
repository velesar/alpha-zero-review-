---
name: vp-s06-dependency-graph
version: 2.0
dependencies: [vp-f01-tech-stack, vp-s01-module-hierarchy, vp-s02-layer-architecture]
mcp_servers: [mental-model, methodology-kb]
---

# VP-S06: Dependency Graph Analysis

## Purpose
Visualize and analyze component dependencies to identify coupling issues, circular dependencies, and potential architectural violations. Also identifies hotspots by combining dependency analysis with code churn metrics.

## Prerequisites
- Module hierarchy identified (VP-S01)
- Architecture layers defined (VP-S02)
- Git history available for churn analysis

## Instructions

### Step 1: Get Current Context

Call `mental-model/get_model` and extract:
- `structure.source_roots` - directories to analyze
- `architecture.layers` - for layer violation detection
- `module_hierarchy.modules` - for module context

### Step 2: Build Internal Dependency Graph

Analyze imports/requires to build the dependency graph:

#### Python
```bash
# Use pydeps for visualization
pydeps src/ --max-bacon=2 --cluster

# Or use import analysis
grep -rh "^from\|^import" src/ --include="*.py" | sort | uniq -c | sort -rn
```

#### JavaScript/TypeScript
```bash
# Use madge for dependency analysis
npx madge --circular src/
npx madge --image graph.svg src/

# Or manual analysis
grep -rh "^import\|require(" src/ --include="*.ts" --include="*.js"
```

### Step 3: Analyze External Dependencies

Check package dependencies for issues:

```bash
# Python - check for outdated/vulnerable packages
pip-audit
pip list --outdated

# Node.js - check for vulnerabilities
npm audit
npm outdated
```

### Step 4: Detect Circular Dependencies

Identify circular dependency chains:

```bash
# Python
pydeps src/ --show-cycles

# JavaScript/TypeScript
npx madge --circular src/
```

Document each cycle:
- Files involved
- Impact on testability
- Potential breaking points

### Step 5: Calculate Coupling Metrics

For each module, calculate:

| Metric | Description | Formula |
|--------|-------------|---------|
| Afferent Coupling (Ca) | Incoming dependencies | Count of modules depending on this |
| Efferent Coupling (Ce) | Outgoing dependencies | Count of modules this depends on |
| Instability (I) | Change sensitivity | Ce / (Ca + Ce) |
| Abstractness (A) | Interface vs impl | Abstract classes / Total classes |

```
I close to 0 → Stable (many depend on it)
I close to 1 → Unstable (depends on many)
```

### Step 6: Detect Layer Violations

Cross-reference dependencies with architecture layers (VP-S02):

```yaml
allowed_dependencies:
  presentation: [application, domain]
  application: [domain, infrastructure]
  domain: []  # Domain should not depend on anything
  infrastructure: [domain]
```

Flag violations:
- Domain importing from infrastructure
- Presentation importing from infrastructure directly
- Circular layer dependencies

### Step 7: Calculate Hotspot Scores

Combine dependency metrics with code churn:

```bash
# Get files changed in last 6 months with change count
git log --since="6 months ago" --pretty=format: --name-only | \
  sort | uniq -c | sort -rn | head -50
```

Calculate hotspot score:
```
hotspot_score = normalize(churn) * 0.3 + normalize(coupling) * 0.4 + normalize(complexity) * 0.3
```

Where:
- `churn`: Commit count in last 6 months
- `coupling`: Sum of afferent + efferent coupling
- `complexity`: Cyclomatic/cognitive complexity

Risk levels:
- **CRITICAL**: Score ≥ 80
- **HIGH**: Score ≥ 60
- **MEDIUM**: Score ≥ 40
- **LOW**: Score < 40

### Step 8: Identify Dependency Smells

Look for these anti-patterns:

| Smell | Description | Detection |
|-------|-------------|-----------|
| Hub modules | Too many dependencies in/out | Ca + Ce > 20 |
| Cyclic deps | Circular import chains | Madge/pydeps |
| Hidden deps | Dependencies through globals | Manual review |
| Feature envy | Module uses another module's data heavily | Import frequency |
| God module | Central module everyone depends on | Ca > 15 |

### Step 9: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S06"
data:
  dependency_graph:
    total_modules: 45
    total_edges: 120
    average_coupling: 2.7
    max_coupling: 15

  coupling_metrics:
    by_module:
      - module: "src/orders"
        afferent: 12
        efferent: 3
        instability: 0.2
        classification: "stable_core"
      - module: "src/api/handlers"
        afferent: 2
        efferent: 8
        instability: 0.8
        classification: "unstable_peripheral"

  circular_dependencies:
    count: 2
    cycles:
      - ["src/orders/service.py", "src/payments/processor.py", "src/orders/validator.py"]
      - ["src/users/auth.py", "src/users/permissions.py"]
    severity: "high"
    recommendation: "Break cycles through interface extraction"

  layer_violations:
    count: 3
    violations:
      - from_layer: "domain"
        to_layer: "infrastructure"
        source: "src/domain/order.py"
        target: "src/infrastructure/database.py"
        severity: "critical"

  external_dependencies:
    total: 45
    outdated: 8
    vulnerable: 2
    deprecated: 3

  hotspots:
    files:
      - path: "src/orders/order_service.py"
        churn: 45
        coupling: 15
        complexity: 35.5
        score: 82.5
        risk: "CRITICAL"
        reasons:
          - "High change frequency (45 commits)"
          - "Hub module (15 connections)"
          - "High cyclomatic complexity (35.5)"
      - path: "src/api/handlers/user_handler.py"
        churn: 30
        coupling: 10
        complexity: 25.0
        score: 65.0
        risk: "HIGH"
        reasons:
          - "High coupling (10 dependencies)"
          - "Complex authentication logic"

  dependency_smells:
    - smell: "hub_module"
      location: "src/common/utils.py"
      afferent: 25
      recommendation: "Split into focused utility modules"
    - smell: "god_module"
      location: "src/core/service.py"
      afferent: 18
      recommendation: "Extract domain-specific services"
```

## Output Schema

```yaml
dependency_graph:
  total_modules: number
  total_edges: number
  average_coupling: number
  max_coupling: number

  coupling_metrics:
    by_module:
      - module: string
        afferent: number      # Incoming deps
        efferent: number      # Outgoing deps
        instability: number   # 0-1 scale
        classification: stable_core | stable_peripheral | unstable_core | unstable_peripheral

  circular_dependencies:
    count: number
    cycles: [[string]]        # Arrays of file paths in cycle
    severity: low | medium | high | critical
    recommendation: string

  layer_violations:
    count: number
    violations:
      - from_layer: string
        to_layer: string
        source: string
        target: string
        severity: string

  external_dependencies:
    total: number
    outdated: number
    vulnerable: number
    deprecated: number

  hotspots:
    files:
      - path: string
        churn: number
        coupling: number
        complexity: number
        score: number        # 0-100
        risk: CRITICAL | HIGH | MEDIUM | LOW
        reasons: [string]

  dependency_smells:
    - smell: string
      location: string
      metric: number
      recommendation: string
```

## Quality Indicators

- ✅ **Healthy**: No circular deps, clear layering, low coupling average
- ⚠️ **Warning**: Few cycles, some layer violations, moderate coupling
- ❌ **Critical**: Multiple cycles, layer violations, high coupling, hub modules

## Coupling Interpretation

### Stable Abstractions Principle (SAP)

```
         High Abstractness
              │
   Zone of    │   Ideal
   Uselessness│
              │
──────────────┼──────────────
              │
   Ideal      │   Zone of
              │   Pain
              │
         Low Abstractness
    Stable         Unstable
   (Low I)        (High I)
```

- **Zone of Pain**: Concrete + Stable = Hard to change, many dependents
- **Zone of Uselessness**: Abstract + Unstable = Over-engineered, unused

## Hotspot Interpretation

### High Coupling + High Churn = 🔥 Critical Hotspot
- Needs immediate refactoring attention
- High risk for cascading bugs
- Expensive to maintain

### High Coupling + Low Churn = 📦 Stable Hub
- Many depend on it but rarely changes
- Consider: should this be more abstract?

### Low Coupling + High Churn = 🔄 Active Development
- Frequently updated, well isolated
- Monitor for coupling growth

### Low Coupling + Low Churn = ✅ Stable Isolated
- Ideal state
- Low maintenance burden

## Dependency Reduction Strategies

| Problem | Solution |
|---------|----------|
| Circular deps | Extract interface, use events |
| High coupling | Apply dependency injection |
| Hub modules | Split by responsibility |
| Layer violations | Introduce adapters |
| God modules | Extract bounded contexts |

## Next Viewpoint

After VP-S06, proceed to **VP-S07: Architecture Decisions Analysis**.

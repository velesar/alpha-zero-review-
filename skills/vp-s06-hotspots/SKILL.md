---
name: vp-s06-hotspots
version: 1.0
dependencies: [vp-f01-tech-stack, vp-s01-module-hierarchy, vp-s02-layer-architecture]
mcp_servers: [mental-model, methodology-kb]
---

# VP-S06: Hotspots Analysis

## Purpose
Identify high-risk files by combining code churn (change frequency) with complexity metrics to find areas that deserve focused attention during quality analysis.

## Prerequisites
- Module hierarchy and architecture layers identified
- Git history available for churn analysis

## Instructions

### Step 1: Get Current Context
Call `mental-model/get_model` and extract:
- `structure.source_roots` - directories to analyze
- `architecture.layers` - for context enrichment
- `module_hierarchy.modules` - for module context

### Step 2: Calculate Code Churn

Use git history to identify frequently changed files:

```bash
# Get files changed in last 6 months with change count
git log --since="6 months ago" --pretty=format: --name-only | \
  sort | uniq -c | sort -rn | head -50
```

Alternative with more detail:
```bash
# Get file change frequency
git log --format=format: --name-only --since="6 months ago" | \
  egrep -v '^$' | \
  sort | uniq -c | sort -rg | head -50
```

For each file, capture:
- Number of commits (churn)
- Unique authors
- Recent vs old changes

### Step 3: Calculate Complexity Metrics

Based on language from VP-F01:

#### Python - Using Radon
```bash
# Cyclomatic complexity
radon cc src/ -a -s

# Maintainability index
radon mi src/ -s
```

#### JavaScript/TypeScript - Using ESLint complexity
```bash
# Or use complexity report tools
npx complexity-report --format json src/
```

#### General approach
For each file, measure:
- Cyclomatic complexity
- Cognitive complexity (if available)
- Lines of code
- Number of functions/methods
- Nesting depth

### Step 4: Calculate Hotspot Score

Combine churn and complexity into a hotspot score:

```
hotspot_score = normalize(churn) * 0.4 + normalize(complexity) * 0.6
```

Where:
- `normalize()` scales values to 0-100
- Churn weight: 0.4 (40%)
- Complexity weight: 0.6 (60%)

Higher score = higher risk hotspot

### Step 5: Classify Risk Level

Based on hotspot score:
- **CRITICAL**: Score ≥ 80
- **HIGH**: Score ≥ 60
- **MEDIUM**: Score ≥ 40
- **LOW**: Score < 40

### Step 6: Identify Reasons

For each hotspot, document why it's a hotspot:
- High change frequency
- High cyclomatic complexity
- Large file size
- Deep nesting
- Many dependencies
- Core business logic location

### Step 7: Cross-Reference with Architecture

Enrich hotspot data with context:
- Which bounded context does it belong to?
- Which architecture layer?
- Is it in a core domain area?

Files in core domain + high hotspot score = highest priority.

### Step 8: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S06"
data:
  files:
    - path: "src/orders/order_service.py"
      churn: 45
      complexity: 35.5
      score: 82.5
      risk: "CRITICAL"
      reasons:
        - "High change frequency (45 commits in 6 months)"
        - "High cyclomatic complexity (35.5)"
        - "Core domain: OrderManagement"
    - path: "src/api/handlers/user_handler.py"
      churn: 30
      complexity: 25.0
      score: 65.0
      risk: "HIGH"
      reasons:
        - "Moderate change frequency"
        - "Complex authentication logic"
    - path: "src/utils/helpers.py"
      churn: 50
      complexity: 10.0
      score: 45.0
      risk: "MEDIUM"
      reasons:
        - "Frequently changed but low complexity"
        - "Generic utilities"
```

## Output Schema

```yaml
hotspots:
  files:
    - path: string
      churn: number           # Commit count
      complexity: number      # Complexity score
      score: number          # Hotspot score (0-100)
      risk: CRITICAL | HIGH | MEDIUM | LOW
      reasons: [string]
```

## Quality Indicators

- ✅ **Healthy**: Few hotspots, hotspots not in core domain, trending down
- ⚠️ **Warning**: Several hotspots, some in core domain
- ❌ **Critical**: Many critical hotspots, core domain affected, trending up

## Hotspot Interpretation

### High Churn + High Complexity = 🔥 Critical Hotspot
- Needs immediate refactoring attention
- High risk for bugs
- Expensive to maintain

### High Churn + Low Complexity = 🔄 Change-Heavy
- Frequently updated but manageable
- Watch for complexity creep
- May indicate evolving requirements

### Low Churn + High Complexity = 📦 Stable Complex
- Complex but not actively changing
- Lower immediate risk
- Consider refactoring when touched

### Low Churn + Low Complexity = ✅ Stable Simple
- Ideal state
- Low maintenance burden
- Not a concern

## Hotspot Reduction Strategies

| Problem | Solution |
|---------|----------|
| High complexity | Extract methods, simplify logic |
| High churn | Stabilize interface, reduce coupling |
| Large files | Split into focused modules |
| Deep nesting | Use early returns, extract conditions |
| Many dependencies | Apply dependency injection |

## Prioritization Matrix

```
         High Complexity
              │
   Refactor   │   Critical
   When       │   Hotspot
   Touched    │
──────────────┼──────────────
              │
   Stable     │   Watch
   Simple     │   For Growth
              │
         Low Complexity
    Low Churn    High Churn
```

## Structure Phase Complete

After VP-S06, the Structure phase is complete. The mental model now contains:
- Module hierarchy (VP-S01)
- Architecture layers (VP-S02)
- Domain model (VP-S03)
- Entity model (VP-S04)
- Interface surface (VP-S05)
- Hotspots (VP-S06)

Proceed to **Quality Phase** starting with **VP-Q01: Security Analysis**.

---
name: vp-s08-team-topologies
version: 1.0
status: optional
dependencies: [vp-s01-module-hierarchy, vp-s03-domain-model]
mcp_servers: [mental-model, methodology-kb]
---

# VP-S08: Team Topologies Mapping `[OPTIONAL]`

## Purpose
Analyze the alignment between code architecture and team structure (Conway's Law alignment) to identify ownership gaps, cognitive load issues, and cross-team friction.

## When to Use

- Enterprise projects with multiple teams
- Audit before team reorganization
- Identifying causes of low velocity
- Planning monolith modularization

## Rationale

Conway's Law states that system architecture reflects the communication structure of the organization. Misalignment between team structure and code ownership creates friction and reduces velocity.

## Team Types (per Team Topologies)

| Type | Description | Architectural Impact |
|------|-------------|---------------------|
| Stream-aligned | Delivers business value | Owns bounded context end-to-end |
| Platform | Provides internal services | Shared infrastructure, APIs |
| Enabling | Helps other teams | Cross-cutting concerns |
| Complicated-subsystem | Specialized knowledge | Isolated complex components |

## Instructions

### Step 1: Get Current Context

Call `mental-model/get_model` and extract:
- `domain_model.bounded_contexts` - domain boundaries
- `module_hierarchy.modules` - code structure

### Step 2: Find Ownership Information

Look for explicit ownership:
```bash
# Find CODEOWNERS
cat .github/CODEOWNERS

# Check for ownership in package files
grep -r "author\|maintainer" --include="package.json" --include="pyproject.toml"
```

### Step 3: Analyze Git History for Implicit Ownership

```bash
# Analyze commit authors by directory (last 6 months)
git log --format='%ae' --since='6 months ago' -- src/payments/ | sort | uniq -c | sort -rn

# Find cross-team changes
git log --format='%h %ae' --name-only --since='3 months ago' | \
  awk '/^[a-f0-9]/{author=$2} /^src\//{print author, $0}'

# Find files with multiple authors
git log --format='%ae' --since='6 months ago' -- src/orders/ | sort -u | wc -l
```

### Step 4: Map Teams to Modules

For each identified team:
1. List modules they primarily own
2. Calculate ownership confidence based on commit frequency
3. Identify shared ownership areas

### Step 5: Calculate Cognitive Load

For each team, assess cognitive load based on:
- Number of domains owned
- Lines of code owned
- Number of external dependencies
- Complexity of owned code

Score 1-10 (>7 is overloaded)

### Step 6: Identify Conway Alignment Issues

Check if:
- Team boundaries align with bounded contexts
- Single team owns related modules
- Cross-cutting changes require cross-team coordination

### Step 7: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S08"
data:
  teams:
    - name: "Payments Team"
      type: "stream-aligned"
      owned_modules:
        - "src/payments"
        - "src/billing"
      cognitive_load_score: 6

    - name: "Platform Team"
      type: "platform"
      owned_modules:
        - "src/infrastructure"
        - "src/common"
      cognitive_load_score: 5

  ownership_analysis:
    clear_ownership:
      - module: "src/payments"
        team: "Payments Team"
        confidence: 0.92
    shared_ownership:
      - module: "src/shared/models"
        teams: ["Payments Team", "Orders Team"]
        conflict_risk: "medium"
    orphaned_modules:
      - "src/legacy/reports"

  conway_alignment:
    aligned:
      - module: "src/payments"
        team: "Payments Team"
        interaction_mode: "x-as-a-service"
    misaligned:
      - module: "src/orders/payment_integration"
        current_owner: "Orders Team"
        natural_owner: "Payments Team"
        recommendation: "Move to Payments Team or create clear API boundary"

  cognitive_load:
    by_team:
      - team: "Payments Team"
        domains_owned: 2
        lines_of_code: 15000
        external_dependencies: 8
        score: 6
        recommendation: "Within acceptable range"
      - team: "Core Platform"
        domains_owned: 5
        lines_of_code: 45000
        external_dependencies: 20
        score: 8
        recommendation: "Consider splitting into smaller teams"

  interaction_modes:
    - teams: ["Payments Team", "Orders Team"]
      mode: "x-as-a-service"
      interface: "src/payments/api/"
      friction_indicators:
        - "12 cross-team commits in last month"
```

## Output Schema

```yaml
team_topologies:
  teams:
    - name: string
      type: stream-aligned | platform | enabling | complicated-subsystem
      owned_modules: [string]
      cognitive_load_score: number  # 1-10

  ownership_analysis:
    clear_ownership:
      - module: string
        team: string
        confidence: number
    shared_ownership:
      - module: string
        teams: [string]
        conflict_risk: low | medium | high
    orphaned_modules: [string]

  conway_alignment:
    aligned:
      - module: string
        team: string
        interaction_mode: string
    misaligned:
      - module: string
        current_owner: string
        natural_owner: string
        recommendation: string

  cognitive_load:
    by_team:
      - team: string
        domains_owned: number
        lines_of_code: number
        external_dependencies: number
        score: number
        recommendation: string

  interaction_modes:
    - teams: [string]
      mode: collaboration | x-as-a-service | facilitating
      interface: string
      friction_indicators: [string]
```

## Quality Indicators

- ✅ Clear module ownership (CODEOWNERS complete)
- ✅ Team boundaries align with bounded contexts
- ✅ Cognitive load balanced across teams
- 🚩 Shared ownership without clear interfaces
- 🚩 One team owns unrelated modules
- 🚩 High cross-team commit coupling
- 🚩 Orphaned modules (no clear owner)

## Metrics

| Metric | Target | Description |
|--------|--------|-------------|
| Ownership clarity | > 90% | % modules with single owner |
| Cognitive load score | < 7 per team | Complexity score 1-10 |
| Cross-team commits | < 10% | Changes touching multiple team's code |
| Conway alignment | > 80% | % modules owned by "natural" team |

## Next Viewpoint

After VP-S08:
- For compliance audits: proceed to **VP-S09: Building Block Compliance**
- Otherwise: proceed to **VP-Q01: Security Analysis**

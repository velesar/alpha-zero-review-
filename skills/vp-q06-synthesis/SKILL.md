---
name: vp-q06-synthesis
version: 2.0
dependencies: [vp-q01-security, vp-q02-performance, vp-q03-testability, vp-q04-code-style, vp-q05-documentation]
mcp_servers: [mental-model, methodology-kb]
---

# VP-Q06: Technical Debt Synthesis

## Purpose
Aggregate, classify, and prioritize all findings using the Fowler Quadrant model, then generate the final audit report with actionable recommendations.

## Prerequisites
- All Quality viewpoints completed (VP-Q01 through VP-Q05)
- Findings accumulated in mental model

## Fowler Quadrant: Technical Debt Classification

Technical Debt is classified along two dimensions:

```
                    DELIBERATE                    INADVERTENT
              (Свідомо прийнятий)            (Випадково створений)
         ┌────────────────────────────┬────────────────────────────┐
         │                            │                            │
         │   "We must ship now and    │   "Now we know how we      │
PRUDENT  │    deal with consequences" │    should have done it"    │
(Розсудливий) │                            │                            │
         │   → Schedule payback       │   → Refactor as you learn  │
         │   → Track explicitly       │   → Update standards       │
         │                            │                            │
         ├────────────────────────────┼────────────────────────────┤
         │                            │                            │
         │   "We don't have time      │   "What's layering?"       │
RECKLESS │    for design"             │                            │
(Безрозсудний) │                            │                            │
         │   → High risk, costly fix  │   → Training needed        │
         │   → Often leads to rewrites│   → May require rewrite    │
         │                            │                            │
         └────────────────────────────┴────────────────────────────┘
```

### Quadrant Strategies

| Quadrant | Strategy | Example |
|----------|----------|---------|
| Prudent-Deliberate | Track, schedule payback | "Hardcoded config for MVP, ticket to externalize" |
| Prudent-Inadvertent | Refactor incrementally | "Discovered better pattern, apply Boy Scout Rule" |
| Reckless-Deliberate | Urgent remediation | "Skipped tests to meet deadline — critical risk" |
| Reckless-Inadvertent | Training + remediation | "Team didn't know about N+1 queries" |

## Instructions

### Step 1: Verify Prerequisites

Call `mental-model/get_completed_viewpoints` and verify:
```
["VP-F01", "VP-F02", "VP-F03",
 "VP-S01", "VP-S02", "VP-S03", "VP-S04", "VP-S05", "VP-S06", "VP-S07",
 "VP-Q01", "VP-Q02", "VP-Q03", "VP-Q04", "VP-Q05"]
```

Optional viewpoints (VP-S08, VP-S09, VP-S10) may also be present.

### Step 2: Get All Findings

Call `mental-model/get_findings` to retrieve all findings from quality viewpoints.

Review the findings:
- Total count
- Distribution by category
- Distribution by severity
- Distribution by affected area

### Step 3: Classify Each Finding by Quadrant

For each finding, determine its Fowler Quadrant:

**Prudent-Deliberate indicators:**
- TODO/FIXME comments explaining the shortcut
- Tickets referencing the issue
- Time-sensitive business constraints
- Explicit trade-off documentation

**Prudent-Inadvertent indicators:**
- New patterns emerged after implementation
- Industry best practices changed
- Better libraries became available
- Lessons learned from production

**Reckless-Deliberate indicators:**
- No tests "to save time"
- Skipped code review
- Copied code instead of refactoring
- Ignored security warnings

**Reckless-Inadvertent indicators:**
- Anti-patterns from lack of knowledge
- Missing basic error handling
- N+1 queries from ORM misuse
- Security vulnerabilities from unawareness

### Step 4: Synthesize Root Causes

Call `mental-model/synthesize`:
```json
{
  "algorithm": "category_based"
}
```

The synthesize function will:
1. Cluster findings by category
2. Identify patterns across findings
3. Calculate aggregate impact
4. Generate root cause descriptions
5. Create recommendations

### Step 5: Analyze Quadrant Distribution

Calculate quadrant distribution:
```yaml
by_quadrant:
  prudent_deliberate: count
  prudent_inadvertent: count
  reckless_deliberate: count
  reckless_inadvertent: count
```

**Interpretation:**
- High Prudent counts → Team makes conscious trade-offs (healthy)
- High Reckless-Deliberate → Process problems (rushed delivery)
- High Reckless-Inadvertent → Knowledge gaps (training needed)

### Step 6: Identify Root Causes

For each root cause, identify:
- Why does this debt exist?
- Is it systemic (same cause for multiple findings)?
- What would prevent this in the future?

### Step 7: Prioritize Using Impact/Effort Matrix

```
                    IMPACT
                High        Low
         ┌─────────────┬─────────────┐
    Low  │  QUICK WIN  │   BACKLOG   │
EFFORT   │   Do Now    │   Consider  │
         ├─────────────┼─────────────┤
    High │  STRATEGIC  │   AVOID     │
         │   Plan      │   Usually   │
         └─────────────┴─────────────┘
```

### Step 8: Generate Prioritized Backlog

Categorize into:
- **Immediate** (0-1 week): Critical security, reckless-deliberate items
- **Short-term** (1-4 weeks): High impact, quick wins
- **Medium-term** (1-3 months): Strategic improvements
- **Long-term** (3+ months): Nice to have

### Step 9: Get Report Templates

Call `methodology-kb/get_template`:
```json
{
  "template_type": "executive_summary",
  "format": "markdown"
}
```

Also get:
```json
{
  "template_type": "root_cause",
  "format": "markdown"
}
```

### Step 10: Generate Reports

Create the following deliverables:

1. **executive_summary.md** - Stakeholder report
2. **root_cause_analysis.md** - Technical root causes with Fowler Quadrant
3. **detailed_findings.md** - All findings with context
4. **mental_model.yaml** - Complete audit data
5. **technical_debt_inventory.yaml** - Debt items by quadrant

### Step 11: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-Q06"
data:
  summary:
    total_items: 45
    by_category:
      security: 12
      reliability: 8
      maintainability: 15
      performance: 5
      testability: 5
    by_severity:
      critical: 2
      high: 8
      medium: 20
      low: 15
    by_quadrant:
      prudent_deliberate: 8
      prudent_inadvertent: 12
      reckless_deliberate: 5
      reckless_inadvertent: 20
    estimated_total_effort: "3-4 developer-weeks"

  quadrant_analysis:
    prudent_deliberate:
      count: 8
      examples:
        - "Hardcoded pagination limit (ticket TECH-123)"
      strategy: "Track explicitly, schedule in roadmap"

    prudent_inadvertent:
      count: 12
      examples:
        - "Discovered repository pattern after initial implementation"
      strategy: "Refactor incrementally, update team knowledge"

    reckless_deliberate:
      count: 5
      examples:
        - "Skipped tests for payment integration"
      strategy: "Urgent remediation, process review"
      risk_assessment: "High risk - payment flow untested"

    reckless_inadvertent:
      count: 20
      examples:
        - "N+1 queries in user listing"
      strategy: "Training, pair programming, code review improvement"
      knowledge_gaps:
        - "ORM eager loading patterns"
        - "Input validation best practices"

  prioritized_backlog:
    immediate:
      - "Add tests for payment integration"
      - "Fix SQL injection in search endpoint"
    short_term:
      - "Implement eager loading for user queries"
      - "Add input validation middleware"
    medium_term:
      - "Refactor to repository pattern"
      - "Improve test coverage to 80%"
    long_term:
      - "Migrate to event-driven architecture"

  root_cause_summary:
    - cause: "Missing input validation layer"
      debt_items: ["F-001", "F-003", "F-007"]
      systemic: true
      recommendation: "Implement centralized validation middleware"

    - cause: "Lack of ORM best practices knowledge"
      debt_items: ["F-012", "F-015", "F-018"]
      systemic: true
      recommendation: "Team training on SQLAlchemy patterns"
```

## Output Schema

```yaml
technical_debt:
  summary:
    total_items: number
    by_category: {}
    by_severity: {}
    by_quadrant:
      prudent_deliberate: number
      prudent_inadvertent: number
      reckless_deliberate: number
      reckless_inadvertent: number
    estimated_total_effort: string

  items:
    - id: string
      category: security | reliability | maintainability | performance
      severity: critical | high | medium | low
      quadrant: prudent_deliberate | prudent_inadvertent | reckless_deliberate | reckless_inadvertent
      description: string
      location: string
      impact: string
      effort: S | M | L | XL
      recommendation: string
      root_cause: string

  quadrant_analysis:
    prudent_deliberate:
      count: number
      examples: [string]
      strategy: string

    prudent_inadvertent:
      count: number
      examples: [string]
      strategy: string

    reckless_deliberate:
      count: number
      examples: [string]
      strategy: string
      risk_assessment: string

    reckless_inadvertent:
      count: number
      examples: [string]
      strategy: string
      knowledge_gaps: [string]

  prioritized_backlog:
    immediate: [string]
    short_term: [string]
    medium_term: [string]
    long_term: [string]

  root_cause_summary:
    - cause: string
      debt_items: [string]
      systemic: boolean
      recommendation: string
```

## Quality Indicators

- ✅ Majority in Prudent quadrants (healthy team)
- ✅ Clear root causes identified
- ✅ Actionable remediation plan
- 🚩 High Reckless-Deliberate count (process problem)
- 🚩 High Reckless-Inadvertent count (knowledge problem)
- 🚩 Same root cause across multiple findings (systemic issue)

## AI Prompt Template

```
Analyze all findings from previous viewpoints:
[FINDINGS]

For each finding, determine:
1. Fowler Quadrant classification (with reasoning)
2. Root cause (why does this debt exist?)
3. Is this part of a pattern (systemic issue)?
4. Impact if not addressed
5. Remediation effort

Then synthesize:
1. Top root causes (fix these → resolve multiple symptoms)
2. Quadrant distribution (team health indicator)
3. Prioritized backlog with rationale
4. Knowledge gaps (for reckless-inadvertent items)
```

## Report Quality Checklist

- [ ] Executive summary fits on 2 pages
- [ ] Root causes are actionable (not just "fix bugs")
- [ ] Fowler Quadrant distribution analyzed
- [ ] Knowledge gaps identified for reckless-inadvertent items
- [ ] Recommendations have clear owners
- [ ] Findings include specific file locations
- [ ] Technical debt is quantified

## Root Cause Quality Criteria

A good root cause:
- **Explains WHY** issues exist (not just WHAT)
- **Groups related** findings together
- **Points to** systemic issues
- **Enables** targeted remediation
- **Reduces** cognitive load vs raw findings

Example:
```
❌ Bad: "There are 15 security findings"
✅ Good: "Insufficient input validation in API layer causes 15 injection vulnerabilities across user, order, and payment endpoints. Root cause: No centralized validation middleware. Quadrant: Reckless-Inadvertent (team unaware of validation patterns)."
```

## Synthesis Complete

After VP-Q06:
- Mental model contains complete audit data
- Technical debt classified by Fowler Quadrant
- Root causes are synthesized
- Reports are generated
- Recommendations are prioritized
- Knowledge gaps identified

The audit is complete. Deliverables are ready for client presentation.

## Follow-up Actions

1. **Present findings** to stakeholders
2. **Address knowledge gaps** (training for reckless-inadvertent)
3. **Review processes** (if high reckless-deliberate)
4. **Create tickets** for immediate actions
5. **Plan sprints** for short-term improvements
6. **Schedule follow-up** audit (3-6 months)
7. **Track metrics** improvement over time

## Audit Success Metrics

- [ ] 3-5 root causes identified (not 100+ findings)
- [ ] Quadrant distribution analyzed
- [ ] Knowledge gaps documented
- [ ] All recommendations are actionable
- [ ] Client understands top priorities
- [ ] Estimated remediation effort provided
- [ ] Follow-up audit scheduled

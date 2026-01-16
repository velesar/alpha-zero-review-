---
name: vp-q06-synthesis
version: 1.0
dependencies: [vp-q01-security, vp-q02-performance, vp-q03-testability, vp-q04-code-style, vp-q05-documentation]
mcp_servers: [mental-model, methodology-kb]
---

# VP-Q06: Synthesis

## Purpose
Synthesize all findings into root causes, generate the final audit report, and provide actionable recommendations.

## Prerequisites
- All Quality viewpoints completed (VP-Q01 through VP-Q05)
- Findings accumulated in mental model

## Instructions

### Step 1: Verify Prerequisites
Call `mental-model/get_completed_viewpoints` and verify:
```
["VP-F01", "VP-F02", "VP-F03",
 "VP-S01", "VP-S02", "VP-S03", "VP-S04", "VP-S05", "VP-S06",
 "VP-Q01", "VP-Q02", "VP-Q03", "VP-Q04", "VP-Q05"]
```

If any viewpoints are missing, complete them first.

### Step 2: Get All Findings
Call `mental-model/get_findings` to retrieve all findings from quality viewpoints.

Review the findings:
- Total count
- Distribution by category
- Distribution by severity
- Distribution by affected area

### Step 3: Synthesize Root Causes

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

Alternative algorithm for location-based clustering:
```json
{
  "algorithm": "location_based"
}
```

### Step 4: Review Root Causes

Examine the synthesized root causes:
- Are they meaningful groupings?
- Do they identify real underlying issues?
- Are the recommendations actionable?

Aim for 3-5 root causes, not 50+ individual findings.

### Step 5: Get Report Template

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

### Step 6: Generate Executive Summary

Using the template, create the executive summary with:

#### Project Overview
- Project name and description
- Audit date
- Codebase size (from module hierarchy)
- Tech stack summary

#### Health Score
Calculate overall health score:
```
health_score = 100 - (critical_count * 15 + high_count * 8 + medium_count * 3 + low_count * 1)
```

Clamp to 0-100 range.

#### Category Health
For each category, calculate status:
- Security: Based on VP-Q01 findings
- Performance: Based on VP-Q02 findings
- Testability: Based on VP-Q03 findings
- Maintainability: Based on VP-Q04 findings
- Documentation: Based on VP-Q05 findings

#### Root Cause Summary
List top 3-5 root causes with:
- Title
- Impact level
- Affected finding count
- Key recommendation

### Step 7: Generate Recommendations

Prioritize recommendations using the matrix:

| Business Impact | Fix Effort | Priority |
|-----------------|------------|----------|
| High | Low | Immediate |
| High | High | Short-term |
| Low | Low | Short-term |
| Low | High | Long-term |

Categorize into:
- **Immediate Actions**: Quick wins, critical fixes
- **Short-term Improvements**: Within 2-4 weeks
- **Long-term Strategic**: Architectural changes

### Step 8: Generate Technical Report

Create detailed technical report with:
- All findings by category
- Detailed root cause analysis
- Code examples where relevant
- Specific file/line references

### Step 9: Generate Final Mental Model

Call `mental-model/get_model` to export the complete mental model.

This includes:
- All viewpoint data
- All findings with context
- Root causes
- Constraints derived

### Step 10: Create Deliverables

Generate the following files:

1. **executive_summary.md** - For stakeholders
2. **root_cause_analysis.md** - For technical leads
3. **detailed_findings.md** - For developers
4. **mental_model.yaml** - Complete audit data

## Output Structure

```
audit_output/
├── executive_summary.md      # Stakeholder report
├── root_cause_analysis.md    # Root causes + recommendations
├── detailed_findings.md      # All findings with details
├── mental_model.yaml         # Complete mental model
└── findings.json             # Raw findings data
```

## Report Quality Checklist

- [ ] Executive summary fits on 2 pages
- [ ] Root causes are actionable (not just "fix bugs")
- [ ] Recommendations have clear owners
- [ ] Findings include specific file locations
- [ ] Severity levels are consistent
- [ ] Context explains why issues matter
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
✅ Good: "Insufficient input validation in API layer causes 15 injection vulnerabilities across user, order, and payment endpoints. Root cause: No centralized validation middleware."
```

## Recommendation Quality Criteria

A good recommendation:
- **Specific**: What exactly to do
- **Measurable**: How to verify it's done
- **Achievable**: Within team's capability
- **Relevant**: Addresses the root cause
- **Time-bound**: Suggested timeframe

Example:
```
❌ Bad: "Improve security"
✅ Good: "Implement input validation middleware for all API endpoints using Pydantic models. Create validation schemas for CreateUserRequest, CreateOrderRequest, and PaymentRequest. Target: 100% API endpoint coverage within 2 weeks."
```

## Synthesis Complete

After VP-Q06:
- Mental model contains complete audit data
- Root causes are synthesized
- Reports are generated
- Recommendations are prioritized

The audit is complete. Deliverables are ready for client presentation.

## Follow-up Actions

1. **Present findings** to stakeholders
2. **Create tickets** for immediate actions
3. **Plan sprints** for short-term improvements
4. **Schedule follow-up** audit (3-6 months)
5. **Track metrics** improvement over time

## Audit Success Metrics

- [ ] 3-5 root causes identified (not 100+ findings)
- [ ] All recommendations are actionable
- [ ] Client understands top priorities
- [ ] Estimated remediation effort provided
- [ ] Follow-up audit scheduled

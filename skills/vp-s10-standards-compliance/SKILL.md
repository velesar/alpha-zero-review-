---
name: vp-s10-standards-compliance
version: 1.0
status: optional
dependencies: [vp-f01-tech-stack, vp-s02-layer-architecture]
mcp_servers: [mental-model, methodology-kb]
---

# VP-S10: Standards Compliance `[OPTIONAL]`

## Purpose
Verify compliance with technology standards, architecture principles, and security policies defined by the organization.

## When to Use

- Enterprise compliance audit
- Security certification preparation
- Vendor assessment
- Internal standards enforcement

## Standards Categories

| Category | Examples |
|----------|----------|
| Technology Standards | Approved languages, frameworks, databases |
| Architecture Principles | "Domain must be framework-agnostic" |
| Security Policies | "No secrets in code", "All endpoints authenticated" |
| Coding Standards | "100% type coverage", "Max complexity 10" |

## Input: Standards Definition

Standards should be defined in a YAML file:

```yaml
# standards/company_standards.yaml
technology_standards:
  approved_languages:
    - python: ">=3.11"
    - typescript: ">=5.0"
  approved_frameworks:
    backend: [fastapi, django]
    frontend: [react, nextjs]
  deprecated:
    - flask: "migrate by Q4 2026"
    - express: "migrate by Q2 2026"
  databases:
    approved: [postgresql, redis]
    restricted: [mongodb]  # Requires architecture review

architecture_principles:
  - id: AP-01
    name: "Domain Independence"
    rule: "Domain layer must not import from infrastructure"
    severity: high

  - id: AP-02
    name: "Explicit Dependencies"
    rule: "All dependencies via constructor injection"
    severity: medium

security_policies:
  - id: SP-01
    name: "No Hardcoded Secrets"
    rule: "No API keys, passwords in source code"
    severity: critical

  - id: SP-02
    name: "Authenticated by Default"
    rule: "All endpoints require authentication unless explicitly public"
    severity: high
```

## Instructions

### Step 1: Load Standards Definition

Check for standards file in:
- `standards/company_standards.yaml`
- `docs/standards.yaml`
- `.standards.yaml`

If no standards file exists, use default enterprise standards.

### Step 2: Check Technology Compliance

Verify approved technologies:

```bash
# Check language versions
python --version
node --version

# Check frameworks in dependencies
grep -E "fastapi|django|flask" requirements.txt
grep -E "express|nestjs|nextjs" package.json

# Check databases
grep -r "DATABASE_URL\|postgresql\|mongodb\|mysql" --include="*.env*"
```

### Step 3: Check Architecture Principles

For each principle, verify compliance:

**AP-01: Domain Independence**
```bash
# Check for forbidden imports in domain layer
grep -r "from infrastructure\|from fastapi\|from sqlalchemy" --include="*.py" src/domain/
```

**AP-02: Explicit Dependencies**
```bash
# Look for hidden instantiation
grep -r "= SomeService()\|Service.instance()" --include="*.py" --include="*.ts"
```

### Step 4: Check Security Policies

**SP-01: No Hardcoded Secrets**
```bash
# Check for potential secrets
grep -rE "api_key\s*=\s*['\"][^'\"]+['\"]|password\s*=\s*['\"][^'\"]+['\"]" --include="*.py" --include="*.ts"

# Use tools
semgrep --config p/secrets .
```

**SP-02: Authenticated Endpoints**
```bash
# Check for unauthenticated routes
grep -r "@router\." --include="*.py" | grep -v "Depends.*auth\|dependencies="
```

### Step 5: Check Deprecated Technologies

Look for deprecated technology usage:
```bash
# Check for deprecated frameworks
grep -r "from flask\|import flask" --include="*.py"
grep -r "express" package.json
```

### Step 6: Calculate Compliance Score

```
compliance_score = (compliant_checks / total_checks) * 100

Scoring:
- Critical violation: -20 points
- High violation: -10 points
- Medium violation: -5 points
- Low violation: -2 points
```

### Step 7: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S10"
data:
  overall_score: 85

  technology_compliance:
    compliant:
      - technology: "python"
        version: "3.11"
        status: "approved"
      - technology: "fastapi"
        version: "0.100.0"
        status: "approved"

    non_compliant:
      - technology: "flask"
        current_version: "2.0"
        required: "deprecated"
        severity: "medium"
        remediation: "Migrate to FastAPI by Q4 2026"

    deprecated_in_use:
      - technology: "requests"
        locations:
          - "src/infrastructure/http_client.py"
        migration_deadline: "Q2 2026"
        effort_estimate: "2 days"

  architecture_compliance:
    - principle_id: "AP-01"
      status: "compliant"
      evidence:
        compliant:
          - "src/domain/"
        violations: []
      remediation: null

    - principle_id: "AP-02"
      status: "partial"
      evidence:
        compliant:
          - "src/application/handlers/"
        violations:
          - path: "src/services/email_service.py"
            description: "Direct instantiation of SMTPClient"
      remediation: "Inject SMTPClient through constructor"

  security_compliance:
    - policy_id: "SP-01"
      status: "violated"
      violations:
        - path: "src/config/settings.py"
          line: 45
          description: "Hardcoded API key for development"
      severity: "critical"
      remediation: "Move to environment variables"

    - policy_id: "SP-02"
      status: "compliant"
      violations: []

  summary:
    critical_violations: 1
    high_violations: 0
    medium_violations: 1
    compliance_score: 85
    top_remediation_actions:
      - "Remove hardcoded API key from settings.py"
      - "Migrate Flask endpoints to FastAPI"
      - "Implement dependency injection for SMTPClient"
```

## Output Schema

```yaml
standards_compliance:
  overall_score: number  # percentage

  technology_compliance:
    compliant:
      - technology: string
        version: string
        status: approved
    non_compliant:
      - technology: string
        current_version: string
        required: string
        severity: string
        remediation: string
    deprecated_in_use:
      - technology: string
        locations: [string]
        migration_deadline: string
        effort_estimate: string

  architecture_compliance:
    - principle_id: string
      status: compliant | partial | violated
      evidence:
        compliant: [string]
        violations:
          - path: string
            description: string
      remediation: string

  security_compliance:
    - policy_id: string
      status: compliant | violated
      violations:
        - path: string
          line: number
          description: string
      severity: critical | high | medium | low
      remediation: string

  summary:
    critical_violations: number
    high_violations: number
    medium_violations: number
    compliance_score: number
    top_remediation_actions: [string]
```

## Quality Indicators

- ✅ 100% compliance with critical policies
- ✅ No deprecated technologies without migration plan
- ✅ All architecture principles followed
- 🚩 Critical security policy violations
- 🚩 Use of non-approved technologies
- 🚩 Systematic architecture principle violations

## Fitness Functions

```python
def check_no_hardcoded_secrets(project_path: str) -> bool:
    """SP-01: No hardcoded secrets in source code."""
    SECRET_PATTERNS = [
        r'api_key\s*=\s*["\'][^"\']+["\']',
        r'password\s*=\s*["\'][^"\']+["\']',
        r'secret\s*=\s*["\'][^"\']+["\']',
        r'AWS_SECRET_ACCESS_KEY\s*=\s*["\'][^"\']+["\']',
    ]

    violations = []
    for pattern in SECRET_PATTERNS:
        for py_file in glob(f"{project_path}/**/*.py", recursive=True):
            with open(py_file) as f:
                for i, line in enumerate(f, 1):
                    if re.search(pattern, line, re.IGNORECASE):
                        violations.append((py_file, i, line.strip()))

    assert len(violations) == 0, f"Hardcoded secrets found: {violations}"
    return True


def check_all_endpoints_authenticated(project_path: str) -> bool:
    """SP-02: All endpoints must have authentication."""
    PUBLIC_WHITELIST = ["/health", "/metrics", "/docs", "/openapi.json"]

    violations = []
    for route_file in glob(f"{project_path}/**/routes*.py", recursive=True):
        routes = extract_routes(route_file)
        for route in routes:
            if route["path"] in PUBLIC_WHITELIST:
                continue
            if not route.get("dependencies") or \
               "auth" not in str(route["dependencies"]).lower():
                violations.append(route)

    assert len(violations) == 0, \
        f"Unauthenticated endpoints: {[v['path'] for v in violations]}"
    return True
```

## Next Viewpoint

After VP-S10, the Structure phase is complete.
Proceed to **Quality Phase** starting with **VP-Q01: Security Analysis**.

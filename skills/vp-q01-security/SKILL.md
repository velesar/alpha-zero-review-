---
name: vp-q01-security
version: 1.0
dependencies: [vp-s02-layer-architecture, vp-s05-interface-surface, vp-s06-dependency-graph]
mcp_servers: [mental-model, methodology-kb]
---

# VP-Q01: Security Analysis

## Purpose
Identify security vulnerabilities and weaknesses in the codebase, with context-aware severity based on the mental model.

## Prerequisites
- Structure phase completed (VP-S01 through VP-S06)
- Constraints derived from mental model
- Interface surface documented

## Instructions

### Step 1: Get Analysis Constraints
Call `mental-model/get_constraints` to get:
- `security_focus_paths` - prioritized paths for security analysis
- `high_priority_paths` - paths requiring extra attention

### Step 2: Run Security Scanners

Based on language from VP-F01, run appropriate scanners:

#### Python - Bandit
```bash
# Scan with JSON output
bandit -r src/ -f json -o bandit_results.json

# Focus on high-confidence findings
bandit -r src/ -ll -f json
```

#### Python - Semgrep
```bash
# Run Python security rules
semgrep --config=p/python --json --output=semgrep_results.json src/
```

#### TypeScript/JavaScript - ESLint Security
```bash
# With security plugin
npx eslint --format json --output-file eslint_results.json src/
```

#### TypeScript - Semgrep
```bash
semgrep --config=p/typescript --json src/
```

#### Rust - Cargo Audit + Clippy
```bash
cargo audit --json
cargo clippy -- -W clippy::pedantic 2>&1
```

### Step 3: Focus on Priority Paths

Filter results to focus on:
1. `security_focus_paths` from constraints
2. API/adapter layers (external exposure)
3. Authentication/authorization code
4. Data handling code

### Step 4: Classify Each Finding

For each security finding, call `methodology-kb/classify_finding`:

```json
{
  "tool": "bandit",
  "rule_id": "B105",
  "context": {
    "bounded_context_type": "core",
    "layer": "domain",
    "is_hotspot": true
  }
}
```

This returns adjusted severity based on context.

### Step 5: Get Context for Each Finding

For findings in key files, call `mental-model/get_context`:
```json
{
  "file_path": "src/domain/auth/handler.py"
}
```

This provides:
- Bounded context type (core/supporting/generic)
- Architecture layer
- Hotspot status

### Step 6: Add Enriched Findings

For each significant finding, call `mental-model/add_finding`:

```json
{
  "viewpoint": "VP-Q01",
  "category": "security_injection",
  "title": "Potential SQL injection in user query",
  "description": "User input is concatenated directly into SQL query without parameterization",
  "file_path": "src/data/user_repository.py",
  "line_number": 45,
  "base_severity": "HIGH",
  "rule_id": "B608",
  "recommendation": "Use parameterized queries or ORM methods"
}
```

### Step 7: Security Categories to Check

#### Injection (OWASP A03)
- SQL injection
- Command injection
- LDAP injection
- XPath injection
- NoSQL injection

#### Broken Authentication (OWASP A07)
- Hardcoded credentials
- Weak password requirements
- Missing MFA
- Session fixation

#### Sensitive Data Exposure (OWASP A02)
- Unencrypted data
- Weak cryptography
- Secrets in code
- Logging sensitive data

#### XXE (OWASP A05)
- Unsafe XML parsing
- External entity resolution

#### Broken Access Control (OWASP A01)
- Missing authorization checks
- IDOR vulnerabilities
- Privilege escalation

#### Security Misconfiguration (OWASP A05)
- Debug mode in production
- Default credentials
- Missing security headers

#### XSS (OWASP A03)
- Reflected XSS
- Stored XSS
- DOM-based XSS

#### Insecure Deserialization (OWASP A08)
- Pickle usage
- Unsafe JSON deserialization
- Type confusion

#### SSRF
- User-controlled URLs
- Internal service exposure

### Step 8: Manual Review Points

In addition to automated scanning, manually review:

1. **Authentication flow**
   - Login implementation
   - Token validation
   - Session management

2. **Authorization checks**
   - Permission validation
   - Role-based access
   - Resource ownership

3. **Input handling**
   - User input validation
   - File upload handling
   - URL parameter handling

4. **Cryptography**
   - Password hashing
   - Token generation
   - Data encryption

5. **Error handling**
   - Error message content
   - Stack trace exposure
   - Information leakage

## Output Integration

All findings added via `mental-model/add_finding` will be:
- Enriched with context
- Severity adjusted based on location
- Available for synthesis in VP-Q06

## Quality Indicators

- ✅ **Healthy**: 0 critical, ≤3 high, all in generic contexts
- ⚠️ **Warning**: 1-2 critical, ≤10 high, some in core contexts
- ❌ **Critical**: >2 critical in core contexts, authentication issues

## OWASP Top 10 Coverage

| OWASP ID | Category | Tools/Checks |
|----------|----------|--------------|
| A01 | Broken Access Control | Manual review, authorization audit |
| A02 | Cryptographic Failures | Bandit, secrets detection |
| A03 | Injection | Semgrep, Bandit, SQLi checks |
| A04 | Insecure Design | Architecture review |
| A05 | Security Misconfiguration | Config review, Semgrep |
| A06 | Vulnerable Components | Dependency scanning |
| A07 | Authentication Failures | Manual review, Bandit |
| A08 | Data Integrity Failures | Deserialization checks |
| A09 | Logging Failures | Log review, sensitive data |
| A10 | SSRF | Semgrep, URL handling |

## Next Viewpoint
After completing VP-Q01, proceed to **VP-Q02: Performance Analysis**.

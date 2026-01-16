---
name: vp-q04-code-style
version: 1.0
dependencies: [vp-f01-tech-stack, vp-s06-hotspots]
mcp_servers: [mental-model, methodology-kb]
---

# VP-Q04: Code Style Analysis

## Purpose
Analyze code quality metrics including complexity, duplication, and adherence to language conventions.

## Prerequisites
- Tech stack identified
- Hotspots identified for prioritization

## Instructions

### Step 1: Get Context
Call `mental-model/get_model` to get:
- `tech_stack.primary_language`
- `hotspots.files` - priority files for analysis
- `constraints.high_priority_paths`

### Step 2: Measure Complexity Metrics

#### Python - Radon
```bash
# Cyclomatic complexity (CC)
radon cc src/ --json --total-average

# Cognitive complexity (requires radon >= 5.1)
# Or use flake8-cognitive-complexity

# Maintainability index
radon mi src/ --json
```

#### TypeScript - ESLint Complexity
```bash
# With complexity rule enabled
npx eslint src/ --rule 'complexity: ["error", 10]' --format json
```

#### Rust - Cargo Clippy
```bash
cargo clippy -- -W clippy::cognitive_complexity
```

### Step 3: Detect Code Duplication

#### Python - Pylint/Flake8
```bash
# Pylint duplicate code detection
pylint --disable=all --enable=duplicate-code src/
```

#### TypeScript - jscpd
```bash
npx jscpd src/ --min-lines 5 --format json
```

#### General - PMD CPD
```bash
pmd cpd --minimum-tokens 50 --language python --files src/
```

### Step 4: Run Linters

#### Python
```bash
# Flake8
flake8 src/ --format=json

# Pylint
pylint src/ --output-format=json

# Ruff (fast, modern)
ruff check src/ --format json
```

#### TypeScript
```bash
# ESLint
npx eslint src/ --format json
```

#### Rust
```bash
cargo clippy --message-format=json
```

### Step 5: Check Naming Conventions

Verify:
- Classes use PascalCase
- Functions/methods use appropriate case (snake_case for Python, camelCase for TS)
- Constants use UPPER_SNAKE_CASE
- Variables are descriptive
- Abbreviations are consistent

### Step 6: Analyze File/Function Size

Check for:
- Files > 500 lines
- Functions > 50 lines
- Classes > 300 lines
- Too many parameters (> 5)
- Deep nesting (> 4 levels)

### Step 7: Get Thresholds
Call `methodology-kb/get_thresholds`:
```json
{
  "project_type": "mature",
  "language": "python"
}
```

Compare metrics against thresholds.

### Step 8: Add Findings

For code style issues, call `mental-model/add_finding`:

```json
{
  "viewpoint": "VP-Q04",
  "category": "maintainability_complexity",
  "title": "High cognitive complexity in order processing",
  "description": "Function process_order has cognitive complexity of 35 (threshold: 15)",
  "file_path": "src/services/order_service.py",
  "line_number": 120,
  "base_severity": "MEDIUM",
  "recommendation": "Extract complex conditions into named methods, reduce nesting depth"
}
```

```json
{
  "viewpoint": "VP-Q04",
  "category": "maintainability_duplication",
  "title": "Duplicated validation logic",
  "description": "45 lines of duplicate code between user_validator.py and order_validator.py",
  "file_path": "src/validators/user_validator.py",
  "line_number": 30,
  "base_severity": "LOW",
  "recommendation": "Extract common validation logic into shared module"
}
```

## Code Style Categories

| Category | Description |
|----------|-------------|
| `maintainability_complexity` | High cyclomatic/cognitive complexity |
| `maintainability_duplication` | Code duplication |
| `maintainability_naming` | Poor naming conventions |
| `maintainability_size` | Oversized files/functions |

## Quality Indicators

- ✅ **Healthy**: Low complexity, minimal duplication, consistent style
- ⚠️ **Warning**: Some complexity issues, occasional duplication
- ❌ **Critical**: High complexity in core code, significant duplication

## Complexity Thresholds

| Metric | Good | Warning | Critical |
|--------|------|---------|----------|
| Cyclomatic Complexity | ≤10 | 11-20 | >20 |
| Cognitive Complexity | ≤15 | 16-25 | >25 |
| Maintainability Index | ≥65 | 45-64 | <45 |
| Function Lines | ≤30 | 31-50 | >50 |
| File Lines | ≤300 | 301-500 | >500 |
| Parameters | ≤4 | 5-6 | >6 |
| Nesting Depth | ≤3 | 4 | >4 |

## Common Code Style Issues

| Issue | Impact | Solution |
|-------|--------|----------|
| Deep nesting | Hard to read/maintain | Extract methods, early returns |
| Long functions | Hard to test/understand | Split into focused functions |
| Magic numbers | Unclear meaning | Use named constants |
| Duplicated code | Maintenance burden | Extract to shared function |
| Inconsistent naming | Confusion | Apply naming conventions |
| God class | Too many responsibilities | Split into focused classes |

## Style Improvement Strategies

### Reducing Complexity
```python
# Before: Deep nesting
def process(data):
    if data:
        if data.valid:
            if data.type == "A":
                # process A
            else:
                # process B

# After: Early returns, extracted methods
def process(data):
    if not data or not data.valid:
        return

    if data.type == "A":
        return process_type_a(data)
    return process_type_b(data)
```

### Reducing Duplication
```python
# Before: Duplicated validation
def validate_user(user):
    if not user.email:
        raise ValueError("Email required")
    if not re.match(EMAIL_REGEX, user.email):
        raise ValueError("Invalid email")

def validate_contact(contact):
    if not contact.email:
        raise ValueError("Email required")
    if not re.match(EMAIL_REGEX, contact.email):
        raise ValueError("Invalid email")

# After: Shared validation
def validate_email(email):
    if not email:
        raise ValueError("Email required")
    if not re.match(EMAIL_REGEX, email):
        raise ValueError("Invalid email")
```

## Next Viewpoint
After completing VP-Q04, proceed to **VP-Q05: Documentation Analysis**.

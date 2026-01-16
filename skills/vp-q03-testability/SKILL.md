---
name: vp-q03-testability
version: 1.0
dependencies: [vp-f02-structure, vp-s02-layer-architecture, vp-s06-hotspots]
mcp_servers: [mental-model, methodology-kb]
---

# VP-Q03: Testability Analysis

## Purpose
Assess test coverage, test quality, and code testability to ensure adequate testing protects code quality.

## Prerequisites
- Structure phase completed
- Test roots identified in VP-F02
- Hotspots identified for coverage priority

## Instructions

### Step 1: Get Context
Call `mental-model/get_model` to get:
- `structure.test_roots` - test locations
- `hotspots.files` - priority coverage targets
- `constraints.coverage_critical_paths` - must-have coverage

### Step 2: Measure Test Coverage

Run coverage tools based on language:

#### Python - Coverage.py/Pytest-cov
```bash
# Run tests with coverage
pytest --cov=src --cov-report=json --cov-report=html tests/

# Get coverage report
coverage report --show-missing
```

#### TypeScript/JavaScript - Jest/Istanbul
```bash
# Run tests with coverage
npm test -- --coverage --coverageReporters=json

# Or with Jest
jest --coverage
```

#### Rust - Cargo Tarpaulin
```bash
cargo tarpaulin --out Json --out Html
```

### Step 3: Analyze Coverage Metrics

Collect these metrics:
- **Line Coverage**: % of code lines executed
- **Branch Coverage**: % of decision branches tested
- **Function Coverage**: % of functions called
- **File Coverage**: Files with/without tests

### Step 4: Check Hotspot Coverage

Cross-reference hotspots with coverage:
```
For each hotspot:
  - Is there a corresponding test file?
  - What is the file's coverage percentage?
  - Are critical paths tested?
```

Files that are hotspots AND have low coverage are highest priority.

### Step 5: Assess Test Quality

Beyond coverage numbers, assess:

#### Test Isolation
```python
# Bad - tests depend on each other
class TestUser:
    def test_create(self):
        self.user = create_user()  # State shared

    def test_update(self):
        self.user.name = "new"  # Depends on test_create

# Good - independent tests
class TestUser:
    def test_create(self):
        user = create_user()
        assert user.id is not None

    def test_update(self):
        user = create_user()  # Fresh setup
        user.name = "new"
        assert user.name == "new"
```

#### Test Clarity
- Clear test names describing behavior
- Arrange-Act-Assert pattern
- Minimal setup/teardown

#### Test Types Present
- Unit tests (isolated)
- Integration tests (component interaction)
- E2E tests (full flow)
- API tests (endpoint testing)

### Step 6: Identify Testability Issues

#### Hard to Test Patterns

**Global State:**
```python
# Bad - uses global
def get_config():
    return GLOBAL_CONFIG["setting"]

# Good - injectable
def get_config(config_provider):
    return config_provider.get("setting")
```

**Hidden Dependencies:**
```python
# Bad - hidden dependency
def send_notification(user):
    client = EmailClient()  # Hidden creation
    client.send(user.email, "Hello")

# Good - injected dependency
def send_notification(user, email_client):
    email_client.send(user.email, "Hello")
```

**Static Methods/Singletons:**
```python
# Bad - hard to mock
def process_order(order):
    PaymentService.instance().process(order)

# Good - injectable
def process_order(order, payment_service):
    payment_service.process(order)
```

**Time Dependencies:**
```python
# Bad - non-deterministic
def is_expired(item):
    return item.expires_at < datetime.now()

# Good - injectable clock
def is_expired(item, clock=datetime.now):
    return item.expires_at < clock()
```

### Step 7: Check Test Infrastructure

Verify:
- Tests run in CI/CD
- Test database/fixtures available
- Mocking infrastructure in place
- Test configuration separate from production

### Step 8: Get Thresholds
Call `methodology-kb/get_thresholds`:
```json
{
  "project_type": "mature",
  "language": "python"
}
```

Compare coverage against thresholds.

### Step 9: Add Findings

For testability issues, call `mental-model/add_finding`:

```json
{
  "viewpoint": "VP-Q03",
  "category": "testability_coverage",
  "title": "Critical hotspot lacks test coverage",
  "description": "order_service.py is a critical hotspot (score: 85) but has only 23% test coverage",
  "file_path": "src/services/order_service.py",
  "base_severity": "HIGH",
  "recommendation": "Add unit tests for order processing logic, especially error paths"
}
```

```json
{
  "viewpoint": "VP-Q03",
  "category": "testability_coupling",
  "title": "Hidden dependency prevents unit testing",
  "description": "PaymentService instantiated directly in order handler, preventing isolation testing",
  "file_path": "src/handlers/order_handler.py",
  "line_number": 45,
  "base_severity": "MEDIUM",
  "recommendation": "Inject PaymentService dependency through constructor or parameter"
}
```

## Testability Categories

| Category | Description |
|----------|-------------|
| `testability_coverage` | Insufficient test coverage |
| `testability_coupling` | Tight coupling preventing testing |
| `testability_quality` | Poor test quality or patterns |
| `testability_infrastructure` | Missing test infrastructure |

## Quality Indicators

- ✅ **Healthy**: Coverage ≥80%, hotspots covered, good isolation
- ⚠️ **Warning**: Coverage 60-80%, some hotspots uncovered
- ❌ **Critical**: Coverage <60%, hotspots untested, no isolation

## Coverage Targets by Area

| Area | Minimum | Recommended |
|------|---------|-------------|
| Core Domain | 90% | 95% |
| Application Services | 80% | 90% |
| API Handlers | 70% | 85% |
| Infrastructure | 60% | 75% |
| Utilities | 50% | 70% |
| Hotspots | 85% | 95% |

## Test Quality Checklist

- [ ] Tests are independent (no shared state)
- [ ] Tests have clear names describing behavior
- [ ] Tests follow Arrange-Act-Assert pattern
- [ ] Tests use appropriate assertions
- [ ] Tests cover happy path and error cases
- [ ] Tests don't test implementation details
- [ ] Tests run quickly (unit < 100ms)
- [ ] Integration tests are isolated from unit tests

## Next Viewpoint
After completing VP-Q03, proceed to **VP-Q04: Code Style Analysis**.

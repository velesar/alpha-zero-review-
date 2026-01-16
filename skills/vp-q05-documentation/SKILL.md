---
name: vp-q05-documentation
version: 1.0
dependencies: [vp-s05-interface-surface]
mcp_servers: [mental-model, methodology-kb]
---

# VP-Q05: Documentation Analysis

## Purpose
Assess the state of documentation including API docs, code comments, and project documentation.

## Prerequisites
- Interface surface documented (VP-S05)
- Public API endpoints identified

## Instructions

### Step 1: Get Context
Call `mental-model/get_model` to get:
- `interface_surface.api_endpoints` - APIs needing docs
- `interface_surface.public_modules` - public interfaces
- `interface_surface.exported_types` - public types

### Step 2: Check API Documentation

#### OpenAPI/Swagger
Look for:
- `openapi.yaml`, `swagger.yaml`, `openapi.json`
- Generated docs from code annotations
- API documentation site

Verify coverage:
- All endpoints documented
- Request/response schemas defined
- Error responses documented
- Authentication documented

#### Python - FastAPI
FastAPI generates OpenAPI automatically. Check:
```python
# Endpoints should have docstrings
@router.get("/users/{user_id}")
async def get_user(user_id: int) -> User:
    """
    Get user by ID.

    - **user_id**: The unique identifier of the user
    - Returns: User object
    - Raises: 404 if user not found
    """
```

#### TypeScript - NestJS
```typescript
// Swagger decorators
@ApiTags('users')
@ApiOperation({ summary: 'Get user by ID' })
@ApiResponse({ status: 200, description: 'User found', type: User })
@ApiResponse({ status: 404, description: 'User not found' })
@Get(':id')
findOne(@Param('id') id: string): User {
```

### Step 3: Check Code Documentation

#### Docstrings/JSDoc Coverage

**Python:**
```python
# Look for docstrings on:
# - Modules (top of file)
# - Classes
# - Public methods
# - Complex functions

def complex_algorithm(data: List[Item], threshold: float) -> Result:
    """
    Process items using adaptive threshold algorithm.

    Args:
        data: List of items to process
        threshold: Minimum score for inclusion

    Returns:
        Result containing processed items and statistics

    Raises:
        ValueError: If threshold is not in range [0, 1]
    """
```

**TypeScript:**
```typescript
/**
 * Process items using adaptive threshold algorithm.
 *
 * @param data - List of items to process
 * @param threshold - Minimum score for inclusion
 * @returns Result containing processed items and statistics
 * @throws {ValidationError} If threshold is not in range [0, 1]
 */
function complexAlgorithm(data: Item[], threshold: number): Result {
```

**Rust:**
```rust
/// Process items using adaptive threshold algorithm.
///
/// # Arguments
///
/// * `data` - List of items to process
/// * `threshold` - Minimum score for inclusion
///
/// # Returns
///
/// Result containing processed items and statistics
///
/// # Errors
///
/// Returns `ValidationError` if threshold is not in range [0, 1]
pub fn complex_algorithm(data: Vec<Item>, threshold: f64) -> Result<Output, Error> {
```

### Step 4: Check Project Documentation

Look for and assess:

#### README.md
- [ ] Project description
- [ ] Installation instructions
- [ ] Quick start guide
- [ ] Configuration options
- [ ] Contributing guidelines
- [ ] License

#### Architecture Documentation
- [ ] System overview
- [ ] Component diagrams
- [ ] Data flow documentation
- [ ] Decision records (ADRs)

#### Development Documentation
- [ ] Local setup guide
- [ ] Testing instructions
- [ ] Deployment guide
- [ ] Troubleshooting guide

### Step 5: Identify Documentation Gaps

Compare:
1. Public APIs vs documented APIs
2. Complex functions vs documented functions
3. Configuration options vs documented options
4. External dependencies vs documented setup

### Step 6: Add Findings

For documentation issues, call `mental-model/add_finding`:

```json
{
  "viewpoint": "VP-Q05",
  "category": "documentation_api",
  "title": "Undocumented API endpoint",
  "description": "POST /api/orders endpoint lacks OpenAPI documentation and parameter descriptions",
  "file_path": "src/api/orders.py",
  "line_number": 45,
  "base_severity": "LOW",
  "recommendation": "Add FastAPI description, parameter docs, and response schemas"
}
```

```json
{
  "viewpoint": "VP-Q05",
  "category": "documentation_code",
  "title": "Complex algorithm lacks explanation",
  "description": "risk_calculation function (CC: 25) has no docstring explaining the algorithm",
  "file_path": "src/services/risk_service.py",
  "line_number": 89,
  "base_severity": "LOW",
  "recommendation": "Add docstring explaining algorithm, parameters, and edge cases"
}
```

## Documentation Categories

| Category | Description |
|----------|-------------|
| `documentation_api` | Missing/incomplete API documentation |
| `documentation_code` | Missing code comments/docstrings |
| `documentation_project` | Missing project-level documentation |
| `documentation_architecture` | Missing architecture documentation |

## Quality Indicators

- ✅ **Healthy**: All APIs documented, complex code explained, README complete
- ⚠️ **Warning**: Some API gaps, partial docstrings, basic README
- ❌ **Critical**: APIs undocumented, no docstrings, no README

## Documentation Metrics

| Metric | Good | Warning | Critical |
|--------|------|---------|----------|
| API Coverage | 100% | 80-99% | <80% |
| Public Function Docs | 90% | 70-89% | <70% |
| README Completeness | All sections | Most sections | Missing key sections |
| Architecture Docs | Current | Outdated | Missing |

## Documentation Priorities

Prioritize documenting:
1. **Public APIs** - External consumers depend on these
2. **Complex algorithms** - Future maintainers need understanding
3. **Configuration** - Operations team needs this
4. **Integration points** - How to connect with other systems
5. **Error handling** - What can go wrong and how to handle it

## Documentation vs Comments

### Document (Yes)
- Public API contracts
- Complex business logic
- Non-obvious design decisions
- Integration requirements
- Error scenarios

### Don't Over-Document (No)
- Obvious code (self-documenting)
- Implementation details that change
- Comments that repeat code
- Outdated information

## Good Documentation Examples

```python
# API endpoint - Good
@router.post(
    "/orders",
    response_model=OrderResponse,
    status_code=201,
    summary="Create new order",
    description="Creates a new order for the authenticated user",
    responses={
        400: {"description": "Invalid order data"},
        401: {"description": "Not authenticated"},
        422: {"description": "Validation error"},
    },
)
async def create_order(order: CreateOrderRequest) -> OrderResponse:
    """
    Create a new order.

    The order will be validated against inventory and user credit limits.
    If successful, an order confirmation email will be sent.

    Raises:
        InsufficientInventoryError: If items are out of stock
        CreditLimitExceededError: If order exceeds user's credit limit
    """
```

## Next Viewpoint
After completing VP-Q05, proceed to **VP-Q06: Synthesis**.

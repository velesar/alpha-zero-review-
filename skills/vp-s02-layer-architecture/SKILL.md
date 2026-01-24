---
name: vp-s02-layer-architecture
version: 2.0
dependencies: [vp-f01-tech-stack, vp-f02-structure, vp-s01-module-hierarchy]
mcp_servers: [mental-model, methodology-kb, codegraph]
---

# VP-S02: Layer Architecture Analysis

## Purpose
Identify the architectural layering pattern used in the project and validate that layer boundaries are respected.

## Prerequisites
- VP-S01 (Module Hierarchy) completed
- Verify by calling `mental-model/get_model` and confirming module_hierarchy is populated
- (Optional) SCIP indexes in `.audit/indexes/` for semantic analysis

## Instructions

### Step 0: Load SCIP Indexes (If Available)

If the project has SCIP indexes (created with `setup-audit --with-index`), load them first:

```
codegraph/load_project_indexes
  project_path: "."
  build_if_missing: false
```

This enables semantic layer violation detection:
- `get_callers(symbol_id)` - Find cross-layer dependencies
- `get_module_deps(module_path)` - Get layer dependencies
- `find_symbol(pattern)` - Search for symbols in specific layers

### Step 1: Get Current Context
Call `mental-model/get_model`, extract:
- `tech_stack.framework` - helps identify expected patterns
- `module_hierarchy.modules` - structure to analyze

### Step 2: Detect Architecture Pattern

Analyze directory structure for common patterns:

#### Clean Architecture
Look for these directories:
- `domain/`, `entities/`, `core/` → Domain layer
- `application/`, `use_cases/`, `usecases/` → Application layer
- `adapters/`, `interfaces/`, `controllers/` → Adapter layer
- `infrastructure/`, `frameworks/` → Infrastructure layer

Characteristics:
- Dependencies point inward
- Domain has no dependencies on outer layers
- Use cases orchestrate domain entities

#### Layered Architecture
Look for:
- `presentation/`, `views/`, `ui/` → Presentation layer
- `controllers/`, `api/`, `handlers/` → Application layer
- `services/`, `business/` → Business/Service layer
- `data/`, `repositories/`, `models/` → Data layer

Characteristics:
- Dependencies flow downward
- Each layer uses only the layer below

#### Hexagonal Architecture (Ports & Adapters)
Look for:
- `domain/`, `core/` → Core hexagon
- `ports/`, `interfaces/` → Port definitions
- `adapters/inbound/`, `adapters/primary/` → Driving adapters
- `adapters/outbound/`, `adapters/secondary/` → Driven adapters

Characteristics:
- Core defines ports (interfaces)
- Adapters implement ports
- No adapter-to-adapter dependencies

#### MVC Pattern
Look for:
- `models/` → Model layer
- `views/`, `templates/` → View layer
- `controllers/`, `handlers/` → Controller layer

### Step 3: Map Detected Layers

For each layer identified:
1. Name the layer
2. List paths/directories belonging to it
3. Define its purpose
4. Identify allowed dependencies

### Step 4: Validate Layer Compliance

Call `methodology-kb/check_compliance` with:
```json
{
  "standard": "clean_architecture|layered_architecture|hexagonal_architecture",
  "detected_pattern": {
    "layers": [
      {"name": "domain", "paths": ["src/domain"]},
      {"name": "application", "paths": ["src/application"]}
    ],
    "violations": []
  }
}
```

### Step 5: Find Layer Violations

#### Using Codegraph (Preferred if SCIP Index Loaded)

For semantic layer violation detection:

```
# Find what domain layer symbols depend on
codegraph/get_module_deps
  module_path: "src/domain"

# Check if domain symbols reference infrastructure
codegraph/find_symbol
  pattern: "database"

# For suspicious symbols, trace callers across layers
codegraph/get_callers
  symbol_id: "<symbol_from_find>"
```

This provides:
- Exact symbol-level cross-layer dependencies
- Distinction between type-only and runtime violations
- Full reference chain for each violation

#### Manual Fallback (Without SCIP Index)

Analyze imports/dependencies from VP-S01 to find violations:

#### Violation Types

**Dependency Inversion Violation:**
Inner layer depends on outer layer
```python
# In domain/entity.py (BAD)
from infrastructure.database import Session
```

**Layer Skip Violation:**
Layer bypasses intermediate layer
```python
# In presentation/view.py (BAD - skipping service layer)
from data.repository import UserRepository
```

**Circular Layer Dependency:**
Layers depend on each other
```python
# In service/user_service.py
from controller.auth import get_current_user  # BAD
```

For each violation, document:
- Source layer and file
- Target layer and file
- Import/dependency statement
- Line number if available
- Why it's a violation

### Step 6: Assess Confidence

- **High**: Clear layer structure, matches known pattern, few violations
- **Medium**: Pattern recognizable but some inconsistencies
- **Low**: Unclear structure, multiple patterns mixed, many violations

### Step 7: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S02"
data:
  pattern: "clean_architecture"
  confidence: "high|medium|low"
  layers:
    - name: "domain"
      paths:
        - "src/domain"
      purpose: "Business entities and rules"
      allowed_dependencies: []
    - name: "application"
      paths:
        - "src/application"
        - "src/use_cases"
      purpose: "Use case orchestration"
      allowed_dependencies:
        - "domain"
    - name: "adapters"
      paths:
        - "src/api"
        - "src/controllers"
      purpose: "External interface adapters"
      allowed_dependencies:
        - "domain"
        - "application"
    - name: "infrastructure"
      paths:
        - "src/infrastructure"
        - "src/db"
      purpose: "Frameworks and drivers"
      allowed_dependencies:
        - "domain"
        - "application"
        - "adapters"
  violations:
    - from_layer: "domain"
      to_layer: "infrastructure"
      file_path: "src/domain/user.py"
      import_path: "src/infrastructure/database"
      line_number: 5
      description: "Domain entity depends on infrastructure"
```

## Output Schema

```yaml
architecture:
  pattern: string           # Detected pattern name
  confidence: high | medium | low
  layers:
    - name: string
      paths: [string]
      purpose: string
      allowed_dependencies: [string]
  violations:
    - from_layer: string
      to_layer: string
      file_path: string
      import_path: string
      line_number: number
      description: string
```

## Quality Indicators

- ✅ **Healthy**: Clear boundaries, 0-2 violations, high confidence
- ⚠️ **Warning**: Minor violations (3-10), medium confidence
- ❌ **Critical**: Many violations (>10), circular dependencies, low confidence

## Framework-Specific Patterns

### FastAPI (Python)
Typical Clean/Layered hybrid:
```
src/
├── domain/         # Domain models, entities
├── application/    # Use cases, services
├── api/           # FastAPI routers (adapters)
└── infrastructure/ # Database, external services
```

### NestJS (TypeScript)
Module-based with layering:
```
src/
├── modules/
│   └── users/
│       ├── domain/
│       ├── application/
│       ├── infrastructure/
│       └── users.module.ts
└── common/
```

### Django (Python)
MVT (Model-View-Template):
```
app/
├── models/        # Data layer
├── views/         # Business logic
├── templates/     # Presentation
├── serializers/   # API adapters
└── services/      # Optional service layer
```

## Next Viewpoint
After completing VP-S02, proceed to **VP-S03: Domain Model Analysis**.

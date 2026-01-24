---
name: vp-s01-module-hierarchy
version: 2.0
dependencies: [vp-f01-tech-stack, vp-f02-structure]
mcp_servers: [mental-model, methodology-kb, codegraph]
---

# VP-S01: Module Hierarchy Analysis

## Purpose
Map the module/package hierarchy of the project to understand code organization, identify modules, and detect circular dependencies.

## Prerequisites
- Foundation phase completed (VP-F01, VP-F02, VP-F03)
- Source roots identified
- (Optional) SCIP indexes in `.audit/indexes/` for semantic analysis

## Instructions

### Step 0: Load SCIP Indexes (If Available)

If the project has SCIP indexes (created with `setup-audit --with-index`), load them first:

```
codegraph/load_project_indexes
  project_path: "."
  build_if_missing: false
```

This enables automated module analysis:
- `get_module_deps(module_path)` - Get all dependencies of a module
- `get_file_symbols(file_path)` - Get all exports from a module
- `find_hotspot_symbols(min_callers)` - Find heavily-imported modules (god modules)

### Step 1: Verify Prerequisites
Call `mental-model/get_model` and confirm:
- `structure.source_roots` is populated
- `tech_stack.primary_language` is known

### Step 2: Map Module Structure

Based on the language from VP-F01:

#### Python
- Each directory with `__init__.py` is a package
- Top-level directories in source roots are main modules
- Note relative vs absolute imports
- Identify `__all__` exports

Map structure:
```python
# Look for patterns like:
src/
├── module_a/
│   ├── __init__.py
│   ├── submodule1.py
│   └── submodule2.py
└── module_b/
    ├── __init__.py
    └── utils.py
```

#### TypeScript/JavaScript
- Each directory with `index.ts` is often a module boundary
- Look for barrel exports (`index.ts` re-exporting)
- Check `tsconfig.json` paths for module aliases
- Note ES modules vs CommonJS

Map structure:
```typescript
// Look for patterns like:
src/
├── moduleA/
│   ├── index.ts     // barrel export
│   ├── service.ts
│   └── types.ts
└── moduleB/
    ├── index.ts
    └── handler.ts
```

#### Rust
- Each directory with `mod.rs` or file with same name is a module
- `lib.rs` defines library module tree
- `main.rs` defines binary module tree
- Workspace members are separate crates

```rust
// Look for patterns like:
src/
├── lib.rs          // pub mod declarations
├── module_a/
│   ├── mod.rs
│   └── submodule.rs
└── module_b.rs
```

#### Go
- Each directory is a package
- Package name from `package` declaration
- Internal packages (`internal/`) restrict visibility

### Step 3: Analyze Module Dependencies

For each module, identify:
- What it imports/depends on
- What depends on it
- Internal vs external dependencies

#### Using Codegraph (Preferred if SCIP Index Loaded)

For each major module, use semantic analysis:

```
# Get dependencies of a module
codegraph/get_module_deps
  module_path: "src/module_a"

# Get all exports from a module
codegraph/get_file_symbols
  file_path: "src/module_a/mod.rs"

# Find modules with many dependents (god modules)
codegraph/find_hotspot_symbols
  min_callers: 10
  path_filter: "src/"
```

This provides accurate dependency information:
- Exact symbol references (not just file-level imports)
- Caller/callee relationships between functions
- Detection of unused exports

#### Manual Fallback (Without SCIP Index)

**Python:**
```python
# Analyze import statements
import module_a
from module_b import something
from . import relative
```

**TypeScript:**
```typescript
// Analyze import statements
import { Something } from './moduleA';
import { Other } from '../moduleB';
import { External } from 'external-package';
```

**Rust:**
```rust
// Analyze use statements
use crate::module_a::Something;
use super::module_b::Other;
use external_crate::External;
```

### Step 4: Detect Circular Dependencies

Look for circular import patterns:
- Module A imports from Module B
- Module B imports from Module A

Common circular dependency patterns:
- Direct cycles: A → B → A
- Indirect cycles: A → B → C → A
- Type-only cycles (sometimes acceptable)

Document any cycles found with:
- Involved modules
- Import chain
- Severity (blocking vs type-only)

### Step 5: Calculate Module Metrics

For each significant module, note:
- Lines of code (approximate)
- Number of submodules
- Number of external dependencies
- Number of internal dependencies

### Step 6: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S01"
data:
  modules:
    - name: "module_a"
      path: "src/module_a"
      submodules:
        - "submodule1"
        - "submodule2"
      dependencies:
        - "module_b"
        - "external_lib"
      lines_of_code: 1500
    - name: "module_b"
      path: "src/module_b"
      submodules: []
      dependencies:
        - "module_a"  # Potential cycle!
      lines_of_code: 800
  circular_dependencies:
    - ["module_a", "module_b"]  # Direct cycle
```

## Output Schema

```yaml
module_hierarchy:
  modules:
    - name: string           # Module name
      path: string           # Path to module
      submodules:            # Child modules
        - string
      dependencies:          # Modules this depends on
        - string
      lines_of_code: number  # Approximate LOC
  circular_dependencies:     # Detected cycles
    - [string]               # List of modules in cycle
```

## Quality Indicators

- ✅ **Healthy**: Clear hierarchy, no circular dependencies, reasonable module sizes
- ⚠️ **Warning**: Minor circular dependencies, some large modules
- ❌ **Critical**: Many circular dependencies, chaotic module structure

## Common Anti-Patterns

### God Module
One module that everything depends on and depends on everything:
- Solution: Split into focused modules

### Circular Dependencies
Modules that mutually depend on each other:
- Solution: Extract shared code, use dependency injection, or interfaces

### Deep Nesting
Too many levels of module nesting (>4-5):
- Solution: Flatten structure, use domain-based organization

### Orphan Modules
Modules with no dependencies and nothing depending on them:
- Solution: Remove if unused, or connect to appropriate consumers

## Next Viewpoint
After completing VP-S01, proceed to **VP-S02: Layer Architecture Analysis**.

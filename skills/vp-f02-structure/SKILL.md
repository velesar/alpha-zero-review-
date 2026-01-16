---
name: vp-f02-structure
version: 1.0
dependencies: [vp-f01-tech-stack]
mcp_servers: [mental-model, methodology-kb]
---

# VP-F02: Project Structure Analysis

## Purpose
Analyze the project's file and directory structure to understand organization patterns, identify source roots, test locations, and configuration files.

## Prerequisites
- VP-F01 (Tech Stack Analysis) completed
- Verify by calling `mental-model/get_model` and confirming tech_stack is populated

## Instructions

### Step 1: Verify Prerequisites
Call `mental-model/get_model` and confirm:
- `tech_stack.primary_language` is set
- `tech_stack.framework` is set
- `completed_viewpoints` includes "VP-F01"

If not complete, execute VP-F01 first.

### Step 2: Identify Root Layout Type

Analyze the project root to determine structure type:

#### Monorepo Indicators
- `packages/`, `apps/`, `libs/` directories
- `pnpm-workspace.yaml`, `lerna.json`
- Multiple `package.json` files (Node)
- Multiple `Cargo.toml` files (Rust)
- `nx.json` (Nx monorepo)
- `turbo.json` (Turborepo)

#### Single Application Indicators
- Single `src/` directory
- Single package manifest at root
- Clear entry point (e.g., `main.py`, `index.ts`)

#### Multi-Package Indicators
- `setup.py` with packages in subdirectories
- Go modules with multiple packages

### Step 3: Identify Source Roots

Based on language and framework from VP-F01:

#### Python
- `src/`, `app/`, `<project_name>/`
- Check `pyproject.toml` for `[tool.setuptools.packages]`

#### TypeScript/JavaScript
- `src/`, `lib/`, `app/`
- Check `tsconfig.json` for `rootDir`, `include`

#### Rust
- `src/` (single crate)
- Member directories for workspace

#### Go
- Directories with `.go` files
- Check `go.mod` for module path

### Step 4: Identify Test Roots

Common test locations by language:

#### Python
- `tests/`, `test/`
- `*_test.py`, `test_*.py` files
- Pytest convention: `tests/` parallel to `src/`

#### TypeScript/JavaScript
- `tests/`, `test/`, `__tests__/`
- `*.test.ts`, `*.spec.ts` files
- Jest convention: `__tests__/` or co-located

#### Rust
- Inline `#[cfg(test)]` modules
- `tests/` directory for integration tests

### Step 5: Identify Configuration Files

Look for:
- **Build**: `Makefile`, `build.gradle`, `CMakeLists.txt`
- **Linting**: `.eslintrc`, `pylintrc`, `.flake8`, `clippy.toml`
- **Formatting**: `.prettierrc`, `black.toml`, `rustfmt.toml`
- **Type Checking**: `tsconfig.json`, `mypy.ini`, `pyright`
- **CI/CD**: `.github/workflows/`, `.gitlab-ci.yml`, `Jenkinsfile`
- **Docker**: `Dockerfile`, `docker-compose.yml`
- **Environment**: `.env`, `.env.example`

### Step 6: Identify Entry Points

Find application entry points:

#### Python
- `__main__.py`
- `main.py`, `app.py`, `server.py`
- `manage.py` (Django)
- `asgi.py`, `wsgi.py`

#### TypeScript/JavaScript
- `index.ts`, `main.ts`, `server.ts`
- `package.json` `main` field
- `src/index.ts` or `src/main.ts`

#### Rust
- `src/main.rs` (binary)
- `src/lib.rs` (library)

### Step 7: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-F02"
data:
  root_layout: "monorepo|single-app|multi-package"
  source_roots:
    - "src/"
    - "lib/"
  test_roots:
    - "tests/"
    - "__tests__/"
  config_files:
    - "pyproject.toml"
    - "tsconfig.json"
    - ".eslintrc.js"
  entry_points:
    - "src/main.py"
    - "src/index.ts"
```

## Output Schema

```yaml
structure:
  root_layout: monorepo | single-app | multi-package
  source_roots:
    - string                    # Paths to source code directories
  test_roots:
    - string                    # Paths to test directories
  config_files:
    - string                    # Important config files
  entry_points:
    - string                    # Application entry points
```

## Quality Indicators

- ✅ **Healthy**: Clear structure, separated source/test, conventional layout
- ⚠️ **Warning**: Mixed conventions, unclear separation, missing test directory
- ❌ **Critical**: Chaotic structure, no clear organization, tests mixed with source

## Structure Anti-Patterns

Watch for these issues:
- Tests in production source directories
- Configuration scattered throughout codebase
- Deeply nested structure without clear hierarchy
- Multiple conflicting organization patterns
- No clear separation between library and application code

## Common Patterns

### Python FastAPI
```
project/
├── src/
│   └── app/
│       ├── __init__.py
│       ├── main.py
│       ├── api/
│       ├── core/
│       ├── models/
│       └── services/
├── tests/
│   ├── conftest.py
│   └── test_*.py
├── pyproject.toml
└── README.md
```

### TypeScript NestJS
```
project/
├── src/
│   ├── main.ts
│   ├── app.module.ts
│   ├── modules/
│   │   └── feature/
│   └── common/
├── test/
│   ├── app.e2e-spec.ts
│   └── jest-e2e.json
├── package.json
├── tsconfig.json
└── nest-cli.json
```

### Rust Workspace
```
project/
├── Cargo.toml (workspace)
├── crates/
│   ├── core/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── api/
│       ├── Cargo.toml
│       └── src/
└── tests/
```

## Next Viewpoint
After completing VP-F02, proceed to **VP-F03: Build & Deployment Analysis**.

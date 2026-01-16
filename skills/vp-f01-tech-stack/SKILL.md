---
name: vp-f01-tech-stack
version: 1.0
dependencies: []
mcp_servers: [mental-model, methodology-kb]
---

# VP-F01: Technology Stack Analysis

## Purpose
Identify and document the technology stack used in the project, including primary programming language, framework, version information, and key dependencies.

## Prerequisites
- Project directory accessible
- Mental Model MCP server running
- If this is a fresh audit, call `mental-model/init_model` first with project name and path

## Instructions

### Step 1: Initialize Mental Model (if needed)
If starting a new audit, initialize the mental model:
```
Call mental-model/init_model with:
- name: <project name from directory or package file>
- path: <project root path>
- description: <brief description if available>
```

### Step 2: Identify Primary Language and Framework

#### For Python Projects
Look for these files (in order of priority):
1. `pyproject.toml` - Modern Python packaging
2. `setup.py` - Traditional Python packaging
3. `requirements.txt` - Dependency list
4. `Pipfile` - Pipenv projects
5. `setup.cfg` - Configuration file

Extract:
- Python version from `[project]` or `python_requires`
- Framework from dependencies (Django, FastAPI, Flask, etc.)
- Framework version

#### For Node/TypeScript Projects
Look for these files:
1. `package.json` - Primary source
2. `tsconfig.json` - TypeScript indicator
3. `pnpm-lock.yaml`, `yarn.lock`, `package-lock.json` - Package manager

Extract:
- Node version from `engines.node`
- TypeScript presence and version
- Framework (Express, NestJS, Next.js, etc.)

#### For Rust Projects
Look for:
1. `Cargo.toml` - Rust manifest
2. `rust-toolchain.toml` - Rust version

Extract:
- Rust edition
- Key framework crates (actix-web, axum, rocket, etc.)

#### For Go Projects
Look for:
1. `go.mod` - Go modules
2. `go.sum` - Dependencies

Extract:
- Go version
- Framework (gin, echo, fiber, etc.)

### Step 3: Identify Key Dependencies

Categorize dependencies into:
- **Core Framework**: Main web/app framework
- **Database**: ORM, database drivers
- **Authentication**: Auth libraries
- **Testing**: Test frameworks
- **Utilities**: Helper libraries

Focus on dependencies with security or architectural implications.

### Step 4: Determine Confidence Level

- **High**: All indicators consistent, version explicitly specified
- **Medium**: Some version info missing or inferred
- **Low**: Multiple conflicting indicators or minimal metadata

### Step 5: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-F01"
data:
  primary_language: "<language>"
  version: "<version>"
  framework: "<framework name>"
  framework_version: "<version>"
  confidence: "high|medium|low"
  additional_languages:
    - "<secondary language>"
  dependencies:
    - name: "<dep name>"
      version: "<version>"
      category: "<core|database|auth|testing|utility>"
```

### Step 6: Get Appropriate Thresholds

Based on detected tech stack, call `methodology-kb/get_thresholds`:
```
project_type: "greenfield|mature|legacy|startup|enterprise"
language: "<detected language>"
```

Store the thresholds for use in subsequent viewpoints.

## Output Schema

```yaml
tech_stack:
  primary_language: string      # e.g., "python", "typescript", "rust"
  version: string               # e.g., "3.11", "5.0", "2021"
  framework: string             # e.g., "FastAPI", "NestJS", "Axum"
  framework_version: string     # e.g., "0.100.0"
  confidence: high | medium | low
  additional_languages:         # Secondary languages
    - string
  dependencies:                 # Key dependencies
    - name: string
      version: string
      category: string
```

## Quality Indicators

- ✅ **Healthy**: Clear tech stack identification, all versions known, high confidence
- ⚠️ **Warning**: Some versions unknown, mixed indicators, medium confidence
- ❌ **Critical**: Unable to determine tech stack, conflicting information

## Common Patterns

### Python + FastAPI
```yaml
primary_language: python
framework: FastAPI
typical_dependencies:
  - pydantic (validation)
  - uvicorn (ASGI server)
  - sqlalchemy (ORM)
  - alembic (migrations)
```

### TypeScript + NestJS
```yaml
primary_language: typescript
framework: NestJS
typical_dependencies:
  - @nestjs/common
  - @nestjs/core
  - typeorm or prisma
  - passport (auth)
```

### Rust + Axum
```yaml
primary_language: rust
framework: Axum
typical_dependencies:
  - tokio (async runtime)
  - tower (middleware)
  - sqlx or diesel (database)
  - serde (serialization)
```

## Next Viewpoint
After completing VP-F01, proceed to **VP-F02: Project Structure Analysis**.

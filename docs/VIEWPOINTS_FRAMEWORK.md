# Mental Model Viewpoints Framework

## Системний підхід до AI-driven Software Quality Audit

> **Версія:** 2.0 | **Light IT Global** | Січень 2026

-----

## 1. Концепція

### Проблема One-Shot Analysis

|Проблема             |Наслідок                                     |
|---------------------|---------------------------------------------|
|Context collapse     |Findings без розуміння архітектури           |
|Token exhaustion     |Поверхневий аналіз великих codebase          |
|Finding drift        |Випадкові знахідки замість системного аналізу|
|False positives      |Findings, що не є проблемами в контексті     |
|Symptom vs Root Cause|Індикатори замість справжніх причин          |

### Рішення: Mental Model First

```
Surface Scan → Mental Model → Deep Analysis → Synthesis
     ↓              ↓              ↓             ↓
  Artifacts     Viewpoints    Contextual    Actionable
  Discovery     Definition    Findings      Report
```

### Теоретичні основи

Фреймворк базується на принципах:

- **ISO/IEC/IEEE 42010** — архітектура як набір viewpoints для різних stakeholders
- **C4 Model** — ієрархічний zoom (Context → Containers → Components → Code)
- **arc42** — структурований шаблон архітектурної документації
- **TOGAF Building Blocks** — переиспользовувані архітектурні компоненти

-----

## 2. Viewpoint Structure

```yaml
viewpoint:
  id: VP-XXX
  name: Human-readable name
  category: foundational | structural | quality
  status: required | optional  # [OPTIONAL] для необов'язкових
  purpose: Що хочемо зрозуміти
  artifacts:
    input: Що шукаємо в проєкті
    output: Що створюємо як результат
  tools:
    discovery: Інструменти для виявлення
    analysis: Інструменти для аналізу
    ai_prompts: Шаблони промптів для LLM
  key_questions: Питання, на які відповідаємо
  quality_indicators:
    positive: Ознаки здорової системи
    negative: Red flags
  metrics: Кількісні показники
  common_findings: Типові проблеми
  fitness_function: Executable specification (optional)
```

-----

## 3. Viewpoint Catalog

### Layer 1: Foundational Viewpoints

|ID    |Name            |Purpose                   |Key Artifacts                           |
|------|----------------|--------------------------|----------------------------------------|
|VP-F01|Technology Stack|Визначити стек технологій |package.json, pyproject.toml, Dockerfile|
|VP-F02|File Structure  |Зрозуміти організацію коду|Directory tree, naming patterns         |
|VP-F03|Build & Deploy  |Зрозуміти CI/CD pipeline  |GitHub Actions, Dockerfile, K8s         |

### Layer 2: Structural Viewpoints

|ID    |Name                     |Status      |Purpose                     |Key Artifacts                  |
|------|-------------------------|------------|----------------------------|-------------------------------|
|VP-S01|Module Hierarchy (C4)    |Required    |Побудувати C4 модель        |Import graph, entry points     |
|VP-S02|Layer Architecture       |Required    |Валідувати архітектурні шари|Layer definitions, violations  |
|VP-S03|Domain Model             |Required    |Зрозуміти доменну модель    |Bounded contexts, aggregates   |
|VP-S04|Entity Model             |Required    |Структура даних             |ORM models, schemas            |
|VP-S05|Interface Surface        |Required    |API та UI інтерфейси        |Routes, OpenAPI, screens       |
|VP-S06|Dependency Graph         |Required    |Внутрішні та зовнішні deps  |Import analysis, package audit |
|VP-S07|Architecture Decisions   |Required    |Виявити implicit ADRs       |Patterns, trade-offs, rationale|
|VP-S08|Team Topologies Mapping  |`[OPTIONAL]`|Conway's Law alignment      |Team structure, ownership      |
|VP-S09|Building Block Compliance|`[OPTIONAL]`|ABB/SBB gap analysis        |Reference architecture match   |
|VP-S10|Standards Compliance     |`[OPTIONAL]`|Policies verification       |Technology standards, policies |

### Layer 3: Quality Viewpoints

|ID    |Name            |Purpose                       |Key Artifacts               |
|------|----------------|------------------------------|----------------------------|
|VP-Q01|Security Surface|Security posture              |Auth, authz, vulnerabilities|
|VP-Q02|Performance     |Performance patterns          |N+1, caching, indexes       |
|VP-Q03|Testability     |Тестова інфраструктура        |Coverage, test types        |
|VP-Q04|Code Style      |Coding standards              |Linter configs, violations  |
|VP-Q05|Documentation   |Документація                  |README, API docs, ADRs      |
|VP-Q06|Technical Debt  |Синтез боргу (Fowler Quadrant)|All findings aggregated     |

-----

## 4. Detailed Viewpoint Specifications

### VP-F01: Technology Stack

**Purpose:** Визначити технологічний стек проєкту

**Input Artifacts:**

- `package.json` — Node.js deps
- `pyproject.toml` / `requirements.txt` — Python deps
- `Dockerfile` — Container config
- `.tool-versions` — Runtime versions

**Output:**

```yaml
technology_stack:
  primary_language: {name, version}
  frameworks: {backend: [], frontend: [], testing: []}
  runtime: {type, version, containerized}
  dev_tools: {linters: [], formatters: [], type_checkers: []}
```

**Discovery Commands:**

```bash
find . -name "package.json" -o -name "*.toml" -o -name "requirements*.txt"
jq '.dependencies' package.json
```

**AI Prompt Template:**

```
Analyze these configuration files:
[FILES]

Extract:
1. Primary language and version
2. Key frameworks (backend, frontend, testing)
3. Development tools (linters, formatters)
4. Confidence level for each identification
```

**Quality Indicators:**

- ✅ Locked dependency versions
- ✅ Явно вказані runtime versions
- 🚩 Wildcard versions ("*")
- 🚩 Missing lock files

**Metrics:**

|Metric              |Target                       |
|--------------------|-----------------------------|
|Dependency freshness|> 80% within 2 major versions|
|Lock file coverage  |100%                         |
|Version pinning     |> 95%                        |

**Fitness Function (Optional):**

```python
def check_dependency_pinning(project_path: str) -> bool:
    """All dependencies must have pinned versions."""
    package_json = load_json(f"{project_path}/package.json")
    deps = {**package_json.get("dependencies", {}),
            **package_json.get("devDependencies", {})}

    unpinned = [name for name, version in deps.items()
                if version.startswith("^") or version == "*"]

    assert len(unpinned) == 0, f"Unpinned dependencies: {unpinned}"
    return True
```

-----

### VP-F02: File Structure

**Purpose:** Зрозуміти організацію файлів та директорій

**Discovery Commands:**

```bash
tree -I 'node_modules|__pycache__|.git|dist' -L 4
scc --no-cocomo .
find . -type f -name "*.py" | xargs wc -l | sort -rn | head -20
```

**Output:**

```yaml
file_structure:
  root_layout: {type: monorepo|single-app, conventions}
  directories: {source_code, tests, configuration}
  hotspots: {largest_files, deepest_nesting}
  conventions: {naming_style, test_location}
```

**AI Prompt Template:**

```
Analyze this directory structure:
[TREE OUTPUT]

Determine:
1. Project type (monorepo, single app, microservices)
2. Architectural style (MVC, Clean Architecture, Feature-based)
3. Source code organization pattern
4. Key directories for business logic
5. Non-standard or concerning patterns
```

**Quality Indicators:**

- ✅ Чітка логічна структура
- ✅ Консистентний naming
- 🚩 Файли > 1000 LOC
- 🚩 Директорії > 50 файлів
- 🚩 Вкладеність > 6 рівнів

**Fitness Function (Optional):**

```python
def check_file_size_limits(project_path: str, max_loc: int = 500) -> bool:
    """No source file should exceed max_loc lines."""
    violations = []
    for py_file in glob(f"{project_path}/**/*.py", recursive=True):
        if "__pycache__" in py_file or "test" in py_file:
            continue
        loc = count_lines(py_file)
        if loc > max_loc:
            violations.append((py_file, loc))

    assert len(violations) == 0, f"Files exceeding {max_loc} LOC: {violations}"
    return True
```

-----

### VP-F03: Build & Deploy

**Purpose:** Зрозуміти CI/CD pipeline та quality gates

**Input Artifacts:**

- `.github/workflows/*.yml`
- `Makefile`
- `docker-compose.yml`
- `kubernetes/*.yaml`

**Output:**

```yaml
build_deploy:
  ci_cd: {platform, pipelines: [{name, trigger, stages}]}
  quality_gates:
    linting: boolean
    type_checking: boolean
    unit_tests: boolean
    security_scan: boolean
    coverage_threshold: number
  deployment: {strategy, environments: []}
```

**AI Prompt Template:**

```
Analyze this CI/CD configuration:
[CONFIG]

Determine:
1. Pipeline structure and stages
2. Quality gates implemented
3. Deployment strategy and environments
4. Missing quality gates
5. Security concerns in pipeline
```

**Quality Indicators:**

- ✅ Повний CI/CD pipeline
- ✅ Всі quality gates на місці
- 🚩 No tests in pipeline
- 🚩 Secrets в коді
- 🚩 No staging environment

**Fitness Function (Optional):**

```python
def check_required_quality_gates(ci_config: dict) -> bool:
    """CI must include all required quality gates."""
    required_gates = {"lint", "test", "type-check", "security-scan"}

    found_gates = set()
    for job in ci_config.get("jobs", {}).values():
        for step in job.get("steps", []):
            step_name = step.get("name", "").lower()
            if "lint" in step_name: found_gates.add("lint")
            if "test" in step_name: found_gates.add("test")
            if "type" in step_name: found_gates.add("type-check")
            if "security" in step_name or "snyk" in step_name:
                found_gates.add("security-scan")

    missing = required_gates - found_gates
    assert len(missing) == 0, f"Missing quality gates: {missing}"
    return True
```

-----

### VP-S01: Module Hierarchy (C4 Model)

**Purpose:** Побудувати C4 модель системи (Context → Container → Component → Code)

**Discovery:**

```bash
# External systems
grep -r "https?://" --include="*.py" --include="*.ts" | grep -v test

# Entry points
find . -name "main.py" -o -name "index.ts" -o -name "app.py"

# Module dependencies
npx madge --circular src/
pydeps src/ --max-bacon=2
```

**Output:**

```yaml
c4_model:
  level_1_context:
    system_name: string
    users: [{name, type, interaction}]
    external_systems: [{name, type, interaction}]
  level_2_containers: [{id, name, type, technology}]
  level_3_components: [{id, name, type, responsibilities, dependencies}]
```

**AI Prompt Template:**

```
Based on these entry points and configurations:
[FILES]

Identify:
1. C4 Level 1: What is this system? Users? External dependencies?
2. C4 Level 2: Distinct containers/deployable units
3. C4 Level 3: Major components per container
4. Communication patterns between components
```

**Quality Indicators:**

- ✅ Чіткі boundaries між containers
- ✅ Низька coupling між компонентами
- 🚩 Circular dependencies
- 🚩 God components
- 🚩 Direct database access з presentation

**Fitness Function (Optional):**

```python
def check_no_circular_dependencies(project_path: str) -> bool:
    """No circular dependencies between modules."""
    import_graph = build_import_graph(project_path)
    cycles = find_cycles(import_graph)

    assert len(cycles) == 0, f"Circular dependencies found: {cycles}"
    return True
```

-----

### VP-S02: Layer Architecture

**Purpose:** Валідувати дотримання архітектурних шарів

**Patterns to Detect:**

- Traditional Layered: Presentation → Business → Data
- Clean Architecture: Entities ← UseCases ← Adapters ← Frameworks
- Hexagonal: Domain + Ports + Adapters
- Vertical Slices: Feature-based

**Output:**

```yaml
layer_architecture:
  detected_pattern: string
  confidence: high|medium|low
  layers: [{name, directories, dependencies_to}]
  violations: [{from_layer, to_layer, location, type}]
```

**AI Prompt Template:**

```
Analyze project structure and imports:
[DIRECTORY STRUCTURE]
[SAMPLE IMPORTS]

Determine:
1. Which architectural pattern is used?
2. Layer definitions and boundaries
3. Dependency direction violations
4. Framework leakage into domain
```

**Quality Indicators:**

- ✅ Явні layer boundaries
- ✅ Domain layer без external deps
- 🚩 Framework annotations в domain
- 🚩 Business logic в controllers
- 🚩 Layer skip violations

**Fitness Function (Optional):**

```python
def check_layer_dependencies(project_path: str) -> bool:
    """Domain layer must not depend on infrastructure."""
    FORBIDDEN_IMPORTS = {
        "domain": ["infrastructure", "adapters", "fastapi", "sqlalchemy", "flask"],
        "application": ["infrastructure", "fastapi", "flask"],
    }

    violations = []
    for layer, forbidden in FORBIDDEN_IMPORTS.items():
        layer_path = f"{project_path}/{layer}"
        if not os.path.exists(layer_path):
            continue

        for py_file in glob(f"{layer_path}/**/*.py", recursive=True):
            imports = extract_imports(py_file)
            for imp in imports:
                if any(f in imp for f in forbidden):
                    violations.append((py_file, imp))

    assert len(violations) == 0, f"Layer violations: {violations}"
    return True
```

-----

### VP-S03: Domain Model

**Purpose:** Зрозуміти доменну модель (DDD)

**DDD Concepts:**

- Bounded Contexts
- Aggregates & Aggregate Roots
- Entities vs Value Objects
- Domain Events
- Ubiquitous Language

**Output:**

```yaml
domain_model:
  bounded_contexts: [{name, type: core|supporting|generic, aggregates}]
  aggregates: [{name, root, entities, value_objects, invariants}]
  domain_events: [{name, aggregate, payload}]
  ubiquitous_language: {term: definition}
```

**AI Prompt Template:**

```
Analyze this codebase for DDD patterns:
[ENTITY CLASSES]
[API ENDPOINTS]

Identify:
1. Bounded contexts and their boundaries
2. Aggregates and aggregate roots
3. Domain richness (behaviors vs just data)
4. Ubiquitous language terms
5. DDD violations (anemic model, leaky aggregates)
```

**Quality Indicators:**

- ✅ Чітко визначені bounded contexts
- ✅ Багата domain model з behaviors
- 🚩 Anemic domain model
- 🚩 Прямі references між aggregates
- 🚩 Shared database між contexts

-----

### VP-S04: Entity Model

**Purpose:** Зрозуміти структуру даних

**Output:**

```yaml
entity_model:
  entities: [{name, attributes, relationships, scope: user|tenant|global}]
  relationships_graph: {nodes, edges}
  patterns: {soft_delete, audit_trail, multi_tenancy}
```

**Quality Indicators:**

- ✅ Нормалізована структура
- ✅ Індекси на foreign keys
- ✅ Audit fields скрізь
- 🚩 God entity (>30 fields)
- 🚩 Missing FK constraints
- 🚩 No indexes

-----

### VP-S05: Interface Surface

**Purpose:** Визначити всі інтерфейси системи

**Interface Types:**

- REST API endpoints
- GraphQL schema
- UI screens/pages
- Events (published/consumed)
- CLI commands

**Output:**

```yaml
interface_surface:
  api: [{method, path, handler, authentication, domain}]
  ui: [{name, path, components, api_calls}]
  events: {published: [], consumed: []}
  summary: {total_endpoints, by_domain, public_count}
```

**Quality Indicators:**

- ✅ OpenAPI documentation
- ✅ Consistent REST naming
- 🚩 Undocumented endpoints
- 🚩 Missing auth on sensitive endpoints
- 🚩 God endpoints

-----

### VP-S06: Dependency Graph

**Purpose:** Аналіз внутрішніх та зовнішніх залежностей

**Discovery Commands:**

```bash
# Internal dependencies
npx madge --circular src/

# External dependencies analysis
npm audit
npm outdated
pip-audit
```

**Output:**

```yaml
dependency_graph:
  internal: {modules: [{imports, imported_by, coupling_score}], cycles: []}
  external: {direct: [], analysis: {outdated, vulnerable, deprecated}}
  runtime: {services: [{name, type, criticality}]}
```

**Quality Indicators:**

- ✅ No circular dependencies
- ✅ All deps up-to-date
- 🚩 Vulnerable dependencies
- 🚩 Abandoned packages
- 🚩 Excessive coupling

**Fitness Function (Optional):**

```python
def check_no_vulnerable_dependencies(project_path: str) -> bool:
    """No known vulnerabilities in dependencies."""
    result = subprocess.run(
        ["pip-audit", "--format", "json"],
        cwd=project_path,
        capture_output=True
    )
    vulnerabilities = json.loads(result.stdout)

    critical = [v for v in vulnerabilities if v["severity"] in ("high", "critical")]
    assert len(critical) == 0, f"Critical vulnerabilities: {critical}"
    return True
```

-----

### VP-S07: Architecture Decisions

**Purpose:** Виявити implicit Architecture Decision Records (ADRs) через patterns у коді

**Rationale:** Більшість архітектурних рішень не документовані явно. Цей viewpoint реконструює "чому" з "що" — аналізуючи patterns, trade-offs та consistency рішень.

**Decision Categories:**

|Category              |Examples                            |
|----------------------|------------------------------------|
|Technology Choices    |"Why PostgreSQL over MongoDB?"      |
|Architectural Patterns|"Why Clean Architecture?"           |
|Integration Patterns  |"Why REST over GraphQL?"            |
|Data Patterns         |"Why Event Sourcing?"               |
|Security Patterns     |"Why OAuth2 over custom auth?"      |
|Trade-offs            |"Why consistency over availability?"|

**Discovery Methods:**

```bash
# Find configuration patterns
grep -r "DATABASE_URL\|REDIS_URL\|KAFKA" --include="*.py" --include="*.env*"

# Find architectural patterns
find . -type d -name "domain" -o -name "adapters" -o -name "infrastructure"

# Find integration patterns
grep -r "@router\|@app.route\|GraphQL" --include="*.py"

# Find explicit ADRs
find . -path "*/docs/*" -name "*.md" | xargs grep -l "Decision\|ADR"
```

**Output:**

```yaml
architecture_decisions:
  explicit_adrs:
    - id: "ADR-001"
      title: string
      status: accepted|deprecated|superseded
      location: path

  implicit_decisions:
    - category: technology|pattern|integration|security|trade-off
      decision: string
      evidence:
        - file: path
          pattern: string
          confidence: high|medium|low
      rationale_hypothesis: string  # AI-generated explanation
      alternatives_considered: []   # What else could have been chosen
      trade_offs:
        benefits: []
        drawbacks: []
      consistency_score: float  # How consistently applied across codebase

  decision_conflicts:
    - decisions: [id1, id2]
      conflict_type: string
      locations: []
      recommendation: string
```

**AI Prompt Template:**

```
Analyze these code patterns and configurations:
[FILES AND PATTERNS]

For each significant decision found:
1. What decision was made?
2. What evidence supports this (files, patterns)?
3. Why might this decision have been made? (hypothesis)
4. What alternatives existed?
5. What are the trade-offs?
6. Is it consistently applied?
7. Are there any conflicting decisions?
```

**Key Questions:**

- Які технологічні вибори були зроблені?
- Чи є consistency у застосуванні patterns?
- Які trade-offs прийняті (явно чи неявно)?
- Чи є конфлікти між рішеннями?
- Які рішення варто задокументувати як explicit ADRs?

**Quality Indicators:**

- ✅ Explicit ADRs для ключових рішень
- ✅ Consistent patterns across codebase
- ✅ Clear rationale for technology choices
- 🚩 Conflicting patterns (REST + GraphQL without clear separation)
- 🚩 Inconsistent application of chosen patterns
- 🚩 No documentation for non-obvious choices
- 🚩 "Accidental architecture" — patterns without intent

**Common Findings:**

|Finding                       |Severity|Recommendation                     |
|------------------------------|--------|-----------------------------------|
|Mixed architectural styles    |Medium  |Document boundaries, plan migration|
|Undocumented technology choice|Low     |Create ADR retroactively           |
|Conflicting patterns          |High    |Resolve or document rationale      |
|Inconsistent error handling   |Medium  |Establish and document standard    |

**Fitness Function (Optional):**

```python
def check_architectural_consistency(project_path: str) -> bool:
    """Architectural patterns must be consistently applied."""
    # Check if all modules follow same layering
    expected_structure = {"domain", "application", "infrastructure"}

    modules = find_top_level_modules(project_path)
    inconsistent = []

    for module in modules:
        module_dirs = set(os.listdir(f"{project_path}/{module}"))
        if not expected_structure.issubset(module_dirs):
            missing = expected_structure - module_dirs
            inconsistent.append((module, missing))

    assert len(inconsistent) == 0, \
        f"Modules with inconsistent structure: {inconsistent}"
    return True


def check_explicit_adrs_exist(project_path: str) -> bool:
    """Key decisions must have explicit ADRs."""
    required_decisions = [
        "database_choice",
        "authentication_method",
        "api_style",
    ]

    adr_path = f"{project_path}/docs/adr"
    if not os.path.exists(adr_path):
        raise AssertionError("No ADR directory found at docs/adr")

    adrs = [f.lower() for f in os.listdir(adr_path)]
    missing = []

    for decision in required_decisions:
        if not any(decision in adr for adr in adrs):
            missing.append(decision)

    assert len(missing) == 0, f"Missing ADRs for: {missing}"
    return True
```

-----

### VP-S08: Team Topologies Mapping `[OPTIONAL]`

**Purpose:** Аналіз відповідності архітектури структурі команд (Conway's Law alignment)

**Rationale:** Conway's Law стверджує, що архітектура системи відображає комунікаційну структуру організації. Невідповідність між team structure та code ownership створює friction та знижує velocity.

**When to Use:**

- Enterprise проєкти з кількома командами
- Аудит перед реорганізацією команд
- Виявлення причин низької velocity
- Планування модуляризації монолітів

**Team Types (за Team Topologies):**

|Type                 |Description               |Architectural Impact           |
|---------------------|--------------------------|-------------------------------|
|Stream-aligned       |Delivers business value   |Owns bounded context end-to-end|
|Platform             |Provides internal services|Shared infrastructure, APIs    |
|Enabling             |Helps other teams         |Cross-cutting concerns         |
|Complicated-subsystem|Specialized knowledge     |Isolated complex components    |

**Input Artifacts:**

- CODEOWNERS file
- Git history (commit authors by module)
- Team structure documentation
- Jira/Linear project boards

**Discovery Commands:**

```bash
# Find CODEOWNERS
cat .github/CODEOWNERS

# Analyze commit authors by directory
git log --format='%ae' --since='6 months ago' -- src/payments/ | sort | uniq -c

# Find cross-team changes
git log --format='%h %ae' --name-only --since='3 months ago' | \
  awk '/^[a-f0-9]/{author=$2} /^src\//{print author, $0}'
```

**Output:**

```yaml
team_topologies:
  teams:
    - name: string
      type: stream-aligned|platform|enabling|complicated-subsystem
      owned_modules: [paths]
      cognitive_load_score: 1-10

  ownership_analysis:
    clear_ownership: [{module, team, confidence}]
    shared_ownership: [{module, teams: [], conflict_risk}]
    orphaned_modules: [paths]  # No clear owner

  conway_alignment:
    aligned: [{module, team, interaction_mode}]
    misaligned:
      - module: path
        current_owner: team
        natural_owner: team  # Based on dependencies
        recommendation: string

  cognitive_load:
    by_team:
      - team: string
        domains_owned: int
        lines_of_code: int
        external_dependencies: int
        score: int  # 1-10, >7 is overloaded
        recommendation: string

  interaction_modes:
    - teams: [team1, team2]
      mode: collaboration|x-as-a-service|facilitating
      interface: path  # API/module boundary
      friction_indicators: []
```

**AI Prompt Template:**

```
Analyze code ownership and team structure:
[CODEOWNERS]
[GIT COMMIT ANALYSIS]
[TEAM DOCUMENTATION]

Determine:
1. Which team owns which modules?
2. Are there modules with unclear ownership?
3. Does team structure align with architectural boundaries?
4. What is cognitive load per team?
5. Where is cross-team friction likely?
6. Recommendations for re-alignment
```

**Quality Indicators:**

- ✅ Clear module ownership (CODEOWNERS complete)
- ✅ Team boundaries align with bounded contexts
- ✅ Cognitive load balanced across teams
- 🚩 Shared ownership without clear interfaces
- 🚩 One team owns unrelated modules
- 🚩 High cross-team commit coupling
- 🚩 Orphaned modules (no clear owner)

**Metrics:**

|Metric              |Target      |Description                          |
|--------------------|------------|-------------------------------------|
|Ownership clarity   |> 90%       |% modules with single owner          |
|Cognitive load score|< 7 per team|Complexity score 1-10                |
|Cross-team commits  |< 10%       |Changes touching multiple team's code|
|Conway alignment    |> 80%       |% modules owned by "natural" team    |

-----

### VP-S09: Building Block Compliance `[OPTIONAL]`

**Purpose:** Порівняти архітектуру з reference architecture через призму TOGAF Building Blocks

**Rationale:** Building Blocks (ABB/SBB) дозволяють оцінити completeness та consistency архітектури проти еталонної моделі.

**When to Use:**

- Compliance audit для enterprise стандартів
- Migration planning до target architecture
- Оцінка technical debt scope
- Onboarding нових архітекторів

**Concepts:**

|Concept               |Description                                |
|----------------------|-------------------------------------------|
|ABB (Architecture BB) |Abstract capability ("Authentication")     |
|SBB (Solution BB)     |Concrete implementation ("Keycloak OAuth2")|
|Reference Architecture|Target set of ABBs for project type        |
|Gap                   |Missing or non-compliant building block    |

**Reference Architectures Available:**

```yaml
reference_architectures:
  clean_architecture_python:
    domain_layer:
      - abb: "Entity Base"
        expected_sbbs: ["Pydantic BaseModel", "dataclass"]
      - abb: "Repository Interface"
        expected_sbbs: ["Protocol/ABC definition"]
      - abb: "Domain Event"
        expected_sbbs: ["Event class with publish()"]
    application_layer:
      - abb: "Use Case"
        expected_sbbs: ["Command Handler", "Query Handler"]
      - abb: "Unit of Work"
        expected_sbbs: ["Context manager pattern"]
    infrastructure_layer:
      - abb: "Repository Implementation"
        expected_sbbs: ["SQLAlchemy Repository"]
      - abb: "HTTP API"
        expected_sbbs: ["FastAPI Router"]
```

**Output:**

```yaml
building_block_compliance:
  reference_architecture: string
  overall_compliance: percentage

  by_layer:
    domain:
      expected: int
      found: int
      compliance: percentage
      gaps: [{abb, impact, recommendation}]
      concerns: [{sbb, issue, severity}]

    application:
      # same structure

    infrastructure:
      # same structure

  sbb_inventory:
    - abb: string
      sbb: string
      location: path
      compliance: full|partial|non-compliant
      issues: []

  recommendations:
    priority_1: []  # Critical gaps
    priority_2: []  # Important improvements
    priority_3: []  # Nice to have
```

**AI Prompt Template:**

```
Compare this codebase against reference architecture:
[REFERENCE: Clean Architecture Python]
[CODEBASE STRUCTURE]
[KEY FILES]

For each expected building block:
1. Is it present? Where?
2. Is implementation compliant with reference?
3. What issues exist?
4. What is impact of gaps?
5. Prioritized recommendations
```

**Quality Indicators:**

- ✅ All ABBs from reference present
- ✅ SBBs follow reference patterns
- ✅ Consistent implementation across modules
- 🚩 Missing critical building blocks
- 🚩 Custom implementations where standard exists
- 🚩 Inconsistent SBBs for same ABB

-----

### VP-S10: Standards Compliance `[OPTIONAL]`

**Purpose:** Перевірити відповідність technology standards, architecture principles та security policies

**Rationale:** Організації мають standards та policies, які повинні бути дотримані. Цей viewpoint формалізує перевірку compliance.

**When to Use:**

- Enterprise compliance audit
- Security certification preparation
- Vendor assessment
- Internal standards enforcement

**Standards Categories:**

|Category               |Examples                                           |
|-----------------------|---------------------------------------------------|
|Technology Standards   |Approved languages, frameworks, databases          |
|Architecture Principles|"Domain must be framework-agnostic"                |
|Security Policies      |"No secrets in code", "All endpoints authenticated"|
|Coding Standards       |"100% type coverage", "Max complexity 10"          |

**Input: Standards Definition**

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

**Output:**

```yaml
standards_compliance:
  overall_score: percentage

  technology_compliance:
    compliant: [{technology, version, status: approved}]
    non_compliant:
      - technology: string
        current_version: string
        required: string
        severity: string
        remediation: string
    deprecated_in_use:
      - technology: string
        locations: [paths]
        migration_deadline: date
        effort_estimate: string

  architecture_compliance:
    - principle_id: string
      status: compliant|partial|violated
      evidence:
        compliant: [paths]
        violations: [{path, description}]
      remediation: string

  security_compliance:
    - policy_id: string
      status: compliant|violated
      violations: [{path, line, description}]
      severity: critical|high|medium|low
      remediation: string

  summary:
    critical_violations: int
    high_violations: int
    compliance_score: percentage
    top_remediation_actions: []
```

**Fitness Functions:**

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


def check_approved_dependencies(project_path: str, standards: dict) -> bool:
    """Only approved technologies in use."""
    approved = set(standards["technology_standards"]["approved_frameworks"]["backend"])

    requirements = parse_requirements(f"{project_path}/requirements.txt")
    non_approved = []

    for req in requirements:
        framework = req.split("==")[0].lower()
        if framework not in approved and is_framework(framework):
            non_approved.append(framework)

    assert len(non_approved) == 0, f"Non-approved frameworks: {non_approved}"
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

**Quality Indicators:**

- ✅ 100% compliance with critical policies
- ✅ No deprecated technologies without migration plan
- ✅ All architecture principles followed
- 🚩 Critical security policy violations
- 🚩 Use of non-approved technologies
- 🚩 Systematic architecture principle violations

-----

### VP-Q01: Security Surface

**Purpose:** Оцінити security posture

**Security Domains:**

- Authentication (JWT, OAuth, Session)
- Authorization (RBAC, ABAC)
- Data Protection (encryption)
- Secrets Management
- Vulnerability Management

**Output:**

```yaml
security_surface:
  authentication: {mechanisms: [], public_endpoints: []}
  authorization: {model, roles: [], permissions: []}
  vulnerabilities: {dependencies: [], code_findings: []}
```

**Quality Indicators:**

- ✅ Modern auth (OAuth 2.0, JWT)
- ✅ Strong password hashing (Argon2)
- 🚩 Hardcoded secrets
- 🚩 Missing auth on endpoints
- 🚩 SQL injection patterns

**Fitness Function (Optional):**

```python
def check_security_headers(project_path: str) -> bool:
    """Security headers must be configured."""
    required_headers = [
        "X-Content-Type-Options",
        "X-Frame-Options",
        "Strict-Transport-Security",
    ]

    # Check middleware configuration
    middleware_files = glob(f"{project_path}/**/middleware*.py", recursive=True)
    found_headers = set()

    for f in middleware_files:
        content = open(f).read()
        for header in required_headers:
            if header in content:
                found_headers.add(header)

    missing = set(required_headers) - found_headers
    assert len(missing) == 0, f"Missing security headers: {missing}"
    return True
```

-----

### VP-Q02: Performance

**Purpose:** Performance patterns analysis

**Focus Areas:**

- Database: N+1, indexes, query optimization
- Caching: layers, strategies, invalidation
- Scalability: stateless, horizontal scaling

**Quality Indicators:**

- ✅ Eager loading configured
- ✅ Multi-layer caching
- 🚩 N+1 queries
- 🚩 Missing indexes
- 🚩 Unbounded queries

-----

### VP-Q03: Testability

**Purpose:** Тестова інфраструктура

**Test Types:**

- Unit tests
- Integration tests
- E2E tests
- Contract tests

**Output:**

```yaml
testability:
  inventory: {unit_tests, integration_tests, e2e_tests}
  coverage: {line: %, branch: %, by_module: {}}
  gaps: {untested_modules: [], missing_test_types: []}
```

**Quality Indicators:**

- ✅ Coverage > 80%
- ✅ All test types present
- 🚩 Coverage < 50%
- 🚩 No integration tests
- 🚩 Flaky tests

**Fitness Function (Optional):**

```python
def check_coverage_threshold(project_path: str, min_coverage: float = 80.0) -> bool:
    """Code coverage must meet minimum threshold."""
    result = subprocess.run(
        ["pytest", "--cov", "--cov-report=json"],
        cwd=project_path,
        capture_output=True
    )

    coverage_data = json.loads(open(f"{project_path}/coverage.json").read())
    total_coverage = coverage_data["totals"]["percent_covered"]

    assert total_coverage >= min_coverage, \
        f"Coverage {total_coverage}% below threshold {min_coverage}%"
    return True
```

-----

### VP-Q04: Code Style

**Purpose:** Coding standards compliance

**Output:**

```yaml
code_style:
  tools: {linters: [], formatters: []}
  compliance: {violations: {total, by_rule}, formatter_passing: %}
  consistency: {naming: %, imports: %}
```

**Quality Indicators:**

- ✅ Linter + Formatter configured
- ✅ Pre-commit hooks
- 🚩 No linting
- 🚩 Thousands of violations

-----

### VP-Q05: Documentation

**Purpose:** Documentation assessment

**Documentation Types:**

- README files
- API docs (OpenAPI)
- Architecture docs (ADRs)
- Code documentation (docstrings)

**Quality Indicators:**

- ✅ Comprehensive README
- ✅ OpenAPI documentation
- 🚩 No README
- 🚩 Outdated docs
- 🚩 No type hints

-----

### VP-Q06: Technical Debt (Synthesis)

**Purpose:** Aggregate, classify, and prioritize all findings

**Input:** All previous viewpoints

**Debt Classification: Fowler Quadrant**

Technical Debt класифікується за двома вимірами:

```
                    DELIBERATE                    INADVERTENT
              (Свідомо прийнятий)            (Випадково створений)
         ┌────────────────────────────┬────────────────────────────┐
         │                            │                            │
         │   "We must ship now and    │   "Now we know how we      │
PRUDENT  │    deal with consequences" │    should have done it"    │
(Розсудливий) │                            │                            │
         │   → Schedule payback       │   → Refactor as you learn  │
         │   → Track explicitly       │   → Update standards       │
         │                            │                            │
         ├────────────────────────────┼────────────────────────────┤
         │                            │                            │
         │   "We don't have time      │   "What's layering?"       │
RECKLESS │    for design"             │                            │
(Безрозсудний) │                            │                            │
         │   → High risk, costly fix  │   → Training needed        │
         │   → Often leads to rewrites│   → May require rewrite    │
         │                            │                            │
         └────────────────────────────┴────────────────────────────┘
```

**Quadrant Strategies:**

|Quadrant            |Strategy               |Example                                          |
|--------------------|-----------------------|-------------------------------------------------|
|Prudent-Deliberate  |Track, schedule payback|"Hardcoded config for MVP, ticket to externalize"|
|Prudent-Inadvertent |Refactor incrementally |"Discovered better pattern, apply Boy Scout Rule"|
|Reckless-Deliberate |Urgent remediation     |"Skipped tests to meet deadline — critical risk" |
|Reckless-Inadvertent|Training + remediation |"Team didn't know about N+1 queries"             |

**Output:**

```yaml
technical_debt:
  summary:
    total_items: int
    by_category: {}
    by_severity: {}
    by_quadrant:
      prudent_deliberate: int
      prudent_inadvertent: int
      reckless_deliberate: int
      reckless_inadvertent: int
    estimated_total_effort: string

  items:
    - id: string
      category: security|reliability|maintainability|performance
      severity: critical|high|medium|low
      quadrant: prudent_deliberate|prudent_inadvertent|reckless_deliberate|reckless_inadvertent
      description: string
      location: path
      impact: string
      effort: S|M|L|XL
      recommendation: string
      root_cause: string  # Why this debt exists

  quadrant_analysis:
    prudent_deliberate:
      count: int
      examples: []
      strategy: "Track explicitly, schedule in roadmap"

    prudent_inadvertent:
      count: int
      examples: []
      strategy: "Refactor incrementally, update team knowledge"

    reckless_deliberate:
      count: int
      examples: []
      strategy: "Urgent remediation, process review"
      risk_assessment: string

    reckless_inadvertent:
      count: int
      examples: []
      strategy: "Training, pair programming, code review improvement"
      knowledge_gaps: []

  prioritized_backlog:
    immediate: []   # Critical security, reckless-deliberate
    short_term: []  # High impact, quick wins
    medium_term: [] # Strategic improvements
    long_term: []   # Nice to have

  root_cause_summary:
    - cause: string
      debt_items: [ids]
      systemic: boolean
      recommendation: string
```

**Prioritization Matrix:**

```
                    IMPACT
                High        Low
         ┌─────────────┬─────────────┐
    Low  │  QUICK WIN  │   BACKLOG   │
EFFORT   │   Do Now    │   Consider  │
         ├─────────────┼─────────────┤
    High │  STRATEGIC  │   AVOID     │
         │   Plan      │   Usually   │
         └─────────────┴─────────────┘
```

**AI Prompt Template:**

```
Analyze all findings from previous viewpoints:
[FINDINGS]

For each finding, determine:
1. Fowler Quadrant classification (with reasoning)
2. Root cause (why does this debt exist?)
3. Is this part of a pattern (systemic issue)?
4. Impact if not addressed
5. Remediation effort

Then synthesize:
1. Top root causes (fix these → resolve multiple symptoms)
2. Quadrant distribution (team health indicator)
3. Prioritized backlog with rationale
4. Knowledge gaps (for reckless-inadvertent items)
```

**Quality Indicators:**

- ✅ Majority in Prudent quadrants
- ✅ Clear root causes identified
- ✅ Actionable remediation plan
- 🚩 High Reckless-Deliberate count (process problem)
- 🚩 High Reckless-Inadvertent count (knowledge problem)
- 🚩 Same root cause across multiple findings (systemic issue)

-----

## 5. Execution Flow

### Phase 1: Foundation (1-2 hours)

1. VP-F01 → VP-F02 → VP-F03

### Phase 2: Structure (3-5 hours)

1. VP-S01 → VP-S02 → VP-S06
1. VP-S03 → VP-S04 → VP-S05
1. VP-S07 (Architecture Decisions)
1. `[OPTIONAL]` VP-S08, VP-S09, VP-S10

### Phase 3: Quality (2-4 hours)

1. VP-Q01 → VP-Q02 → VP-Q03 → VP-Q04 → VP-Q05

### Phase 4: Synthesis (1-2 hours)

1. VP-Q06 (aggregates all findings with Fowler Quadrant)

-----

## 6. Fitness Functions

### Concept

Fitness Functions — це executable specifications, що автоматично перевіряють архітектурні характеристики. Вони забезпечують continuous validation архітектурних рішень.

### Integration with Viewpoints

Кожен viewpoint може мати асоційовані fitness functions:

```yaml
viewpoint_fitness:
  VP-S02:
    - name: "Layer Dependencies"
      function: check_layer_dependencies
      frequency: per_commit

  VP-S06:
    - name: "No Vulnerabilities"
      function: check_no_vulnerable_dependencies
      frequency: daily

  VP-Q01:
    - name: "Security Headers"
      function: check_security_headers
      frequency: per_commit
```

### Implementation Pattern

```python
# fitness_functions/architecture.py

from dataclasses import dataclass
from typing import Callable, List
from enum import Enum

class Frequency(Enum):
    PER_COMMIT = "per_commit"
    DAILY = "daily"
    WEEKLY = "weekly"
    ON_DEMAND = "on_demand"

@dataclass
class FitnessFunction:
    name: str
    viewpoint: str
    function: Callable[[str], bool]
    frequency: Frequency
    severity: str = "high"

    def execute(self, project_path: str) -> dict:
        try:
            result = self.function(project_path)
            return {"status": "pass", "name": self.name}
        except AssertionError as e:
            return {"status": "fail", "name": self.name, "error": str(e)}

# Registry
FITNESS_FUNCTIONS: List[FitnessFunction] = [
    FitnessFunction(
        name="Layer Dependencies",
        viewpoint="VP-S02",
        function=check_layer_dependencies,
        frequency=Frequency.PER_COMMIT,
    ),
    FitnessFunction(
        name="No Circular Dependencies",
        viewpoint="VP-S01",
        function=check_no_circular_dependencies,
        frequency=Frequency.PER_COMMIT,
    ),
    # ... more functions
]

def run_fitness_suite(project_path: str, frequency: Frequency = None) -> dict:
    """Run all fitness functions, optionally filtered by frequency."""
    results = []
    functions = FITNESS_FUNCTIONS

    if frequency:
        functions = [f for f in functions if f.frequency == frequency]

    for ff in functions:
        results.append(ff.execute(project_path))

    passed = sum(1 for r in results if r["status"] == "pass")
    return {
        "total": len(results),
        "passed": passed,
        "failed": len(results) - passed,
        "results": results,
    }
```

### CI Integration

```yaml
# .github/workflows/fitness.yml
name: Architecture Fitness

on: [push, pull_request]

jobs:
  fitness:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Fitness Functions
        run: |
          python -m fitness_functions.runner \
            --frequency per_commit \
            --fail-on-error
```

-----

## 7. Context Engineering Layers

```yaml
context_layers:
  project_metadata:
    - technology_stack
    - conventions

  structural_context:
    - c4_model
    - layer_definitions
    - domain_model
    - architecture_decisions

  compliance_context:  # NEW
    - building_block_inventory
    - standards_baseline
    - team_ownership_map

  semantic_index:
    - code_embeddings
    - pattern_library

  external_knowledge:
    - framework_docs (via MCP)
    - vulnerability_db
    - best_practices KB
```

-----

## 8. Tool Integration Matrix

|Viewpoint|Static Tools         |AI Support                             |
|---------|---------------------|---------------------------------------|
|VP-F01   |jq, find             |Dependency categorization              |
|VP-F02   |tree, scc            |Structure analysis                     |
|VP-F03   |CI parsers           |Gap identification                     |
|VP-S01   |madge, pydeps        |Module grouping                        |
|VP-S02   |deptrac, ArchUnit    |Violation detection                    |
|VP-S06   |npm audit, pip-audit |Risk assessment                        |
|VP-S07   |grep, AST analysis   |Decision reconstruction                |
|VP-S08   |git log, CODEOWNERS  |Ownership analysis                     |
|VP-S09   |custom matchers      |Compliance scoring                     |
|VP-S10   |semgrep, custom rules|Policy verification                    |
|VP-Q01   |semgrep, bandit      |Vulnerability analysis                 |
|VP-Q03   |coverage tools       |Gap analysis                           |
|VP-Q04   |eslint, flake8, ruff |Consistency check                      |
|VP-Q06   |—                    |Prioritization, Quadrant classification|

-----

## 9. Viewpoint Dependencies

```
VP-F01 (Tech Stack)
    │
    ├──► VP-F02 (Structure)
    │       │
    │       ├──► VP-F03 (Build/Deploy)
    │       │
    │       └──► VP-S01 (Module Hierarchy)
    │               │
    │               ├──► VP-S02 (Layers)
    │               │       │
    │               │       └──► VP-S03 (Domain Model)
    │               │               │
    │               │               └──► VP-S04 (Entities)
    │               │                       │
    │               │                       └──► VP-S05 (Interfaces)
    │               │
    │               ├──► VP-S06 (Dependencies)
    │               │
    │               └──► VP-S07 (Architecture Decisions) ◄── NEW
    │
    ├──► [OPTIONAL] VP-S08 (Team Topologies)
    ├──► [OPTIONAL] VP-S09 (Building Block Compliance)
    └──► [OPTIONAL] VP-S10 (Standards Compliance)

    └──► VP-Q01..Q05 (Quality Viewpoints)
                │
                └──► VP-Q06 (Synthesis + Fowler Quadrant) ◄── ENHANCED
```

-----

## 10. Next Steps

1. **Prompt Templates** — детальні промпти для кожного viewpoint
1. **Discovery Scripts** — автоматизація збору даних
1. **MCP Servers** — knowledge access для external context
1. **Integration** — mapping на CodeClimate/SonarQube/CodeRabbit метрики
1. **Reporting** — шаблони звітів та візуалізації
1. **Fitness Function Library** — готові перевірки для common patterns
1. **Standards Library** — типові standards для різних industries

-----

## Appendix A: Glossary

|Term            |Definition                                                                          |
|----------------|------------------------------------------------------------------------------------|
|ADR             |Architecture Decision Record — документування архітектурних рішень                  |
|ABB             |Architecture Building Block — абстрактна capability                                 |
|SBB             |Solution Building Block — конкретна реалізація ABB                                  |
|Fitness Function|Executable specification для архітектурної характеристики                           |
|Fowler Quadrant |Класифікація technical debt за dimensions Deliberate/Inadvertent та Prudent/Reckless|
|Conway's Law    |Архітектура системи відображає комунікаційну структуру організації                  |

## Appendix B: Reference Architectures

Доступні reference architectures для VP-S09:

|Name                     |Languages |Key Patterns                            |
|-------------------------|----------|----------------------------------------|
|Clean Architecture Python|Python    |Domain/Application/Infrastructure layers|
|Hexagonal TypeScript     |TypeScript|Ports & Adapters                        |
|Modular Monolith         |Any       |Bounded context modules                 |
|Microservices            |Any       |Service mesh, API Gateway               |

-----

*Документ підготовлено для Light IT Global*
*Версія 2.0 — Січень 2026*

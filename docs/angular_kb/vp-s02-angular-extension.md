---
name: vp-s02-angular-extension
version: 1.0
extends: VP-S02
framework: angular
dependencies:
  - vp-f01-tech-stack
  - vp-f02-structure
---

# VP-S02 Extension: Angular Layer Architecture Analysis

## Purpose

Specialized layer architecture analysis for Angular applications. Extends standard VP-S02 by leveraging Angular's module system, dependency injection hierarchy, and TypeScript import patterns for boundary enforcement.

## Angular-Specific Context

Angular provides **runtime boundary enforcement** through:
- NgModule imports/exports (what's visible between modules)
- Hierarchical injector (service scope management)
- Lazy-loaded module isolation (separate injector branches)
- TypeScript path aliases (enforce import patterns)

Unlike compile-time enforcement (Rust workspaces), Angular violations **will compile but cause runtime issues** or architectural debt.

## Detection Strategy

### Step 1: Determine Project Structure Type

```bash
# Check for Nx workspace
if [ -f "nx.json" ]; then
  echo "NX_WORKSPACE"
elif [ -f "angular.json" ]; then
  echo "STANDALONE_APP"
fi

# Check for standalone components (Angular 14+)
grep -r "standalone: true" src/app --include="*.ts" | wc -l
```

**Decision Tree:**
- Nx Workspace → Analyze lib boundaries via `project.json` tags
- Standalone App → Analyze module imports and folder structure
- Standalone Components → Analyze direct imports between components

### Step 2: Module Boundary Analysis (NgModule-based)

```bash
# Find all module files
find src -name "*.module.ts" -type f

# Extract imports for each module
for module in $(find src -name "*.module.ts"); do
  echo "=== $module ==="
  grep -A 30 "imports:" "$module" | grep -E "Module|Component" | head -15
done

# Find cross-feature imports (potential violations)
grep -rn "import.*from.*features/" src/app/features --include="*.ts" | \
  grep -v "\.spec\.ts" | \
  awk -F: '{print $1 " -> " $0}'
```

**Expected Pattern (Healthy):**
```
features/users/users.module.ts:
  imports: [SharedModule, CoreModule]  ✅

features/orders/orders.module.ts:
  imports: [SharedModule, CoreModule]  ✅
```

**Violation Pattern:**
```
features/users/users.module.ts:
  imports: [OrdersModule]  # ❌ Feature imports another feature
```

### Step 3: Domain Layer Purity Check

```bash
# Domain folder should have ZERO Angular imports
echo "=== Angular imports in domain layer ==="
grep -rn "@angular" src/app/domain --include="*.ts"
grep -rn "from '@angular" src/app/domain --include="*.ts"

# Should only see pure TypeScript
echo "=== Expected: Only pure TS imports ==="
grep -rn "^import" src/app/domain --include="*.ts" | head -20
```

**Healthy Output:**
```
domain/models/user.model.ts: import { Address } from './address.model';
domain/interfaces/repository.ts: import { Observable } from 'rxjs';
# NO @angular imports
```

**Violation Output:**
```
domain/services/pricing.service.ts: import { Injectable } from '@angular/core';  # ❌
```

### Step 4: Smart/Dumb Component Analysis

```bash
# Find components with service injections (smart)
echo "=== Smart Components (with inject/constructor DI) ==="
grep -rln "inject(" src/app/features --include="*.component.ts"
grep -rln "constructor.*private.*Service" src/app/features --include="*.component.ts"

# Find components with only Input/Output (dumb)
echo "=== Dumb Components (Input/Output only) ==="
for comp in $(find src/app -name "*.component.ts"); do
  if grep -q "@Input\|@Output" "$comp" && ! grep -q "inject(\|constructor.*private" "$comp"; then
    echo "$comp"
  fi
done

# Check OnPush usage on dumb components
echo "=== Dumb components missing OnPush ==="
for comp in $(find src/app/shared -name "*.component.ts"); do
  if ! grep -q "ChangeDetectionStrategy.OnPush" "$comp"; then
    echo "MISSING OnPush: $comp"
  fi
done
```

### Step 5: Service Layer Analysis

```bash
# Find services with providedIn: 'root' (global singletons)
echo "=== Root-provided services ==="
grep -rln "providedIn: 'root'" src/app --include="*.service.ts"

# Find services provided in modules (scoped)
echo "=== Module-scoped services ==="
grep -rn "providers:" src/app --include="*.module.ts" | grep -v "\.spec"

# Find facade services
echo "=== Facade services ==="
grep -rln "\.facade\." src/app --include="*.ts"
find src -name "*.facade.ts" -type f
```

### Step 6: Subscription Leak Detection

```bash
# Find manual subscriptions
echo "=== Manual subscriptions (potential leaks) ==="
grep -rn "\.subscribe(" src/app --include="*.component.ts" | grep -v "\.spec\."

# Check for proper cleanup
echo "=== Components with subscriptions but no cleanup ==="
for comp in $(grep -rl "\.subscribe(" src/app --include="*.component.ts"); do
  if ! grep -q "takeUntil\|takeUntilDestroyed\|ngOnDestroy\|async" "$comp"; then
    echo "POTENTIAL LEAK: $comp"
  fi
done
```

### Step 7: Nx Library Boundary Analysis (if applicable)

```bash
# Check library tags
echo "=== Library tags ==="
for proj in libs/*/project.json; do
  echo "=== $proj ==="
  jq '.tags' "$proj"
done

# Run Nx lint for boundary violations
npx nx run-many --target=lint --all 2>&1 | grep "error.*enforce-module-boundaries"
```

## Output Schema

```yaml
angular_layer_architecture:
  structure_type: "nx_workspace" | "standalone_app"
  angular_version: string
  
  # Module organization
  module_analysis:
    total_modules: int
    feature_modules: list[string]
    lazy_loaded: list[string]
    eagerly_loaded: list[string]
    cross_feature_imports: list[CrossFeatureViolation]
  
  # Domain layer assessment
  domain_purity:
    has_domain_folder: boolean
    angular_imports_count: int
    violations: list[DomainViolation]
    purity_score: 0-100
  
  # Component classification
  component_analysis:
    total_components: int
    smart_components: list[ComponentInfo]
    dumb_components: list[ComponentInfo]
    unclassified: list[string]
    onpush_coverage: percentage
  
  # Service layer
  service_analysis:
    root_provided: list[string]
    module_scoped: list[string]
    facades: list[string]
    missing_abstractions: list[string]
  
  # RxJS health
  rxjs_analysis:
    manual_subscriptions: int
    potential_leaks: list[string]
    async_pipe_usage: int
    proper_cleanup: list[string]
  
  # Nx-specific (if applicable)
  nx_analysis:
    library_count: int
    boundary_violations: list[NxViolation]
    tag_coverage: percentage
  
  # Overall assessment
  pattern_detected: "Clean Architecture" | "Feature Modules" | "Mixed" | "Unknown"
  confidence: "high" | "medium" | "low"
  
  violations:
    - type: string
      location: string
      severity: "critical" | "high" | "medium" | "low"
      description: string
      fix_suggestion: string
```

## Severity Adjustment Rules

### Angular-Specific Severity Multipliers

| Finding | Base Severity | Angular Adjustment | Rationale |
|---------|---------------|-------------------|-----------|
| Feature imports another feature | MEDIUM | → HIGH (×1.5) | Breaks module isolation |
| Angular imports in domain | HIGH | → HIGH (×1.3) | Framework coupling |
| Missing OnPush on dumb component | LOW | → MEDIUM (×1.5) | Performance impact |
| Subscription without cleanup | MEDIUM | → HIGH (×1.5) | Memory leak risk |
| HTTP in component | MEDIUM | → MEDIUM (×1.2) | Architecture violation |
| God component (>300 lines) | MEDIUM | → HIGH (×1.3) | Maintainability risk |
| No lazy loading for large feature | LOW | → MEDIUM (×1.5) | Bundle size impact |

### Context Multipliers from Mental Model

```yaml
severity_multipliers:
  core_feature_module: 1.5      # User auth, payments, etc.
  high_traffic_component: 1.4    # Dashboard, list views
  shared_component: 1.3          # Widely reused
  low_test_coverage: 1.2
```

## AI Prompt Template

```markdown
Analyze this Angular project for layer architecture compliance:

## Project Context
- Structure: {{structure_type}}
- Angular Version: {{angular_version}}
- State Management: {{state_library}}
- Total Components: {{component_count}}

## Collected Data

### Module Structure
```
{{module_tree}}
```

### Module Imports
```
{{module_imports}}
```

### Domain Layer Files
```
{{domain_files}}
```

### Angular Imports in Domain
```
{{domain_angular_imports}}
```

### Smart Components (with injections)
```
{{smart_components}}
```

### Dumb Components (Input/Output only)
```
{{dumb_components}}
```

### Subscription Analysis
```
{{subscription_analysis}}
```

## Analysis Required

1. **Module Boundary Analysis**:
   - Are feature modules properly isolated?
   - Any cross-feature imports?
   - Which modules should be lazy-loaded but aren't?

2. **Domain Layer Purity**:
   - Does domain folder exist and follow conventions?
   - Any Angular framework imports in domain?
   - Can domain be extracted to shared library?

3. **Component Architecture**:
   - Is smart/dumb separation clear?
   - Are dumb components using OnPush?
   - Any god components (>200 lines)?

4. **State Management**:
   - Is there a consistent state management approach?
   - Are facades hiding implementation details?
   - Is state properly scoped?

5. **RxJS Health**:
   - Any subscription leaks?
   - Proper cleanup patterns used?
   - Async pipe preferred over manual subscriptions?

6. **Dependency Injection**:
   - Appropriate use of providedIn scopes?
   - Interface-based injection where needed?
   - Service layer properly abstracted?

## Output

Provide findings in the schema format above. Include specific file paths
and line numbers where possible. Prioritize by impact on maintainability.
```

## Quality Indicators

### Healthy Angular Architecture ✅

- Feature modules lazy-loaded via router
- Domain folder with zero `@angular` imports
- Clear smart/dumb component separation
- OnPush on all presentational components
- Facades abstract state management
- Async pipe preferred over manual subscriptions
- InjectionToken used for swappable dependencies
- Nx tags enforce boundaries (if monorepo)

### Warning Signs ⚠️

- Components > 200 lines
- Mixed smart/dumb patterns in same folder
- Direct HTTP calls in some components
- Manual subscriptions without clear cleanup
- Shared module importing feature-specific code
- providedIn: 'root' for feature-specific services

### Critical Issues 🚨

- Feature module imports another feature module
- Angular decorators in domain layer
- God components (> 300 lines, > 5 services)
- Observable subscriptions without any cleanup pattern
- Circular dependencies between modules
- Business logic in templates
- State mutation outside designated services

## Constraints Produced

After VP-S02 Angular analysis:

```yaml
constraints:
  security_focus_paths:
    - "src/app/core/guards/**"
    - "src/app/core/interceptors/**"
    - "**/auth/**"
  
  performance_critical:
    - path: string
      reason: "missing_onpush" | "large_list" | "heavy_computation"
  
  refactoring_candidates:
    - path: string
      reason: "god_component" | "feature_coupling" | "subscription_leak"
      effort: "S" | "M" | "L"
  
  architecture_pattern: "Clean Architecture" | "Feature Modules" | "Mixed"
  
  angular_specific:
    needs_lazy_loading: list[string]
    domain_purity_violations: list[string]
    subscription_leak_risks: list[string]
    missing_facades: list[string]
```

---

## Examples

### Example 1: Clean Feature Module Structure

```
src/app/features/users/
├── containers/
│   └── user-list/
│       └── user-list.container.ts    # inject(UserFacade)
├── components/
│   └── user-card/
│       └── user-card.component.ts    # OnPush, @Input/@Output only
├── services/
│   ├── user.facade.ts
│   └── user-api.service.ts
├── state/
│   └── user.store.ts
└── users.module.ts                    # imports: [SharedModule]
```

**Assessment:**
- ✅ Smart/dumb separation clear
- ✅ Facade abstracts state
- ✅ No cross-feature imports

### Example 2: Problematic Structure

```typescript
// features/users/user-list.component.ts
@Component({
  // Missing OnPush!
})
export class UserListComponent implements OnInit {
  users: User[] = [];
  
  constructor(
    private http: HttpClient,           // ❌ Direct HTTP
    private orderService: OrderService  // ❌ Cross-feature dependency
  ) {}
  
  ngOnInit() {
    this.http.get('/api/users').subscribe(users => {  // ❌ Subscription leak
      this.users = users;
      this.orderService.loadOrdersForUsers(users);     // ❌ Feature coupling
    });
  }
}
```

**Assessment:**
- 🚨 Direct HTTP call in component
- 🚨 Cross-feature service injection
- 🚨 Subscription without cleanup
- ⚠️ Missing OnPush change detection
- 🚨 Component mixing smart/dumb concerns

**Recommendation:**
```typescript
// user-list.container.ts (Smart)
@Component({
  template: `<app-user-list [users]="users$ | async" />`,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class UserListContainerComponent {
  private facade = inject(UserFacade);
  users$ = this.facade.users$;
  
  ngOnInit() {
    this.facade.loadUsers();
  }
}

// user-list.component.ts (Dumb)
@Component({
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class UserListComponent {
  @Input({ required: true }) users!: User[];
  // NO injections, NO subscriptions
}
```

### Example 3: Domain Purity Violation

```typescript
// ❌ domain/services/pricing.service.ts
import { Injectable } from '@angular/core';  // VIOLATION!

@Injectable({ providedIn: 'root' })
export class PricingService {
  calculateDiscount(order: Order): number {
    // Business logic...
  }
}
```

**Assessment:**
- 🚨 Angular decorator in domain layer
- Cannot extract to shared library
- Tightly coupled to Angular DI

**Recommendation:**
```typescript
// ✅ domain/services/pricing.service.ts (Pure TypeScript)
export class PricingService {
  calculateDiscount(order: Order): number {
    // Business logic...
  }
}

// ✅ core/providers.ts (Angular wiring)
import { PricingService } from '@domain/services/pricing.service';

export const PRICING_SERVICE = new InjectionToken<PricingService>('PricingService');

export const domainProviders = [
  { provide: PRICING_SERVICE, useClass: PricingService }
];
```

---

## Angular Version Considerations

### Angular 14-15

- Check for standalone component adoption
- Typed forms usage
- `inject()` function vs constructor DI

### Angular 16+

- Signal usage for state
- `takeUntilDestroyed()` for cleanup
- Required inputs

### Angular 17+

- New control flow syntax (`@if`, `@for`)
- Deferrable views for lazy loading
- Check migration from structural directives

**Version Detection:**
```bash
cat package.json | jq '.dependencies["@angular/core"]'
```

---

*Extension for VP-S02 Layer Architecture Viewpoint*
*Part of AI Code Audit Agent Methodology Knowledge Base*

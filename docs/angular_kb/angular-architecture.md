# Angular Architecture Glossary

> Extension for Methodology KB Glossary | AI Code Audit Agent

---

## Core Concepts

### Smart Component (Container Component)
**Definition:** A component that handles orchestration, state management, and service interaction. Injects services, manages data flow, and delegates presentation to dumb components.

**Architecture Role:** The "controller" in MVC terms. Acts as adapter between application state and UI presentation.

**Characteristics:**
- Injects facade/state services
- Handles user interaction events
- Minimal template logic
- Often uses `| async` pipe

**Example:**
```typescript
@Component({ template: `<app-user-list [users]="users$ | async" />` })
export class UserContainerComponent {
  private facade = inject(UserFacade);
  users$ = this.facade.users$;
}
```

### Dumb Component (Presentational Component)
**Definition:** A component focused purely on presentation. Receives data via `@Input()`, emits events via `@Output()`, and has zero service dependencies.

**Architecture Role:** Pure UI rendering. Can be reused across features without modification.

**Characteristics:**
- Zero service injections
- Only `@Input()` and `@Output()`
- OnPush change detection
- Highly testable in isolation

**Example:**
```typescript
@Component({ changeDetection: ChangeDetectionStrategy.OnPush })
export class UserCardComponent {
  @Input({ required: true }) user!: User;
  @Output() selected = new EventEmitter<User>();
}
```

### Facade Service
**Definition:** A service that provides a simplified interface to a complex subsystem (state management, multiple API services, etc.).

**Architecture Role:** Acts as "port" in hexagonal architecture. Components depend on facades, never on implementation details.

**Pattern:**
```typescript
@Injectable({ providedIn: 'root' })
export class UserFacade {
  readonly users$ = this.state.users$;  // Expose observables
  
  loadUsers(): void { /* orchestrate */ }  // Expose commands
}
```

### Feature Module
**Definition:** An Angular NgModule that encapsulates a specific feature area of the application, typically lazy-loaded.

**Architecture Role:** Bounded context in DDD terms. Self-contained unit with its own components, services, and state.

**Characteristics:**
- Lazy-loaded via router
- Own injector scope
- Imports SharedModule, not other features
- Self-contained routing

---

## Angular Module System

### NgModule
**Definition:** Angular's mechanism for organizing related components, directives, pipes, and services into cohesive blocks.

**Types:**
- **Root Module (AppModule):** Bootstrap module
- **Core Module:** Singleton services, guards, interceptors
- **Shared Module:** Reusable components, pipes, directives
- **Feature Module:** Domain-specific functionality

### Standalone Component (Angular 14+)
**Definition:** A component that declares its own dependencies without requiring an NgModule.

**Migration Pattern:**
```typescript
@Component({
  standalone: true,
  imports: [CommonModule, ButtonComponent],
  // ...
})
export class UserCardComponent { }
```

### Lazy Loading
**Definition:** Loading feature modules on-demand when the user navigates to a route, rather than at application startup.

**Implementation:**
```typescript
{ path: 'users', loadChildren: () => import('./features/users/users.module').then(m => m.UsersModule) }
```

**Impact:** Reduces initial bundle size, improves time-to-interactive.

### Barrel Export (index.ts)
**Definition:** A file that re-exports multiple modules from a single entry point.

**Risk:** Can cause circular dependencies and slow compilation when overused.

**Best Practice:** Flatten exports, be explicit about what's public API.

---

## Dependency Injection

### Hierarchical Injector
**Definition:** Angular's DI system where injectors form a tree that mirrors the component tree, with child injectors inheriting from parents.

**Scopes:**
- Root Injector → Application singletons
- Module Injector → Per lazy-loaded module
- Component Injector → Per component instance

### InjectionToken
**Definition:** A token that enables injection of non-class dependencies or abstract interfaces.

**Use Case:** Dependency inversion, swappable implementations.

**Example:**
```typescript
export const USER_REPOSITORY = new InjectionToken<UserRepository>('UserRepository');
providers: [{ provide: USER_REPOSITORY, useClass: HttpUserRepository }]
```

### providedIn
**Definition:** Metadata option that specifies where a service should be registered.

**Options:**
- `'root'` → Application singleton
- `'platform'` → Shared across apps
- `FeatureModule` → Module-scoped (for lazy modules)

### inject() Function (Angular 14+)
**Definition:** Functional alternative to constructor injection.

**Advantage:** Works in injection contexts (field initializers, factory functions).

**Example:**
```typescript
export class UserService {
  private http = inject(HttpClient);
}
```

---

## State Management

### BehaviorSubject
**Definition:** RxJS subject that requires an initial value and emits current value to new subscribers.

**Use Case:** Local state management in services.

**Pattern:**
```typescript
private usersSubject = new BehaviorSubject<User[]>([]);
readonly users$ = this.usersSubject.asObservable();
```

### Signal (Angular 16+)
**Definition:** Angular's reactive primitive for state management with fine-grained reactivity.

**Types:**
- `signal()` → Writable state
- `computed()` → Derived state
- `effect()` → Side effects

**Example:**
```typescript
users = signal<User[]>([]);
userCount = computed(() => this.users().length);
```

### NgRx Store
**Definition:** Redux-inspired state management library for Angular with strict unidirectional data flow.

**Components:**
- Actions → Events describing state changes
- Reducers → Pure functions computing new state
- Selectors → Functions deriving data from state
- Effects → Side effect handlers

### toSignal() (Angular 16+)
**Definition:** Utility to convert an Observable to a Signal.

**Example:**
```typescript
users = toSignal(this.userService.users$, { initialValue: [] });
```

---

## Change Detection

### OnPush Change Detection
**Definition:** Change detection strategy that only checks a component when its inputs change by reference, an event originates from the component, or an observable bound with async pipe emits.

**Importance:** Critical for performance in presentational components.

**Trigger Conditions:**
1. `@Input()` reference changes
2. DOM event from component or child
3. `async` pipe receives emission
4. `markForCheck()` called manually

### Default Change Detection
**Definition:** Angular checks the entire component tree on every async event (click, timer, HTTP response).

**When to Use:** Smart components where inputs change frequently, debugging scenarios.

### ChangeDetectorRef
**Definition:** Service to interact with change detection manually.

**Methods:**
- `markForCheck()` → Schedule check (OnPush)
- `detectChanges()` → Immediate check
- `detach()` → Stop checking

### Zoneless (Angular 18+)
**Definition:** Running Angular without Zone.js, using signals for change detection triggers.

**Status:** Experimental, gaining stability.

---

## RxJS Patterns

### Subscription Leak
**Definition:** Memory leak caused by not unsubscribing from observables when a component is destroyed.

**Symptoms:** Memory growth, stale callbacks, duplicate API calls.

**Solutions:**
- `async` pipe (automatic)
- `takeUntilDestroyed()` (Angular 16+)
- `takeUntil(destroy$)` pattern
- Explicit `unsubscribe()` in `ngOnDestroy`

### takeUntilDestroyed() (Angular 16+)
**Definition:** Operator that automatically completes an observable when the injection context is destroyed.

**Example:**
```typescript
this.service.data$.pipe(
  takeUntilDestroyed()
).subscribe(data => this.process(data));
```

### Async Pipe
**Definition:** Angular pipe that subscribes to an observable, returns emitted values, and automatically unsubscribes on destroy.

**Best Practice:** Preferred over manual subscriptions for template bindings.

**Example:**
```html
<div *ngIf="users$ | async as users">
  {{ users.length }} users
</div>
```

### Marble Testing
**Definition:** Testing technique using ASCII diagrams to represent observable sequences over time.

**Example:**
```typescript
const source$ = cold('--a--b--c|', { a: 1, b: 2, c: 3 });
const expected = '   --x--y--z|';
expectObservable(source$.pipe(map(x => x * 10))).toBe(expected, { x: 10, y: 20, z: 30 });
```

---

## Performance Terms

### trackBy
**Definition:** Function that helps Angular identify items in an `*ngFor` loop to minimize DOM manipulation.

**Importance:** Without it, entire list re-renders on any change.

**Example:**
```typescript
trackByUserId(index: number, user: User): string {
  return user.id;
}
```

### Virtual Scrolling
**Definition:** Rendering only visible items in a long list, recycling DOM elements as user scrolls.

**Implementation:** `@angular/cdk/scrolling` module.

### Tree Shaking
**Definition:** Build optimization that removes unused code from the final bundle.

**Enabled By:** ES modules, `providedIn: 'root'`, avoiding barrel exports.

### Preloading Strategy
**Definition:** Configuring how/when lazy-loaded modules are fetched.

**Options:**
- `NoPreloading` → Load on navigation only
- `PreloadAllModules` → Load all after initial
- Custom strategy → Selective preloading

---

## Testing Terms

### TestBed
**Definition:** Angular's testing utility for configuring testing modules, creating components, and injecting dependencies.

**Example:**
```typescript
TestBed.configureTestingModule({
  imports: [UserCardComponent],
  providers: [{ provide: UserService, useValue: mockService }]
});
```

### HttpTestingController
**Definition:** Service for mocking HTTP requests in unit tests.

**Pattern:**
```typescript
const req = httpMock.expectOne('/api/users');
req.flush(mockUsers);
httpMock.verify();
```

### Shallow Testing
**Definition:** Testing a component in isolation by stubbing child components.

**Use Case:** Unit testing smart components without their children.

### Spectator
**Definition:** Popular testing library that simplifies Angular component testing with cleaner API.

**Example:**
```typescript
const spectator = createComponentFactory(UserCardComponent);
const component = spectator.createComponent({ props: { user: mockUser } });
```

---

## Architecture Anti-Patterns

### God Component
**Definition:** Component with too many responsibilities, typically > 300 lines, > 5 service injections.

**Symptoms:** Hard to test, hard to understand, frequent merge conflicts.

**Fix:** Extract to smart/dumb hierarchy, delegate to services.

### Service Locator
**Definition:** Anti-pattern of using `Injector.get()` to dynamically resolve dependencies instead of constructor injection.

**Problem:** Hidden dependencies, harder testing, breaks IDE tooling.

### Shared Module Bloat
**Definition:** SharedModule that imports too much, used everywhere, causing bundle bloat.

**Fix:** Granular modules or standalone components.

### Barrel Export Hell
**Definition:** Deep chains of re-exports causing circular dependencies and slow builds.

**Fix:** Flatten exports, explicit imports, lint rules.

---

## Structural Terms

### Core Module
**Definition:** Module containing singleton services, guards, interceptors—imported once by AppModule.

**Contents:**
- Global error handler
- Auth service
- HTTP interceptors
- Route guards

### Domain Layer
**Definition:** Pure TypeScript layer containing business entities, interfaces, and logic without Angular dependencies.

**Requirement:** Zero `@angular/*` imports.

### Nx Workspace
**Definition:** Monorepo tooling for Angular (and other frameworks) with library organization, caching, and dependency graph analysis.

**Key Feature:** `@nx/enforce-module-boundaries` lint rule for architectural enforcement.

### Library Tags (Nx)
**Definition:** Metadata tags on Nx libraries used to define dependency rules.

**Common Tags:**
- `type:domain` → Pure domain logic
- `type:data-access` → State and API
- `type:feature` → Feature modules
- `type:ui` → Shared UI components

---

## Angular-Specific Control Flow (v17+)

### @if / @else
**Definition:** Built-in template syntax replacing `*ngIf` structural directive.

**Example:**
```html
@if (users.length > 0) {
  <app-user-list [users]="users" />
} @else {
  <p>No users found</p>
}
```

### @for / @empty
**Definition:** Built-in template syntax replacing `*ngFor` with required track expression.

**Example:**
```html
@for (user of users; track user.id) {
  <app-user-card [user]="user" />
} @empty {
  <p>No users</p>
}
```

### @defer
**Definition:** Declarative lazy loading for template sections.

**Example:**
```html
@defer (on viewport) {
  <app-heavy-component />
} @placeholder {
  <app-skeleton />
}
```

---

*Glossary Extension for Angular Architecture*
*Part of AI Code Audit Agent Methodology Knowledge Base*

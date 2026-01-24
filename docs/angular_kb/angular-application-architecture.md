# Angular Application Architecture Knowledge Base

> **Methodology KB** | AI Code Audit Agent | January 2026

---

## 1. Overview

This document provides comprehensive guidance for auditing Angular applications, particularly enterprise-scale SPAs. The core insight is that **Angular's module system and dependency injection already provide architectural guardrails**—you structure to leverage these mechanisms, not fight them.

### Why Clean Architecture Maps Well to Angular

Angular's built-in features align naturally with clean architecture concepts:

| Clean Architecture Concept | Angular Implementation |
|---------------------------|------------------------|
| Use Cases / Interactors | Facade Services |
| Entities / Domain | Pure TypeScript classes in `/domain` |
| Interface Adapters | Smart Components, Services |
| Frameworks & Drivers | Angular itself, HTTP, Storage |
| Dependency Inversion | InjectionToken + useClass |
| Bounded Contexts | Feature Modules (lazy-loaded) |

This mapping provides **runtime boundary enforcement** through Angular's hierarchical injector and module system.

---

## 2. Structural Patterns

### 2.1 Standalone Application Structure

**Use when:** Single application, team < 10 developers

```
src/app/
├── core/                       # Singleton services, guards, interceptors
│   ├── services/
│   │   ├── auth.service.ts
│   │   └── error-handler.service.ts
│   ├── guards/
│   ├── interceptors/
│   └── core.module.ts          # Imported once in AppModule
│
├── shared/                     # Reusable, stateless components
│   ├── components/
│   │   ├── button/
│   │   └── modal/
│   ├── directives/
│   ├── pipes/
│   └── shared.module.ts        # Imported in feature modules
│
├── domain/                     # Pure TypeScript - NO Angular imports
│   ├── models/
│   │   ├── user.model.ts
│   │   └── order.model.ts
│   ├── interfaces/
│   │   ├── user-repository.interface.ts
│   │   └── order-repository.interface.ts
│   └── services/               # Pure business logic
│       └── pricing.service.ts
│
├── features/                   # Feature modules (lazy-loaded)
│   ├── users/
│   │   ├── containers/         # Smart components
│   │   │   └── user-list/
│   │   ├── components/         # Dumb components
│   │   │   └── user-card/
│   │   ├── services/
│   │   │   ├── user.facade.ts
│   │   │   └── user-api.service.ts
│   │   ├── state/              # Feature-specific state
│   │   └── users.module.ts
│   └── orders/
│       └── ...
│
└── app.module.ts
```

**Module Dependency Rules:**
- `core` → imports `domain`, Angular core
- `shared` → **no** service dependencies, pure presentation
- `domain` → **zero** Angular imports
- `features/*` → imports `core`, `shared`, `domain`
- Features **never** import from other features directly

### 2.2 Nx Monorepo Structure

**Use when:** Multiple applications, team > 10 developers, shared libraries needed

```
workspace/
├── apps/
│   ├── customer-portal/        # Main customer app
│   │   └── src/
│   └── admin-dashboard/        # Admin app
│       └── src/
│
├── libs/
│   ├── domain/                 # Pure TypeScript domain
│   │   ├── src/
│   │   │   ├── models/
│   │   │   └── interfaces/
│   │   └── project.json
│   │
│   ├── data-access/            # State + API layer
│   │   ├── user/
│   │   │   ├── src/
│   │   │   │   ├── +state/     # NgRx or signal state
│   │   │   │   ├── services/
│   │   │   │   └── user.facade.ts
│   │   │   └── project.json
│   │   └── order/
│   │
│   ├── feature/                # Feature libraries
│   │   ├── user-management/
│   │   │   ├── src/
│   │   │   │   ├── containers/
│   │   │   │   ├── components/
│   │   │   │   └── feature-user-management.module.ts
│   │   │   └── project.json
│   │   └── order-processing/
│   │
│   └── ui/                     # Shared UI components
│       ├── src/
│       │   ├── button/
│       │   └── modal/
│       └── project.json
│
├── nx.json
└── tsconfig.base.json
```

**Nx Dependency Constraints (eslint):**
```json
{
  "@nx/enforce-module-boundaries": [
    "error",
    {
      "depConstraints": [
        { "sourceTag": "type:app", "onlyDependOnLibsWithTags": ["type:feature", "type:ui", "type:data-access"] },
        { "sourceTag": "type:feature", "onlyDependOnLibsWithTags": ["type:ui", "type:data-access", "type:domain"] },
        { "sourceTag": "type:data-access", "onlyDependOnLibsWithTags": ["type:domain"] },
        { "sourceTag": "type:ui", "onlyDependOnLibsWithTags": ["type:domain"] },
        { "sourceTag": "type:domain", "onlyDependOnLibsWithTags": [] }
      ]
    }
  ]
}
```

---

## 3. Component Patterns

### 3.1 Smart/Dumb (Container/Presentational) Split

The fundamental component architecture pattern in Angular:

| Aspect | Smart (Container) | Dumb (Presentational) |
|--------|-------------------|----------------------|
| Purpose | Orchestration | Display |
| Services | Injects facades | **None** |
| Data | Fetches from state | Receives via `@Input()` |
| Events | Handles directly | Emits via `@Output()` |
| Change Detection | Default | **OnPush** |
| Testing | Integration (with mocks) | Unit (isolated) |

### 3.2 Smart Component Example

```typescript
// user-list.container.ts
@Component({
  selector: 'app-user-list-container',
  template: `
    <app-user-list
      [users]="users$ | async"
      [loading]="loading$ | async"
      (userSelected)="onUserSelect($event)"
      (deleteRequested)="onDelete($event)"
    />
  `,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class UserListContainerComponent {
  private facade = inject(UserFacade);
  
  users$ = this.facade.users$;
  loading$ = this.facade.loading$;
  
  onUserSelect(user: User): void {
    this.facade.selectUser(user.id);
  }
  
  onDelete(userId: string): void {
    this.facade.deleteUser(userId);
  }
}
```

### 3.3 Dumb Component Example

```typescript
// user-list.component.ts
@Component({
  selector: 'app-user-list',
  templateUrl: './user-list.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush  // CRITICAL
})
export class UserListComponent {
  @Input({ required: true }) users!: User[];
  @Input() loading = false;
  
  @Output() userSelected = new EventEmitter<User>();
  @Output() deleteRequested = new EventEmitter<string>();
  
  // NO constructor injections
  // NO service dependencies
  // Pure presentation logic only
  
  trackByUserId(index: number, user: User): string {
    return user.id;
  }
}
```

### 3.4 When to Split

| Indicator | Action |
|-----------|--------|
| Component > 150 lines | Consider splitting |
| Multiple service injections | Extract container |
| Reusable presentation | Extract dumb component |
| Complex template logic | Extract to component or pipe |
| Multiple responsibilities | Split by concern |

---

## 4. State Management Patterns

### 4.1 Facade Pattern (Recommended Default)

The facade acts as the **port** in hexagonal architecture—components never know implementation details.

```typescript
// user.facade.ts
@Injectable({ providedIn: 'root' })
export class UserFacade {
  private state = inject(UserStateService);
  private api = inject(UserApiService);
  
  // Expose state as observables (read-only to consumers)
  readonly users$ = this.state.users$;
  readonly selectedUser$ = this.state.selectedUser$;
  readonly loading$ = this.state.loading$;
  
  // Commands (imperative API)
  loadUsers(): void {
    this.state.setLoading(true);
    this.api.getUsers().pipe(
      tap(users => this.state.setUsers(users)),
      finalize(() => this.state.setLoading(false))
    ).subscribe();
  }
  
  selectUser(id: string): void {
    this.state.selectUser(id);
  }
  
  deleteUser(id: string): void {
    this.api.deleteUser(id).pipe(
      tap(() => this.state.removeUser(id))
    ).subscribe();
  }
}
```

```typescript
// user-state.service.ts
@Injectable({ providedIn: 'root' })
export class UserStateService {
  private usersSubject = new BehaviorSubject<User[]>([]);
  private selectedIdSubject = new BehaviorSubject<string | null>(null);
  private loadingSubject = new BehaviorSubject<boolean>(false);
  
  readonly users$ = this.usersSubject.asObservable();
  readonly loading$ = this.loadingSubject.asObservable();
  readonly selectedUser$ = combineLatest([this.users$, this.selectedIdSubject]).pipe(
    map(([users, id]) => users.find(u => u.id === id) ?? null)
  );
  
  setUsers(users: User[]): void {
    this.usersSubject.next(users);
  }
  
  setLoading(loading: boolean): void {
    this.loadingSubject.next(loading);
  }
  
  // ... other mutations
}
```

### 4.2 Signal-Based State (Angular 16+)

Modern approach using Angular Signals:

```typescript
// user.store.ts
@Injectable({ providedIn: 'root' })
export class UserStore {
  // State
  private users = signal<User[]>([]);
  private selectedId = signal<string | null>(null);
  private loading = signal(false);
  
  // Selectors (computed)
  readonly userList = this.users.asReadonly();
  readonly isLoading = this.loading.asReadonly();
  readonly selectedUser = computed(() => 
    this.users().find(u => u.id === this.selectedId()) ?? null
  );
  readonly userCount = computed(() => this.users().length);
  
  // Actions
  setUsers(users: User[]): void {
    this.users.set(users);
  }
  
  addUser(user: User): void {
    this.users.update(current => [...current, user]);
  }
  
  setLoading(value: boolean): void {
    this.loading.set(value);
  }
}
```

### 4.3 NgRx (Complex State)

**Use when:** Complex state with time-travel debugging, undo/redo, or strict immutability requirements.

```
feature/
├── +state/
│   ├── user.actions.ts
│   ├── user.reducer.ts
│   ├── user.selectors.ts
│   ├── user.effects.ts
│   └── user.facade.ts      # Still use facade to hide NgRx
└── ...
```

**Critical Rule:** Even with NgRx, wrap in facade. Components should never directly dispatch actions or select from store.

---

## 5. Dependency Injection Patterns

### 5.1 Interface-Based Injection

Angular's DI enables the Dependency Inversion Principle:

```typescript
// domain/interfaces/user-repository.interface.ts
export interface UserRepository {
  getAll(): Observable<User[]>;
  getById(id: string): Observable<User>;
  save(user: User): Observable<User>;
  delete(id: string): Observable<void>;
}

// Injection token
export const USER_REPOSITORY = new InjectionToken<UserRepository>('UserRepository');
```

```typescript
// infrastructure/http-user.repository.ts
@Injectable()
export class HttpUserRepository implements UserRepository {
  private http = inject(HttpClient);
  
  getAll(): Observable<User[]> {
    return this.http.get<UserDto[]>('/api/users').pipe(
      map(dtos => dtos.map(dto => this.toDomain(dto)))
    );
  }
  
  // ... other methods
}
```

```typescript
// app.module.ts or providers array
providers: [
  { provide: USER_REPOSITORY, useClass: HttpUserRepository }
]

// For testing
providers: [
  { provide: USER_REPOSITORY, useClass: MockUserRepository }
]
```

```typescript
// user.facade.ts - Depends on interface, not implementation
export class UserFacade {
  private repo = inject(USER_REPOSITORY);
  
  loadUsers(): void {
    this.repo.getAll().subscribe(/* ... */);
  }
}
```

### 5.2 Hierarchical Injector Scoping

| Provider Location | Scope | Use Case |
|-------------------|-------|----------|
| `providedIn: 'root'` | Application singleton | Global services, state |
| Feature Module providers | Per lazy-loaded module | Feature-specific state |
| Component providers | Per component instance | Component-local state |

```typescript
// Feature-scoped service (new instance per lazy module)
@Injectable()  // NOT providedIn: 'root'
export class FeatureStateService { }

@NgModule({
  providers: [FeatureStateService]  // Scoped to this module
})
export class FeatureModule { }
```

---

## 6. Error Handling

### 6.1 Global Error Handling

```typescript
// core/services/global-error-handler.ts
@Injectable()
export class GlobalErrorHandler implements ErrorHandler {
  private logger = inject(LoggerService);
  private notification = inject(NotificationService);
  
  handleError(error: Error): void {
    // Log to monitoring service
    this.logger.error(error);
    
    // Show user-friendly message
    if (error instanceof HttpErrorResponse) {
      this.notification.showError(this.getHttpErrorMessage(error));
    } else {
      this.notification.showError('An unexpected error occurred');
    }
    
    // Re-throw in development
    if (!environment.production) {
      console.error(error);
    }
  }
}
```

### 6.2 HTTP Interceptor Error Handling

```typescript
// core/interceptors/error.interceptor.ts
export const errorInterceptor: HttpInterceptorFn = (req, next) => {
  return next(req).pipe(
    catchError((error: HttpErrorResponse) => {
      if (error.status === 401) {
        // Redirect to login
        inject(Router).navigate(['/login']);
      } else if (error.status === 403) {
        // Show forbidden message
        inject(NotificationService).showError('Access denied');
      }
      
      return throwError(() => error);
    })
  );
};
```

### 6.3 Component-Level Error States

```typescript
// Using async pipe with error handling
@Component({
  template: `
    @if (error()) {
      <app-error-message [error]="error()" (retry)="load()" />
    } @else if (loading()) {
      <app-spinner />
    } @else {
      <app-user-list [users]="users()" />
    }
  `
})
export class UserContainerComponent {
  private facade = inject(UserFacade);
  
  users = toSignal(this.facade.users$, { initialValue: [] });
  loading = toSignal(this.facade.loading$, { initialValue: false });
  error = toSignal(this.facade.error$, { initialValue: null });
  
  load(): void {
    this.facade.loadUsers();
  }
}
```

---

## 7. Performance Patterns

### 7.1 Change Detection Optimization

```typescript
// ✅ ALWAYS use OnPush for dumb components
@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  // ...
})
export class UserCardComponent {
  @Input({ required: true }) user!: User;
}
```

**OnPush triggers change detection only when:**
- `@Input()` reference changes
- Event originates from component
- Observable emits (via async pipe)
- Manually triggered via `ChangeDetectorRef`

### 7.2 TrackBy for Lists

```typescript
// ✅ ALWAYS use trackBy
@Component({
  template: `
    @for (user of users; track user.id) {
      <app-user-card [user]="user" />
    }
  `
})

// Or with *ngFor
template: `
  <app-user-card 
    *ngFor="let user of users; trackBy: trackByUserId"
    [user]="user"
  />
`

trackByUserId(index: number, user: User): string {
  return user.id;
}
```

### 7.3 Lazy Loading

```typescript
// app-routing.module.ts
const routes: Routes = [
  {
    path: 'users',
    loadChildren: () => import('./features/users/users.module')
      .then(m => m.UsersModule)
  },
  {
    path: 'orders',
    loadComponent: () => import('./features/orders/order-list.component')
      .then(c => c.OrderListComponent)  // Standalone component
  }
];
```

### 7.4 Virtual Scrolling

```typescript
// For large lists (> 100 items)
import { ScrollingModule } from '@angular/cdk/scrolling';

@Component({
  template: `
    <cdk-virtual-scroll-viewport itemSize="50" class="viewport">
      <app-user-card *cdkVirtualFor="let user of users" [user]="user" />
    </cdk-virtual-scroll-viewport>
  `
})
```

---

## 8. Anti-Patterns Detection Guide

### 8.1 Subscription Leaks

**Symptom:** Memory leaks, stale callbacks, duplicate HTTP calls

```typescript
// ❌ BAD: Manual subscription without cleanup
ngOnInit() {
  this.userService.getUsers().subscribe(users => {
    this.users = users;
  });
}

// ✅ GOOD: Async pipe (automatic cleanup)
users$ = this.userService.getUsers();
// Template: {{ users$ | async }}

// ✅ GOOD: takeUntilDestroyed (Angular 16+)
users$ = this.userService.getUsers().pipe(
  takeUntilDestroyed()
);

// ✅ GOOD: Explicit cleanup
private destroy$ = new Subject<void>();

ngOnInit() {
  this.userService.getUsers().pipe(
    takeUntil(this.destroy$)
  ).subscribe(users => this.users = users);
}

ngOnDestroy() {
  this.destroy$.next();
  this.destroy$.complete();
}
```

### 8.2 God Components

**Symptom:** Component > 300 lines, > 5 injected services, mixed concerns

**Detection:**
```bash
# Find large components
find src -name "*.component.ts" -exec wc -l {} + | sort -rn | head -20

# Count injected services
grep -rn "inject(" src/app/features --include="*.component.ts" | \
  awk -F: '{print $1}' | sort | uniq -c | sort -rn
```

**Fix:** Extract to smart/dumb hierarchy

### 8.3 Template Logic Abuse

```typescript
// ❌ BAD: Complex logic in template
template: `
  <div *ngIf="user && user.roles && user.roles.includes('admin') && !user.suspended">
    {{ user.firstName + ' ' + user.lastName | uppercase }}
  </div>
`

// ✅ GOOD: Logic in component
template: `
  <div *ngIf="isActiveAdmin">
    {{ fullName | uppercase }}
  </div>
`

get isActiveAdmin(): boolean {
  return this.user?.roles?.includes('admin') && !this.user?.suspended;
}

get fullName(): string {
  return `${this.user?.firstName} ${this.user?.lastName}`;
}

// ✅ BETTER: With signals (Angular 16+)
isActiveAdmin = computed(() => 
  this.user()?.roles?.includes('admin') && !this.user()?.suspended
);
```

### 8.4 Direct HTTP in Components

```typescript
// ❌ BAD: HTTP call in component
@Component({ /* ... */ })
export class UserListComponent {
  private http = inject(HttpClient);
  
  ngOnInit() {
    this.http.get('/api/users').subscribe(/* ... */);
  }
}

// ✅ GOOD: Via facade/service
@Component({ /* ... */ })
export class UserListComponent {
  private facade = inject(UserFacade);
  users$ = this.facade.users$;
  
  ngOnInit() {
    this.facade.loadUsers();
  }
}
```

### 8.5 Shared Module Bloat

```typescript
// ❌ BAD: Shared module imports everything
@NgModule({
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    HttpClientModule,      // Should be in Core
    RouterModule,          // Usually not needed
    MaterialModule,        // Too broad
    // ... 50 more imports
  ],
  declarations: [
    // 100 components
  ],
  exports: [
    // 100 components
  ]
})
export class SharedModule { }

// ✅ GOOD: Granular shared modules or standalone
// shared/button/button.component.ts
@Component({
  standalone: true,
  imports: [CommonModule],
  // ...
})
export class ButtonComponent { }
```

---

## 9. Testing Strategy

### 9.1 Dumb Component Testing (Isolated)

```typescript
describe('UserCardComponent', () => {
  let component: UserCardComponent;
  let fixture: ComponentFixture<UserCardComponent>;
  
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [UserCardComponent]  // Standalone
    }).compileComponents();
    
    fixture = TestBed.createComponent(UserCardComponent);
    component = fixture.componentInstance;
  });
  
  it('should display user name', () => {
    component.user = { id: '1', name: 'John Doe', email: 'john@test.com' };
    fixture.detectChanges();
    
    const nameElement = fixture.nativeElement.querySelector('.user-name');
    expect(nameElement.textContent).toContain('John Doe');
  });
  
  it('should emit event when delete clicked', () => {
    component.user = { id: '1', name: 'John Doe', email: 'john@test.com' };
    const deleteSpy = jest.spyOn(component.deleteRequested, 'emit');
    
    fixture.detectChanges();
    fixture.nativeElement.querySelector('.delete-btn').click();
    
    expect(deleteSpy).toHaveBeenCalledWith('1');
  });
});
```

### 9.2 Smart Component Testing (With Mocks)

```typescript
describe('UserListContainerComponent', () => {
  let component: UserListContainerComponent;
  let fixture: ComponentFixture<UserListContainerComponent>;
  let mockFacade: jest.Mocked<UserFacade>;
  
  beforeEach(async () => {
    mockFacade = {
      users$: of([{ id: '1', name: 'John' }]),
      loading$: of(false),
      loadUsers: jest.fn(),
      deleteUser: jest.fn()
    } as any;
    
    await TestBed.configureTestingModule({
      imports: [UserListContainerComponent],
      providers: [
        { provide: UserFacade, useValue: mockFacade }
      ]
    }).compileComponents();
    
    fixture = TestBed.createComponent(UserListContainerComponent);
    component = fixture.componentInstance;
  });
  
  it('should load users on init', () => {
    fixture.detectChanges();
    expect(mockFacade.loadUsers).toHaveBeenCalled();
  });
});
```

### 9.3 Service Testing with HttpTestingController

```typescript
describe('UserApiService', () => {
  let service: UserApiService;
  let httpMock: HttpTestingController;
  
  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [HttpClientTestingModule],
      providers: [UserApiService]
    });
    
    service = TestBed.inject(UserApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });
  
  afterEach(() => {
    httpMock.verify();  // Ensure no outstanding requests
  });
  
  it('should fetch users', () => {
    const mockUsers = [{ id: '1', name: 'John' }];
    
    service.getUsers().subscribe(users => {
      expect(users).toEqual(mockUsers);
    });
    
    const req = httpMock.expectOne('/api/users');
    expect(req.request.method).toBe('GET');
    req.flush(mockUsers);
  });
});
```

---

## 10. Viewpoint Integration

### VP-F01: Tech Stack Detection

**Angular-specific checks:**
- Angular version (check `package.json`)
- CLI version
- State management library (NgRx, NGXS, Akita, Signals)
- UI framework (Material, PrimeNG, etc.)
- Build optimizer settings

**Artifact:** `package.json`, `angular.json`

### VP-F02: File Structure

**Angular-specific patterns:**
- Standalone app vs Nx monorepo
- Core/Shared/Feature module pattern
- Smart/Dumb component organization
- State management structure

### VP-S02: Layer Architecture

**Critical for Angular:**
- Module import boundaries
- Service injection hierarchy
- Feature module isolation
- Domain layer purity

**Severity Adjustment:**
| Finding | Severity |
|---------|----------|
| Feature imports from another feature | HIGH |
| Angular imports in domain | HIGH |
| HTTP calls in components | MEDIUM |
| Missing OnPush on dumb components | LOW |

### VP-Q02: Performance

**Angular-specific:**
- Change detection strategy usage
- Lazy loading coverage
- Bundle size analysis
- trackBy usage in templates

### VP-Q03: Testability

**Angular-specific:**
- TestBed configuration patterns
- Mock service patterns
- Coverage by component type

---

## 11. Summary: Audit Checklist

### Architecture
- [ ] Clear Core/Shared/Feature module separation
- [ ] Feature modules are lazy-loaded
- [ ] Domain folder has no Angular imports
- [ ] Facade pattern abstracts state management

### Components
- [ ] Smart/Dumb separation maintained
- [ ] OnPush on all presentational components
- [ ] No service injections in dumb components
- [ ] Components under 200 lines

### State Management
- [ ] Single source of truth for each data type
- [ ] Immutable update patterns
- [ ] Facade hides implementation details
- [ ] State changes are traceable

### RxJS & Subscriptions
- [ ] No subscription leaks (async pipe or cleanup)
- [ ] Proper error handling in streams
- [ ] takeUntilDestroyed or explicit cleanup

### Performance
- [ ] trackBy on all *ngFor / @for
- [ ] Virtual scrolling for large lists
- [ ] Lazy loading for feature modules
- [ ] Preload strategy configured

### Testing
- [ ] Dumb components tested in isolation
- [ ] Smart components tested with mocks
- [ ] Services tested with HttpTestingController
- [ ] Coverage by layer (domain > services > components)

---

*Part of AI Code Audit Agent Methodology Knowledge Base*

---
name: vp-s05-interface-surface
version: 1.0
dependencies: [vp-s02-layer-architecture, vp-s03-domain-model]
mcp_servers: [mental-model, methodology-kb]
---

# VP-S05: Interface Surface Analysis

## Purpose
Map the external interfaces of the application including API endpoints, public modules, and exported types to understand the attack surface and integration points.

## Prerequisites
- VP-S02 (Layer Architecture) completed
- VP-S03 (Domain Model) completed

## Instructions

### Step 1: Get Current Context
Call `mental-model/get_model` and extract:
- `tech_stack.framework` - helps identify API patterns
- `architecture.layers` - especially adapter/API layers
- `domain_model.bounded_contexts` - for context boundaries

### Step 2: Identify API Endpoints

Based on framework:

#### FastAPI (Python)
```python
# Look for router decorators
@router.get("/users/{user_id}")
@router.post("/users")
@app.get("/items")
```

Files to check:
- `api/`, `routes/`, `endpoints/`
- Files with `router` or `app` instances

#### Django REST Framework (Python)
```python
# ViewSets
class UserViewSet(ModelViewSet):
    ...

# Function-based views
@api_view(['GET', 'POST'])
def user_list(request):
    ...
```

Files to check:
- `views.py`, `viewsets.py`
- `urls.py` for URL patterns

#### Express/NestJS (TypeScript)
```typescript
// Express
router.get('/users', getUsers);
router.post('/users', createUser);

// NestJS
@Controller('users')
export class UsersController {
    @Get()
    findAll() {}

    @Post()
    create(@Body() dto: CreateUserDto) {}
}
```

Files to check:
- `*.controller.ts`
- `routes/` directory
- Files with `@Controller` decorator

#### Axum/Actix (Rust)
```rust
// Axum
Router::new()
    .route("/users", get(list_users))
    .route("/users/:id", get(get_user))

// Actix
web::scope("/users")
    .route("", web::get().to(list_users))
```

### Step 3: Document Each Endpoint

For each API endpoint, capture:
- HTTP method (GET, POST, PUT, DELETE, PATCH)
- Path (including parameters)
- Handler function/method name
- File location
- Line number
- Authentication requirement (if visible)
- Request body type (if applicable)
- Response type (if visible)

### Step 4: Analyze Authentication/Authorization

Look for authentication patterns:
- JWT validation
- API key requirements
- OAuth/OIDC
- Session-based auth

Common patterns:
```python
# FastAPI
@router.get("/protected", dependencies=[Depends(get_current_user)])

# NestJS
@UseGuards(AuthGuard)
@Get('protected')
```

Document:
- Which endpoints require auth
- Which endpoints are public
- Authorization levels (admin, user, etc.)

### Step 5: Identify Public Modules

Find modules/packages meant for external consumption:

#### Python
- `__all__` exports in `__init__.py`
- Public functions (no underscore prefix)
- Documented public API

#### TypeScript
- Barrel exports in `index.ts`
- Public exports from package
- TypeScript declaration files (`.d.ts`)

#### Rust
- `pub` visibility
- `pub use` re-exports
- Items in `lib.rs`

### Step 6: Map Exported Types

Identify types exposed to consumers:
- DTOs (Data Transfer Objects)
- Request/Response schemas
- API models
- Shared interfaces

### Step 7: Assess Attack Surface

Consider security implications:
- Number of public endpoints
- Endpoints handling user input
- Endpoints with file operations
- Endpoints with database operations
- Endpoints without authentication

### Step 8: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S05"
data:
  api_endpoints:
    - method: "GET"
      path: "/api/users"
      handler: "list_users"
      file_path: "src/api/users.py"
      line_number: 25
      authentication: "jwt"
    - method: "POST"
      path: "/api/users"
      handler: "create_user"
      file_path: "src/api/users.py"
      line_number: 45
      authentication: "none"
    - method: "GET"
      path: "/api/users/{id}"
      handler: "get_user"
      file_path: "src/api/users.py"
      line_number: 65
      authentication: "jwt"
  public_modules:
    - "src/api"
    - "src/schemas"
  exported_types:
    - "UserDTO"
    - "CreateUserRequest"
    - "UserResponse"
```

## Output Schema

```yaml
interface_surface:
  api_endpoints:
    - method: string
      path: string
      handler: string
      file_path: string
      line_number: number
      authentication: string
  public_modules: [string]
  exported_types: [string]
```

## Quality Indicators

- ✅ **Healthy**: All endpoints documented, clear auth requirements, minimal public surface
- ⚠️ **Warning**: Some undocumented endpoints, unclear auth, large attack surface
- ❌ **Critical**: Many unauthenticated endpoints, inconsistent patterns, excessive exposure

## Security Considerations

### API Security Checklist
- [ ] Authentication on sensitive endpoints
- [ ] Authorization checks
- [ ] Input validation
- [ ] Rate limiting
- [ ] CORS configuration
- [ ] HTTPS enforcement
- [ ] Request size limits
- [ ] Proper error handling (no stack traces)

### Common API Vulnerabilities
| Vulnerability | Check For |
|--------------|-----------|
| IDOR | Direct object references without auth |
| Mass Assignment | Accepting all fields from request |
| Broken Auth | Missing or weak authentication |
| Injection | Unsanitized input in queries |
| SSRF | User-controlled URLs |

## Interface Patterns

### RESTful API
```
GET    /resources         - List
GET    /resources/{id}    - Read
POST   /resources         - Create
PUT    /resources/{id}    - Update
DELETE /resources/{id}    - Delete
```

### GraphQL
Look for:
- `/graphql` endpoint
- Schema definitions
- Resolvers

### gRPC
Look for:
- `.proto` files
- Service definitions
- Generated code

## Next Viewpoint
After completing VP-S05, proceed to **VP-S06: Hotspots Analysis**.

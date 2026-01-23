# Rust Server Architecture Knowledge Base

> **Methodology KB** | AI Code Audit Agent | January 2026

---

## 1. Overview

This document provides comprehensive guidance for auditing Rust server applications, particularly protocol servers like MCP implementations. The core insight is that **Rust's type system naturally implements architectural patterns** that other languages require frameworks or runtime checks to enforce.

### Why Hexagonal Architecture Dominates in Rust

Among major architectural patterns—Clean, Hexagonal (Ports and Adapters), and Onion Architecture—the Rust community gravitates toward Hexagonal Architecture because its core abstraction (ports as interfaces, adapters as implementations) translates directly to Rust constructs:

| Hexagonal Concept | Rust Construct |
|-------------------|----------------|
| Port (interface) | Trait |
| Adapter (implementation) | Struct implementing trait |
| Domain Core | Pure structs, enums, business logic |
| Dependency Injection | Generic parameters or `Arc<dyn Trait>` |

This isn't just theoretical elegance—it's ergonomic in practice and provides **compile-time architecture enforcement**.

---

## 2. Structural Patterns

### 2.1 Single Crate Organization

**Use when:** Project under ~10,000 LOC, compile times under 2 minutes

```
src/
├── domain/
│   ├── models.rs       # Pure business entities
│   ├── services.rs     # Service traits and implementations
│   └── repository.rs   # Repository trait definitions (outbound ports)
├── inbound/
│   ├── http/
│   │   └── handlers.rs # HTTP handlers (inbound adapters)
│   └── protocol/
│       └── server.rs   # Protocol listeners
└── outbound/
    ├── sqlite.rs       # impl Repository for Sqlite (outbound adapters)
    └── postgres.rs     # impl Repository for Postgres
```

**Module Dependency Rules:**
- `domain` → **no dependencies** except `thiserror`
- `inbound` → depends on `domain`
- `outbound` → depends on `domain`
- `inbound` and `outbound` **never depend on each other**

### 2.2 Workspace Organization (Flat Pattern)

**Use when:**
- Project exceeds ~15,000 LOC
- Incremental compile times exceed 2 minutes
- Multiple developers need compile-time boundary enforcement

```
my_server/
├── Cargo.toml          # [workspace] with members = ["crates/*"]
└── crates/
    ├── server/         # Binary entry point
    │   ├── Cargo.toml  # depends on all other crates
    │   └── src/
    │       └── main.rs # Wires everything together
    ├── domain/         # Business entities and repository traits
    │   ├── Cargo.toml  # dependencies: [thiserror] ONLY
    │   └── src/
    │       ├── lib.rs
    │       ├── models.rs
    │       └── repository.rs
    ├── application/    # Use cases and service orchestration
    │   ├── Cargo.toml  # depends on: domain
    │   └── src/
    ├── infrastructure/ # Database implementations
    │   ├── Cargo.toml  # depends on: domain, application, sqlx, etc.
    │   └── src/
    └── common/         # Shared error types, config
        ├── Cargo.toml
        └── src/
```

**The Key Guarantee:** When domain crate's `Cargo.toml` cannot list infrastructure crate, you **physically cannot** import database code into business logic. The compiler becomes your architecture enforcer.

### 2.3 Workspace Cargo.toml Conventions

```toml
# Root Cargo.toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
# Declare once, reference everywhere
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }
thiserror = "1.0"
anyhow = "1.0"
```

```toml
# crates/domain/Cargo.toml
[package]
name = "domain"
version.workspace = true
edition.workspace = true

[dependencies]
thiserror.workspace = true
# NOTHING ELSE - domain stays pure
```

---

## 3. Dependency Injection Patterns

### 3.1 The Rust Approach

Rust doesn't need Spring or DI containers. The trait system provides **compile-time dependency injection** that's more performant and explicit than reflection-based alternatives.

### 3.2 Pattern: Generic Parameters

**Use when:** Performance-critical hot paths where monomorphization matters

```rust
// Domain layer defines the port
pub trait MessageRepository: Clone + Send + Sync + 'static {
    fn save(&self, msg: &Message) -> impl Future<Output = Result<(), RepoError>> + Send;
    fn find_by_id(&self, id: Uuid) -> impl Future<Output = Result<Option<Message>, RepoError>> + Send;
}

// Service depends on the trait, not concrete implementation
pub struct MessageService<R: MessageRepository> {
    repo: R,
}

impl<R: MessageRepository> MessageService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
    
    pub async fn process(&self, msg: Message) -> Result<(), ServiceError> {
        // Business logic completely unaware of database choice
        self.repo.save(&msg).await?;
        Ok(())
    }
}
```

**Trade-off:** Complex type signatures propagate through application

### 3.3 Pattern: Trait Objects (Recommended Default)

**Use when:** Most applications—better ergonomics, simpler types

```rust
pub struct MessageService {
    repo: Arc<dyn MessageRepository + Send + Sync>,
}

impl MessageService {
    pub fn new(repo: Arc<dyn MessageRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}
```

**Trade-off:** Small runtime cost from dynamic dispatch (usually negligible)

### 3.4 Pattern: Newtype Wrapper

**Use when:** Need to hide implementation details from public API

```rust
// Public API only exposes the wrapper
pub struct Connection(Arc<dyn Db + Send + Sync>);

impl Connection {
    pub fn postgres(url: &str) -> Result<Self, Error> {
        Ok(Self(Arc::new(PostgresDb::connect(url)?)))
    }
    
    pub fn sqlite(path: &str) -> Result<Self, Error> {
        Ok(Self(Arc::new(SqliteDb::open(path)?)))
    }
}
```

---

## 4. Error Handling Strategy

### 4.1 The Two-Library Pattern

| Layer | Library | Rationale |
|-------|---------|-----------|
| Domain | `thiserror` | Typed enums for matching on specific variants |
| Application/Handlers | `anyhow` | Context attachment, aggregation, logging |

### 4.2 Domain Errors with thiserror

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("User not found: {0}")]
    UserNotFound(UserId),
    
    #[error("Validation failed: {0}")]
    ValidationError(String),
    
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },
    
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
}
```

**Why typed errors matter:** Calling code can match on specific variants:

```rust
match service.transfer(from, to, amount).await {
    Ok(_) => HttpResponse::Ok(),
    Err(DomainError::UserNotFound(_)) => HttpResponse::NotFound(),
    Err(DomainError::InsufficientFunds { .. }) => HttpResponse::BadRequest(),
    Err(DomainError::ValidationError(_)) => HttpResponse::BadRequest(),
    Err(e) => {
        tracing::error!(?e, "Internal error");
        HttpResponse::InternalServerError()
    }
}
```

### 4.3 Application Errors with anyhow

```rust
use anyhow::{Context, Result};

pub async fn handle_request(req: Request) -> Result<Response> {
    let user = db.get_user(req.user_id)
        .await
        .context("Failed to fetch user from database")?;
    
    let processed = process_data(&user.data)
        .context("Data processing failed")?;
    
    Ok(Response::new(processed))
}
```

### 4.4 Boundary Mapping

Map domain errors to HTTP at handler boundaries:

```rust
impl From<DomainError> for HttpResponse {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::UserNotFound(_) => 
                HttpResponse::NotFound().json(ErrorBody::from(&e)),
            DomainError::ValidationError(_) => 
                HttpResponse::BadRequest().json(ErrorBody::from(&e)),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
```

---

## 5. State Management for Concurrent Servers

### 5.1 Shared State Patterns

| Pattern | Use Case |
|---------|----------|
| `Arc<T>` | Shared immutable state (config, static data) |
| `Arc<Mutex<T>>` | General mutable shared state |
| `Arc<RwLock<T>>` | Read-heavy workloads |
| Mutex Sharding | High-contention scenarios |

### 5.2 Mutex Sharding Pattern

For high-contention scenarios, shard state across multiple locks:

```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::sync::Arc;

type ShardedDb = Arc<Vec<Mutex<HashMap<String, Vec<u8>>>>>;

fn create_sharded_db(num_shards: usize) -> ShardedDb {
    Arc::new(
        (0..num_shards)
            .map(|_| Mutex::new(HashMap::new()))
            .collect()
    )
}

fn get_shard<'a>(db: &'a ShardedDb, key: &str) -> std::sync::MutexGuard<'a, HashMap<String, Vec<u8>>> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    let shard_idx = (hasher.finish() as usize) % db.len();
    db[shard_idx].lock().unwrap()
}
```

### 5.3 Async Lock Considerations

| Scenario | Use |
|----------|-----|
| Lock held across `.await` | `tokio::sync::Mutex` |
| Quick synchronous section | `std::sync::Mutex` |

**Critical Rule:** Keep lock scopes minimal—acquire, operate, release, **then** await.

```rust
// ✅ GOOD: Lock released before await
async fn process(state: Arc<Mutex<State>>) {
    let value = {
        let guard = state.lock().unwrap();
        guard.value.clone()
    }; // Lock released here
    
    expensive_async_operation(value).await; // Safe to await now
}

// ❌ BAD: Lock held across await
async fn process_bad(state: Arc<Mutex<State>>) {
    let guard = state.lock().unwrap();
    expensive_async_operation(guard.value.clone()).await; // DEADLOCK RISK
}
```

---

## 6. Testing Strategy

### 6.1 Trait-Based Testing

The same traits that enable swappable production backends enable fast, deterministic tests.

```rust
// In-memory implementation for testing
#[derive(Clone, Default)]
pub struct InMemoryRepo {
    messages: Arc<Mutex<HashMap<Uuid, Message>>>,
}

impl MessageRepository for InMemoryRepo {
    async fn save(&self, msg: &Message) -> Result<(), RepoError> {
        self.messages.lock().unwrap().insert(msg.id, msg.clone());
        Ok(())
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, RepoError> {
        Ok(self.messages.lock().unwrap().get(&id).cloned())
    }
}
```

### 6.2 Parameterized Test Pattern

Write generic test functions, instantiate with different backends:

```rust
async fn test_message_roundtrip<R: MessageRepository>(repo: R) {
    let msg = Message::new("test content");
    repo.save(&msg).await.unwrap();
    
    let retrieved = repo.find_by_id(msg.id).await.unwrap();
    assert_eq!(Some(msg), retrieved);
}

#[tokio::test]
async fn in_memory_roundtrip() {
    let repo = InMemoryRepo::default();
    test_message_roundtrip(repo).await;
}

#[tokio::test]
async fn sqlite_roundtrip() {
    let repo = SqliteRepo::in_memory().await.unwrap();
    test_message_roundtrip(repo).await;
}

#[tokio::test]
#[ignore] // Run only in CI with real database
async fn postgres_roundtrip() {
    let repo = PostgresRepo::from_env().await.unwrap();
    test_message_roundtrip(repo).await;
}
```

### 6.3 Test Organization Rules

| Requirement | Rationale |
|-------------|-----------|
| `cargo test` works after `git clone` | Zero external setup for contributors |
| External deps marked `#[ignore]` | Separate CI job for integration tests |
| Domain tests have zero I/O | Fast, deterministic, reveal design issues |

---

## 7. Handler Design

### 7.1 Thin Handlers Principle

Handlers should be **thin orchestration layers**:
1. Deserialize request
2. Validate input
3. Delegate to domain services
4. Serialize response

**All meaningful logic lives in domain services** that accept only domain types.

### 7.2 Example: Proper Handler

```rust
// ✅ GOOD: Thin handler, business logic in service
async fn create_user(
    State(service): State<Arc<UserService>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Validate input (could be in middleware)
    let validated = req.validate()?;
    
    // Delegate to service
    let user = service.create_user(validated.into()).await?;
    
    // Transform to response
    Ok(Json(user.into()))
}
```

### 7.3 Anti-pattern: Fat Handler

```rust
// ❌ BAD: Business logic in handler
async fn create_user(
    State(db): State<Arc<DbPool>>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    // Business logic leaked into handler
    if req.email.contains("test") {
        return Err(AppError::InvalidEmail);
    }
    
    // Direct database access
    let user = sqlx::query_as!(User, "INSERT INTO users...")
        .fetch_one(&*db)
        .await?;
    
    // More business logic
    if user.is_admin {
        send_admin_notification(&user).await?;
    }
    
    Ok(Json(user.into()))
}
```

---

## 8. Anti-Patterns Detection Guide

### 8.1 Excessive Clone

**Symptom:** Frequent `.clone()` calls throughout codebase

**Root Cause:** Ownership model confusion, fighting the borrow checker

**Fix:**
- Use `Arc` for shared ownership
- Use references where lifetime allows
- Consider `Cow<'a, T>` for flexibility

**Audit Command:**
```bash
grep -rn "\.clone()" src/ | wc -l
```

### 8.2 Blocking in Async

**Symptom:** `std::fs::*`, `std::net::*`, or other blocking calls in async functions

**Root Cause:** Misunderstanding async runtime behavior

**Fix:**
```rust
// ❌ BAD
async fn bad() {
    let data = std::fs::read("file.txt").unwrap(); // Blocks runtime!
}

// ✅ GOOD: Use async alternatives
async fn good() {
    let data = tokio::fs::read("file.txt").await.unwrap();
}

// ✅ GOOD: Or spawn_blocking for CPU-intensive work
async fn good_cpu() {
    let result = tokio::task::spawn_blocking(|| {
        expensive_computation()
    }).await.unwrap();
}
```

### 8.3 Unwrap in Production

**Symptom:** `unwrap()` or `expect()` outside test code

**Root Cause:** Laziness, prototype code in production

**Fix:** Proper error propagation with `?` operator

### 8.4 Generic Box<dyn Error>

**Symptom:** `Box<dyn std::error::Error>` return types

**Root Cause:** Avoiding error type design

**Problem:** Loses type information, calling code can't match on variants

**Fix:** Use `thiserror` for domain errors

### 8.5 Lock Across Await

**Symptom:** `MutexGuard` or `RwLockGuard` held across `.await`

**Root Cause:** Not understanding async execution model

**Problem:** Deadlocks, runtime blocking

**Fix:** Scope locks tightly, release before await

---

## 9. Viewpoint Integration

### VP-F01: Tech Stack Detection

**Rust-specific checks:**
- Rust edition (2018, 2021, 2024)
- Async runtime (tokio, async-std, smol)
- Workspace vs single crate
- Key framework choices (axum, actix-web, etc.)

**Artifact:** `Cargo.toml` analysis

### VP-F02: File Structure

**Rust-specific patterns:**
- Flat workspace (`crates/`) vs nested
- Module organization (domain/inbound/outbound)
- Binary vs library crate separation

### VP-S02: Layer Architecture

**Critical for Rust:**
- Validate trait definitions in domain layer
- Check for infrastructure imports in domain
- Verify handler thickness (orchestration only)

**Severity Adjustment:**
| Finding | Severity |
|---------|----------|
| Domain imports from infrastructure | CRITICAL |
| Business logic in handlers | HIGH |
| Framework types in domain | HIGH |
| Missing trait abstractions | MEDIUM |

### VP-S03: Domain Model

**Rust mapping:**
- Bounded contexts → Crates or module trees
- Aggregates → Structs with `impl` blocks
- Value Objects → Newtype pattern or structs

### VP-Q01: Security

**Rust-specific:**
- `unwrap()` usage outside tests (panic risk)
- Unsafe blocks and their justification
- Dependency audit (`cargo audit`)

### VP-Q03: Testability

**Rust-specific:**
- Trait-based mocking capability
- Test organization (unit vs integration)
- `#[ignore]` usage for external deps

---

## 10. Reference Implementations

### ripgrep

**Repository:** https://github.com/BurntSushi/ripgrep

**Study for:**
- Workspace organization for complex CLI tool
- Nine member crates with clear separation
- `grep` facade over specialized sub-crates
- How to decompose problems into reusable, testable units

### Axum

**Repository:** https://github.com/tokio-rs/axum

**Study for:**
- Modern HTTP server patterns
- Macro-free API design using type system
- Tower middleware ecosystem integration
- How to compose existing, well-tested components

### Tokio mini-redis

**Repository:** https://github.com/tokio-rs/mini-redis

**Study for:**
- Learning example (~3,000 LOC)
- Realistic async patterns
- Connection handling
- Protocol parsing
- Shared state management

---

## 11. Summary: Audit Checklist

### Architecture
- [ ] Layer boundaries enforced (workspace) or documented (modules)
- [ ] Domain crate/module has no framework dependencies
- [ ] Repository traits defined in domain layer
- [ ] Handlers are thin orchestration only

### Dependency Injection
- [ ] Traits used for external interactions
- [ ] Generic parameters or trait objects (appropriate choice)
- [ ] Wiring happens in main.rs/binary crate

### Error Handling
- [ ] `thiserror` for domain errors
- [ ] `anyhow` for application orchestration
- [ ] Proper `From` implementations at boundaries
- [ ] No `unwrap()` in production code

### State Management
- [ ] `Arc<T>` for shared immutable state
- [ ] Appropriate mutex/rwlock choice
- [ ] No locks held across await points

### Testing
- [ ] Domain tests have zero I/O
- [ ] In-memory implementations for traits
- [ ] `cargo test` works without external deps
- [ ] External tests marked `#[ignore]`

### Anti-Patterns
- [ ] No excessive `.clone()` calls
- [ ] No blocking I/O in async code
- [ ] No `Box<dyn Error>` returns
- [ ] No framework leakage into domain

---

*Part of AI Code Audit Agent Methodology Knowledge Base*

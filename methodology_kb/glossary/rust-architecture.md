# Rust Architecture Glossary

> Extension for Methodology KB Glossary | AI Code Audit Agent

---

## Core Concepts

### Trait (Rust)
**Definition:** A Rust language construct that defines shared behavior, similar to interfaces in other languages but with additional capabilities like associated types and default implementations.

**Architecture Role:** Traits serve as **ports** in hexagonal architecture—they define contracts that adapters implement.

**Example:**
```rust
pub trait MessageRepository {
    fn save(&self, msg: &Message) -> Result<(), Error>;
}
```

### Trait Object
**Definition:** A pointer to a concrete type that implements a trait, using dynamic dispatch (`dyn Trait`). Commonly wrapped in `Box<dyn Trait>` or `Arc<dyn Trait>`.

**Architecture Role:** Enables runtime polymorphism for dependency injection without generic parameters propagating through the entire codebase.

**Trade-off:** Small runtime cost from dynamic dispatch vs. simpler type signatures.

### Monomorphization
**Definition:** Rust compiler technique that generates specialized code for each concrete type used with a generic function, eliminating runtime dispatch.

**Architecture Role:** When using generic parameters (`fn process<R: Repository>(repo: R)`), the compiler creates separate optimized versions for each repository type.

**Trade-off:** Faster execution but larger binary size and complex type signatures.

### Newtype Pattern
**Definition:** Wrapping a type in a single-field struct to create a distinct type, typically for type safety or encapsulation.

**Architecture Role:** Creates semantic types (`UserId(i64)` vs raw `i64`) and hides implementation details.

**Example:**
```rust
pub struct UserId(i64);
pub struct Connection(Arc<dyn Db + Send + Sync>);
```

---

## Cargo & Project Structure

### Workspace
**Definition:** A Cargo feature that manages multiple related packages (crates) that are developed in tandem.

**Architecture Role:** Enables **compile-time architecture enforcement**. When crate A cannot list crate B in dependencies, A cannot import from B.

**When to Use:** Projects over ~15,000 LOC or when compile-time boundary enforcement is needed.

### Flat Workspace Pattern
**Definition:** Workspace organization where all member crates are in a single `crates/` directory at repository root, with a virtual manifest at top level.

**Structure:**
```
project/
├── Cargo.toml          # [workspace]
└── crates/
    ├── domain/
    ├── infrastructure/
    └── server/
```

### Virtual Manifest
**Definition:** A `Cargo.toml` that contains only `[workspace]` configuration and no `[package]` section—it doesn't define a crate itself.

### Workspace Dependencies
**Definition:** Dependencies declared in root `Cargo.toml` under `[workspace.dependencies]` that member crates reference with `dependency.workspace = true`.

**Purpose:** Single source of truth for version management across workspace.

### Feature Flag (Cargo)
**Definition:** Conditional compilation feature that enables optional functionality or dependencies.

**Best Practice:** Features should be **additive only**—enabling a feature adds functionality rather than changing existing behavior.

---

## Error Handling

### thiserror
**Definition:** Rust derive macro library for creating custom error types with minimal boilerplate.

**Architecture Role:** Used in **domain layer** for typed error enums that calling code can match on.

**Example:**
```rust
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("User not found: {0}")]
    UserNotFound(UserId),
}
```

### anyhow
**Definition:** Rust library for idiomatic error handling in applications, providing easy error context and propagation.

**Architecture Role:** Used at **application/handler layer** for aggregating errors and attaching context.

**Example:**
```rust
operation().context("Failed during processing")?;
```

### Error Propagation (?)
**Definition:** Rust's `?` operator that returns early from a function if a `Result` is `Err`, propagating the error.

**Pattern:** Enables clean error handling without explicit matching at every call site.

---

## Async & Concurrency

### Arc (Atomic Reference Count)
**Definition:** Thread-safe reference-counting pointer for shared ownership across threads.

**Architecture Role:** Foundation for shared state in concurrent servers: `Arc<T>` for immutable, `Arc<Mutex<T>>` for mutable.

### tokio::sync::Mutex vs std::sync::Mutex
**Definition:** Tokio's async-aware mutex vs standard library's synchronous mutex.

**When to Use:**
- `tokio::sync::Mutex`: Lock held across `.await` points
- `std::sync::Mutex`: Quick synchronous critical sections

### Mutex Sharding
**Definition:** Distributing state across multiple independent mutexes, selected by hashing keys.

**Architecture Role:** Reduces lock contention in high-throughput scenarios.

### spawn_blocking
**Definition:** Tokio function that runs blocking code on a dedicated thread pool, preventing async runtime blocking.

**When to Use:** CPU-intensive work or unavoidable blocking I/O in async context.

---

## Hexagonal Architecture (Rust-Specific)

### Port (Rust)
**Definition:** A trait that defines an interface between application core and external world.

**Types:**
- **Inbound Port:** Trait defining operations the outside world can request (e.g., service interface)
- **Outbound Port:** Trait defining operations the core needs from external systems (e.g., repository trait)

### Adapter (Rust)
**Definition:** A struct that implements a port trait, providing concrete functionality.

**Types:**
- **Inbound Adapter:** HTTP handler, CLI parser, gRPC service
- **Outbound Adapter:** Database implementation, external API client

### Domain Purity
**Definition:** Degree to which domain layer is free of framework and infrastructure dependencies.

**Measurement:** Percentage of domain files with no external dependencies beyond `thiserror`.

**Target:** 100% for domain crate/module.

### Handler Thickness
**Definition:** Amount of logic in HTTP/protocol handlers.

**Assessment:**
- **Thin:** Deserialize → Validate → Delegate → Serialize (target state)
- **Thick:** Contains business logic, multiple conditionals, database calls (anti-pattern)

---

## Anti-Patterns (Rust-Specific)

### Excessive Clone
**Definition:** Overuse of `.clone()` calls indicating ownership model issues.

**Symptom:** More than 10-15 `.clone()` calls per 1000 LOC outside tests.

**Fix:** Use `Arc` for shared ownership, references where lifetimes allow.

### Lock Across Await
**Definition:** Holding a `MutexGuard` or `RwLockGuard` while calling `.await`.

**Problem:** Can cause deadlocks and blocks async runtime.

**Fix:** Scope locks tightly—acquire, operate, release, then await.

### Blocking in Async
**Definition:** Using synchronous I/O (`std::fs`, `std::net`) in async functions.

**Problem:** Blocks the entire async runtime thread.

**Fix:** Use async alternatives (`tokio::fs`) or `spawn_blocking`.

### Framework Leakage
**Definition:** Framework-specific types (e.g., `axum::Json`, `actix_web::web::Data`) appearing in domain layer.

**Problem:** Couples domain to specific framework, prevents testing and migration.

**Fix:** Domain accepts only domain types; framework types converted at handler boundary.

---

## Metrics & Quality

### Domain Purity Score
**Definition:** Percentage of files in domain layer with no external dependencies.

**Calculation:** `(files with only thiserror) / (total domain files) × 100`

**Target:** ≥ 95% for well-architected Rust projects.

### Trait Abstraction Coverage
**Definition:** Percentage of external interactions (DB, APIs, etc.) that have trait abstractions.

**Calculation:** `(interactions with traits) / (total external interactions) × 100`

**Target:** 100% for hexagonal compliance.

### Handler LOC
**Definition:** Lines of code in handler functions, excluding imports and comments.

**Thresholds:**
- ✅ Healthy: < 30 lines average
- ⚠️ Warning: 30-50 lines average
- 🚨 Critical: > 50 lines average

---

## Reference Terms

### rmcp
**Definition:** Rust crate for implementing MCP (Model Context Protocol) servers.

**Relevance:** Primary implementation target for AI Code Audit Agent MCP servers.

### tree-sitter
**Definition:** Parser generator tool and incremental parsing library, widely used for code analysis.

**Relevance:** Can be used to build Codegraph for Rust projects.

### SARIF
**Definition:** Static Analysis Results Interchange Format—JSON-based standard for expressing static analysis results.

**Relevance:** Output format for findings from AI Code Audit Agent.

---

*Glossary Extension for Rust Architecture*
*Part of AI Code Audit Agent Methodology Knowledge Base*

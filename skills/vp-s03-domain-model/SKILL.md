---
name: vp-s03-domain-model
version: 1.0
dependencies: [vp-s01-module-hierarchy, vp-s02-layer-architecture]
mcp_servers: [mental-model, methodology-kb]
---

# VP-S03: Domain Model Analysis

## Purpose
Identify bounded contexts and domain concepts to understand the business domains the application serves and their relationships.

## Prerequisites
- VP-S02 (Layer Architecture) completed
- Layer boundaries identified

## Instructions

### Step 1: Get Current Context
Call `mental-model/get_model` and extract:
- `architecture.layers` - especially domain/core layers
- `module_hierarchy.modules` - for structure reference

### Step 2: Identify Bounded Contexts

A bounded context is a cohesive area of the domain with:
- Distinct vocabulary/ubiquitous language
- Clear boundaries
- Internal consistency

#### Finding Bounded Contexts

Look for:
1. **Top-level modules** in domain/core layer that represent business areas
2. **Feature modules** that encapsulate specific functionality
3. **Aggregate roots** that define consistency boundaries
4. **Shared kernel** code used across contexts

Common bounded context patterns:
```
src/domain/
├── users/          # User Management BC
├── orders/         # Order Management BC
├── inventory/      # Inventory BC
├── payments/       # Payment BC
└── shared/         # Shared Kernel
```

Or in modular monolith:
```
src/modules/
├── user-management/
├── order-processing/
├── fulfillment/
└── billing/
```

### Step 3: Classify Bounded Context Types

For each bounded context, classify as:

#### Core Domain
- Provides competitive advantage
- Contains unique business logic
- Highest priority for quality
- Examples: Order processing for e-commerce, Risk assessment for insurance

#### Supporting Domain
- Supports the core domain
- Important but not differentiating
- Can potentially be outsourced
- Examples: User management, Notifications

#### Generic Domain
- Standard functionality
- Could use off-the-shelf solutions
- Lowest business differentiation
- Examples: Authentication, File storage, Logging

### Step 4: Identify Domain Entities

Within each bounded context, find:

**Entities:**
- Objects with identity that persists over time
- Usually have an ID field
- Examples: User, Order, Product

**Value Objects:**
- Objects defined by their attributes
- Immutable, no identity
- Examples: Address, Money, DateRange

**Aggregate Roots:**
- Entities that control access to a cluster of objects
- Ensure consistency within the aggregate
- Examples: Order (containing OrderItems)

### Step 5: Map Context Relationships

Identify how bounded contexts interact:

**Integration Patterns:**
- **Shared Kernel**: Common code shared between contexts
- **Customer-Supplier**: One context provides for another
- **Conformist**: One context follows another's model
- **Anti-Corruption Layer**: Translation between contexts
- **Open Host Service**: Context provides a standardized interface
- **Published Language**: Shared domain language (e.g., events)

### Step 6: Document Context Boundaries

For each bounded context, note:
- Entry points (APIs, services)
- Data stores (own database/schema)
- Events published/consumed
- External dependencies

### Step 7: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S03"
data:
  bounded_contexts:
    - name: "OrderManagement"
      type: "core"
      paths:
        - "src/domain/orders"
        - "src/modules/orders"
      entities:
        - "Order"
        - "OrderItem"
        - "OrderStatus"
      description: "Handles order lifecycle from creation to fulfillment"
    - name: "UserManagement"
      type: "supporting"
      paths:
        - "src/domain/users"
      entities:
        - "User"
        - "UserProfile"
        - "Role"
      description: "User registration, authentication, and profile management"
    - name: "Infrastructure"
      type: "generic"
      paths:
        - "src/shared"
        - "src/common"
      entities:
        - "AuditLog"
        - "Configuration"
      description: "Cross-cutting infrastructure concerns"
  aggregates:
    Order:
      - "OrderItem"
      - "OrderStatus"
      - "ShippingInfo"
    User:
      - "UserProfile"
      - "UserPreferences"
```

## Output Schema

```yaml
domain_model:
  bounded_contexts:
    - name: string
      type: core | supporting | generic
      paths: [string]
      entities: [string]
      description: string
  aggregates:
    <aggregate_root>:
      - string     # Entities in aggregate
```

## Quality Indicators

- ✅ **Healthy**: Clear contexts, appropriate classification, defined boundaries
- ⚠️ **Warning**: Overlapping contexts, unclear boundaries, missing classification
- ❌ **Critical**: No clear domain structure, everything in one context, anemic domain

## Domain Model Anti-Patterns

### Anemic Domain Model
- Entities are just data containers
- All logic in services
- Missing encapsulation

### Big Ball of Mud
- No clear boundaries
- Everything depends on everything
- No bounded contexts

### Leaky Abstractions
- Domain concepts leak to other layers
- Infrastructure concerns in domain
- UI concepts in business logic

### God Object
- One entity does everything
- Too many responsibilities
- Hard to understand and maintain

## DDD Tactical Patterns to Look For

- **Repository Pattern**: Abstraction over data access
- **Factory Pattern**: Complex object creation
- **Domain Events**: Communication between contexts
- **Specification Pattern**: Business rules encapsulation
- **Domain Services**: Operations not belonging to entities

## Next Viewpoint
After completing VP-S03, proceed to **VP-S04: Entity Model Analysis**.

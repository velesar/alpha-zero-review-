---
name: vp-s04-entity-model
version: 1.0
dependencies: [vp-s03-domain-model]
mcp_servers: [mental-model, methodology-kb]
---

# VP-S04: Entity Model Analysis

## Purpose
Analyze data entities, their relationships, and data modeling patterns to understand the data layer architecture.

## Prerequisites
- VP-S03 (Domain Model) completed
- Bounded contexts identified

## Instructions

### Step 1: Get Current Context
Call `mental-model/get_model` and extract:
- `domain_model.bounded_contexts` - for context reference
- `domain_model.aggregates` - for aggregate structure

### Step 2: Locate Entity Definitions

Based on framework from VP-F01:

#### Python/SQLAlchemy
```python
# Look for patterns like:
class User(Base):
    __tablename__ = 'users'
    id = Column(Integer, primary_key=True)
    email = Column(String, unique=True)
```

#### Python/Django
```python
class User(models.Model):
    email = models.EmailField(unique=True)
    created_at = models.DateTimeField(auto_now_add=True)
```

#### TypeScript/TypeORM
```typescript
@Entity()
export class User {
    @PrimaryGeneratedColumn()
    id: number;

    @Column({ unique: true })
    email: string;
}
```

#### TypeScript/Prisma
```prisma
model User {
    id    Int     @id @default(autoincrement())
    email String  @unique
}
```

#### Rust/Diesel
```rust
#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub email: String,
}
```

### Step 3: Map Entity Relationships

For each entity, identify relationships:

#### Relationship Types

**One-to-One:**
```
User ──── Profile
```

**One-to-Many:**
```
User ──┬── Order
       ├── Order
       └── Order
```

**Many-to-Many:**
```
User ──┬──┬── Role
       │  │
       └──┴── Role
```

#### Relationship Analysis
For each relationship, note:
- Type (1:1, 1:N, M:N)
- Navigation direction (unidirectional/bidirectional)
- Cascade behavior
- Lazy/eager loading

### Step 4: Identify Entity Patterns

Look for these patterns:

#### Inheritance Patterns
- **Single Table**: All types in one table
- **Joined Table**: Type hierarchy in related tables
- **Concrete Table**: Each type in separate table

#### Audit Patterns
- `created_at`, `updated_at` timestamps
- `created_by`, `updated_by` user references
- Soft delete (`deleted_at`, `is_deleted`)

#### Versioning Patterns
- Version column for optimistic locking
- History tables for audit trail

### Step 5: Analyze Data Integrity

Check for:

**Constraints:**
- Primary keys defined
- Foreign keys defined
- Unique constraints
- Not null constraints
- Check constraints

**Indexes:**
- Primary key indexes (automatic)
- Foreign key indexes
- Query optimization indexes
- Unique indexes

**Missing Integrity:**
- Orphan records possible
- No foreign key constraints
- Missing unique constraints
- Implicit relationships (by convention only)

### Step 6: Identify Entity Anti-Patterns

**God Entity:**
- Entity with too many fields (>20-30)
- Multiple responsibilities
- Everything related to it

**Primitive Obsession:**
- Using primitives instead of value objects
- Email as string instead of Email type
- Money as float instead of Money type

**Anemic Entity:**
- Entity with only getters/setters
- No business logic
- Validation elsewhere

**Implicit Relationships:**
- Related entities not properly linked
- IDs stored as strings
- No foreign key constraints

### Step 7: Update Mental Model

Call `mental-model/update_viewpoint` with:
```yaml
viewpoint: "VP-S04"
data:
  entities:
    - name: "User"
      path: "src/domain/users/models.py"
      type: "aggregate_root"
      fields:
        - "id"
        - "email"
        - "password_hash"
        - "created_at"
      relationships:
        - target: "Profile"
          relationship_type: "one_to_one"
          cardinality: "1:1"
        - target: "Order"
          relationship_type: "one_to_many"
          cardinality: "1:N"
    - name: "Order"
      path: "src/domain/orders/models.py"
      type: "aggregate_root"
      fields:
        - "id"
        - "user_id"
        - "status"
        - "total"
      relationships:
        - target: "User"
          relationship_type: "many_to_one"
          cardinality: "N:1"
        - target: "OrderItem"
          relationship_type: "one_to_many"
          cardinality: "1:N"
```

## Output Schema

```yaml
entity_model:
  entities:
    - name: string
      path: string
      type: string           # aggregate_root, entity, value_object
      fields: [string]
      relationships:
        - target: string
          relationship_type: string
          cardinality: string
```

## Quality Indicators

- ✅ **Healthy**: Clear entity boundaries, proper relationships, constraints defined
- ⚠️ **Warning**: Some missing constraints, unclear relationships
- ❌ **Critical**: No constraints, god entities, anemic models

## Common Data Modeling Issues

| Issue | Impact | Solution |
|-------|--------|----------|
| Missing FK constraints | Data integrity | Add foreign key constraints |
| N+1 query potential | Performance | Use eager loading or batch queries |
| No indexes on FK | Performance | Add indexes on foreign keys |
| Implicit relationships | Integrity | Make relationships explicit |
| God entity | Maintainability | Split into focused entities |

## Entity Modeling Best Practices

- [ ] Primary key on every entity
- [ ] Foreign keys for relationships
- [ ] Indexes on frequently queried columns
- [ ] Audit fields (created_at, updated_at)
- [ ] Soft delete consideration
- [ ] Version field for optimistic locking
- [ ] Value objects for complex attributes
- [ ] Appropriate cascade settings

## Next Viewpoint
After completing VP-S04, proceed to **VP-S05: Interface Surface Analysis**.

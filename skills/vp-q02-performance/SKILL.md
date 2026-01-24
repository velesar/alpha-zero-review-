---
name: vp-q02-performance
version: 1.0
dependencies: [vp-s02-layer-architecture, vp-s04-entity-model, vp-s06-dependency-graph]
mcp_servers: [mental-model, methodology-kb]
---

# VP-Q02: Performance Analysis

## Purpose
Identify performance issues and bottlenecks, with focus on high-impact areas identified in the mental model.

## Prerequisites
- Structure phase completed
- Entity model documented
- Hotspots identified

## Instructions

### Step 1: Get Analysis Priorities
Call `mental-model/get_constraints` and `mental-model/get_model` to get:
- High priority paths
- Hotspot files
- API endpoints (from interface surface)
- Database entities

### Step 2: Database Performance Analysis

#### N+1 Query Detection

Look for patterns that suggest N+1 queries:

**Python/SQLAlchemy:**
```python
# Potential N+1 - accessing relationship in loop
for user in users:
    print(user.orders)  # Each iteration triggers a query

# Better: eager loading
users = session.query(User).options(joinedload(User.orders)).all()
```

**Python/Django:**
```python
# Potential N+1
for user in User.objects.all():
    print(user.profile.name)  # Each iteration queries profile

# Better: select_related/prefetch_related
User.objects.select_related('profile').all()
```

**TypeScript/TypeORM:**
```typescript
// Potential N+1
const users = await userRepo.find();
for (const user of users) {
    const orders = await user.orders;  // Query per user
}

// Better: relations option
const users = await userRepo.find({ relations: ['orders'] });
```

#### Missing Indexes

Check for:
- Foreign key columns without indexes
- Frequently filtered columns
- Sort columns
- Search columns

#### Expensive Queries

Look for:
- `SELECT *` without limits
- Missing pagination
- Complex JOINs without indexes
- Subqueries in loops

### Step 3: Algorithm Efficiency

#### Time Complexity Issues

Look for:
```python
# O(n²) - nested loops over same data
for item in items:
    for other in items:
        if item == other:
            ...

# O(n²) - list operations in loop
for item in items:
    if item in other_list:  # O(n) lookup
        ...

# Better: use set for O(1) lookup
other_set = set(other_list)
for item in items:
    if item in other_set:
        ...
```

#### Space Complexity Issues

Look for:
- Loading entire files into memory
- Unbounded list growth
- Large object accumulation
- Missing generators/iterators

### Step 4: Async/Concurrency Issues

#### Blocking in Async Context

**Python:**
```python
# Bad - blocking call in async function
async def get_data():
    response = requests.get(url)  # Blocks event loop
    return response.json()

# Good - async HTTP client
async def get_data():
    async with aiohttp.ClientSession() as session:
        async with session.get(url) as response:
            return await response.json()
```

**TypeScript:**
```typescript
// Missing await leads to fire-and-forget
async function saveData() {
    database.save(data);  // Missing await!
}

// Missing Promise.all for parallel operations
async function getData() {
    const a = await fetchA();  // Sequential
    const b = await fetchB();  // when parallel is possible
    return [a, b];
}
```

### Step 5: Memory Issues

Look for:
- Large object creation in loops
- Missing cleanup/disposal
- Unbounded caches
- Circular references preventing GC
- Memory leaks in event listeners

### Step 6: API Performance

#### Response Size
- Returning too much data
- Missing pagination
- No filtering support
- Including unused fields

#### Caching Opportunities
- Static data not cached
- Repeated computations
- Missing HTTP cache headers
- No query result caching

### Step 7: Add Findings (Batch Preferred)

**For multiple findings (preferred - ADR-0005):**
Call `mental-model/add_findings` with all performance findings at once:

```json
{
  "findings": [
    {
      "viewpoint": "VP-Q02",
      "category": "performance_database",
      "title": "N+1 query in user listing",
      "description": "User orders accessed in loop without eager loading",
      "file_path": "src/services/user_service.py",
      "line_number": 78,
      "base_severity": "MEDIUM",
      "recommendation": "Use eager loading: query.options(joinedload(User.orders))"
    },
    {
      "viewpoint": "VP-Q02",
      "category": "performance_algorithm",
      "title": "O(n²) complexity in search",
      "description": "Nested loops over same collection",
      "file_path": "src/utils/matcher.py",
      "line_number": 45,
      "base_severity": "MEDIUM",
      "recommendation": "Use set for O(1) lookup"
    }
  ]
}
```

**For single finding:**
Call `mental-model/add_finding`:

```json
{
  "viewpoint": "VP-Q02",
  "category": "performance_database",
  "title": "N+1 query in user listing",
  "description": "User orders accessed in loop without eager loading, causing N+1 database queries",
  "file_path": "src/services/user_service.py",
  "line_number": 78,
  "base_severity": "MEDIUM",
  "recommendation": "Use eager loading: query.options(joinedload(User.orders))"
}
```

## Performance Categories

| Category | Examples |
|----------|----------|
| `performance_database` | N+1 queries, missing indexes, expensive queries |
| `performance_algorithm` | O(n²) complexity, inefficient data structures |
| `performance_memory` | Memory leaks, large allocations, unbounded growth |
| `performance_network` | Blocking calls, missing parallel execution |
| `performance_caching` | Missing cache, cache invalidation issues |

## Quality Indicators

- ✅ **Healthy**: No critical issues, efficient patterns used
- ⚠️ **Warning**: Some N+1 queries, minor inefficiencies
- ❌ **Critical**: Blocking async, O(n²) in hot paths, memory leaks

## Common Performance Anti-Patterns

| Anti-Pattern | Impact | Solution |
|--------------|--------|----------|
| N+1 Queries | Database overload | Eager loading, batch queries |
| SELECT * | Network/memory overhead | Select specific columns |
| Missing pagination | Memory/response time | Add limit/offset or cursor |
| Sync in async | Thread blocking | Use async alternatives |
| Loop allocations | GC pressure | Pre-allocate, reuse objects |
| String concatenation | O(n²) for many concats | Use StringBuilder/join |

## Performance Review Checklist

- [ ] Database queries use appropriate indexes
- [ ] No N+1 query patterns
- [ ] Pagination on list endpoints
- [ ] Async operations properly awaited
- [ ] Large datasets processed in batches
- [ ] Caching for expensive computations
- [ ] Efficient data structures used
- [ ] No blocking calls in async context

## Next Viewpoint
After completing VP-Q02, proceed to **VP-Q03: Testability Analysis**.

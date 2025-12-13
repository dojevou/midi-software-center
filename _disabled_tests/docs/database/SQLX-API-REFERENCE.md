# SQLx 0.7 API Reference - Quick Guide

## Common Mistakes Fixed

### MISTAKE 1: Passing parameters to `query_as()`

```rust
// WRONG - query_as() only takes SQL string
let row: (i64,) = sqlx::query_as(&sql, &params)
    .fetch_one(pool)
    .await?;

// CORRECT - use .bind() for each parameter
let row: (i64,) = sqlx::query_as::<_, (i64,)>(
    "SELECT id FROM users WHERE id = $1"
)
.bind(user_id)
.fetch_one(pool)
.await?;
```

### MISTAKE 2: Assuming `query_as()` accepts second arg

```rust
// WRONG - second parameter doesn't exist
sqlx::query_as(&query, vec![param1, param2])

// CORRECT - chain .bind() calls
sqlx::query_as::<_, MyType>(sql)
    .bind(param1)
    .bind(param2)
    .fetch_one(pool)
    .await?
```

### MISTAKE 3: Using `query_as()` for dynamic queries

```rust
// WRONG - can't use query_as with dynamic SQL easily
let results = sqlx::query_as::<_, MyStruct>(&dynamic_sql, params)
    .fetch_all(pool)
    .await?;

// CORRECT - use query() for dynamic, then map
let rows = sqlx::query(&dynamic_sql)
    .fetch_all(pool)
    .await?;

let results: Vec<MyStruct> = rows.into_iter()
    .map(|row| MyStruct {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
    })
    .collect::<Result<_, _>>()?;
```

---

## SQLx 0.7 API Overview

### `sqlx::query()` - Runtime Query (Most Flexible)

**When to use:** Dynamic WHERE clauses, runtime SQL building

```rust
let row = sqlx::query("SELECT id, name FROM users WHERE id = $1")
    .bind(user_id)
    .fetch_one(pool)
    .await?;

let id: i64 = row.try_get("id")?;
let name: String = row.try_get("name")?;
```

**Pros:**
- No compile-time checking needed
- Works with dynamic SQL
- Flexible parameter passing with `.bind()`

**Cons:**
- No type safety at compile time
- Must use `.try_get()` for value extraction

---

### `sqlx::query_as()` - Typed Query (Best for Static SQL)

**When to use:** Static SQL with compile-time tuple types

```rust
let rows: Vec<(i64, String)> = sqlx::query_as::<_, (i64, String)>(
    "SELECT id, name FROM users WHERE id = $1"
)
.bind(user_id)
.fetch_all(pool)
.await?;

for (id, name) in rows {
    println!("{}: {}", id, name);
}
```

**Pros:**
- Type safe at compile time
- Access values via tuple indexing
- Zero runtime overhead

**Cons:**
- Can't use with dynamic SQL
- Tuple type must match column order exactly
- Column order matters, not names

---

### `sqlx::query_as!()` - Macro Query (Maximum Type Safety)

**When to use:** Maximum compile-time safety (requires `.sqlx` metadata)

```rust
let rows = sqlx::query_as!(
    User,
    "SELECT id, name, email FROM users WHERE id = $1",
    user_id
)
.fetch_all(pool)
.await?;
```

**Pros:**
- Full compile-time type checking
- Uses struct fields, not tuples
- Automatic mapping

**Cons:**
- Requires `.sqlx` metadata (run during build)
- Can't use with fully dynamic SQL
- Build-time dependency on database

---

### `sqlx::query_builder::QueryBuilder` - Dynamic Parameterized

**When to use:** Dynamic SQL with SQL injection protection

```rust
use sqlx::QueryBuilder;

let mut qb = QueryBuilder::new("SELECT * FROM users WHERE 1=1");

if let Some(name) = filter.name {
    qb.push(" AND name = ").push_bind(name);
}

if let Some(email) = filter.email {
    qb.push(" AND email = ").push_bind(email);
}

let rows = qb.build().fetch_all(pool).await?;
```

**Pros:**
- Safe from SQL injection
- Flexible dynamic building
- Proper parameterization with `.push_bind()`

**Cons:**
- No tuple mapping built-in
- Still need `.map()` for struct conversion
- Slightly more verbose

---

## Extraction Methods

### From `Row` (returned by `query()`)

```rust
// Safe extraction with error handling
let value: String = row.try_get("column_name")?;

// With default
let value: String = row.try_get("column_name").unwrap_or_default();

// With fallback
let value: String = row.try_get("column_name")
    .unwrap_or_else(|_| "default".to_string());

// By index (use carefully)
let value: String = row.get(0);

// Optional column
let maybe_value: Option<String> = row.try_get("column_name").ok();
```

### From Tuple (returned by `query_as()`)

```rust
let (id, name): (i64, String) = row;

// Or destructure in iteration
for (id, name) in rows {
    println!("{}: {}", id, name);
}
```

---

## Common Patterns

### Pattern 1: Static Query with Parameters

```rust
let user = sqlx::query_as::<_, (i64, String, String)>(
    "SELECT id, name, email FROM users WHERE id = $1"
)
.bind(user_id)
.fetch_optional(pool)
.await?;
```

### Pattern 2: Dynamic WHERE with QueryBuilder

```rust
let mut qb = QueryBuilder::new(
    "SELECT id, name, email FROM users WHERE 1=1"
);

if let Some(name) = filters.name {
    qb.push(" AND name ILIKE ").push_bind(format!("%{}%", name));
}

let rows = qb.build().fetch_all(pool).await?;

for row in rows {
    let id: i64 = row.try_get("id")?;
    let name: String = row.try_get("name")?;
}
```

### Pattern 3: Dynamic WHERE with String Building (Use with Caution)

```rust
let mut where_clause = String::new();

if let Some(min_age) = filters.min_age {
    where_clause.push_str(&format!(" AND age >= {}", min_age));
}

if let Some(name) = &filters.name {
    // IMPORTANT: Escape quotes to prevent SQL injection
    where_clause.push_str(&format!(
        " AND name ILIKE '%{}%'",
        name.replace("'", "''")
    ));
}

let sql = format!("SELECT * FROM users WHERE 1=1{}", where_clause);
let rows = sqlx::query(&sql).fetch_all(pool).await?;
```

**Note:** This pattern (used in the fix) works but is less safe than QueryBuilder. Use QueryBuilder for production code.

### Pattern 4: Struct Mapping from Row

```rust
#[derive(sqlx::FromRow)]
struct User {
    id: i64,
    name: String,
    email: String,
}

let users = sqlx::query_as::<_, User>(
    "SELECT id, name, email FROM users WHERE active = true"
)
.fetch_all(pool)
.await?;
```

---

## Error Handling Patterns

### Pattern 1: Propagate with `?`

```rust
let row = sqlx::query("SELECT * FROM users WHERE id = $1")
    .bind(id)
    .fetch_one(pool)
    .await?;  // ? propagates sqlx::Error
```

### Pattern 2: Custom Error Message

```rust
let row = sqlx::query("SELECT * FROM users WHERE id = $1")
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to fetch user: {}", e))?;
```

### Pattern 3: Handle Optional

```rust
let row = sqlx::query("SELECT * FROM users WHERE id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await?;  // Returns Ok(None) if not found

match row {
    Some(row) => {
        let name: String = row.try_get("name")?;
        Ok(Some(name))
    }
    None => Ok(None),
}
```

### Pattern 4: Batch with Try Collect

```rust
let names: Result<Vec<String>, sqlx::Error> = sqlx::query(
    "SELECT name FROM users"
)
.fetch_all(pool)
.await?
.into_iter()
.map(|row| row.try_get("name"))
.collect();
```

---

## The Fix Applied

**Location:** `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/commands/database.rs`

**Lines Changed:** 48-129 (entire `database_search()` function)

**Pattern Used:** Pattern 3 (Dynamic WHERE with String Building) + Pattern 4 (Struct Mapping)

**Reason:** The function needed dynamic WHERE clause building based on optional filter parameters, so `query()` with manual string building was the most straightforward approach.

**For Production:** Consider migrating to QueryBuilder (Pattern 2) for better SQL injection protection.

---

## SQLx 0.7 Signature Reference

```rust
// query() - returns Query<'_, Postgres, PgRow>
pub fn query(sql: &str) -> Query<'_, Postgres, PgRow>

// query_as<T>() - returns QueryAs<'_, Postgres, T>
pub fn query_as<'q, T>(sql: &'q str) -> QueryAs<'q, Postgres, T>

// query!() - macro, compile-time validated
// Requires: sqlx-cli + .sqlx metadata

// QueryBuilder::new() - dynamic SQL builder
pub fn new(sql: &str) -> Self
```

**Key Point:** `query()` and `query_as()` are functions that take ONLY the SQL string. Parameters are added via `.bind()`, NOT as function arguments.

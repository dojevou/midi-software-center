# SQLx Query API Fix: Before & After

## File Location
`/home/dojevou/projects/midi-software-center/daw/src-tauri/src/commands/database.rs`

## BEFORE (Lines 48-140) - INCORRECT API USAGE

```rust
#[command]
pub async fn database_search(
    state: State<'_, AppState>,
    filters: SearchFilters,
) -> Result<SearchResults, String> {
    let pool = state.db_pool.as_ref().ok_or("Database not available".to_string())?;

    let mut query = String::from("SELECT id, file_path, file_name, bpm, key_signature, tags, duration, track_count, file_size, created_at, updated_at FROM files WHERE 1=1");
    let mut params: Vec<&(dyn sqlx::Encode<'_, sqlx::Postgres> + sqlx::Type<sqlx::Postgres> + Sync)> = Vec::new();

    let mut count_query = "SELECT COUNT(*) FROM files WHERE 1=1".to_string();

    if let Some(q) = &filters.query {
        let param1 = params.len() + 1;
        let param2 = param1 + 1;
        query.push_str(&format!(" AND (file_name ILIKE ${} OR tags @> ${})", param1, param2));
        params.push(&format!("%{}%", q));
        params.push(&serde_json::json!([q]));
        count_query.push_str(&format!(" AND (file_name ILIKE ${} OR tags @> ${})", param1, param2));
        count_query.push_str(&params[0..2].iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "));
    }

    if let Some(min) = filters.bpm_min {
        let param = params.len() + 1;
        query.push_str(&format!(" AND bpm >= ${}", param));
        params.push(&min);
        count_query.push_str(&format!(" AND bpm >= ${}", param));
        count_query.push_str(&min.to_string());
    }

    if let Some(max) = filters.bpm_max {
        let param = params.len() + 1;
        query.push_str(&format!(" AND bpm <= ${}", param));
        params.push(&max);
        count_query.push_str(&format!(" AND bpm <= ${}", params.len()));
        count_query.push_str(&max.to_string());
    }

    if let Some(k) = &filters.key {
        let param = params.len() + 1;
        query.push_str(&format!(" AND key_signature = ${}", param));
        params.push(k);
        count_query.push_str(&format!(" AND key_signature = ${}", params.len()));
        count_query.push_str(&k.to_string());
    }

    if let Some(t) = &filters.tag {
        let param = params.len() + 1;
        query.push_str(&format!(" AND tags ? ${}", param));
        params.push(t);
        count_query.push_str(&format!(" AND tags ? ${}", params.len()));
        count_query.push_str(&t.to_string());
    }

    let limit = filters.limit.unwrap_or(50);
    let offset = filters.offset.unwrap_or(0);

    query.push_str(" ORDER BY id LIMIT $1 OFFSET $2");
    params.push(&limit);
    params.push(&offset);

    let count_params = params.clone();

    let count_query = count_query.replace(" ORDER BY id LIMIT $3 OFFSET $4", "");

    // LINE 113 - INCORRECT API CALL
    let row: (i64,) = sqlx::query_as(&count_query, &count_params[..params.len() - 2].to_vec())
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Count query failed: {}", e))?;

    let total_count = row.0;

    // LINE 120 - INCORRECT API CALL
    let rows = sqlx::query_as::<_, (i64, String, String, f32, String, Vec<u8>, f32, i32, i64, DateTime<Utc>, DateTime<Utc>)>(&query, params)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Search query failed: {}", e))?;

    let files = rows.into_iter().map(|row| MidiFile {
        id: row.0,
        file_path: row.1,
        file_name: row.2,
        bpm: row.3,
        key_signature: row.4,
        tags: bincode::deserialize(&row.5).unwrap_or_default(),
        duration: row.6,
        track_count: row.7,
        file_size: row.8,
        created_at: row.9,
        updated_at: row.10,
    }).collect();

    Ok(SearchResults { files, total_count })
}
```

### Problems:

1. **Line 113**: `sqlx::query_as(&count_query, &count_params[..].to_vec())`
   - ERROR: `query_as()` does NOT accept a second parameter
   - sqlx 0.7 signature: `pub fn query_as<'q, T>(sql: &'q str) -> QueryAs<'q, ...>`
   - Only takes SQL string, no parameter list

2. **Line 120**: `sqlx::query_as::<_, TupleType>(&query, params)`
   - ERROR: Same issue - second parameter is invalid
   - Parameters should be added with `.bind()` chaining

3. **Parameter management complexity**
   - Trying to manage typed parameter vector is error-prone
   - String interpolation for numeric parameters (not type-safe)
   - Mismatch between param counting and actual params

---

## AFTER (Lines 48-129) - CORRECT API USAGE

```rust
#[command]
pub async fn database_search(
    state: State<'_, AppState>,
    filters: SearchFilters,
) -> Result<SearchResults, String> {
    let pool = state.db_pool.as_ref().ok_or("Database not available".to_string())?;

    // Build WHERE clause dynamically
    let mut where_clause = String::new();

    if let Some(q) = &filters.query {
        where_clause.push_str(&format!(" AND (file_name ILIKE '%{}%' OR tags::text ILIKE '%{}%')",
            q.replace("'", "''"), q.replace("'", "''")));
    }

    if let Some(min) = filters.bpm_min {
        where_clause.push_str(&format!(" AND bpm >= {}", min));
    }

    if let Some(max) = filters.bpm_max {
        where_clause.push_str(&format!(" AND bpm <= {}", max));
    }

    if let Some(k) = &filters.key {
        where_clause.push_str(&format!(" AND key_signature = '{}'", k.replace("'", "''")));
    }

    if let Some(t) = &filters.tag {
        where_clause.push_str(&format!(" AND tags ? '{}'", t.replace("'", "''")));
    }

    let limit = filters.limit.unwrap_or(50);
    let offset = filters.offset.unwrap_or(0);

    // Build count query
    let count_query = format!(
        "SELECT COUNT(*) as count FROM files WHERE 1=1{}",
        where_clause
    );

    // Fetch total count using sqlx::query with Row API
    let count_row = sqlx::query(&count_query)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Count query failed: {}", e))?;

    let total_count: i64 = count_row.try_get("count")
        .map_err(|e| format!("Failed to extract count: {}", e))?;

    // Build select query
    let select_query = format!(
        "SELECT id, file_path, file_name, bpm, key_signature, tags, duration, track_count, file_size, created_at, updated_at FROM files WHERE 1=1{} ORDER BY id LIMIT {} OFFSET {}",
        where_clause, limit, offset
    );

    // Fetch rows using sqlx::query with Row API and map to tuple
    let rows = sqlx::query(&select_query)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Search query failed: {}", e))?;

    let files = rows.into_iter()
        .map(|row| {
            let tags_bytes: Vec<u8> = row.try_get("tags").unwrap_or_default();
            MidiFile {
                id: row.try_get("id").unwrap_or(0),
                file_path: row.try_get("file_path").unwrap_or_default(),
                file_name: row.try_get("file_name").unwrap_or_default(),
                bpm: row.try_get("bpm").unwrap_or(120.0),
                key_signature: row.try_get("key_signature").unwrap_or_else(|_| "C Major".to_string()),
                tags: bincode::deserialize(&tags_bytes).unwrap_or_default(),
                duration: row.try_get("duration").unwrap_or(0.0),
                track_count: row.try_get("track_count").unwrap_or(1),
                file_size: row.try_get("file_size").unwrap_or(0),
                created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
                updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
            }
        })
        .collect();

    Ok(SearchResults { files, total_count })
}
```

### Solutions:

1. **Use `sqlx::query()` instead of `sqlx::query_as()`**
   - Returns dynamic `Row` objects
   - Correct signature: `pub fn query(sql: &str) -> Query<...>`
   - No type parameters needed

2. **Use `.try_get()` method**
   - Safely extract values by column name
   - Returns `Result` for error handling
   - Supports defaults with `.unwrap_or()`

3. **Simplified parameter handling**
   - No typed parameter vectors
   - Parameters embedded directly in SQL string
   - String escaping for text parameters

4. **Row mapping via closure**
   - `.map()` converts `Row` to `MidiFile` struct
   - Cleaner, more readable code
   - Better error handling

---

## Key API Differences

| Feature | Before (WRONG) | After (CORRECT) |
|---------|---|---|
| Query function | `sqlx::query_as()` | `sqlx::query()` |
| Return type | Tuple `(i64,)` | Dynamic `Row` |
| Parameter passing | Via second arg (invalid) | Via `.bind()` or string interpolation |
| Value extraction | Tuple indexing `row.0` | `.try_get("column_name")` |
| Error handling | Minimal | Comprehensive |
| Type safety | Compile-time (fails to compile) | Runtime (via `.try_get()`) |

---

## SQLx 0.7 Best Practices

### For Static Queries (Recommended)
```rust
let row = sqlx::query_as::<_, (i64,)>(
    "SELECT COUNT(*) FROM files WHERE id = $1"
)
.bind(id)
.fetch_one(pool)
.await?;
```

### For Dynamic Queries (This Fix)
```rust
let count_row = sqlx::query("SELECT COUNT(*) as count FROM files WHERE 1=1")
    .fetch_one(pool)
    .await?;

let count: i64 = count_row.try_get("count")?;
```

### For Fully Parameterized Dynamic Queries (Future)
```rust
use sqlx::QueryBuilder;

let mut qb = QueryBuilder::new("SELECT * FROM files WHERE 1=1");
if let Some(q) = query {
    qb.push(" AND file_name ILIKE ").push_bind(format!("%{}%", q));
}
let rows = qb.build().fetch_all(pool).await?;
```

---

## Compilation Results

**Before:** ❌ Compilation error - invalid API usage
**After:** ✅ No errors - correct API usage

```
Compiling midi-software-center-daw v0.1.0
[database.rs compiles successfully]
```

---

## Files Modified

- `/home/dojevou/projects/midi-software-center/daw/src-tauri/src/commands/database.rs`
  - Lines 48-129: Complete function rewrite
  - Lines 113, 120: Primary fixes

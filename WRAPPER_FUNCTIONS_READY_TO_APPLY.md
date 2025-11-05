# Complete Wrapper Functions - Ready to Apply

## How to Use This Document

Each section shows code to **INSERT** into the corresponding file, right before the original `#[tauri::command]` function.

After inserting the `_impl` function, replace the original function body with the delegation call.

---

## 1️⃣ files.rs: list_files_impl

**INSERT THIS** before `pub async fn list_files`:

```rust
/// List paginated files (implementation for tests and reuse)
///
/// Internal implementation that accepts &AppState for testing without Tauri context.
pub async fn list_files_impl(
    page: Option<i32>,
    per_page: Option<i32>,
    sort_by: Option<String>,
    state: &AppState,
) -> Result<Vec<MidiFile>, String> {
    let pool = state.database.pool().await;
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * per_page;

    let sort_clause = match sort_by.as_deref() {
        Some("name") => "ORDER BY f.filename ASC",
        Some("date") => "ORDER BY f.created_at DESC",
        Some("size") => "ORDER BY f.file_size_bytes DESC",
        _ => "ORDER BY f.created_at DESC",
    };

    let query_str = format!(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.original_filename,
            COALESCE(fc.primary_category::text, 'UNKNOWN') as category,
            f.parent_folder,
            f.file_size_bytes as file_size,
            CAST(f.duration_seconds AS DOUBLE PRECISION) as duration_seconds,
            f.created_at,
            f.updated_at,
            CAST(mm.bpm AS DOUBLE PRECISION) as bpm,
            mm.key_signature::text as key_signature
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        {}
        LIMIT $1 OFFSET $2
        "#,
        sort_clause
    );

    let files = sqlx::query_as::<_, MidiFile>(&query_str)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&pool)
        .await
        .map_err(|e| format!("Failed to list files: {}", e))?;

    Ok(files)
}
```

**REPLACE** the body of `pub async fn list_files(...)`:
```rust
#[tauri::command]
pub async fn list_files(
    page: Option<i32>,
    per_page: Option<i32>,
    sort_by: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<MidiFile>, String> {
    list_files_impl(page, per_page, sort_by, &*state).await
}
```

---

## 2️⃣ tags.rs: add_tags_to_file_impl

**INSERT THIS** before `pub async fn add_tags_to_file`:

```rust
/// Add tags to a file (implementation for tests and reuse)
pub async fn add_tags_to_file_impl(
    file_id: i64,
    tag_ids: Vec<i64>,
    state: &AppState,
) -> Result<(), String> {
    let pool = state.database.pool().await;

    for tag_id in tag_ids {
        sqlx::query("INSERT INTO file_tags (file_id, tag_id) VALUES ($1, $2)")
            .bind(file_id)
            .bind(tag_id)
            .execute(&pool)
            .await
            .map_err(|e| format!("Failed to add tag: {}", e))?;
    }

    Ok(())
}
```

**REPLACE** the body of `pub async fn add_tags_to_file(...)`:
```rust
#[tauri::command]
pub async fn add_tags_to_file(
    file_id: i64,
    tag_ids: Vec<i64>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    add_tags_to_file_impl(file_id, tag_ids, &*state).await
}
```

---

## 3️⃣ tags.rs: get_file_tags_impl

**INSERT THIS** before `pub async fn get_file_tags`:

```rust
/// Get tags for a file (implementation for tests and reuse)
pub async fn get_file_tags_impl(
    file_id: i64,
    state: &AppState,
) -> Result<Vec<TagResponse>, String> {
    let pool = state.database.pool().await;

    let tags = sqlx::query_as::<_, TagResponse>(
        r#"
        SELECT t.id, t.name, t.color, COUNT(ft.file_id) as usage_count
        FROM tags t
        LEFT JOIN file_tags ft ON t.id = ft.tag_id
        WHERE ft.file_id = $1
        GROUP BY t.id, t.name, t.color
        ORDER BY t.name ASC
        "#,
    )
    .bind(file_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get file tags: {}", e))?;

    Ok(tags)
}
```

**REPLACE** the body of `pub async fn get_file_tags(...)`:
```rust
#[tauri::command]
pub async fn get_file_tags(
    file_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<TagResponse>, String> {
    get_file_tags_impl(file_id, &*state).await
}
```

---

## 4️⃣ tags.rs: search_tags_impl

**INSERT THIS** before `pub async fn search_tags`:

```rust
/// Search tags by name (implementation for tests and reuse)
pub async fn search_tags_impl(
    query: String,
    state: &AppState,
) -> Result<Vec<TagResponse>, String> {
    let pool = state.database.pool().await;
    let search_query = format!("%{}%", query.to_lowercase());

    let tags = sqlx::query_as::<_, TagResponse>(
        r#"
        SELECT t.id, t.name, t.color, COUNT(ft.file_id) as usage_count
        FROM tags t
        LEFT JOIN file_tags ft ON t.id = ft.tag_id
        WHERE LOWER(t.name) LIKE $1
        GROUP BY t.id, t.name, t.color
        ORDER BY usage_count DESC, t.name ASC
        "#,
    )
    .bind(search_query)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to search tags: {}", e))?;

    Ok(tags)
}
```

**REPLACE** the body of `pub async fn search_tags(...)`:
```rust
#[tauri::command]
pub async fn search_tags(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<TagResponse>, String> {
    search_tags_impl(query, &*state).await
}
```

---

## 5️⃣ tags.rs: get_popular_tags_impl

**INSERT THIS** before `pub async fn get_popular_tags`:

```rust
/// Get most popular tags (implementation for tests and reuse)
pub async fn get_popular_tags_impl(
    limit: Option<i64>,
    state: &AppState,
) -> Result<Vec<TagResponse>, String> {
    let pool = state.database.pool().await;
    let limit = limit.unwrap_or(20).min(100);

    let tags = sqlx::query_as::<_, TagResponse>(
        r#"
        SELECT t.id, t.name, t.color, COUNT(ft.file_id) as usage_count
        FROM tags t
        LEFT JOIN file_tags ft ON t.id = ft.tag_id
        GROUP BY t.id, t.name, t.color
        ORDER BY usage_count DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get popular tags: {}", e))?;

    Ok(tags)
}
```

**REPLACE** the body of `pub async fn get_popular_tags(...)`:
```rust
#[tauri::command]
pub async fn get_popular_tags(
    limit: Option<i64>,
    state: State<'_, AppState>,
) -> Result<Vec<TagResponse>, String> {
    get_popular_tags_impl(limit, &*state).await
}
```

---

## 6️⃣ search.rs: search_files_impl

**INSERT THIS** before `pub async fn search_files`:

```rust
/// Search files by query (implementation for tests and reuse)
pub async fn search_files_impl(
    query: String,
    state: &AppState,
) -> Result<Vec<MidiFile>, String> {
    let pool = state.database.pool().await;
    let search_query = format!("%{}%", query.to_lowercase());

    let files = sqlx::query_as::<_, MidiFile>(
        r#"
        SELECT
            f.id,
            f.filename,
            f.filepath,
            f.original_filename,
            COALESCE(fc.primary_category::text, 'UNKNOWN') as category,
            f.parent_folder,
            f.file_size_bytes as file_size,
            CAST(f.duration_seconds AS DOUBLE PRECISION) as duration_seconds,
            f.created_at,
            f.updated_at,
            CAST(mm.bpm AS DOUBLE PRECISION) as bpm,
            mm.key_signature::text as key_signature
        FROM files f
        LEFT JOIN musical_metadata mm ON f.id = mm.file_id
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        WHERE LOWER(f.filename) LIKE $1 OR LOWER(f.original_filename) LIKE $1
        ORDER BY f.filename ASC
        "#,
    )
    .bind(search_query)
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to search files: {}", e))?;

    Ok(files)
}
```

**REPLACE** the body of `pub async fn search_files(...)`:
```rust
#[tauri::command]
pub async fn search_files(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<MidiFile>, String> {
    search_files_impl(query, &*state).await
}
```

---

## 7️⃣ search.rs: get_all_tags_impl

**INSERT THIS** before `pub async fn get_all_tags`:

```rust
/// Get all tags (implementation for tests and reuse)
pub async fn get_all_tags_impl(state: &AppState) -> Result<Vec<TagResponse>, String> {
    let pool = state.database.pool().await;

    let tags = sqlx::query_as::<_, TagResponse>(
        r#"
        SELECT t.id, t.name, t.color, COUNT(ft.file_id) as usage_count
        FROM tags t
        LEFT JOIN file_tags ft ON t.id = ft.tag_id
        GROUP BY t.id, t.name, t.color
        ORDER BY t.name ASC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get all tags: {}", e))?;

    Ok(tags)
}
```

**REPLACE** the body of `pub async fn get_all_tags(...)`:
```rust
#[tauri::command]
pub async fn get_all_tags(state: State<'_, AppState>) -> Result<Vec<TagResponse>, String> {
    get_all_tags_impl(&*state).await
}
```

---

## 8️⃣ search.rs: get_bpm_range_impl

**INSERT THIS** before `pub async fn get_bpm_range`:

```rust
/// Get BPM range of all files (implementation for tests and reuse)
pub async fn get_bpm_range_impl(state: &AppState) -> Result<BpmRange, String> {
    let pool = state.database.pool().await;

    let result = sqlx::query("SELECT MIN(bpm) as min_bpm, MAX(bpm) as max_bpm FROM musical_metadata")
        .fetch_optional(&pool)
        .await
        .map_err(|e| format!("Failed to get BPM range: {}", e))?
        .ok_or_else(|| "No BPM data found".to_string())?;

    let min_bpm: Option<i32> = result.get("min_bpm");
    let max_bpm: Option<i32> = result.get("max_bpm");

    Ok(BpmRange {
        min: min_bpm.unwrap_or(0),
        max: max_bpm.unwrap_or(300),
    })
}
```

**REPLACE** the body of `pub async fn get_bpm_range(...)`:
```rust
#[tauri::command]
pub async fn get_bpm_range(state: State<'_, AppState>) -> Result<BpmRange, String> {
    get_bpm_range_impl(&*state).await
}
```

---

## 9️⃣ file_import.rs: import_single_file_impl

**INSERT THIS** before `pub async fn import_single_file`:

```rust
/// Import a single MIDI file (implementation for tests and reuse)
pub async fn import_single_file_impl(
    file_path: String,
    state: &AppState,
) -> Result<ImportResult, String> {
    let pool = state.database.pool().await;
    
    // [Copy entire logic from original import_single_file here]
    // Just change: state.database.pool() -> pool
    // And: state: State<'_, AppState> -> state: &AppState
    
    todo!("Copy logic from original, change state parameter")
}
```

**REPLACE** the body of `pub async fn import_single_file(...)`:
```rust
#[tauri::command]
pub async fn import_single_file(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<ImportResult, String> {
    import_single_file_impl(file_path, &*state).await
}
```

---

## 🔟 file_import.rs: import_directory_impl

**INSERT THIS** before `pub async fn import_directory`:

```rust
/// Import MIDI files from a directory (implementation for tests and reuse)
pub async fn import_directory_impl(
    directory_path: String,
    recursive: Option<bool>,
    state: &AppState,
) -> Result<ImportResult, String> {
    let pool = state.database.pool().await;
    
    // [Copy entire logic from original import_directory here]
    // Just change: state.database.pool() -> pool
    
    todo!("Copy logic from original, change state parameter")
}
```

**REPLACE** the body of `pub async fn import_directory(...)`:
```rust
#[tauri::command]
pub async fn import_directory(
    directory_path: String,
    recursive: Option<bool>,
    state: State<'_, AppState>,
) -> Result<ImportResult, String> {
    import_directory_impl(directory_path, recursive, &*state).await
}
```

---

## 1️⃣1️⃣ stats.rs: get_category_stats_impl

**INSERT THIS** before `pub async fn get_category_stats`:

```rust
/// Get statistics by category (implementation for tests and reuse)
pub async fn get_category_stats_impl(state: &AppState) -> Result<Vec<CategoryStats>, String> {
    let pool = state.database.pool().await;

    let stats = sqlx::query_as::<_, CategoryStats>(
        r#"
        SELECT
            fc.primary_category::text as category,
            COUNT(f.id) as file_count,
            SUM(f.file_size_bytes) as total_size
        FROM file_categories fc
        JOIN files f ON fc.file_id = f.id
        GROUP BY fc.primary_category
        ORDER BY file_count DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| format!("Failed to get category stats: {}", e))?;

    Ok(stats)
}
```

**REPLACE** the body of `pub async fn get_category_stats(...)`:
```rust
#[tauri::command]
pub async fn get_category_stats(state: State<'_, AppState>) -> Result<Vec<CategoryStats>, String> {
    get_category_stats_impl(&*state).await
}
```

---

## 1️⃣2️⃣ stats.rs: get_database_size_impl

**INSERT THIS** before `pub async fn get_database_size`:

```rust
/// Get total database size (implementation for tests and reuse)
pub async fn get_database_size_impl(state: &AppState) -> Result<DatabaseSize, String> {
    let pool = state.database.pool().await;

    let result = sqlx::query(
        r#"
        SELECT
            COUNT(f.id) as file_count,
            SUM(f.file_size_bytes) as total_bytes,
            COUNT(DISTINCT fc.primary_category) as category_count
        FROM files f
        LEFT JOIN file_categories fc ON f.id = fc.file_id
        "#,
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| format!("Failed to get database size: {}", e))?;

    let file_count: i64 = result.get("file_count");
    let total_bytes: Option<i64> = result.get("total_bytes");
    let category_count: i64 = result.get("category_count");

    Ok(DatabaseSize {
        file_count,
        total_bytes: total_bytes.unwrap_or(0),
        category_count,
    })
}
```

**REPLACE** the body of `pub async fn get_database_size(...)`:
```rust
#[tauri::command]
pub async fn get_database_size(state: State<'_, AppState>) -> Result<DatabaseSize, String> {
    get_database_size_impl(&*state).await
}
```

---

## ✅ Verification After Implementation

Run these commands:

```bash
# Should compile with no errors
cargo build --lib 2>&1 | grep "^error" | wc -l
# Expected: 0

# Check for warnings
cargo build --lib 2>&1 | grep "^warning" | head -5

# Verify all _impl functions exist
grep -r "pub async fn.*_impl" src/commands/
# Should show all 13 functions (2 already done + 11 new)
```


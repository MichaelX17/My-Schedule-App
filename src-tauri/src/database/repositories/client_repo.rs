use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::errors::{AppError, AppResult};
use crate::models::client::{
    Client, ClientArchiveFilter, ClientListResult, ClientSortBy, ListClientsParams,
};

const SELECT_COLUMNS: &str =
    "id, full_name, phone, email, address, notes, is_archived, archived_at, created_at, updated_at";

const DEFAULT_PAGE_LIMIT: i64 = 50;
const MAX_PAGE_LIMIT: i64 = 200;

/// Escapes `%`, `_`, and `\` so user search queries are matched literally in SQLite `LIKE`.
fn escape_like_pattern(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len() + 2);
    escaped.push('%');
    for ch in raw.chars() {
        match ch {
            '%' | '_' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped.push('%');
    escaped
}

fn push_filters(
    builder: &mut QueryBuilder<'_, Sqlite>,
    filter: ClientArchiveFilter,
    search_pattern: Option<String>,
) {
    let mut has_where = false;
    match filter {
        ClientArchiveFilter::Active => {
            builder.push(" WHERE is_archived = 0");
            has_where = true;
        }
        ClientArchiveFilter::Archived => {
            builder.push(" WHERE is_archived = 1");
            has_where = true;
        }
        ClientArchiveFilter::All => {}
    }

    if let Some(pattern) = search_pattern {
        if has_where {
            builder.push(" AND ");
        } else {
            builder.push(" WHERE ");
        }
        builder.push("(full_name LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" ESCAPE '\\' OR email LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" ESCAPE '\\' OR phone LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" ESCAPE '\\' OR notes LIKE ");
        builder.push_bind(pattern);
        builder.push(" ESCAPE '\\')");
    }
}

/// Lists clients with search, archive filtering, sorting, and pagination.
pub async fn list(pool: &SqlitePool, params: &ListClientsParams) -> AppResult<ClientListResult> {
    let filter = params.filter.unwrap_or_default();
    let sort_by = params.sort_by.unwrap_or_default();
    let limit = params
        .limit
        .unwrap_or(DEFAULT_PAGE_LIMIT)
        .clamp(1, MAX_PAGE_LIMIT);
    let offset = params.offset.unwrap_or(0).max(0);

    let search_pattern = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(escape_like_pattern);

    // Filtered total count
    let mut count_qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new("SELECT COUNT(*) FROM clients");
    push_filters(&mut count_qb, filter, search_pattern.clone());
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    // Active / archived summary counts
    let active_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE is_archived = 0")
            .fetch_one(pool)
            .await?;
    let archived_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE is_archived = 1")
            .fetch_one(pool)
            .await?;

    // Paginated items
    let mut items_qb: QueryBuilder<'_, Sqlite> =
        QueryBuilder::new(format!("SELECT {SELECT_COLUMNS} FROM clients"));
    push_filters(&mut items_qb, filter, search_pattern);

    let order_clause = match sort_by {
        ClientSortBy::NameAsc => " ORDER BY full_name COLLATE NOCASE ASC, created_at DESC",
        ClientSortBy::NameDesc => " ORDER BY full_name COLLATE NOCASE DESC, created_at DESC",
        ClientSortBy::RecentlyUpdated => " ORDER BY updated_at DESC, full_name COLLATE NOCASE ASC",
        ClientSortBy::Newest => " ORDER BY created_at DESC, full_name COLLATE NOCASE ASC",
    };
    items_qb.push(order_clause);
    items_qb.push(" LIMIT ");
    items_qb.push_bind(limit);
    items_qb.push(" OFFSET ");
    items_qb.push_bind(offset);

    let items = items_qb.build_query_as::<Client>().fetch_all(pool).await?;

    Ok(ClientListResult {
        items,
        total,
        active_count,
        archived_count,
    })
}

/// Fetches a single client by stable ID.
pub async fn get_by_id(pool: &SqlitePool, id: &str) -> AppResult<Option<Client>> {
    let query = format!("SELECT {SELECT_COLUMNS} FROM clients WHERE id = ?1");
    let client = sqlx::query_as::<_, Client>(&query)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(client)
}

/// Inserts a new client record inside an existing transaction.
pub async fn insert_tx(tx: &mut sqlx::Transaction<'_, Sqlite>, client: &Client) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO clients (
            id, full_name, phone, email, address, notes,
            is_archived, archived_at, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
    )
    .bind(&client.id)
    .bind(&client.full_name)
    .bind(&client.phone)
    .bind(&client.email)
    .bind(&client.address)
    .bind(&client.notes)
    .bind(if client.is_archived { 1 } else { 0 })
    .bind(&client.archived_at)
    .bind(&client.created_at)
    .bind(&client.updated_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Inserts a new client record.
pub async fn insert(pool: &SqlitePool, client: &Client) -> AppResult<Client> {
    sqlx::query(
        r#"
        INSERT INTO clients (
            id, full_name, phone, email, address, notes,
            is_archived, archived_at, created_at, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
    )
    .bind(&client.id)
    .bind(&client.full_name)
    .bind(&client.phone)
    .bind(&client.email)
    .bind(&client.address)
    .bind(&client.notes)
    .bind(if client.is_archived { 1 } else { 0 })
    .bind(&client.archived_at)
    .bind(&client.created_at)
    .bind(&client.updated_at)
    .execute(pool)
    .await?;

    get_by_id(pool, &client.id)
        .await?
        .ok_or_else(|| AppError::Internal("Inserted client could not be reloaded.".into()))
}

/// Updates an existing client's editable profile fields.
pub async fn update(pool: &SqlitePool, client: &Client) -> AppResult<Client> {
    let result = sqlx::query(
        r#"
        UPDATE clients
        SET full_name = ?1,
            phone = ?2,
            email = ?3,
            address = ?4,
            notes = ?5,
            updated_at = ?6
        WHERE id = ?7
        "#,
    )
    .bind(&client.full_name)
    .bind(&client.phone)
    .bind(&client.email)
    .bind(&client.address)
    .bind(&client.notes)
    .bind(&client.updated_at)
    .bind(&client.id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Client not found.".into()));
    }

    get_by_id(pool, &client.id)
        .await?
        .ok_or_else(|| AppError::Internal("Updated client could not be reloaded.".into()))
}

/// Updates archive state for a client (archive or restore).
pub async fn set_archived(
    pool: &SqlitePool,
    id: &str,
    is_archived: bool,
    archived_at: Option<&str>,
    updated_at: &str,
) -> AppResult<Client> {
    let result = sqlx::query(
        r#"
        UPDATE clients
        SET is_archived = ?1,
            archived_at = ?2,
            updated_at = ?3
        WHERE id = ?4
        "#,
    )
    .bind(if is_archived { 1 } else { 0 })
    .bind(archived_at)
    .bind(updated_at)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Client not found.".into()));
    }

    get_by_id(pool, id)
        .await?
        .ok_or_else(|| AppError::Internal("Updated client could not be reloaded.".into()))
}

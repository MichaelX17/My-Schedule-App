use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Persisted client record.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct Client {
    pub id: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub is_archived: bool,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Input payload for creating a new client.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateClientInput {
    pub full_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
}

/// Input payload for updating an existing client.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateClientInput {
    pub id: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
}

/// Archive visibility filter for listing clients.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientArchiveFilter {
    #[default]
    Active,
    Archived,
    All,
}

/// Sort order options for listing clients.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClientSortBy {
    #[default]
    NameAsc,
    NameDesc,
    RecentlyUpdated,
    Newest,
}

/// Query parameters for listing/searching clients.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListClientsParams {
    pub search: Option<String>,
    pub filter: Option<ClientArchiveFilter>,
    pub sort_by: Option<ClientSortBy>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Paginated client list response including active and archived totals.
#[derive(Debug, Clone, Serialize)]
pub struct ClientListResult {
    pub items: Vec<Client>,
    pub total: i64,
    pub active_count: i64,
    pub archived_count: i64,
}

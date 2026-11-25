use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use my_schedule_app_lib::database;
use my_schedule_app_lib::errors::AppError;
use my_schedule_app_lib::models::client::{
    ClientArchiveFilter, ClientSortBy, CreateClientInput, ListClientsParams, UpdateClientInput,
};
use my_schedule_app_lib::platform;
use my_schedule_app_lib::services::client_service;

static COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "my_schedule_app_clients_{label}_{}_{}_{}",
            std::process::id(),
            nanos,
            seq
        ));
        fs::create_dir_all(&dir).expect("create temp dir");
        Self { path: dir }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[tokio::test]
async fn client_validation_and_international_phone_support() {
    let temp = TestDir::new("validation");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = &conn.require_pool().expect("pool");

    // Blank name rejected
    let err = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "   ".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect_err("blank name must fail");
    assert!(matches!(err, AppError::Validation(_)));

    // Invalid email rejected
    let err = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Ana Perez".into(),
            phone: None,
            email: Some("invalid-email".into()),
            address: None,
            notes: None,
        },
    )
    .await
    .expect_err("invalid email must fail");
    assert!(matches!(err, AppError::Validation(_)));

    // International phone numbers and duplicate names allowed
    let c1 = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "  Maria Garcia  ".into(),
            phone: Some("+34 600 123 456".into()),
            email: Some("  ".into()),
            address: Some("".into()),
            notes: Some("Prefers morning sessions.".into()),
        },
    )
    .await
    .expect("create c1");

    assert_eq!(c1.full_name, "Maria Garcia");
    assert_eq!(c1.phone.as_deref(), Some("+34 600 123 456"));
    assert_eq!(c1.email, None);
    assert_eq!(c1.address, None);
    assert!(!c1.is_archived);
    assert!(c1.archived_at.is_none());

    let c2 = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Maria Garcia".into(),
            phone: Some("+81 3-1234-5678".into()),
            email: Some("maria.jp@example.jp".into()),
            address: None,
            notes: None,
        },
    )
    .await
    .expect("duplicate names should be permitted");
    assert_ne!(c1.id, c2.id);

    conn.close().await;
}

#[tokio::test]
async fn client_crud_archive_restore_and_persistence_across_restart() {
    let temp = TestDir::new("crud_restart");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn1, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool1 = &conn1.require_pool().expect("pool1");

    let created = client_service::create_client(
        pool1,
        CreateClientInput {
            full_name: "Carlos Ruiz".into(),
            phone: Some("+1 (555) 234-5678".into()),
            email: Some("carlos@example.com".into()),
            address: Some("42 Oak Ave".into()),
            notes: Some("Initial consultation notes".into()),
        },
    )
    .await
    .expect("create client");

    let updated = client_service::update_client(
        pool1,
        UpdateClientInput {
            id: created.id.clone(),
            full_name: "Carlos Ruiz Vega".into(),
            phone: Some("+1 (555) 999-0000".into()),
            email: Some("carlos.vega@example.com".into()),
            address: None,
            notes: Some("Updated notes".into()),
        },
    )
    .await
    .expect("update client");

    assert_eq!(updated.full_name, "Carlos Ruiz Vega");
    assert_eq!(updated.address, None);
    assert_eq!(updated.notes.as_deref(), Some("Updated notes"));

    let archived = client_service::archive_client(pool1, &created.id)
        .await
        .expect("archive client");
    assert!(archived.is_archived);
    assert!(archived.archived_at.is_some());

    // Default active list excludes archived client
    let active_list = client_service::list_clients(pool1, ListClientsParams::default())
        .await
        .expect("list active");
    assert_eq!(active_list.items.len(), 0);
    assert_eq!(active_list.active_count, 0);
    assert_eq!(active_list.archived_count, 1);

    // Archived client remains accessible by ID for historical records
    let fetched = client_service::get_client(pool1, &created.id)
        .await
        .expect("get archived client by id");
    assert!(fetched.is_archived);

    conn1.close().await;

    // Reopen database (simulate restart)
    let (conn2, _) = database::initialize(&paths.database_path)
        .await
        .expect("reopen db");
    let pool2 = &conn2.require_pool().expect("pool2");

    let archived_list = client_service::list_clients(
        pool2,
        ListClientsParams {
            filter: Some(ClientArchiveFilter::Archived),
            ..Default::default()
        },
    )
    .await
    .expect("list archived after restart");
    assert_eq!(archived_list.items.len(), 1);
    assert_eq!(archived_list.items[0].full_name, "Carlos Ruiz Vega");

    let restored = client_service::restore_client(pool2, &created.id)
        .await
        .expect("restore client");
    assert!(!restored.is_archived);
    assert!(restored.archived_at.is_none());

    let missing_err = client_service::get_client(pool2, "nonexistent-id")
        .await
        .expect_err("missing id must return NotFound");
    assert!(matches!(missing_err, AppError::NotFound(_)));

    conn2.close().await;
}

#[tokio::test]
async fn client_search_sorting_and_pagination() {
    let temp = TestDir::new("search_sort");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");

    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = &conn.require_pool().expect("pool");

    for (name, email, notes) in [
        ("Zara Kim", Some("zara@example.com"), Some("VIP 100%")),
        ("Alice Brown", Some("alice@example.com"), None),
        ("Bruno Diaz", None, Some("Referred by friend")),
    ] {
        client_service::create_client(
            pool,
            CreateClientInput {
                full_name: name.into(),
                phone: None,
                email: email.map(str::to_string),
                address: None,
                notes: notes.map(str::to_string),
            },
        )
        .await
        .expect("seed client");
    }

    // NameAsc with pagination
    let page1 = client_service::list_clients(
        pool,
        ListClientsParams {
            sort_by: Some(ClientSortBy::NameAsc),
            limit: Some(2),
            offset: Some(0),
            ..Default::default()
        },
    )
    .await
    .expect("page 1");
    assert_eq!(page1.total, 3);
    assert_eq!(page1.items.len(), 2);
    assert_eq!(page1.items[0].full_name, "Alice Brown");
    assert_eq!(page1.items[1].full_name, "Bruno Diaz");

    // NameDesc
    let desc = client_service::list_clients(
        pool,
        ListClientsParams {
            sort_by: Some(ClientSortBy::NameDesc),
            ..Default::default()
        },
    )
    .await
    .expect("desc");
    assert_eq!(desc.items[0].full_name, "Zara Kim");

    // Literal '%' search must only match the note containing "100%"
    let percent_search = client_service::list_clients(
        pool,
        ListClientsParams {
            search: Some("100%".into()),
            ..Default::default()
        },
    )
    .await
    .expect("percent search");
    assert_eq!(percent_search.total, 1);
    assert_eq!(percent_search.items[0].full_name, "Zara Kim");

    conn.close().await;
}

#[tokio::test]
async fn delete_client_requires_archived_state_and_removes_record() {
    let temp = TestDir::new("delete_archived_only");
    let paths = platform::AppPaths::from_root(temp.path());
    paths.ensure_directories().expect("create dirs");
    let (conn, _) = database::initialize(&paths.database_path)
        .await
        .expect("init db");
    let pool = &conn.require_pool().expect("pool");

    let client = client_service::create_client(
        pool,
        CreateClientInput {
            full_name: "Delete Candidate".into(),
            phone: None,
            email: None,
            address: None,
            notes: None,
        },
    )
    .await
    .expect("create client");

    // Active client cannot be deleted
    let active_err = client_service::delete_client(pool, &client.id)
        .await
        .expect_err("active client must not be deletable");
    assert!(matches!(active_err, AppError::Validation(_)));

    // Archive first -> now permanent deletion succeeds
    client_service::archive_client(pool, &client.id)
        .await
        .expect("archive client");
    client_service::delete_client(pool, &client.id)
        .await
        .expect("delete archived client");

    let not_found = client_service::get_client(pool, &client.id)
        .await
        .expect_err("deleted client must not exist");
    assert!(matches!(not_found, AppError::NotFound(_)));

    conn.close().await;
}

pub mod backup;
pub mod commands;
pub mod database;
pub mod errors;
pub mod export;
pub mod models;
pub mod notifications;
pub mod platform;
pub mod scheduler;
pub mod services;
pub mod time;
pub mod updater;

use std::time::Duration;

use chrono::Utc;
use database::connection::DatabaseConnection;
use notifications::TauriNotificationSender;
use platform::{ResolvedStorage, StorageMode};
use tauri::async_runtime::JoinHandle;
use tauri::{Emitter, Manager};
use tracing_subscriber::EnvFilter;

/// Application-level runtime configuration.
#[derive(Debug, Default, Clone)]
pub struct AppConfig {
    pub portable_mode: bool,
}

/// Central shared application state managed by Tauri.
#[allow(dead_code)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: AppConfig,
    pub storage: ResolvedStorage,
    pub restore_lock: tokio::sync::Mutex<()>,
    pub background_handles: Vec<JoinHandle<()>>,
}

impl AppState {
    fn new(db: DatabaseConnection, storage: ResolvedStorage) -> Self {
        let portable_mode = storage.mode == StorageMode::Portable;
        Self {
            db,
            config: AppConfig { portable_mode },
            storage,
            restore_lock: tokio::sync::Mutex::new(()),
            background_handles: Vec::new(),
        }
    }
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("my_schedule_app_lib=info,tauri=info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .try_init();
}

/// Configures and launches the Tauri desktop application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_logging();
    tracing::info!("Initializing My Schedule App");

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let fallback_dir = app.path().app_data_dir().ok();
            let resolved_storage =
                platform::resolve_runtime_storage(fallback_dir).map_err(|e| {
                    tracing::error!(error = %e, "Failed to resolve writable storage directory");
                    e
                })?;

            resolved_storage.paths.ensure_directories().map_err(|e| {
                tracing::error!(error = %e, "Failed to create application data directories");
                e
            })?;

            tracing::info!(
                mode = ?resolved_storage.mode,
                "Storage paths resolved"
            );

            let (db, _health) = tauri::async_runtime::block_on(database::initialize(
                &resolved_storage.paths.database_path,
            ))
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to initialize local SQLite database");
                e
            })?;

            let mut state = AppState::new(db, resolved_storage);

            // Spawn the in-process startup backup + reminder scheduler (catch-up + 30s tick).
            {
                let db_handle = state.db.clone();
                let paths_handle = state.storage.paths.clone();
                let app_handle = app.handle().clone();
                let handle = tauri::async_runtime::spawn(async move {
                    if let Ok(pool) = db_handle.require_pool() {
                        let settings = services::settings_service::get_settings(&pool)
                            .await
                            .unwrap_or_default();
                        if settings.auto_backup_enabled {
                            if let Err(err) = backup::run_startup_automatic_backup(
                                &pool,
                                &paths_handle,
                                Some(settings.auto_backup_retention as usize),
                            )
                            .await
                            {
                                tracing::error!(error = %err, "Startup automatic backup failed");
                            }
                        }
                    }

                    let sender = TauriNotificationSender::new(app_handle.clone());
                    if let Ok(pool) = db_handle.require_pool() {
                        match scheduler::process_startup_catchup(&pool, Utc::now(), &sender).await {
                            Ok(summary) if summary.total_new_notifications() > 0 => {
                                let _ = app_handle.emit("reminders-updated", &summary);
                            }
                            Ok(_) => {}
                            Err(err) => {
                                tracing::error!(error = %err, "Startup reminder catch-up failed");
                            }
                        }
                    }

                    let mut ticker = tokio::time::interval(Duration::from_secs(30));
                    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                    // First tick completes immediately; consume it since startup catch-up just ran.
                    ticker.tick().await;

                    loop {
                        ticker.tick().await;
                        if let Ok(pool) = db_handle.require_pool() {
                            match scheduler::process_scheduler_tick(&pool, Utc::now(), &sender)
                                .await
                            {
                                Ok(summary) if summary.total_new_notifications() > 0 => {
                                    let _ = app_handle.emit("reminders-updated", &summary);
                                }
                                Ok(_) => {}
                                Err(err) => {
                                    tracing::error!(error = %err, "Periodic reminder scheduler tick failed");
                                }
                            }
                        }
                    }
                });
                state.background_handles.push(handle);
            }

            app.manage(state);
            tracing::info!("Application state initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::get_storage_info,
            commands::get_database_health,
            commands::business::get_onboarding_status,
            commands::business::get_business_profile,
            commands::business::create_business_profile,
            commands::business::update_business_profile,
            commands::business::upload_business_logo,
            commands::business::get_business_logo,
            commands::business::remove_business_logo,
            commands::clients::list_clients,
            commands::clients::get_client,
            commands::clients::create_client,
            commands::clients::update_client,
            commands::clients::archive_client,
            commands::clients::restore_client,
            commands::clients::delete_client,
            commands::services::list_services,
            commands::services::get_service,
            commands::services::create_service,
            commands::services::update_service,
            commands::services::activate_service,
            commands::services::deactivate_service,
            commands::services::archive_service,
            commands::services::restore_service,
            commands::services::delete_service,
            commands::session_plans::list_session_plans,
            commands::session_plans::get_session_plan,
            commands::session_plans::create_session_plan,
            commands::session_plans::create_session_plan_with_appointments,
            commands::session_plans::update_session_plan,
            commands::session_plans::archive_session_plan,
            commands::session_plans::delete_session_plan,
            commands::appointments::list_appointments,
            commands::appointments::get_appointment,
            commands::appointments::check_appointment_overlaps,
            commands::appointments::create_appointment,
            commands::appointments::update_appointment,
            commands::appointments::reschedule_appointment,
            commands::appointments::complete_appointment,
            commands::appointments::cancel_appointment,
            commands::appointments::delete_appointment,
            commands::calendar::get_calendar_range,
            commands::reminders::list_notifications,
            commands::reminders::get_unread_notification_count,
            commands::reminders::mark_notification_read,
            commands::reminders::mark_all_notifications_read,
            commands::reminders::dismiss_notification,
            commands::reminders::list_upcoming_reminders,
            commands::reminders::run_reminder_check_now,
            commands::reports::get_dashboard_summary,
            commands::reports::get_operational_report,
            commands::backups::export_business_json,
            commands::backups::export_entity_csv,
            commands::backups::list_backups,
            commands::backups::create_manual_backup,
            commands::backups::validate_backup,
            commands::backups::restore_backup,
            commands::settings::get_app_settings,
            commands::settings::update_app_settings,
            commands::updater::get_updater_info,
            commands::updater::check_for_updates,
            commands::updater::download_and_stage_update
        ]);

    if let Err(err) = builder.run(tauri::generate_context!()) {
        tracing::error!(error = %err, "Failed to run My Schedule App");
    }
}

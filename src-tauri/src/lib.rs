pub mod commands;
pub mod crypto;
pub mod db;
pub mod error;
pub mod keychain;
pub mod models;
pub mod parser;

use tauri::Manager;

use commands::AppState;
use crypto::{generate_master_key, MasterKey};
use keychain::Keychain;

fn init_master_key(
    keychain: &dyn Keychain,
    has_encrypted_data: bool,
) -> Result<MasterKey, String> {
    match keychain.get_master_key() {
        Ok(Some(key)) => Ok(key),
        Ok(None) if has_encrypted_data => Err(
            "Master key is missing from the OS keychain, but encrypted data exists. \
             Refusing to generate a new key because existing data would be unrecoverable."
                .to_string(),
        ),
        Ok(None) => {
            let key = generate_master_key();
            keychain
                .set_master_key(&key)
                .map_err(|e| format!("Failed to store master key in OS keychain: {e}"))?;
            Ok(key)
        }
        Err(e) => {
            if has_encrypted_data {
                Err(format!(
                    "OS keychain is unavailable and encrypted data exists: {e}"
                ))
            } else {
                Err(format!("OS keychain is unavailable: {e}"))
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to resolve app data directory: {e}"))?;
            let app_data_dir = if cfg!(debug_assertions) {
                app_data_dir.join("dev")
            } else {
                app_data_dir
            };
            let db_path = app_data_dir.join("envvault.db");

            let conn = match db::open(&db_path) {
                Ok(conn) => conn,
                Err(e) => {
                    app.manage(AppState::new(
                        rusqlite::Connection::open_in_memory().map_err(|e| e.to_string())?,
                        None,
                        Some(format!("Failed to open database: {e}")),
                    ));
                    return Ok(());
                }
            };

            let has_encrypted_data = db::has_encrypted_data(&conn).unwrap_or(false);
            let os_keychain = keychain::OsKeychain;
            let (master_key, init_error) = match init_master_key(&os_keychain, has_encrypted_data) {
                Ok(key) => (Some(key), None),
                Err(msg) => (None, Some(msg)),
            };

            app.manage(AppState::new(conn, master_key, init_error));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_init_status,
            commands::list_projects,
            commands::create_project,
            commands::rename_project,
            commands::delete_project,
            commands::list_environments,
            commands::create_environment,
            commands::rename_environment,
            commands::delete_environment,
            commands::duplicate_environment,
            commands::list_variables,
            commands::create_variable,
            commands::update_variable,
            commands::delete_variable,
            commands::reorder_variables,
            commands::reveal_variable,
            commands::preview_env_import,
            commands::import_env_text,
            commands::export_env_text,
            commands::export_env_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;
use zeroize::Zeroizing;

use crate::crypto::{Aes256GcmCrypto, MasterKey, ValueCrypto, KEY_LEN};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{Environment, EnvironmentSummary, Project, ProjectSummary, RevealedValue, Variable, VariableRow};

pub struct AppState {
    pub conn: Mutex<Connection>,
    pub master_key: Mutex<Option<MasterKey>>,
    pub init_error: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(conn: Connection, master_key: Option<MasterKey>, init_error: Option<String>) -> Self {
        Self {
            conn: Mutex::new(conn),
            master_key: Mutex::new(master_key),
            init_error: Mutex::new(init_error),
        }
    }

    fn ready_key(&self) -> AppResult<MasterKey> {
        if let Some(err) = self
            .init_error
            .lock()
            .map_err(|_| AppError::InvalidState("state lock poisoned".into()))?
            .clone()
        {
            return Err(AppError::InvalidState(err));
        }
        let guard = self
            .master_key
            .lock()
            .map_err(|_| AppError::InvalidState("state lock poisoned".into()))?;
        let key = guard
            .as_ref()
            .ok_or_else(|| AppError::InvalidState("Vault is locked: master key unavailable".into()))?;
        let mut arr = [0u8; KEY_LEN];
        arr.copy_from_slice(key.as_ref());
        Ok(Zeroizing::new(arr))
    }

    fn conn(&self) -> AppResult<std::sync::MutexGuard<'_, Connection>> {
        if let Some(err) = self
            .init_error
            .lock()
            .map_err(|_| AppError::InvalidState("state lock poisoned".into()))?
            .clone()
        {
            return Err(AppError::InvalidState(err));
        }
        self.conn
            .lock()
            .map_err(|_| AppError::InvalidState("database lock poisoned".into()))
    }
}

#[tauri::command]
pub fn get_init_status(state: State<'_, AppState>) -> AppResult<Option<String>> {
    let guard = state
        .init_error
        .lock()
        .map_err(|_| AppError::InvalidState("state lock poisoned".into()))?;
    Ok(guard.clone())
}

#[tauri::command]
pub fn list_projects(state: State<'_, AppState>) -> AppResult<Vec<ProjectSummary>> {
    let conn = state.conn()?;
    db::list_projects(&conn)
}

#[tauri::command]
pub fn create_project(state: State<'_, AppState>, name: String) -> AppResult<Project> {
    let conn = state.conn()?;
    db::create_project(&conn, &name)
}

#[tauri::command]
pub fn rename_project(state: State<'_, AppState>, id: String, name: String) -> AppResult<Project> {
    let conn = state.conn()?;
    db::rename_project(&conn, &id, &name)
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let conn = state.conn()?;
    db::delete_project(&conn, &id)
}

#[tauri::command]
pub fn list_environments(state: State<'_, AppState>, project_id: String) -> AppResult<Vec<EnvironmentSummary>> {
    let conn = state.conn()?;
    db::list_environments(&conn, &project_id)
}

#[tauri::command]
pub fn create_environment(state: State<'_, AppState>, project_id: String, name: String) -> AppResult<Environment> {
    let conn = state.conn()?;
    db::create_environment(&conn, &project_id, &name)
}

#[tauri::command]
pub fn rename_environment(state: State<'_, AppState>, id: String, name: String) -> AppResult<Environment> {
    let conn = state.conn()?;
    db::rename_environment(&conn, &id, &name)
}

#[tauri::command]
pub fn delete_environment(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let conn = state.conn()?;
    db::delete_environment(&conn, &id)
}

#[tauri::command]
pub fn duplicate_environment(state: State<'_, AppState>, source_id: String, new_name: String) -> AppResult<Environment> {
    let conn = state.conn()?;
    db::duplicate_environment(&conn, &source_id, &new_name)
}

#[tauri::command]
pub fn list_variables(state: State<'_, AppState>, environment_id: String) -> AppResult<Vec<VariableRow>> {
    let _ = state.ready_key()?;
    let conn = state.conn()?;
    db::list_variable_rows(&conn, &environment_id)
}

#[tauri::command]
pub fn create_variable(
    state: State<'_, AppState>,
    environment_id: String,
    key: String,
    value: String,
    comment: Option<String>,
    position: Option<i64>,
) -> AppResult<Variable> {
    let master = state.ready_key()?;
    let crypto = Aes256GcmCrypto;
    let (ciphertext, nonce) = crypto.encrypt(&master, value.as_bytes())?;
    let conn = state.conn()?;
    db::create_variable(
        &conn,
        &environment_id,
        &key,
        &ciphertext,
        &nonce,
        comment.as_deref(),
        position,
    )
}

#[tauri::command]
pub fn update_variable(
    state: State<'_, AppState>,
    id: String,
    key: String,
    value: Option<String>,
    comment: Option<String>,
) -> AppResult<Variable> {
    let conn = state.conn()?;
    if let Some(value) = value {
        let master = state.ready_key()?;
        let crypto = Aes256GcmCrypto;
        let (ciphertext, nonce) = crypto.encrypt(&master, value.as_bytes())?;
        db::update_variable(&conn, &id, &key, &ciphertext, &nonce, comment.as_deref())
    } else {
        db::update_variable_meta(&conn, &id, &key, comment.as_deref())
    }
}

#[tauri::command]
pub fn reorder_variables(
    state: State<'_, AppState>,
    environment_id: String,
    ordered_ids: Vec<String>,
) -> AppResult<()> {
    let _ = state.ready_key()?;
    let conn = state.conn()?;
    db::reorder_variables(&conn, &environment_id, &ordered_ids)
}

#[tauri::command]
pub fn delete_variable(state: State<'_, AppState>, id: String) -> AppResult<()> {
    let conn = state.conn()?;
    db::delete_variable(&conn, &id)
}

#[tauri::command]
pub fn reveal_variable(state: State<'_, AppState>, id: String) -> AppResult<RevealedValue> {
    let master = state.ready_key()?;
    let conn = state.conn()?;
    let record = db::get_variable_record(&conn, &id)?;
    let crypto = Aes256GcmCrypto;
    let nonce: [u8; crate::crypto::NONCE_LEN] = record
        .nonce
        .as_slice()
        .try_into()
        .map_err(|_| AppError::Crypto("invalid nonce length".into()))?;
    let plaintext = crypto.decrypt(&master, &record.value_encrypted, &nonce)?;
    let value = String::from_utf8(plaintext)
        .map_err(|_| AppError::Crypto("decrypted value is not valid UTF-8".into()))?;
    Ok(RevealedValue { id: record.id, value })
}

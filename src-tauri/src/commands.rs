use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;
use zeroize::Zeroizing;

use crate::crypto::{Aes256GcmCrypto, MasterKey, ValueCrypto, KEY_LEN};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{
    CompareRow, CompareStatus, Environment, EnvironmentComparison, EnvironmentSummary, ImportPreview,
    ImportPreviewRow, ImportStatus, Project, ProjectSummary, RevealedValue, Variable, VariableRow,
};
use crate::parser;

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

#[tauri::command]
pub fn preview_env_import(state: State<'_, AppState>, environment_id: String, text: String) -> AppResult<ImportPreview> {
    let _ = state.ready_key()?;
    let conn = state.conn()?;
    let existing = db::list_variable_keys(&conn, &environment_id)?;
    let parsed = parser::parse_env(&text);

    let mut rows = Vec::new();
    for v in &parsed.variables {
        let status = if existing.contains(&v.key) { ImportStatus::WillOverwrite } else { ImportStatus::New };
        rows.push(ImportPreviewRow {
            key: Some(v.key.clone()),
            masked_value: if v.value.is_empty() { String::new() } else { "••••••••".to_string() },
            status,
            line: v.line,
            message: None,
            comment: v.comment.clone(),
            value: Some(v.value.clone()),
        });
    }

    for d in &parsed.diagnostics {
        match d.level {
            parser::DiagnosticLevel::Error => rows.push(ImportPreviewRow {
                key: d.key.clone(),
                masked_value: String::new(),
                status: ImportStatus::Invalid,
                line: d.line,
                message: Some(d.message.clone()),
                comment: None,
                value: None,
            }),
            parser::DiagnosticLevel::Warning => {
                if let Some(key) = &d.key {
                    if let Some(row) = rows.iter_mut().find(|r| r.key.as_deref() == Some(key.as_str())) {
                        row.message = Some(d.message.clone());
                    }
                }
            }
        }
    }

    rows.sort_by_key(|r| r.line);
    Ok(ImportPreview { rows })
}

#[tauri::command]
pub fn import_env_text(state: State<'_, AppState>, environment_id: String, text: String) -> AppResult<usize> {
    let master = state.ready_key()?;
    let conn = state.conn()?;
    let parsed = parser::parse_env(&text);
    let crypto = Aes256GcmCrypto;

    let tx = conn.unchecked_transaction()?;
    let mut existing = std::collections::HashMap::new();
    {
        let mut stmt = tx.prepare("SELECT id, key FROM variables WHERE environment_id = ?1")?;
        let rows = stmt.query_map(rusqlite::params![environment_id], |row| {
            Ok((row.get::<_, String>(1)?, row.get::<_, String>(0)?))
        })?;
        for row in rows {
            let (key, id) = row?;
            existing.insert(key, id);
        }
    }

    let mut count = 0usize;
    for v in parsed.variables {
        let (ciphertext, nonce) = crypto.encrypt(&master, v.value.as_bytes())?;
        if let Some(id) = existing.get(&v.key) {
            db::update_variable(&tx, id, &v.key, &ciphertext, &nonce, v.comment.as_deref())?;
        } else {
            db::create_variable(&tx, &environment_id, &v.key, &ciphertext, &nonce, v.comment.as_deref(), None)?;
        }
        count += 1;
    }

    tx.commit()?;
    Ok(count)
}

#[tauri::command]
pub fn export_env_text(state: State<'_, AppState>, environment_id: String) -> AppResult<String> {
    let master = state.ready_key()?;
    let conn = state.conn()?;
    let mut stmt = conn.prepare(
        "SELECT key, value_encrypted, nonce, comment FROM variables WHERE environment_id = ?1 ORDER BY position ASC, key COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map(rusqlite::params![environment_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, Vec<u8>>(1)?,
            row.get::<_, Vec<u8>>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;
    let crypto = Aes256GcmCrypto;
    let mut vars = Vec::new();
    for row in rows {
        let (key, value_encrypted, nonce_bytes, comment) = row?;
        let nonce: [u8; crate::crypto::NONCE_LEN] = nonce_bytes
            .as_slice()
            .try_into()
            .map_err(|_| AppError::Crypto("invalid nonce length".into()))?;
        let plaintext = crypto.decrypt(&master, &value_encrypted, &nonce)?;
        let value = String::from_utf8(plaintext)
            .map_err(|_| AppError::Crypto("decrypted value is not valid UTF-8".into()))?;
        vars.push(parser::ExportVariable { key, value, comment });
    }
    Ok(parser::serialize_env(&vars))
}

#[tauri::command]
pub fn export_env_file(state: State<'_, AppState>, environment_id: String, path: String) -> AppResult<()> {
    let content = export_env_text(state, environment_id)?;
    std::fs::write(&path, content).map_err(|e| AppError::InvalidState(format!("Failed to write file: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| AppError::InvalidState(format!("Failed to set file permissions: {e}")))?;
    }
    Ok(())
}

#[tauri::command]
pub fn compare_environments(
    state: State<'_, AppState>,
    left_environment_id: String,
    right_environment_id: String,
) -> AppResult<EnvironmentComparison> {
    if left_environment_id == right_environment_id {
        return Err(AppError::Validation(
            "select two different environments to compare".into(),
        ));
    }
    let master = state.ready_key()?;
    let conn = state.conn()?;
    let left = db::list_variable_records(&conn, &left_environment_id)?;
    let right = db::list_variable_records(&conn, &right_environment_id)?;
    let crypto = Aes256GcmCrypto;

    let decrypt = |rec: &crate::db::VariableRecord| -> AppResult<Vec<u8>> {
        let nonce: [u8; crate::crypto::NONCE_LEN] = rec
            .nonce
            .as_slice()
            .try_into()
            .map_err(|_| AppError::Crypto("invalid nonce length".into()))?;
        crypto.decrypt(&master, &rec.value_encrypted, &nonce)
    };
    let mask = |rec: &crate::db::VariableRecord| -> String {
        if rec.value_encrypted.is_empty() {
            String::new()
        } else {
            "••••••••".to_string()
        }
    };

    let mut right_map: std::collections::BTreeMap<String, &crate::db::VariableRecord> =
        std::collections::BTreeMap::new();
    for rec in &right {
        right_map.insert(rec.key.clone(), rec);
    }

    let mut rows = Vec::new();
    let mut seen_right = std::collections::HashSet::new();
    for l in &left {
        if let Some(r) = right_map.get(&l.key) {
            seen_right.insert(l.key.as_str());
            let status = if decrypt(l)? == decrypt(r)? {
                CompareStatus::Same
            } else {
                CompareStatus::Different
            };
            rows.push(CompareRow {
                key: l.key.clone(),
                status,
                left_masked: Some(mask(l)),
                right_masked: Some(mask(r)),
                left_variable_id: Some(l.id.clone()),
                right_variable_id: Some(r.id.clone()),
            });
        } else {
            rows.push(CompareRow {
                key: l.key.clone(),
                status: CompareStatus::OnlyLeft,
                left_masked: Some(mask(l)),
                right_masked: None,
                left_variable_id: Some(l.id.clone()),
                right_variable_id: None,
            });
        }
    }
    for r in &right {
        if seen_right.contains(r.key.as_str()) {
            continue;
        }
        rows.push(CompareRow {
            key: r.key.clone(),
            status: CompareStatus::OnlyRight,
            left_masked: None,
            right_masked: Some(mask(r)),
            left_variable_id: None,
            right_variable_id: Some(r.id.clone()),
        });
    }
    rows.sort_by(|a, b| a.key.to_lowercase().cmp(&b.key.to_lowercase()).then(a.key.cmp(&b.key)));

    Ok(EnvironmentComparison {
        left_environment_id,
        right_environment_id,
        rows,
    })
}

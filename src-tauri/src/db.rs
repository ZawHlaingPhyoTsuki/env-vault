use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Environment, EnvironmentSummary, Project, ProjectSummary, Variable, VariableRow};

const SCHEMA_VERSION: &str = "1";

pub fn open_in_memory() -> AppResult<Connection> {
    let conn = Connection::open_in_memory()?;
    init(&conn)?;
    Ok(conn)
}

pub fn open(path: &std::path::Path) -> AppResult<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::Database(format!("failed to create app data directory: {e}"))
        })?;
    }
    let conn = Connection::open(path)?;
    init(&conn)?;
    Ok(conn)
}

fn init(conn: &Connection) -> AppResult<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrate(conn)?;
    Ok(())
}

fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )?;

    let version: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .optional()?;

    match version.as_deref() {
        None => apply_v1(conn)?,
        Some(SCHEMA_VERSION) => {}
        Some(other) => {
            return Err(AppError::Migration(format!(
                "unsupported schema version: {other}"
            )));
        }
    }

    Ok(())
}

fn apply_v1(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS environments (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(project_id, name)
        );

        CREATE TABLE IF NOT EXISTS variables (
            id TEXT PRIMARY KEY,
            environment_id TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
            key TEXT NOT NULL,
            value_encrypted BLOB NOT NULL,
            nonce BLOB NOT NULL,
            comment TEXT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(environment_id, key)
        );

        CREATE INDEX IF NOT EXISTS idx_environments_project_id ON environments(project_id);
        CREATE INDEX IF NOT EXISTS idx_variables_environment_id ON variables(environment_id);
        ",
    )?;
    conn.execute(
        "INSERT INTO meta (key, value) VALUES ('schema_version', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![SCHEMA_VERSION],
    )?;
    Ok(())
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn new_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn has_encrypted_data(conn: &Connection) -> AppResult<bool> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM variables", [], |row| row.get(0))?;
    Ok(count > 0)
}

// Projects

pub fn create_project(conn: &Connection, name: &str) -> AppResult<Project> {
    let name = validate_name(name, "Project name")?;
    let ts = now();
    let project = Project {
        id: new_id(),
        name: name.to_string(),
        created_at: ts.clone(),
        updated_at: ts,
    };
    conn.execute(
        "INSERT INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![project.id, project.name, project.created_at, project.updated_at],
    )?;
    Ok(project)
}

pub fn list_projects(conn: &Connection) -> AppResult<Vec<ProjectSummary>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.created_at, p.updated_at,
                COUNT(DISTINCT e.id) AS environment_count,
                COUNT(v.id) AS variable_count
         FROM projects p
         LEFT JOIN environments e ON e.project_id = p.id
         LEFT JOIN variables v ON v.environment_id = e.id
         GROUP BY p.id
         ORDER BY p.name COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ProjectSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
            environment_count: row.get(4)?,
            variable_count: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn rename_project(conn: &Connection, id: &str, name: &str) -> AppResult<Project> {
    let name = validate_name(name, "Project name")?;
    let ts = now();
    let changed = conn.execute(
        "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, ts, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound("project not found".into()));
    }
    get_project(conn, id)
}

pub fn get_project(conn: &Connection, id: &str) -> AppResult<Project> {
    conn.query_row(
        "SELECT id, name, created_at, updated_at FROM projects WHERE id = ?1",
        params![id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound("project not found".into()))
}

pub fn delete_project(conn: &Connection, id: &str) -> AppResult<()> {
    let changed = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(AppError::NotFound("project not found".into()));
    }
    Ok(())
}

// Environments

pub fn create_environment(conn: &Connection, project_id: &str, name: &str) -> AppResult<Environment> {
    let name = validate_name(name, "Environment name")?;
    ensure_project_exists(conn, project_id)?;
    let ts = now();
    let env = Environment {
        id: new_id(),
        project_id: project_id.to_string(),
        name: name.to_string(),
        created_at: ts.clone(),
        updated_at: ts,
    };
    conn.execute(
        "INSERT INTO environments (id, project_id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![env.id, env.project_id, env.name, env.created_at, env.updated_at],
    )?;
    Ok(env)
}

pub fn list_environments(conn: &Connection, project_id: &str) -> AppResult<Vec<EnvironmentSummary>> {
    ensure_project_exists(conn, project_id)?;
    let mut stmt = conn.prepare(
        "SELECT e.id, e.project_id, e.name, e.created_at, e.updated_at,
                COUNT(v.id) AS variable_count
         FROM environments e
         LEFT JOIN variables v ON v.environment_id = e.id
         WHERE e.project_id = ?1
         GROUP BY e.id
         ORDER BY e.name COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map(params![project_id], |row| {
        Ok(EnvironmentSummary {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
            variable_count: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn rename_environment(conn: &Connection, id: &str, name: &str) -> AppResult<Environment> {
    let name = validate_name(name, "Environment name")?;
    let ts = now();
    let changed = conn.execute(
        "UPDATE environments SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, ts, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound("environment not found".into()));
    }
    get_environment(conn, id)
}

pub fn get_environment(conn: &Connection, id: &str) -> AppResult<Environment> {
    conn.query_row(
        "SELECT id, project_id, name, created_at, updated_at FROM environments WHERE id = ?1",
        params![id],
        |row| {
            Ok(Environment {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound("environment not found".into()))
}

pub fn delete_environment(conn: &Connection, id: &str) -> AppResult<()> {
    let changed = conn.execute("DELETE FROM environments WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(AppError::NotFound("environment not found".into()));
    }
    Ok(())
}

pub fn duplicate_environment(conn: &Connection, source_id: &str, new_name: &str) -> AppResult<Environment> {
    let source = get_environment(conn, source_id)?;
    let tx = conn.unchecked_transaction()?;
    let target = create_environment(&tx, &source.project_id, new_name)?;
    let mut stmt = tx.prepare(
        "SELECT key, value_encrypted, nonce, comment, position, created_at, updated_at
         FROM variables WHERE environment_id = ?1 ORDER BY position ASC",
    )?;
    let mut rows = stmt.query(params![source_id])?;
    let ts = now();
    while let Some(row) = rows.next()? {
        let key: String = row.get(0)?;
        let value_encrypted: Vec<u8> = row.get(1)?;
        let nonce: Vec<u8> = row.get(2)?;
        let comment: Option<String> = row.get(3)?;
        let position: i64 = row.get(4)?;
        tx.execute(
            "INSERT INTO variables (id, environment_id, key, value_encrypted, nonce, comment, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![new_id(), target.id, key, value_encrypted, nonce, comment, position, ts, ts],
        )?;
    }
    drop(rows);
    drop(stmt);
    tx.commit()?;
    Ok(target)
}

// Variables

#[derive(Debug, Clone)]
pub struct VariableRecord {
    pub id: String,
    pub environment_id: String,
    pub key: String,
    pub value_encrypted: Vec<u8>,
    pub nonce: Vec<u8>,
    pub comment: Option<String>,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

pub fn list_variable_keys(conn: &Connection, environment_id: &str) -> AppResult<std::collections::HashSet<String>> {
    ensure_environment_exists(conn, environment_id)?;
    let mut stmt = conn.prepare("SELECT key FROM variables WHERE environment_id = ?1")?;
    let rows = stmt.query_map(params![environment_id], |row| row.get::<_, String>(0))?;
    let mut set = std::collections::HashSet::new();
    for row in rows {
        set.insert(row?);
    }
    Ok(set)
}

pub fn list_variable_rows(conn: &Connection, environment_id: &str) -> AppResult<Vec<VariableRow>> {
    ensure_environment_exists(conn, environment_id)?;
    let mut stmt = conn.prepare(
        "SELECT id, environment_id, key, comment, position, created_at, updated_at,
                length(value_encrypted) > 0 AS has_value
         FROM variables WHERE environment_id = ?1 ORDER BY position ASC, key COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map(params![environment_id], |row| {
        let has_value: bool = row.get(7)?;
        Ok(VariableRow {
            id: row.get(0)?,
            environment_id: row.get(1)?,
            key: row.get(2)?,
            masked_value: if has_value { "••••••••".to_string() } else { String::new() },
            has_value,
            comment: row.get(3)?,
            position: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn get_variable_record(conn: &Connection, id: &str) -> AppResult<VariableRecord> {
    conn.query_row(
        "SELECT id, environment_id, key, value_encrypted, nonce, comment, position, created_at, updated_at
         FROM variables WHERE id = ?1",
        params![id],
        |row| {
            Ok(VariableRecord {
                id: row.get(0)?,
                environment_id: row.get(1)?,
                key: row.get(2)?,
                value_encrypted: row.get(3)?,
                nonce: row.get(4)?,
                comment: row.get(5)?,
                position: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound("variable not found".into()))
}

pub fn list_variable_records(conn: &Connection, environment_id: &str) -> AppResult<Vec<VariableRecord>> {
    ensure_environment_exists(conn, environment_id)?;
    let mut stmt = conn.prepare(
        "SELECT id, environment_id, key, value_encrypted, nonce, comment, position, created_at, updated_at
         FROM variables WHERE environment_id = ?1 ORDER BY position ASC, key COLLATE NOCASE ASC",
    )?;
    let rows = stmt.query_map(params![environment_id], |row| {
        Ok(VariableRecord {
            id: row.get(0)?,
            environment_id: row.get(1)?,
            key: row.get(2)?,
            value_encrypted: row.get(3)?,
            nonce: row.get(4)?,
            comment: row.get(5)?,
            position: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn create_variable(
    conn: &Connection,
    environment_id: &str,
    key: &str,
    value_encrypted: &[u8],
    nonce: &[u8],
    comment: Option<&str>,
    position: Option<i64>,
) -> AppResult<Variable> {
    ensure_environment_exists(conn, environment_id)?;
    let key = validate_variable_key(key)?;
    let ts = now();
    let position = position.unwrap_or(next_variable_position(conn, environment_id)?);
    let variable = Variable {
        id: new_id(),
        environment_id: environment_id.to_string(),
        key: key.to_string(),
        comment: normalize_comment(comment),
        position,
        created_at: ts.clone(),
        updated_at: ts.clone(),
    };
    conn.execute(
        "INSERT INTO variables (id, environment_id, key, value_encrypted, nonce, comment, position, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            variable.id,
            variable.environment_id,
            variable.key,
            value_encrypted,
            nonce,
            variable.comment,
            variable.position,
            variable.created_at,
            variable.updated_at
        ],
    )?;
    Ok(variable)
}

pub fn update_variable(
    conn: &Connection,
    id: &str,
    key: &str,
    value_encrypted: &[u8],
    nonce: &[u8],
    comment: Option<&str>,
) -> AppResult<Variable> {
    let key = validate_variable_key(key)?;
    let ts = now();
    let changed = conn.execute(
        "UPDATE variables SET key = ?1, value_encrypted = ?2, nonce = ?3, comment = ?4, updated_at = ?5 WHERE id = ?6",
        params![key, value_encrypted, nonce, normalize_comment(comment), ts, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound("variable not found".into()));
    }
    get_variable(conn, id)
}

pub fn delete_variable(conn: &Connection, id: &str) -> AppResult<()> {
    let changed = conn.execute("DELETE FROM variables WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(AppError::NotFound("variable not found".into()));
    }
    Ok(())
}

pub fn update_variable_meta(
    conn: &Connection,
    id: &str,
    key: &str,
    comment: Option<&str>,
) -> AppResult<Variable> {
    let key = validate_variable_key(key)?;
    let ts = now();
    let changed = conn.execute(
        "UPDATE variables SET key = ?1, comment = ?2, updated_at = ?3 WHERE id = ?4",
        params![key, normalize_comment(comment), ts, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound("variable not found".into()));
    }
    get_variable(conn, id)
}

pub fn reorder_variables(conn: &Connection, environment_id: &str, ordered_ids: &[String]) -> AppResult<()> {
    ensure_environment_exists(conn, environment_id)?;
    let tx = conn.unchecked_transaction()?;
    for (idx, id) in ordered_ids.iter().enumerate() {
        let changed = tx.execute(
            "UPDATE variables SET position = ?1, updated_at = ?2 WHERE id = ?3 AND environment_id = ?4",
            params![idx as i64, now(), id, environment_id],
        )?;
        if changed == 0 {
            return Err(AppError::NotFound(format!("variable not found: {id}")));
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn get_variable(conn: &Connection, id: &str) -> AppResult<Variable> {
    conn.query_row(
        "SELECT id, environment_id, key, comment, position, created_at, updated_at FROM variables WHERE id = ?1",
        params![id],
        |row| {
            Ok(Variable {
                id: row.get(0)?,
                environment_id: row.get(1)?,
                key: row.get(2)?,
                comment: row.get(3)?,
                position: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    )
    .optional()?
    .ok_or_else(|| AppError::NotFound("variable not found".into()))
}

fn next_variable_position(conn: &Connection, environment_id: &str) -> AppResult<i64> {
    let max: Option<i64> = conn.query_row(
        "SELECT MAX(position) FROM variables WHERE environment_id = ?1",
        params![environment_id],
        |row| row.get(0),
    )?;
    Ok(max.unwrap_or(-1) + 1)
}

fn ensure_project_exists(conn: &Connection, id: &str) -> AppResult<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = ?1)",
        params![id],
        |row| row.get(0),
    )?;
    if exists { Ok(()) } else { Err(AppError::NotFound("project not found".into())) }
}

fn ensure_environment_exists(conn: &Connection, id: &str) -> AppResult<()> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM environments WHERE id = ?1)",
        params![id],
        |row| row.get(0),
    )?;
    if exists { Ok(()) } else { Err(AppError::NotFound("environment not found".into())) }
}

fn validate_name<'a>(name: &'a str, label: &str) -> AppResult<&'a str> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation(format!("{label} cannot be empty")));
    }
    Ok(name)
}

fn validate_variable_key(key: &str) -> AppResult<&str> {
    let key = key.trim();
    let valid = !key.is_empty()
        && key
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !valid {
        return Err(AppError::Validation(
            "Variable key must match [A-Za-z_][A-Za-z0-9_]*".into(),
        ));
    }
    Ok(key)
}

fn normalize_comment(comment: Option<&str>) -> Option<String> {
    comment
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;

    fn sample_value() -> (Vec<u8>, Vec<u8>) {
        (vec![1, 2, 3, 4], vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 11, 12])
    }

    #[test]
    fn migration_creates_schema() {
        let conn = open_in_memory().unwrap();
        let version: String = conn
            .query_row("SELECT value FROM meta WHERE key='schema_version'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn project_crud_and_uniqueness() {
        let conn = open_in_memory().unwrap();
        let p = create_project(&conn, "My Project").unwrap();
        assert_eq!(p.name, "My Project");
        let list = list_projects(&conn).unwrap();
        assert_eq!(list.len(), 1);
        let renamed = rename_project(&conn, &p.id, "Renamed").unwrap();
        assert_eq!(renamed.name, "Renamed");
        let err = create_project(&conn, "Renamed").unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
        delete_project(&conn, &p.id).unwrap();
        assert!(list_projects(&conn).unwrap().is_empty());
    }

    #[test]
    fn environment_crud_uniqueness_and_duplicate() {
        let conn = open_in_memory().unwrap();
        let p = create_project(&conn, "P").unwrap();
        let e1 = create_environment(&conn, &p.id, "dev").unwrap();
        let e2 = create_environment(&conn, &p.id, "stage").unwrap();
        assert_eq!(list_environments(&conn, &p.id).unwrap().len(), 2);
        let err = create_environment(&conn, &p.id, "dev").unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
        let (ct, nonce) = sample_value();
        create_variable(&conn, &e1.id, "API_KEY", &ct, &nonce, Some("comment"), None).unwrap();
        let dup = duplicate_environment(&conn, &e1.id, "dev-copy").unwrap();
        let dup_rows = list_variable_rows(&conn, &dup.id).unwrap();
        assert_eq!(dup_rows.len(), 1);
        assert_eq!(dup_rows[0].key, "API_KEY");
        delete_environment(&conn, &e2.id).unwrap();
        assert_eq!(list_environments(&conn, &p.id).unwrap().len(), 2);
        let _ = dup;
    }

    #[test]
    fn variable_crud_and_uniqueness() {
        let conn = open_in_memory().unwrap();
        let p = create_project(&conn, "P").unwrap();
        let e = create_environment(&conn, &p.id, "dev").unwrap();
        let (ct, nonce) = sample_value();
        let v = create_variable(&conn, &e.id, "FOO_BAR", &ct, &nonce, None, None).unwrap();
        let rows = list_variable_rows(&conn, &e.id).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].masked_value, "••••••••");
        let err = create_variable(&conn, &e.id, "FOO_BAR", &ct, &nonce, None, None).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));
        let updated = update_variable(&conn, &v.id, "FOO_BAZ", &ct, &nonce, Some("c")).unwrap();
        assert_eq!(updated.key, "FOO_BAZ");
        delete_variable(&conn, &v.id).unwrap();
        assert!(list_variable_rows(&conn, &e.id).unwrap().is_empty());
    }

    #[test]
    fn cascade_delete_project() {
        let conn = open_in_memory().unwrap();
        let p = create_project(&conn, "P").unwrap();
        let e = create_environment(&conn, &p.id, "dev").unwrap();
        let (ct, nonce) = sample_value();
        create_variable(&conn, &e.id, "A", &ct, &nonce, None, None).unwrap();
        delete_project(&conn, &p.id).unwrap();
        let env_count: i64 = conn.query_row("SELECT COUNT(*) FROM environments", [], |r| r.get(0)).unwrap();
        let var_count: i64 = conn.query_row("SELECT COUNT(*) FROM variables", [], |r| r.get(0)).unwrap();
        assert_eq!(env_count, 0);
        assert_eq!(var_count, 0);
    }

    #[test]
    fn variable_key_validation() {
        let conn = open_in_memory().unwrap();
        let p = create_project(&conn, "P").unwrap();
        let e = create_environment(&conn, &p.id, "dev").unwrap();
        let (ct, nonce) = sample_value();
        assert!(create_variable(&conn, &e.id, "BAD-KEY", &ct, &nonce, None, None).is_err());
        assert!(create_variable(&conn, &e.id, "1BAD", &ct, &nonce, None, None).is_err());
        assert!(create_variable(&conn, &e.id, "GOOD_KEY_1", &ct, &nonce, None, None).is_ok());
    }
}

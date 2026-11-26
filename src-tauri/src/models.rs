use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub environment_count: i64,
    pub variable_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSummary {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub variable_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub id: String,
    pub environment_id: String,
    pub key: String,
    pub comment: Option<String>,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableRow {
    pub id: String,
    pub environment_id: String,
    pub key: String,
    pub masked_value: String,
    pub has_value: bool,
    pub comment: Option<String>,
    pub position: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevealedValue {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportStatus {
    New,
    WillOverwrite,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreviewRow {
    pub key: Option<String>,
    pub masked_value: String,
    pub status: ImportStatus,
    pub line: usize,
    pub message: Option<String>,
    pub comment: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub rows: Vec<ImportPreviewRow>,
}

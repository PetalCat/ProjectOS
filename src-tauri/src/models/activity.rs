use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivityEntry {
    pub id: i64,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub action: String,
    pub detail: Option<String>,
    pub created_at: i64,
}

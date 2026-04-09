use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabel {
    pub name: String,
    pub color: String,
    pub project_id: Option<String>,
}

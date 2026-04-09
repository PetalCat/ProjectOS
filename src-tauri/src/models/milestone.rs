use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Milestone {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<i64>,
    pub state: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub open_count: Option<i64>,
    pub closed_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMilestone {
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<i64>,
}

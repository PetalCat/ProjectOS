use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issue {
    pub id: String,
    pub project_id: Option<String>,
    pub number: Option<i64>,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub status: Option<String>,
    pub sort_order: f64,
    pub context: Option<String>,
    pub machine_id: Option<String>,
    pub milestone_id: Option<String>,
    pub locked: bool,
    pub pinned: bool,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub closed_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIssue {
    pub project_id: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub status: Option<String>,
    pub machine_id: Option<String>,
    pub milestone_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIssue {
    pub id: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    pub context: Option<String>,
    pub machine_id: Option<String>,
    pub milestone_id: Option<String>,
}

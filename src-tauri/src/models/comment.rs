use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub issue_id: String,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateComment {
    pub issue_id: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateComment {
    pub id: String,
    pub body: String,
}

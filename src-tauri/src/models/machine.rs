use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Machine {
    pub id: String,
    pub name: String,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub user: Option<String>,
    pub os: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateMachine {
    pub name: String,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub user: Option<String>,
    pub os: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMachine {
    pub id: String,
    pub name: Option<String>,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub user: Option<String>,
    pub os: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineDoc {
    pub id: String,
    pub machine_id: String,
    pub title: String,
    pub content: Option<String>,
    pub url: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateMachineDoc {
    pub machine_id: String,
    pub title: String,
    pub content: Option<String>,
    pub url: Option<String>,
}

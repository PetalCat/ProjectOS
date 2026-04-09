use crate::models::label::{CreateLabel, Label};
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn create_label(state: State<AppState>, input: CreateLabel) -> Result<Label, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO labels (id, name, color, project_id) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, input.name, input.color, input.project_id],
    ).map_err(|e| e.to_string())?;
    Ok(Label { id, name: input.name, color: input.color, project_id: input.project_id })
}

#[tauri::command]
pub fn list_labels(state: State<AppState>, project_id: Option<String>) -> Result<Vec<Label>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    // Return global labels + project-specific labels
    let mut stmt = db.prepare(
        "SELECT id, name, color, project_id FROM labels WHERE project_id IS NULL OR project_id = ?1 ORDER BY name"
    ).map_err(|e| e.to_string())?;
    let labels = stmt.query_map([&project_id.unwrap_or_default()], |row| {
        Ok(Label { id: row.get(0)?, name: row.get(1)?, color: row.get(2)?, project_id: row.get(3)? })
    }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(labels)
}

#[tauri::command]
pub fn delete_label(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM labels WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn add_label_to_issue(state: State<AppState>, issue_id: String, label_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR IGNORE INTO issue_labels (issue_id, label_id) VALUES (?1, ?2)",
        rusqlite::params![issue_id, label_id],
    ).map_err(|e| e.to_string())?;

    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    let label_name: String = db.query_row("SELECT name FROM labels WHERE id = ?1", [&label_id], |r| r.get(0)).map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, (SELECT project_id FROM issues WHERE id = ?1), 'labeled', ?2, ?3)",
        rusqlite::params![issue_id, serde_json::json!({"label": label_name}).to_string(), now],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_label_from_issue(state: State<AppState>, issue_id: String, label_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM issue_labels WHERE issue_id = ?1 AND label_id = ?2",
        rusqlite::params![issue_id, label_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_issue_labels(state: State<AppState>, issue_id: String) -> Result<Vec<Label>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT l.id, l.name, l.color, l.project_id FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = ?1 ORDER BY l.name"
    ).map_err(|e| e.to_string())?;
    let labels = stmt.query_map([&issue_id], |row| {
        Ok(Label { id: row.get(0)?, name: row.get(1)?, color: row.get(2)?, project_id: row.get(3)? })
    }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(labels)
}

#[cfg(test)]
mod tests {
    use crate::db;
    use crate::state::AppState;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn test_state() -> AppState {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        db::schema::create_tables(&conn).unwrap();
        AppState { db: Mutex::new(conn) }
    }

    #[test]
    fn test_labels_and_issue_labels() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        db.execute("INSERT INTO labels (id, name, color, project_id) VALUES ('l1', 'bug', '#ff0000', NULL)", []).unwrap();
        db.execute("INSERT INTO issues (id, title, state, status, sort_order, locked, pinned, created_at, updated_at) VALUES ('i1', 'Test', 'open', 'idea', 1.0, 0, 0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issue_labels (issue_id, label_id) VALUES ('i1', 'l1')", []).unwrap();

        let label_name: String = db.query_row(
            "SELECT l.name FROM labels l JOIN issue_labels il ON l.id = il.label_id WHERE il.issue_id = 'i1'",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(label_name, "bug");
    }
}

use crate::models::activity::ActivityEntry;
use crate::state::AppState;
use tauri::State;

fn row_to_entry(row: &rusqlite::Row) -> Result<ActivityEntry, rusqlite::Error> {
    Ok(ActivityEntry {
        id: row.get(0)?,
        issue_id: row.get(1)?,
        project_id: row.get(2)?,
        action: row.get(3)?,
        detail: row.get(4)?,
        created_at: row.get(5)?,
    })
}

#[tauri::command]
pub fn get_activity_log(state: State<AppState>, project_id: Option<String>, limit: Option<i64>) -> Result<Vec<ActivityEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(50);

    if let Some(ref pid) = project_id {
        let mut stmt = db.prepare(
            "SELECT id, issue_id, project_id, action, detail, created_at FROM activity_log WHERE project_id = ?1 ORDER BY created_at DESC LIMIT ?2"
        ).map_err(|e| e.to_string())?;
        let entries: Vec<ActivityEntry> = stmt.query_map(rusqlite::params![pid, limit], row_to_entry)
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(entries)
    } else {
        let mut stmt = db.prepare(
            "SELECT id, issue_id, project_id, action, detail, created_at FROM activity_log ORDER BY created_at DESC LIMIT ?1"
        ).map_err(|e| e.to_string())?;
        let entries: Vec<ActivityEntry> = stmt.query_map(rusqlite::params![limit], row_to_entry)
            .map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(entries)
    }
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
    fn test_activity_log() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (NULL, NULL, 'created', '{\"title\":\"Test\"}', 1000)", []).unwrap();
        db.execute("INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (NULL, NULL, 'closed', NULL, 2000)", []).unwrap();

        let count: i64 = db.query_row("SELECT COUNT(*) FROM activity_log", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
    }
}

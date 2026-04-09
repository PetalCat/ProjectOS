use crate::models::milestone::{CreateMilestone, Milestone};
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
}

#[tauri::command]
pub fn create_milestone(state: State<AppState>, input: CreateMilestone) -> Result<Milestone, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    db.execute(
        "INSERT INTO milestones (id, project_id, title, description, due_date, state, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6, ?7)",
        rusqlite::params![id, input.project_id, input.title, input.description, input.due_date, now, now],
    ).map_err(|e| e.to_string())?;
    Ok(Milestone { id, project_id: input.project_id, title: input.title, description: input.description, due_date: input.due_date, state: "open".to_string(), created_at: now, updated_at: now, open_count: Some(0), closed_count: Some(0) })
}

#[tauri::command]
pub fn list_milestones(state: State<AppState>, project_id: String) -> Result<Vec<Milestone>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT m.id, m.project_id, m.title, m.description, m.due_date, m.state, m.created_at, m.updated_at,
                (SELECT COUNT(*) FROM issues WHERE milestone_id = m.id AND state = 'open') as open_count,
                (SELECT COUNT(*) FROM issues WHERE milestone_id = m.id AND state = 'closed') as closed_count
         FROM milestones m WHERE m.project_id = ?1 ORDER BY m.state ASC, m.created_at DESC"
    ).map_err(|e| e.to_string())?;
    let milestones = stmt.query_map([&project_id], |row| {
        Ok(Milestone {
            id: row.get(0)?,
            project_id: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            due_date: row.get(4)?,
            state: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            open_count: row.get(8)?,
            closed_count: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(milestones)
}

#[tauri::command]
pub fn close_milestone(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    db.execute("UPDATE milestones SET state = 'closed', updated_at = ?1 WHERE id = ?2", rusqlite::params![now, id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_milestone(state: State<AppState>, issue_id: String, milestone_id: Option<String>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    db.execute("UPDATE issues SET milestone_id = ?1, updated_at = ?2 WHERE id = ?3", rusqlite::params![milestone_id, now, issue_id]).map_err(|e| e.to_string())?;
    Ok(())
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
    fn test_milestone_with_issue_counts() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'Test', NULL, NULL, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO milestones (id, project_id, title, description, due_date, state, created_at, updated_at) VALUES ('m1', 'p1', 'v1', NULL, NULL, 'open', 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issues (id, project_id, number, title, state, status, sort_order, milestone_id, locked, pinned, created_at, updated_at) VALUES ('i1', 'p1', 1, 'Open', 'open', 'ready', 1.0, 'm1', 0, 0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issues (id, project_id, number, title, state, status, sort_order, milestone_id, locked, pinned, created_at, updated_at) VALUES ('i2', 'p1', 2, 'Closed', 'closed', NULL, 2.0, 'm1', 0, 0, 1000, 1000)", []).unwrap();

        let (open, closed): (i64, i64) = db.query_row(
            "SELECT (SELECT COUNT(*) FROM issues WHERE milestone_id = 'm1' AND state = 'open'), (SELECT COUNT(*) FROM issues WHERE milestone_id = 'm1' AND state = 'closed')",
            [], |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!(open, 1);
        assert_eq!(closed, 1);
    }
}

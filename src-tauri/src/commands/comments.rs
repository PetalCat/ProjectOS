use crate::models::comment::{Comment, CreateComment, UpdateComment};
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[tauri::command]
pub fn create_comment(state: State<AppState>, input: CreateComment) -> Result<Comment, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Check if issue is locked
    let locked: bool = db.query_row(
        "SELECT locked FROM issues WHERE id = ?1", [&input.issue_id], |row| row.get::<_, i64>(0).map(|v| v != 0),
    ).map_err(|e| e.to_string())?;

    if locked {
        return Err("Issue is locked".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    db.execute(
        "INSERT INTO issue_comments (id, issue_id, body, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, input.issue_id, input.body, now, now],
    ).map_err(|e| e.to_string())?;

    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, (SELECT project_id FROM issues WHERE id = ?1), 'commented', NULL, ?2)",
        rusqlite::params![input.issue_id, now],
    ).map_err(|e| e.to_string())?;

    Ok(Comment { id, issue_id: input.issue_id, body: input.body, created_at: now, updated_at: now })
}

#[tauri::command]
pub fn list_comments(state: State<AppState>, issue_id: String) -> Result<Vec<Comment>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT id, issue_id, body, created_at, updated_at FROM issue_comments WHERE issue_id = ?1 ORDER BY created_at ASC"
    ).map_err(|e| e.to_string())?;
    let comments = stmt.query_map([&issue_id], |row| {
        Ok(Comment {
            id: row.get(0)?,
            issue_id: row.get(1)?,
            body: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(comments)
}

#[tauri::command]
pub fn update_comment(state: State<AppState>, input: UpdateComment) -> Result<Comment, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    db.execute(
        "UPDATE issue_comments SET body = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![input.body, now, input.id],
    ).map_err(|e| e.to_string())?;

    db.query_row(
        "SELECT id, issue_id, body, created_at, updated_at FROM issue_comments WHERE id = ?1",
        [&input.id],
        |row| Ok(Comment { id: row.get(0)?, issue_id: row.get(1)?, body: row.get(2)?, created_at: row.get(3)?, updated_at: row.get(4)? }),
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_comment(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM issue_comments WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn add_reaction(state: State<AppState>, issue_id: Option<String>, comment_id: Option<String>, emoji: String) -> Result<(), String> {
    if issue_id.is_none() && comment_id.is_none() {
        return Err("Must provide issue_id or comment_id".to_string());
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    db.execute(
        "INSERT INTO issue_reactions (id, issue_id, comment_id, emoji, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, issue_id, comment_id, emoji, now],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_reaction(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM issue_reactions WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(serde::Serialize)]
pub struct ReactionGroup {
    pub emoji: String,
    pub count: i64,
    pub ids: Vec<String>,
}

#[tauri::command]
pub fn list_reactions(state: State<AppState>, issue_id: Option<String>, comment_id: Option<String>) -> Result<Vec<ReactionGroup>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let (sql, param) = if let Some(ref iid) = issue_id {
        ("SELECT id, emoji FROM issue_reactions WHERE issue_id = ?1 ORDER BY emoji", iid.clone())
    } else if let Some(ref cid) = comment_id {
        ("SELECT id, emoji FROM issue_reactions WHERE comment_id = ?1 ORDER BY emoji", cid.clone())
    } else {
        return Ok(vec![]);
    };

    let mut stmt = db.prepare(sql).map_err(|e| e.to_string())?;
    let rows: Vec<(String, String)> = stmt.query_map([&param], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut groups: std::collections::HashMap<String, ReactionGroup> = std::collections::HashMap::new();
    for (id, emoji) in rows {
        let group = groups.entry(emoji.clone()).or_insert(ReactionGroup { emoji, count: 0, ids: vec![] });
        group.count += 1;
        group.ids.push(id);
    }

    Ok(groups.into_values().collect())
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
    fn test_comments_crud() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO issues (id, title, state, status, sort_order, locked, pinned, created_at, updated_at) VALUES ('i1', 'Test', 'open', 'idea', 1.0, 0, 0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issue_comments (id, issue_id, body, created_at, updated_at) VALUES ('c1', 'i1', 'Hello', 1000, 1000)", []).unwrap();

        let body: String = db.query_row("SELECT body FROM issue_comments WHERE id = 'c1'", [], |r| r.get(0)).unwrap();
        assert_eq!(body, "Hello");

        db.execute("UPDATE issue_comments SET body = 'Updated' WHERE id = 'c1'", []).unwrap();
        let body: String = db.query_row("SELECT body FROM issue_comments WHERE id = 'c1'", [], |r| r.get(0)).unwrap();
        assert_eq!(body, "Updated");

        db.execute("DELETE FROM issue_comments WHERE id = 'c1'", []).unwrap();
        let count: i64 = db.query_row("SELECT COUNT(*) FROM issue_comments WHERE issue_id = 'i1'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_locked_issue_blocks_comments() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO issues (id, title, state, status, sort_order, locked, pinned, created_at, updated_at) VALUES ('i1', 'Test', 'open', 'idea', 1.0, 1, 0, 1000, 1000)", []).unwrap();

        let locked: bool = db.query_row("SELECT locked FROM issues WHERE id = 'i1'", [], |r| r.get::<_, i64>(0).map(|v| v != 0)).unwrap();
        assert!(locked);
    }

    #[test]
    fn test_reactions() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO issues (id, title, state, status, sort_order, locked, pinned, created_at, updated_at) VALUES ('i1', 'Test', 'open', 'idea', 1.0, 0, 0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issue_reactions (id, issue_id, comment_id, emoji, created_at) VALUES ('r1', 'i1', NULL, '👍', 1000)", []).unwrap();
        db.execute("INSERT INTO issue_reactions (id, issue_id, comment_id, emoji, created_at) VALUES ('r2', 'i1', NULL, '👍', 1001)", []).unwrap();

        let count: i64 = db.query_row("SELECT COUNT(*) FROM issue_reactions WHERE issue_id = 'i1' AND emoji = '👍'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
    }
}

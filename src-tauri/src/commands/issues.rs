use crate::models::issue::{CreateIssue, Issue, UpdateIssue};
use crate::state::AppState;
use tauri::{Emitter, State};
use uuid::Uuid;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn next_issue_number(db: &rusqlite::Connection, project_id: &str) -> Result<i64, rusqlite::Error> {
    let max: Option<i64> = db.query_row(
        "SELECT MAX(number) FROM issues WHERE project_id = ?1",
        [project_id],
        |row| row.get(0),
    )?;
    Ok(max.unwrap_or(0) + 1)
}

fn next_sort_order(db: &rusqlite::Connection, project_id: Option<&str>) -> Result<f64, rusqlite::Error> {
    let max: Option<f64> = match project_id {
        Some(pid) => db.query_row(
            "SELECT MAX(sort_order) FROM issues WHERE project_id = ?1 AND state = 'open'",
            [pid],
            |row| row.get(0),
        )?,
        None => db.query_row(
            "SELECT MAX(sort_order) FROM issues WHERE project_id IS NULL AND state = 'open'",
            [],
            |row| row.get(0),
        )?,
    };
    Ok(max.unwrap_or(0.0) + 1.0)
}

pub fn row_to_issue(row: &rusqlite::Row) -> Result<Issue, rusqlite::Error> {
    Ok(Issue {
        id: row.get(0)?,
        project_id: row.get(1)?,
        number: row.get(2)?,
        title: row.get(3)?,
        body: row.get(4)?,
        state: row.get(5)?,
        status: row.get(6)?,
        sort_order: row.get(7)?,
        context: row.get(8)?,
        machine_id: row.get(9)?,
        milestone_id: row.get(10)?,
        locked: row.get::<_, i64>(11)? != 0,
        pinned: row.get::<_, i64>(12)? != 0,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
        closed_at: row.get(15)?,
    })
}

pub const ISSUE_COLUMNS: &str = "id, project_id, number, title, body, state, status, sort_order, context, machine_id, milestone_id, locked, pinned, created_at, updated_at, closed_at";

#[tauri::command]
pub fn create_issue(app: tauri::AppHandle, state: State<AppState>, input: CreateIssue) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    let status = input.status.as_deref().unwrap_or("idea").to_string();

    let number = match &input.project_id {
        Some(pid) => Some(next_issue_number(&db, pid).map_err(|e| e.to_string())?),
        None => None,
    };

    let sort_order = next_sort_order(&db, input.project_id.as_deref()).map_err(|e| e.to_string())?;

    db.execute(
        "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, context, machine_id, milestone_id, locked, pinned, created_at, updated_at, closed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6, ?7, NULL, ?8, ?9, 0, 0, ?10, ?11, NULL)",
        rusqlite::params![
            id, input.project_id, number, input.title, input.body,
            status, sort_order, input.machine_id, input.milestone_id, now, now
        ],
    ).map_err(|e| e.to_string())?;

    // Log activity
    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, ?2, 'created', NULL, ?3)",
        rusqlite::params![id, input.project_id, now],
    ).map_err(|e| e.to_string())?;

    let issue = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    app.emit("issues-changed", serde_json::json!({"project_id": issue.project_id})).unwrap();
    Ok(issue)
}

#[tauri::command]
pub fn list_issues(
    state: State<AppState>,
    project_id: Option<String>,
    include_closed: Option<bool>,
) -> Result<Vec<Issue>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let include_closed = include_closed.unwrap_or(false);

    let sql = match (&project_id, include_closed) {
        (Some(_), false) => format!(
            "SELECT {} FROM issues WHERE project_id = ?1 AND state = 'open' ORDER BY pinned DESC, sort_order ASC",
            ISSUE_COLUMNS
        ),
        (Some(_), true) => format!(
            "SELECT {} FROM issues WHERE project_id = ?1 ORDER BY pinned DESC, sort_order ASC",
            ISSUE_COLUMNS
        ),
        (None, false) => format!(
            "SELECT {} FROM issues WHERE project_id IS NULL AND state = 'open' ORDER BY pinned DESC, sort_order ASC",
            ISSUE_COLUMNS
        ),
        (None, true) => format!(
            "SELECT {} FROM issues WHERE project_id IS NULL ORDER BY pinned DESC, sort_order ASC",
            ISSUE_COLUMNS
        ),
    };

    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;

    let issues = match &project_id {
        Some(pid) => stmt
            .query_map([pid], row_to_issue)
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect(),
        None => stmt
            .query_map([], row_to_issue)
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect(),
    };

    Ok(issues)
}

#[tauri::command]
pub fn get_issue(state: State<AppState>, id: String) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let issue = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;
    Ok(issue)
}

#[tauri::command]
pub fn update_issue(app: tauri::AppHandle, state: State<AppState>, input: UpdateIssue) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    let existing = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&input.id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    let title = input.title.unwrap_or(existing.title);
    let body = input.body.or(existing.body);
    let status = input.status.or(existing.status);
    let context = input.context.or(existing.context);
    let machine_id = input.machine_id.or(existing.machine_id);
    let milestone_id = input.milestone_id.or(existing.milestone_id);

    // Enforce only one 'next' per project
    if status.as_deref() == Some("next") {
        if let Some(ref pid) = existing.project_id {
            db.execute(
                "UPDATE issues SET status = 'ready', updated_at = ?1 WHERE project_id = ?2 AND status = 'next' AND id != ?3",
                rusqlite::params![now, pid, input.id],
            ).map_err(|e| e.to_string())?;
        }
    }

    db.execute(
        "UPDATE issues SET title = ?1, body = ?2, status = ?3, context = ?4, machine_id = ?5, milestone_id = ?6, updated_at = ?7 WHERE id = ?8",
        rusqlite::params![title, body, status, context, machine_id, milestone_id, now, input.id],
    ).map_err(|e| e.to_string())?;

    let issue = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&input.id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    app.emit("issues-changed", serde_json::json!({"project_id": issue.project_id})).unwrap();
    Ok(issue)
}

#[tauri::command]
pub fn close_issue(app: tauri::AppHandle, state: State<AppState>, id: String) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    db.execute(
        "UPDATE issues SET state = 'closed', closed_at = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![now, now, id],
    ).map_err(|e| e.to_string())?;

    // Re-evaluate blocked issues: if this was a blocker, check if blocked issues now have no open blockers
    let blocked_ids: Vec<String> = {
        let mut stmt = db.prepare(
            "SELECT blocked_id FROM issue_deps WHERE blocker_id = ?1"
        ).map_err(|e| e.to_string())?;
        let ids: Vec<String> = stmt.query_map([&id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        ids
    };

    for blocked_id in &blocked_ids {
        let open_blocker_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM issue_deps d JOIN issues i ON i.id = d.blocker_id WHERE d.blocked_id = ?1 AND i.state = 'open'",
            [blocked_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;

        if open_blocker_count == 0 {
            db.execute(
                "UPDATE issues SET status = 'ready', updated_at = ?1 WHERE id = ?2 AND status = 'blocked'",
                rusqlite::params![now, blocked_id],
            ).map_err(|e| e.to_string())?;
        }
    }

    // Log activity
    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) SELECT ?1, project_id, 'closed', NULL, ?2 FROM issues WHERE id = ?1",
        rusqlite::params![id, now],
    ).map_err(|e| e.to_string())?;

    let issue = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    app.emit("issues-changed", serde_json::json!({"project_id": issue.project_id})).unwrap();
    Ok(issue)
}

#[tauri::command]
pub fn reopen_issue(app: tauri::AppHandle, state: State<AppState>, id: String) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    db.execute(
        "UPDATE issues SET state = 'open', status = 'ready', closed_at = NULL, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, id],
    ).map_err(|e| e.to_string())?;

    // Log activity
    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) SELECT ?1, project_id, 'reopened', NULL, ?2 FROM issues WHERE id = ?1",
        rusqlite::params![id, now],
    ).map_err(|e| e.to_string())?;

    let issue = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    app.emit("issues-changed", serde_json::json!({"project_id": issue.project_id})).unwrap();
    Ok(issue)
}

#[tauri::command]
pub fn delete_issue(app: tauri::AppHandle, state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    // Get project_id before deleting
    let project_id: Option<String> = db.query_row(
        "SELECT project_id FROM issues WHERE id = ?1",
        [&id], |r| r.get(0)
    ).ok().flatten();
    db.execute("DELETE FROM issues WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    app.emit("issues-changed", serde_json::json!({"project_id": project_id})).unwrap();
    Ok(())
}

#[tauri::command]
pub fn reorder_issue(app: tauri::AppHandle, state: State<AppState>, id: String, new_sort_order: f64) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    db.execute(
        "UPDATE issues SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_sort_order, now, id],
    ).map_err(|e| e.to_string())?;

    let issue = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    app.emit("issues-changed", serde_json::json!({"project_id": issue.project_id})).unwrap();
    Ok(issue)
}

#[tauri::command]
pub fn transfer_issue(app: tauri::AppHandle, state: State<AppState>, id: String, to_project_id: String) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    let existing = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    let from_project_id = existing.project_id.clone();
    let new_number = next_issue_number(&db, &to_project_id).map_err(|e| e.to_string())?;
    let new_sort_order = next_sort_order(&db, Some(&to_project_id)).map_err(|e| e.to_string())?;

    db.execute(
        "UPDATE issues SET project_id = ?1, number = ?2, sort_order = ?3, updated_at = ?4 WHERE id = ?5",
        rusqlite::params![to_project_id, new_number, new_sort_order, now, id],
    ).map_err(|e| e.to_string())?;

    // Log activity with from/to detail
    let detail = format!(
        "from:{},to:{}",
        from_project_id.as_deref().unwrap_or("none"),
        to_project_id
    );
    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, ?2, 'transferred', ?3, ?4)",
        rusqlite::params![id, to_project_id, detail, now],
    ).map_err(|e| e.to_string())?;

    let issue = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    // Emit for both source and destination projects
    app.emit("issues-changed", serde_json::json!({"project_id": from_project_id})).unwrap();
    app.emit("issues-changed", serde_json::json!({"project_id": issue.project_id})).unwrap();
    Ok(issue)
}

#[tauri::command]
pub fn promote_idea(app: tauri::AppHandle, state: State<AppState>, id: String) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    let issue = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    // Create project from issue title/body
    let project_id = Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
        rusqlite::params![project_id, issue.title, issue.body, now, now],
    ).map_err(|e| e.to_string())?;

    // Close the issue
    db.execute(
        "UPDATE issues SET state = 'closed', closed_at = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![now, now, id],
    ).map_err(|e| e.to_string())?;

    // Log activity
    let detail = format!("promoted_to_project:{}", project_id);
    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, ?2, 'promoted', ?3, ?4)",
        rusqlite::params![id, project_id, detail, now],
    ).map_err(|e| e.to_string())?;

    let closed_issue = db.query_row(
        &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
        [&id],
        row_to_issue,
    ).map_err(|e| e.to_string())?;

    app.emit("issues-changed", serde_json::json!({"project_id": closed_issue.project_id})).unwrap();
    app.emit("projects-changed", ()).unwrap();
    Ok(closed_issue)
}

#[tauri::command]
pub fn add_dependency(state: State<AppState>, blocker_id: String, blocked_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    db.execute(
        "INSERT OR IGNORE INTO issue_deps (blocker_id, blocked_id) VALUES (?1, ?2)",
        rusqlite::params![blocker_id, blocked_id],
    ).map_err(|e| e.to_string())?;

    // Auto-set blocked issue to 'blocked' status
    db.execute(
        "UPDATE issues SET status = 'blocked', updated_at = ?1 WHERE id = ?2 AND state = 'open'",
        rusqlite::params![now, blocked_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn remove_dependency(state: State<AppState>, blocker_id: String, blocked_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    db.execute(
        "DELETE FROM issue_deps WHERE blocker_id = ?1 AND blocked_id = ?2",
        rusqlite::params![blocker_id, blocked_id],
    ).map_err(|e| e.to_string())?;

    // Re-evaluate: if no more open blockers, unblock
    let still_blocked: i64 = db.query_row(
        "SELECT COUNT(*) FROM issue_deps d JOIN issues i ON d.blocker_id = i.id WHERE d.blocked_id = ?1 AND i.state = 'open'",
        [&blocked_id], |r| r.get(0),
    ).map_err(|e| e.to_string())?;

    if still_blocked == 0 {
        db.execute(
            "UPDATE issues SET status = 'ready', updated_at = ?1 WHERE id = ?2 AND status = 'blocked'",
            rusqlite::params![now, blocked_id],
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn add_relation(state: State<AppState>, issue_a_id: String, issue_b_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    // Store with smaller id first for consistency
    let (a, b) = if issue_a_id < issue_b_id { (issue_a_id, issue_b_id) } else { (issue_b_id, issue_a_id) };
    db.execute(
        "INSERT OR IGNORE INTO issue_relations (issue_a_id, issue_b_id) VALUES (?1, ?2)",
        rusqlite::params![a, b],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn remove_relation(state: State<AppState>, issue_a_id: String, issue_b_id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM issue_relations WHERE (issue_a_id = ?1 AND issue_b_id = ?2) OR (issue_a_id = ?2 AND issue_b_id = ?1)",
        rusqlite::params![issue_a_id, issue_b_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn assign_issue(state: State<AppState>, issue_id: String, assignee_name: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    // Upsert assignee
    let assignee_id: String = match db.query_row("SELECT id FROM assignees WHERE name = ?1", [&assignee_name], |r| r.get(0)) {
        Ok(id) => id,
        Err(_) => {
            let id = Uuid::new_v4().to_string();
            db.execute("INSERT INTO assignees (id, name) VALUES (?1, ?2)", rusqlite::params![id, assignee_name]).map_err(|e| e.to_string())?;
            id
        }
    };
    db.execute(
        "INSERT OR IGNORE INTO issue_assignees (issue_id, assignee_id) VALUES (?1, ?2)",
        rusqlite::params![issue_id, assignee_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn unassign_issue(state: State<AppState>, issue_id: String, assignee_name: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM issue_assignees WHERE issue_id = ?1 AND assignee_id = (SELECT id FROM assignees WHERE name = ?2)",
        rusqlite::params![issue_id, assignee_name],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_issue_assignees(state: State<AppState>, issue_id: String) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT a.name FROM assignees a JOIN issue_assignees ia ON a.id = ia.assignee_id WHERE ia.issue_id = ?1 ORDER BY a.name"
    ).map_err(|e| e.to_string())?;
    let names = stmt.query_map([&issue_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn test_state() -> AppState {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        db::schema::create_tables(&conn).unwrap();
        AppState { db: Mutex::new(conn) }
    }

    fn insert_project(db: &Connection, id: &str, name: &str) {
        db.execute(
            "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES (?1, ?2, NULL, NULL, 1000, 1000)",
            rusqlite::params![id, name],
        ).unwrap();
    }

    fn insert_issue(db: &Connection, id: &str, project_id: Option<&str>, title: &str, status: &str, state: &str, sort_order: f64) {
        db.execute(
            "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, context, machine_id, milestone_id, locked, pinned, created_at, updated_at, closed_at)
             VALUES (?1, ?2, 1, ?3, NULL, ?4, ?5, ?6, NULL, NULL, NULL, 0, 0, 1000, 1000, NULL)",
            rusqlite::params![id, project_id, title, state, status, sort_order],
        ).unwrap();
    }

    #[test]
    fn test_create_issue_with_project() {
        let state = test_state();
        {
            let db = state.db.lock().unwrap();
            insert_project(&db, "p1", "MyProject");
        }

        // Test via direct DB operations matching what create_issue does
        let db = state.db.lock().unwrap();
        let id = Uuid::new_v4().to_string();
        let now = now_ms();
        let number = next_issue_number(&db, "p1").unwrap();
        let sort_order = next_sort_order(&db, Some("p1")).unwrap();

        assert_eq!(number, 1);
        assert!(sort_order > 0.0);

        db.execute(
            "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, context, machine_id, milestone_id, locked, pinned, created_at, updated_at, closed_at)
             VALUES (?1, 'p1', ?2, 'Fix bug', NULL, 'open', 'idea', ?3, NULL, NULL, NULL, 0, 0, ?4, ?5, NULL)",
            rusqlite::params![id, number, sort_order, now, now],
        ).unwrap();

        let issue = db.query_row(
            &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
            [&id],
            row_to_issue,
        ).unwrap();

        assert_eq!(issue.project_id.as_deref(), Some("p1"));
        assert_eq!(issue.number, Some(1));
        assert_eq!(issue.status.as_deref(), Some("idea"));
        assert_eq!(issue.state, "open");
    }

    #[test]
    fn test_create_issue_without_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        let id = Uuid::new_v4().to_string();
        let now = now_ms();
        let sort_order = next_sort_order(&db, None).unwrap();

        assert_eq!(sort_order, 1.0);

        db.execute(
            "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, context, machine_id, milestone_id, locked, pinned, created_at, updated_at, closed_at)
             VALUES (?1, NULL, NULL, 'Inbox item', NULL, 'open', 'idea', ?2, NULL, NULL, NULL, 0, 0, ?3, ?4, NULL)",
            rusqlite::params![id, sort_order, now, now],
        ).unwrap();

        let issue = db.query_row(
            &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
            [&id],
            row_to_issue,
        ).unwrap();

        assert!(issue.project_id.is_none());
        assert!(issue.number.is_none());
        assert_eq!(issue.status.as_deref(), Some("idea"));
    }

    #[test]
    fn test_close_issue_unblocks_dependents() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        insert_project(&db, "p1", "Project");
        // blocker issue
        insert_issue(&db, "blocker1", Some("p1"), "Blocker", "ready", "open", 1.0);
        // blocked issue
        insert_issue(&db, "blocked1", Some("p1"), "Blocked", "blocked", "open", 2.0);

        // Create dependency: blocker1 blocks blocked1
        db.execute(
            "INSERT INTO issue_deps (blocker_id, blocked_id) VALUES ('blocker1', 'blocked1')",
            [],
        ).unwrap();

        let now = now_ms();

        // Close the blocker
        db.execute(
            "UPDATE issues SET state = 'closed', closed_at = ?1, updated_at = ?2 WHERE id = 'blocker1'",
            rusqlite::params![now, now],
        ).unwrap();

        // Re-evaluate blocked issues
        let open_blocker_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM issue_deps d JOIN issues i ON i.id = d.blocker_id WHERE d.blocked_id = 'blocked1' AND i.state = 'open'",
            [],
            |row| row.get(0),
        ).unwrap();

        if open_blocker_count == 0 {
            db.execute(
                "UPDATE issues SET status = 'ready', updated_at = ?1 WHERE id = 'blocked1' AND status = 'blocked'",
                rusqlite::params![now],
            ).unwrap();
        }

        let status: String = db.query_row(
            "SELECT status FROM issues WHERE id = 'blocked1'",
            [],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(status, "ready");
    }

    #[test]
    fn test_only_one_next_per_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        insert_project(&db, "p1", "Project");
        insert_issue(&db, "i1", Some("p1"), "Issue 1", "next", "open", 1.0);
        insert_issue(&db, "i2", Some("p1"), "Issue 2", "ready", "open", 2.0);

        let now = now_ms();

        // Setting i2 to 'next' should clear i1's 'next' status
        db.execute(
            "UPDATE issues SET status = 'ready', updated_at = ?1 WHERE project_id = 'p1' AND status = 'next' AND id != 'i2'",
            rusqlite::params![now],
        ).unwrap();
        db.execute(
            "UPDATE issues SET status = 'next', updated_at = ?1 WHERE id = 'i2'",
            rusqlite::params![now],
        ).unwrap();

        let i1_status: String = db.query_row(
            "SELECT status FROM issues WHERE id = 'i1'",
            [],
            |row| row.get(0),
        ).unwrap();

        let i2_status: String = db.query_row(
            "SELECT status FROM issues WHERE id = 'i2'",
            [],
            |row| row.get(0),
        ).unwrap();

        assert_eq!(i1_status, "ready");
        assert_eq!(i2_status, "next");

        // Verify only one 'next' in project
        let next_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM issues WHERE project_id = 'p1' AND status = 'next'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(next_count, 1);
    }

    #[test]
    fn test_promote_idea_to_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        // Insert an idea issue without a project
        let issue_id = "idea1".to_string();
        let now = now_ms();
        db.execute(
            "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, context, machine_id, milestone_id, locked, pinned, created_at, updated_at, closed_at)
             VALUES ('idea1', NULL, NULL, 'Build new feature', 'Description of feature', 'open', 'idea', 1.0, NULL, NULL, NULL, 0, 0, ?1, ?2, NULL)",
            rusqlite::params![now, now],
        ).unwrap();

        let issue = db.query_row(
            &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
            [&issue_id],
            row_to_issue,
        ).unwrap();

        // Create project from issue
        let project_id = Uuid::new_v4().to_string();
        db.execute(
            "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
            rusqlite::params![project_id, issue.title, issue.body, now, now],
        ).unwrap();

        // Close the issue
        db.execute(
            "UPDATE issues SET state = 'closed', closed_at = ?1, updated_at = ?2 WHERE id = 'idea1'",
            rusqlite::params![now, now],
        ).unwrap();

        // Verify project was created
        let project_name: String = db.query_row(
            "SELECT name FROM projects WHERE id = ?1",
            [&project_id],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(project_name, "Build new feature");

        // Verify issue is closed
        let issue_state: String = db.query_row(
            "SELECT state FROM issues WHERE id = 'idea1'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(issue_state, "closed");

        // Verify activity log entry
        let detail = format!("promoted_to_project:{}", project_id);
        db.execute(
            "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES ('idea1', ?1, 'promoted', ?2, ?3)",
            rusqlite::params![project_id, detail, now],
        ).unwrap();

        let action: String = db.query_row(
            "SELECT action FROM activity_log WHERE issue_id = 'idea1'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(action, "promoted");
    }

    #[test]
    fn test_add_and_remove_dependency() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'T', NULL, NULL, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issues (id, project_id, number, title, state, status, sort_order, locked, pinned, created_at, updated_at) VALUES ('a', 'p1', 1, 'A', 'open', 'ready', 1.0, 0, 0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issues (id, project_id, number, title, state, status, sort_order, locked, pinned, created_at, updated_at) VALUES ('b', 'p1', 2, 'B', 'open', 'ready', 2.0, 0, 0, 1000, 1000)", []).unwrap();

        db.execute("INSERT INTO issue_deps (blocker_id, blocked_id) VALUES ('a', 'b')", []).unwrap();
        db.execute("UPDATE issues SET status = 'blocked' WHERE id = 'b'", []).unwrap();

        db.execute("DELETE FROM issue_deps WHERE blocker_id = 'a' AND blocked_id = 'b'", []).unwrap();
        let still_blocked: i64 = db.query_row("SELECT COUNT(*) FROM issue_deps d JOIN issues i ON d.blocker_id = i.id WHERE d.blocked_id = 'b' AND i.state = 'open'", [], |r| r.get(0)).unwrap();
        if still_blocked == 0 {
            db.execute("UPDATE issues SET status = 'ready' WHERE id = 'b' AND status = 'blocked'", []).unwrap();
        }
        let status: String = db.query_row("SELECT status FROM issues WHERE id = 'b'", [], |r| r.get(0)).unwrap();
        assert_eq!(status, "ready");
    }

    #[test]
    fn test_assignees() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO issues (id, title, state, status, sort_order, locked, pinned, created_at, updated_at) VALUES ('i1', 'T', 'open', 'idea', 1.0, 0, 0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO assignees (id, name) VALUES ('a1', 'parker')", []).unwrap();
        db.execute("INSERT INTO issue_assignees (issue_id, assignee_id) VALUES ('i1', 'a1')", []).unwrap();

        let name: String = db.query_row("SELECT a.name FROM assignees a JOIN issue_assignees ia ON a.id = ia.assignee_id WHERE ia.issue_id = 'i1'", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "parker");
    }
}

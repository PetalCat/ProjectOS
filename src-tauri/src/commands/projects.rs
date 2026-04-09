use crate::models::project::{CreateProject, Project, UpdateProject};
use crate::state::AppState;
use tauri::{Emitter, State};
use uuid::Uuid;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[tauri::command]
pub fn create_project(app: tauri::AppHandle, state: State<AppState>, input: CreateProject) -> Result<Project, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    db.execute(
        "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
        rusqlite::params![id, input.name, input.description, now, now],
    ).map_err(|e| e.to_string())?;
    let project = Project { id, name: input.name, description: input.description, notes: None, created_at: now, updated_at: now };
    app.emit("projects-changed", ()).unwrap();
    Ok(project)
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare("SELECT id, name, description, notes, created_at, updated_at FROM projects ORDER BY name").map_err(|e| e.to_string())?;
    let projects = stmt.query_map([], |row| {
        Ok(Project { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, notes: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)? })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(projects)
}

#[tauri::command]
pub fn update_project(app: tauri::AppHandle, state: State<AppState>, input: UpdateProject) -> Result<Project, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    let existing: Project = db.query_row(
        "SELECT id, name, description, notes, created_at, updated_at FROM projects WHERE id = ?1",
        [&input.id], |row| Ok(Project { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, notes: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)? }),
    ).map_err(|e| e.to_string())?;
    let name = input.name.unwrap_or(existing.name);
    let description = input.description.or(existing.description);
    let notes = input.notes.or(existing.notes);
    db.execute("UPDATE projects SET name = ?1, description = ?2, notes = ?3, updated_at = ?4 WHERE id = ?5",
        rusqlite::params![name, description, notes, now, input.id]).map_err(|e| e.to_string())?;
    app.emit("projects-changed", ()).unwrap();
    Ok(Project { id: input.id, name, description, notes, created_at: existing.created_at, updated_at: now })
}

#[tauri::command]
pub fn delete_project(app: tauri::AppHandle, state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM projects WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    app.emit("projects-changed", ()).unwrap();
    Ok(())
}

#[derive(serde::Serialize)]
pub struct DashboardProject {
    pub project: Project,
    pub next_issue: Option<crate::models::issue::Issue>,
    pub open_count: i64,
}

#[derive(serde::Serialize)]
pub struct Dashboard {
    pub projects: Vec<DashboardProject>,
    pub recent_activity: Vec<crate::models::activity::ActivityEntry>,
}

#[tauri::command]
pub fn get_dashboard(state: State<AppState>) -> Result<Dashboard, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db.prepare("SELECT id, name, description, notes, created_at, updated_at FROM projects ORDER BY name").map_err(|e| e.to_string())?;
    let projects: Vec<Project> = stmt.query_map([], |row| {
        Ok(Project { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, notes: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)? })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let mut dashboard_projects = Vec::new();
    for project in projects {
        let open_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM issues WHERE project_id = ?1 AND state = 'open'",
            [&project.id], |r| r.get(0),
        ).map_err(|e| e.to_string())?;

        let next_issue = db.query_row(
            &format!("SELECT {} FROM issues WHERE project_id = ?1 AND status = 'next' AND state = 'open' LIMIT 1", crate::commands::issues::ISSUE_COLUMNS),
            [&project.id],
            |row| crate::commands::issues::row_to_issue(row),
        ).ok();

        dashboard_projects.push(DashboardProject { project, next_issue, open_count });
    }

    let mut stmt = db.prepare(
        "SELECT id, issue_id, project_id, action, detail, created_at FROM activity_log ORDER BY created_at DESC LIMIT 20"
    ).map_err(|e| e.to_string())?;
    let recent_activity = stmt.query_map([], |row| {
        Ok(crate::models::activity::ActivityEntry { id: row.get(0)?, issue_id: row.get(1)?, project_id: row.get(2)?, action: row.get(3)?, detail: row.get(4)?, created_at: row.get(5)? })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    Ok(Dashboard { projects: dashboard_projects, recent_activity })
}

#[tauri::command]
pub fn scan_developer_folder(app: tauri::AppHandle, state: State<AppState>, path: String) -> Result<Vec<Project>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    let mut created = Vec::new();

    let entries = std::fs::read_dir(&path).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip hidden dirs
        if name.starts_with('.') {
            continue;
        }
        // Skip if project already exists with this name
        let exists: bool = db.query_row(
            "SELECT COUNT(*) > 0 FROM projects WHERE name = ?1",
            [&name],
            |row| row.get(0),
        ).unwrap_or(false);
        if exists {
            continue;
        }

        let id = Uuid::new_v4().to_string();
        let dir_path = entry.path().to_string_lossy().to_string();
        db.execute(
            "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
            rusqlite::params![id, name, dir_path, now, now],
        ).map_err(|e| e.to_string())?;

        db.execute(
            "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (NULL, ?1, 'created', ?2, ?3)",
            rusqlite::params![id, serde_json::json!({"title": name, "source": "scan"}).to_string(), now],
        ).map_err(|e| e.to_string())?;

        created.push(Project {
            id,
            name,
            description: Some(dir_path),
            notes: None,
            created_at: now,
            updated_at: now,
        });
    }

    let _ = app.emit("projects-changed", ());
    Ok(created)
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
    fn test_create_and_list_projects() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'ProjectOS', 'Test', NULL, 1000, 1000)", []).unwrap();
        let mut stmt = db.prepare("SELECT id, name FROM projects").unwrap();
        let names: Vec<String> = stmt.query_map([], |row| row.get(1)).unwrap().filter_map(|r| r.ok()).collect();
        assert_eq!(names, vec!["ProjectOS"]);
    }

    #[test]
    fn test_update_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'Old', NULL, NULL, 1000, 1000)", []).unwrap();
        db.execute("UPDATE projects SET name = 'New', updated_at = 2000 WHERE id = 'p1'", []).unwrap();
        let name: String = db.query_row("SELECT name FROM projects WHERE id = 'p1'", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "New");
    }

    #[test]
    fn test_delete_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'Test', NULL, NULL, 1000, 1000)", []).unwrap();
        db.execute("DELETE FROM projects WHERE id = 'p1'", []).unwrap();
        let count: i64 = db.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}

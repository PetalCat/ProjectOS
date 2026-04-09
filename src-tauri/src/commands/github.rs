use crate::state::AppState;
use tauri::{Emitter, State};
use uuid::Uuid;
use std::process::Command;

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
}

#[tauri::command]
pub fn sync_github_issues(app: tauri::AppHandle, state: State<AppState>, project_id: String) -> Result<u32, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Get github_repo for this project
    let github_repo: Option<String> = db.query_row(
        "SELECT github_repo FROM projects WHERE id = ?1", [&project_id], |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let repo = github_repo.ok_or("Project has no linked GitHub repo")?;

    // Use gh CLI to fetch issues
    let output = Command::new("gh")
        .args(["issue", "list", "--repo", &repo, "--json", "number,title,body,state,url", "--limit", "100"])
        .output()
        .map_err(|e| format!("Failed to run gh CLI: {}", e))?;

    if !output.status.success() {
        return Err(format!("gh CLI error: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let issues: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse gh output: {}", e))?;

    let now = now_ms();
    let mut synced = 0u32;

    for issue in &issues {
        let gh_number = issue["number"].as_i64().unwrap_or(0);
        let title = issue["title"].as_str().unwrap_or("").to_string();
        let body = issue["body"].as_str().map(|s| s.to_string());
        let state = if issue["state"].as_str() == Some("OPEN") { "open" } else { "closed" };
        let url = issue["url"].as_str().unwrap_or("").to_string();
        let external_id = format!("{}#{}", repo, gh_number);

        // Check if already synced
        let exists: bool = db.query_row(
            "SELECT COUNT(*) > 0 FROM issues WHERE external_source = 'github' AND external_id = ?1",
            [&external_id], |row| row.get(0),
        ).unwrap_or(false);

        if exists {
            // Update existing
            db.execute(
                "UPDATE issues SET title = ?1, body = ?2, state = ?3, status = CASE WHEN ?3 = 'closed' THEN NULL ELSE status END, updated_at = ?4 WHERE external_source = 'github' AND external_id = ?5",
                rusqlite::params![title, body, state, now, external_id],
            ).map_err(|e| e.to_string())?;
        } else {
            // Create new
            let id = Uuid::new_v4().to_string();
            let sort_order: f64 = db.query_row(
                "SELECT COALESCE(MAX(sort_order), 0.0) + 1.0 FROM issues WHERE project_id = ?1",
                [&project_id], |row| row.get(0),
            ).unwrap_or(1.0);

            let status = if state == "open" { Some("ready") } else { None };

            db.execute(
                "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, locked, pinned, external_source, external_id, external_url, created_at, updated_at, closed_at)
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, 0, 0, 'github', ?8, ?9, ?10, ?11, CASE WHEN ?5 = 'closed' THEN ?10 ELSE NULL END)",
                rusqlite::params![id, project_id, title, body, state, status, sort_order, external_id, url, now, now],
            ).map_err(|e| e.to_string())?;
        }
        synced += 1;
    }

    let _ = app.emit("issues-changed", serde_json::json!({"project_id": project_id}));
    Ok(synced)
}

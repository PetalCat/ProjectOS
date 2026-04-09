use crate::models::issue::Issue;
use crate::state::AppState;
use tauri::State;

#[derive(serde::Serialize)]
pub struct SearchResults {
    pub issues: Vec<Issue>,
    pub projects: Vec<crate::models::project::Project>,
    pub machines: Vec<crate::models::machine::Machine>,
}

#[tauri::command]
pub fn search_issues(state: State<AppState>, query: String) -> Result<SearchResults, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let fts_query = format!("{}*", query.replace('"', ""));
    let like_query = format!("%{}%", query);

    // FTS search on issues
    let mut stmt = db.prepare(
        "SELECT i.id, i.project_id, i.number, i.title, i.body, i.state, i.status, i.sort_order, i.context, i.machine_id, i.milestone_id, i.locked, i.pinned, i.created_at, i.updated_at, i.closed_at, i.external_source, i.external_id, i.external_url
         FROM issues i JOIN issues_fts f ON i.rowid = f.rowid
         WHERE issues_fts MATCH ?1
         ORDER BY rank LIMIT 20"
    ).map_err(|e| e.to_string())?;

    let issues: Vec<Issue> = stmt.query_map([&fts_query], |row| {
        Ok(Issue {
            id: row.get(0)?, project_id: row.get(1)?, number: row.get(2)?, title: row.get(3)?,
            body: row.get(4)?, state: row.get(5)?, status: row.get(6)?, sort_order: row.get(7)?,
            context: row.get(8)?, machine_id: row.get(9)?, milestone_id: row.get(10)?,
            locked: row.get::<_, i64>(11).map(|v| v != 0)?,
            pinned: row.get::<_, i64>(12).map(|v| v != 0)?,
            created_at: row.get(13)?, updated_at: row.get(14)?, closed_at: row.get(15)?,
            external_source: row.get(16)?, external_id: row.get(17)?, external_url: row.get(18)?,
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // LIKE search on projects
    let mut stmt = db.prepare(
        "SELECT id, name, description, notes, github_repo, created_at, updated_at FROM projects WHERE name LIKE ?1 LIMIT 10"
    ).map_err(|e| e.to_string())?;
    let projects = stmt.query_map([&like_query], |row| {
        Ok(crate::models::project::Project { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, notes: row.get(3)?, github_repo: row.get(4)?, created_at: row.get(5)?, updated_at: row.get(6)? })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // LIKE search on machines
    let mut stmt = db.prepare(
        "SELECT id, name, hostname, ip, user, os, notes, created_at, updated_at FROM machines WHERE name LIKE ?1 OR hostname LIKE ?1 LIMIT 10"
    ).map_err(|e| e.to_string())?;
    let machines = stmt.query_map([&like_query], |row| {
        Ok(crate::models::machine::Machine { id: row.get(0)?, name: row.get(1)?, hostname: row.get(2)?, ip: row.get(3)?, user: row.get(4)?, os: row.get(5)?, notes: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)? })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    Ok(SearchResults { issues, projects, machines })
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
    fn test_fts_search() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO issues (id, title, body, state, status, sort_order, locked, pinned, created_at, updated_at) VALUES ('i1', 'Fix login bug', 'Auth fails on mobile', 'open', 'ready', 1.0, 0, 0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issues (id, title, body, state, status, sort_order, locked, pinned, created_at, updated_at) VALUES ('i2', 'Add dark mode', 'Theme support', 'open', 'idea', 2.0, 0, 0, 1000, 1000)", []).unwrap();

        let mut stmt = db.prepare("SELECT issues.title FROM issues JOIN issues_fts ON issues.rowid = issues_fts.rowid WHERE issues_fts MATCH 'login*' LIMIT 5").unwrap();
        let titles: Vec<String> = stmt.query_map([], |r| r.get(0)).unwrap().filter_map(|r| r.ok()).collect();
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0], "Fix login bug");
    }
}

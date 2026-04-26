use crate::commands::issues::{row_to_issue, ISSUE_COLUMNS};
use crate::models::issue::Issue;
use crate::state::AppState;
use rusqlite::Connection;
use std::process::Command;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

// ── link helpers ──────────────────────────────────────────────────────────────

fn parse_external_id(ext_id: &str) -> Option<(String, i64)> {
    // external_id format is "owner/repo#N"
    let hash = ext_id.rfind('#')?;
    let repo = ext_id[..hash].to_string();
    let number: i64 = ext_id[hash + 1..].parse().ok()?;
    Some((repo, number))
}

fn github_link(db: &Connection, issue_id: &str) -> Option<(String, i64)> {
    let row: Result<(Option<String>, Option<String>), _> = db.query_row(
        "SELECT external_source, external_id FROM issues WHERE id = ?1",
        [issue_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    );
    let (source, ext_id) = row.ok()?;
    if source.as_deref() != Some("github") {
        return None;
    }
    ext_id.as_deref().and_then(parse_external_id)
}

pub struct PushSnapshot {
    repo: String,
    number: i64,
    title: String,
    body: String,
    state: String,
}

// Extract push info while the DB lock is held. Call from within a command,
// then drop the lock before invoking `push_issue_update_async` to avoid
// blocking the DB while `gh` runs.
pub fn snapshot_for_push(db: &Connection, issue_id: &str) -> Option<PushSnapshot> {
    let (repo, number) = github_link(db, issue_id)?;
    let row: Result<(String, Option<String>, String), _> = db.query_row(
        "SELECT title, body, state FROM issues WHERE id = ?1",
        [issue_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    );
    let (title, body, state) = row.ok()?;
    Some(PushSnapshot {
        repo,
        number,
        title,
        body: body.unwrap_or_default(),
        state,
    })
}

fn emit_push_error(app: &AppHandle, issue_id: &str, message: impl Into<String>) {
    let msg = message.into();
    eprintln!("[github push] issue={} error={}", issue_id, msg);
    let _ = app.emit(
        "github-push-error",
        serde_json::json!({ "issue_id": issue_id, "message": msg }),
    );
}

// Format a `Command::new("gh")` spawn error with an actionable hint when
// the gh binary isn't installed. The user-facing banner needs more than
// "No such file or directory".
fn fmt_gh_spawn_err(label: &str, e: &std::io::Error) -> String {
    if e.kind() == std::io::ErrorKind::NotFound {
        format!(
            "{}: GitHub CLI (`gh`) not found. Install from https://cli.github.com and run `gh auth login`.",
            label
        )
    } else {
        format!("{}: {}", label, e)
    }
}

// Format a non-zero gh exit's stderr with hints for the common failure modes
// (not authenticated, rate-limited).
fn fmt_gh_stderr(label: &str, stderr: &[u8]) -> String {
    let msg = String::from_utf8_lossy(stderr).trim().to_string();
    let lower = msg.to_lowercase();
    if lower.contains("authentication required")
        || lower.contains("not logged into")
        || lower.contains("gh auth login")
        || lower.contains("401")
    {
        format!(
            "{}: GitHub authentication required. Run `gh auth login` and try again. ({})",
            label, msg
        )
    } else if lower.contains("rate limit") || lower.contains("api rate") {
        format!(
            "{}: GitHub rate limit hit — try again in a few minutes. ({})",
            label, msg
        )
    } else {
        format!("{}: {}", label, msg)
    }
}

pub fn push_issue_update_snapshot(app: &AppHandle, issue_id: &str, snap: PushSnapshot) {
    let num = snap.number.to_string();

    // Edit title (and body, only when non-empty — otherwise `--body ""` clears
    // the existing GitHub description, which is a footgun for first-time syncs).
    let mut edit = Command::new("gh");
    edit.args([
        "issue",
        "edit",
        &num,
        "--repo",
        &snap.repo,
        "--title",
        &snap.title,
    ]);
    if !snap.body.is_empty() {
        edit.args(["--body", &snap.body]);
    }
    match edit.output() {
        Ok(o) if o.status.success() => {}
        Ok(o) => emit_push_error(app, issue_id, fmt_gh_stderr("gh issue edit", &o.stderr)),
        Err(e) => emit_push_error(app, issue_id, fmt_gh_spawn_err("gh issue edit", &e)),
    }

    // Sync state (idempotent)
    let subcmd = if snap.state == "closed" { "close" } else { "reopen" };
    let state_res = Command::new("gh")
        .args(["issue", subcmd, &num, "--repo", &snap.repo])
        .output();
    // Non-success on close/reopen usually means "already in that state" — ignore.
    if let Err(e) = state_res {
        emit_push_error(
            app,
            issue_id,
            fmt_gh_spawn_err(&format!("gh issue {}", subcmd), &e),
        );
    }
}

pub fn spawn_push_issue_update(app: AppHandle, issue_id: String, snap: PushSnapshot) {
    std::thread::spawn(move || {
        push_issue_update_snapshot(&app, &issue_id, snap);
    });
}

pub fn spawn_push_comment(app: AppHandle, issue_id: String, repo: String, number: i64, body: String) {
    std::thread::spawn(move || {
        let res = Command::new("gh")
            .args([
                "issue",
                "comment",
                &number.to_string(),
                "--repo",
                &repo,
                "--body",
                &body,
            ])
            .output();
        match res {
            Ok(o) if o.status.success() => {}
            Ok(o) => emit_push_error(&app, &issue_id, fmt_gh_stderr("gh issue comment", &o.stderr)),
            Err(e) => emit_push_error(&app, &issue_id, fmt_gh_spawn_err("gh issue comment", &e)),
        }
    });
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn sync_github_issues(
    app: AppHandle,
    state: State<AppState>,
    project_id: String,
) -> Result<u32, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let github_repo: Option<String> = db
        .query_row(
            "SELECT github_repo FROM projects WHERE id = ?1",
            [&project_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let repo = github_repo.ok_or("Project has no linked GitHub repo")?;

    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "--repo",
            &repo,
            "--state",
            "all",
            "--json",
            "number,title,body,state,url",
            "--limit",
            "200",
        ])
        .output()
        .map_err(|e| fmt_gh_spawn_err("gh issue list", &e))?;

    if !output.status.success() {
        return Err(fmt_gh_stderr("gh issue list", &output.stderr));
    }

    let issues: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse gh output: {}", e))?;

    let now = now_ms();
    let mut synced = 0u32;

    for issue in &issues {
        let gh_number = issue["number"].as_i64().unwrap_or(0);
        let title = issue["title"].as_str().unwrap_or("").to_string();
        let body = issue["body"].as_str().map(|s| s.to_string());
        let state = if issue["state"].as_str() == Some("OPEN") {
            "open"
        } else {
            "closed"
        };
        let url = issue["url"].as_str().unwrap_or("").to_string();
        let external_id = format!("{}#{}", repo, gh_number);

        let exists: bool = db
            .query_row(
                "SELECT COUNT(*) > 0 FROM issues WHERE external_source = 'github' AND external_id = ?1",
                [&external_id],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if exists {
            db.execute(
                "UPDATE issues SET title = ?1, body = ?2, state = ?3, status = CASE WHEN ?3 = 'closed' THEN NULL ELSE status END, updated_at = ?4 WHERE external_source = 'github' AND external_id = ?5",
                rusqlite::params![title, body, state, now, external_id],
            ).map_err(|e| e.to_string())?;
        } else {
            let id = Uuid::new_v4().to_string();
            let sort_order: f64 = db
                .query_row(
                    "SELECT COALESCE(MAX(sort_order), 0.0) + 1.0 FROM issues WHERE project_id = ?1",
                    [&project_id],
                    |row| row.get(0),
                )
                .unwrap_or(1.0);

            let status = if state == "open" { Some("ready") } else { None };

            db.execute(
                "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, locked, pinned, external_source, external_id, external_url, created_at, updated_at, closed_at)
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, 0, 0, 'github', ?8, ?9, ?10, ?11, CASE WHEN ?5 = 'closed' THEN ?10 ELSE NULL END)",
                rusqlite::params![id, project_id, title, body, state, status, sort_order, external_id, url, now, now],
            ).map_err(|e| e.to_string())?;
        }
        synced += 1;
    }

    let _ = app.emit(
        "issues-changed",
        serde_json::json!({ "project_id": project_id }),
    );
    Ok(synced)
}

#[tauri::command]
pub fn publish_issue_to_github(
    app: AppHandle,
    state: State<AppState>,
    issue_id: String,
) -> Result<Issue, String> {
    // Fetch issue + project.github_repo inside a scoped lock so we can drop
    // it before invoking `gh` (which can take multiple seconds).
    let (title, body, repo, project_id) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let issue: Issue = db
            .query_row(
                &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
                [&issue_id],
                row_to_issue,
            )
            .map_err(|e| e.to_string())?;

        if issue.external_source.as_deref() == Some("github") && issue.external_id.is_some() {
            return Err("Issue is already linked to GitHub".to_string());
        }

        let project_id = issue
            .project_id
            .as_deref()
            .ok_or("Issue has no project")?
            .to_string();

        let repo: Option<String> = db
            .query_row(
                "SELECT github_repo FROM projects WHERE id = ?1",
                [&project_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;

        let repo = repo.ok_or("Project has no linked GitHub repo")?;
        (
            issue.title,
            issue.body.unwrap_or_default(),
            repo,
            project_id,
        )
    };

    let mut create = Command::new("gh");
    create.args(["issue", "create", "--repo", &repo, "--title", &title]);
    if !body.is_empty() {
        create.args(["--body", &body]);
    }
    let output = create
        .output()
        .map_err(|e| fmt_gh_spawn_err("gh issue create", &e))?;

    if !output.status.success() {
        return Err(fmt_gh_stderr("gh issue create", &output.stderr));
    }

    // Last line of stdout is the URL like https://github.com/owner/repo/issues/123
    let url = String::from_utf8_lossy(&output.stdout)
        .lines()
        .rev()
        .find(|l| l.starts_with("http"))
        .unwrap_or("")
        .trim()
        .to_string();
    if url.is_empty() {
        return Err("gh issue create returned no URL".to_string());
    }

    let number: i64 = url
        .rsplit('/')
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or("Could not parse issue number from gh output")?;

    let external_id = format!("{}#{}", repo, number);
    let now = now_ms();

    let updated = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "UPDATE issues SET external_source = 'github', external_id = ?1, external_url = ?2, updated_at = ?3 WHERE id = ?4",
            rusqlite::params![external_id, url, now, issue_id],
        )
        .map_err(|e| e.to_string())?;

        db.query_row(
            &format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS),
            [&issue_id],
            row_to_issue,
        )
        .map_err(|e| e.to_string())?
    };

    let _ = app.emit(
        "issues-changed",
        serde_json::json!({ "project_id": project_id }),
    );
    Ok(updated)
}

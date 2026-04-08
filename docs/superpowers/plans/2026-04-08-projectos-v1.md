# ProjectOS v1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a local-first Tauri v2 desktop app for managing issues across projects, modeled after GitHub Issues with ordered execution and machine awareness.

**Architecture:** Rust backend owns all data via rusqlite, exposes Tauri commands, and pushes state changes via Tauri events. Svelte 5 frontend renders the UI reactively by subscribing to those events. SQLite FTS5 powers full-text search.

**Tech Stack:** Tauri v2, Svelte 5, Rust, rusqlite (bundled), SQLite FTS5, Vite, vitest, pnpm

---

## File Structure

```
ProjectOS/
├── package.json
├── pnpm-lock.yaml
├── vite.config.ts
├── svelte.config.js
├── tsconfig.json
├── index.html
├── src/                              # Svelte frontend
│   ├── main.ts                       # Mount App
│   ├── App.svelte                    # Root layout: sidebar + content
│   ├── lib/
│   │   ├── types.ts                  # All TypeScript types
│   │   ├── commands.ts               # Tauri invoke wrappers
│   │   ├── events.ts                 # Tauri event subscriptions
│   │   ├── stores/
│   │   │   ├── navigation.svelte.ts  # Current view/route state
│   │   │   ├── projects.svelte.ts    # Projects list
│   │   │   ├── issues.svelte.ts      # Issues for current project
│   │   │   ├── dashboard.svelte.ts   # Dashboard data
│   │   │   ├── machines.svelte.ts    # Machines list
│   │   │   └── search.svelte.ts      # Search results
│   │   └── components/
│   │       ├── Sidebar.svelte
│   │       ├── Dashboard.svelte
│   │       ├── ProjectView.svelte
│   │       ├── IssueList.svelte
│   │       ├── IssueRow.svelte
│   │       ├── IssueDetail.svelte
│   │       ├── MachineView.svelte
│   │       ├── SearchModal.svelte
│   │       ├── QuickCapture.svelte
│   │       ├── ActivityFeed.svelte
│   │       ├── CommentThread.svelte
│   │       ├── LabelBadge.svelte
│   │       ├── StatusBadge.svelte
│   │       └── MilestoneBar.svelte
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── src/
│   │   ├── main.rs                   # Entry point
│   │   ├── lib.rs                    # Tauri setup, command registration
│   │   ├── state.rs                  # AppState with Mutex<Connection>
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs            # All CREATE TABLE + FTS5 statements
│   │   │   └── migrations.rs        # Version-based migration runner
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── project.rs
│   │   │   ├── issue.rs
│   │   │   ├── comment.rs
│   │   │   ├── label.rs
│   │   │   ├── milestone.rs
│   │   │   ├── machine.rs
│   │   │   └── activity.rs
│   │   └── commands/
│   │       ├── mod.rs
│   │       ├── projects.rs
│   │       ├── issues.rs
│   │       ├── comments.rs
│   │       ├── labels.rs
│   │       ├── milestones.rs
│   │       ├── machines.rs
│   │       ├── search.rs
│   │       └── activity.rs
│   └── icons/
```

---

## Task 1: Project Scaffold

**Files:**
- Create: `package.json`, `vite.config.ts`, `svelte.config.js`, `tsconfig.json`, `index.html`
- Create: `src/main.ts`, `src/App.svelte`
- Create: `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`
- Create: `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Initialize the Tauri v2 + Svelte project**

Run:
```bash
cd /path/to/dev/ProjectOS
pnpm create tauri-app@latest . --template svelte-ts --manager pnpm
```

If the directory isn't empty (docs already exist), scaffold in a temp dir and move files:
```bash
cd /tmp && pnpm create tauri-app@latest projectos-scaffold --template svelte-ts --manager pnpm
cp -r /tmp/projectos-scaffold/* /path/to/dev/ProjectOS/
cp /tmp/projectos-scaffold/.gitignore /path/to/dev/ProjectOS/
rm -rf /tmp/projectos-scaffold
cd /path/to/dev/ProjectOS
```

- [ ] **Step 2: Install dependencies**

```bash
cd /path/to/dev/ProjectOS
pnpm install
```

- [ ] **Step 3: Add rusqlite to Cargo.toml**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
rusqlite = { version = "0.31", features = ["bundled", "modern_sqlite"] }
uuid = { version = "1", features = ["v4"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 4: Add vitest for frontend testing**

```bash
pnpm add -D vitest @testing-library/svelte jsdom
```

Add to `vite.config.ts`:
```ts
/// <reference types="vitest" />
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  test: {
    environment: "jsdom",
  },
});
```

Add to `package.json` scripts:
```json
"test": "vitest run",
"test:watch": "vitest"
```

- [ ] **Step 5: Verify the scaffold builds**

```bash
cd /path/to/dev/ProjectOS
pnpm build
cd src-tauri && cargo build
```

Expected: Both build successfully.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri v2 + Svelte 5 project"
```

---

## Task 2: Database Schema + State Management

**Files:**
- Create: `src-tauri/src/state.rs`
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/schema.rs`
- Create: `src-tauri/src/db/migrations.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write test for database initialization**

Create `src-tauri/src/db/mod.rs`:
```rust
pub mod schema;
pub mod migrations;

use rusqlite::Connection;
use std::path::Path;

pub fn open_db(path: &Path) -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    schema::create_tables(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_db_creates_all_tables() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        schema::create_tables(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"issues".to_string()));
        assert!(tables.contains(&"issue_comments".to_string()));
        assert!(tables.contains(&"issue_reactions".to_string()));
        assert!(tables.contains(&"issue_deps".to_string()));
        assert!(tables.contains(&"issue_relations".to_string()));
        assert!(tables.contains(&"labels".to_string()));
        assert!(tables.contains(&"issue_labels".to_string()));
        assert!(tables.contains(&"milestones".to_string()));
        assert!(tables.contains(&"assignees".to_string()));
        assert!(tables.contains(&"issue_assignees".to_string()));
        assert!(tables.contains(&"machines".to_string()));
        assert!(tables.contains(&"machine_docs".to_string()));
        assert!(tables.contains(&"issue_templates".to_string()));
        assert!(tables.contains(&"activity_log".to_string()));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test test_open_db_creates_all_tables
```

Expected: FAIL — `schema` module doesn't exist yet.

- [ ] **Step 3: Implement schema**

Create `src-tauri/src/db/schema.rs`:
```rust
use rusqlite::Connection;

pub fn create_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            notes TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS machines (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            hostname TEXT,
            ip TEXT,
            user TEXT,
            os TEXT,
            notes TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS milestones (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            description TEXT,
            due_date INTEGER,
            state TEXT NOT NULL DEFAULT 'open' CHECK(state IN ('open', 'closed')),
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS issues (
            id TEXT PRIMARY KEY,
            project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
            number INTEGER,
            title TEXT NOT NULL,
            body TEXT,
            state TEXT NOT NULL DEFAULT 'open' CHECK(state IN ('open', 'closed')),
            status TEXT CHECK(status IN ('next', 'ready', 'blocked', 'idea') OR status IS NULL),
            sort_order REAL NOT NULL DEFAULT 0.0,
            context TEXT,
            machine_id TEXT REFERENCES machines(id) ON DELETE SET NULL,
            milestone_id TEXT REFERENCES milestones(id) ON DELETE SET NULL,
            locked INTEGER NOT NULL DEFAULT 0,
            pinned INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            closed_at INTEGER
        );

        CREATE INDEX IF NOT EXISTS idx_issues_project_id ON issues(project_id);
        CREATE INDEX IF NOT EXISTS idx_issues_state ON issues(state);
        CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status);
        CREATE INDEX IF NOT EXISTS idx_issues_sort_order ON issues(sort_order);

        CREATE TABLE IF NOT EXISTS issue_comments (
            id TEXT PRIMARY KEY,
            issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            body TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS issue_reactions (
            id TEXT PRIMARY KEY,
            issue_id TEXT REFERENCES issues(id) ON DELETE CASCADE,
            comment_id TEXT REFERENCES issue_comments(id) ON DELETE CASCADE,
            emoji TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            CHECK(issue_id IS NOT NULL OR comment_id IS NOT NULL)
        );

        CREATE TABLE IF NOT EXISTS issue_deps (
            blocker_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            blocked_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            PRIMARY KEY (blocker_id, blocked_id)
        );

        CREATE TABLE IF NOT EXISTS issue_relations (
            issue_a_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            issue_b_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            PRIMARY KEY (issue_a_id, issue_b_id)
        );

        CREATE TABLE IF NOT EXISTS labels (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT NOT NULL,
            project_id TEXT REFERENCES projects(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS issue_labels (
            issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
            PRIMARY KEY (issue_id, label_id)
        );

        CREATE TABLE IF NOT EXISTS assignees (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS issue_assignees (
            issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
            assignee_id TEXT NOT NULL REFERENCES assignees(id) ON DELETE CASCADE,
            PRIMARY KEY (issue_id, assignee_id)
        );

        CREATE TABLE IF NOT EXISTS machine_docs (
            id TEXT PRIMARY KEY,
            machine_id TEXT NOT NULL REFERENCES machines(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            content TEXT,
            url TEXT,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS issue_templates (
            id TEXT PRIMARY KEY,
            project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            body TEXT NOT NULL,
            labels TEXT NOT NULL DEFAULT '[]'
        );

        CREATE TABLE IF NOT EXISTS activity_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            issue_id TEXT REFERENCES issues(id) ON DELETE SET NULL,
            project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
            action TEXT NOT NULL,
            detail TEXT,
            created_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_activity_log_created_at ON activity_log(created_at);
        CREATE INDEX IF NOT EXISTS idx_activity_log_project_id ON activity_log(project_id);
        "
    )
}
```

- [ ] **Step 4: Create empty migrations module**

Create `src-tauri/src/db/migrations.rs`:
```rust
use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL);"
    )?;

    let version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < 1 {
        conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
    }

    Ok(())
}
```

- [ ] **Step 5: Run tests**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test test_open_db_creates_all_tables
```

Expected: PASS

- [ ] **Step 6: Create AppState**

Create `src-tauri/src/state.rs`:
```rust
use rusqlite::Connection;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Connection>,
}
```

- [ ] **Step 7: Wire up state in lib.rs**

Replace `src-tauri/src/lib.rs`:
```rust
mod state;
mod db;

use state::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("projectos.db");
            let conn = db::open_db(&db_path).expect("failed to open database");
            db::migrations::run_migrations(&conn).expect("failed to run migrations");
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 8: Verify it compiles**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo build
```

Expected: Compiles successfully.

- [ ] **Step 9: Commit**

```bash
cd /path/to/dev/ProjectOS
git add src-tauri/src/state.rs src-tauri/src/db/ src-tauri/src/lib.rs
git commit -m "feat: database schema, state management, and migrations"
```

---

## Task 3: Models

**Files:**
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/models/project.rs`
- Create: `src-tauri/src/models/issue.rs`
- Create: `src-tauri/src/models/comment.rs`
- Create: `src-tauri/src/models/label.rs`
- Create: `src-tauri/src/models/milestone.rs`
- Create: `src-tauri/src/models/machine.rs`
- Create: `src-tauri/src/models/activity.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod models`)

- [ ] **Step 1: Create all model structs**

Create `src-tauri/src/models/mod.rs`:
```rust
pub mod project;
pub mod issue;
pub mod comment;
pub mod label;
pub mod milestone;
pub mod machine;
pub mod activity;
```

Create `src-tauri/src/models/project.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateProject {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProject {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
}
```

Create `src-tauri/src/models/issue.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Issue {
    pub id: String,
    pub project_id: Option<String>,
    pub number: Option<i64>,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub status: Option<String>,
    pub sort_order: f64,
    pub context: Option<String>,
    pub machine_id: Option<String>,
    pub milestone_id: Option<String>,
    pub locked: bool,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub closed_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateIssue {
    pub project_id: Option<String>,
    pub title: String,
    pub body: Option<String>,
    pub status: Option<String>,
    pub machine_id: Option<String>,
    pub milestone_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateIssue {
    pub id: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    pub context: Option<String>,
    pub machine_id: Option<String>,
    pub milestone_id: Option<String>,
}
```

Create `src-tauri/src/models/comment.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub issue_id: String,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateComment {
    pub issue_id: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateComment {
    pub id: String,
    pub body: String,
}
```

Create `src-tauri/src/models/label.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLabel {
    pub name: String,
    pub color: String,
    pub project_id: Option<String>,
}
```

Create `src-tauri/src/models/milestone.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Milestone {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<i64>,
    pub state: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub open_count: Option<i64>,
    pub closed_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMilestone {
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<i64>,
}
```

Create `src-tauri/src/models/machine.rs`:
```rust
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
```

Create `src-tauri/src/models/activity.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActivityEntry {
    pub id: i64,
    pub issue_id: Option<String>,
    pub project_id: Option<String>,
    pub action: String,
    pub detail: Option<String>,
    pub created_at: i64,
}
```

- [ ] **Step 2: Add `mod models` to lib.rs**

Add `mod models;` to the top of `src-tauri/src/lib.rs` alongside the existing mod declarations.

- [ ] **Step 3: Verify it compiles**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo build
```

Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
cd /path/to/dev/ProjectOS
git add src-tauri/src/models/ src-tauri/src/lib.rs
git commit -m "feat: add all data model structs"
```

---

## Task 4: Project Commands

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/projects.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write tests for project CRUD**

Create `src-tauri/src/commands/mod.rs`:
```rust
pub mod projects;
```

Create `src-tauri/src/commands/projects.rs`:
```rust
use crate::models::project::{CreateProject, Project, UpdateProject};
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
pub fn create_project(state: State<AppState>, input: CreateProject) -> Result<Project, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    db.execute(
        "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
        rusqlite::params![id, input.name, input.description, now, now],
    ).map_err(|e| e.to_string())?;

    Ok(Project {
        id,
        name: input.name,
        description: input.description,
        notes: None,
        created_at: now,
        updated_at: now,
    })
}

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, name, description, notes, created_at, updated_at FROM projects ORDER BY name")
        .map_err(|e| e.to_string())?;
    let projects = stmt
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                notes: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(projects)
}

#[tauri::command]
pub fn update_project(state: State<AppState>, input: UpdateProject) -> Result<Project, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    let existing: Project = db
        .query_row(
            "SELECT id, name, description, notes, created_at, updated_at FROM projects WHERE id = ?1",
            [&input.id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    notes: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| e.to_string())?;

    let name = input.name.unwrap_or(existing.name);
    let description = input.description.or(existing.description);
    let notes = input.notes.or(existing.notes);

    db.execute(
        "UPDATE projects SET name = ?1, description = ?2, notes = ?3, updated_at = ?4 WHERE id = ?5",
        rusqlite::params![name, description, notes, now, input.id],
    ).map_err(|e| e.to_string())?;

    Ok(Project {
        id: input.id,
        name,
        description,
        notes,
        created_at: existing.created_at,
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM projects WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
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
    fn test_create_and_list_projects() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        db.execute(
            "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'ProjectOS', 'Test', NULL, 1000, 1000)",
            [],
        ).unwrap();

        let mut stmt = db
            .prepare("SELECT id, name FROM projects")
            .unwrap();
        let names: Vec<String> = stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert_eq!(names, vec!["ProjectOS"]);
    }

    #[test]
    fn test_update_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        db.execute(
            "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'Old', NULL, NULL, 1000, 1000)",
            [],
        ).unwrap();

        db.execute(
            "UPDATE projects SET name = 'New', updated_at = 2000 WHERE id = 'p1'",
            [],
        ).unwrap();

        let name: String = db.query_row("SELECT name FROM projects WHERE id = 'p1'", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "New");
    }

    #[test]
    fn test_delete_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        db.execute(
            "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'Test', NULL, NULL, 1000, 1000)",
            [],
        ).unwrap();

        db.execute("DELETE FROM projects WHERE id = 'p1'", []).unwrap();

        let count: i64 = db.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test commands::projects
```

Expected: All 3 tests PASS.

- [ ] **Step 3: Register commands in lib.rs**

Update `src-tauri/src/lib.rs` to add `mod commands;` and register the project commands:

```rust
mod state;
mod db;
mod models;
mod commands;

use state::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("projectos.db");
            let conn = db::open_db(&db_path).expect("failed to open database");
            db::migrations::run_migrations(&conn).expect("failed to run migrations");
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::projects::create_project,
            commands::projects::list_projects,
            commands::projects::update_project,
            commands::projects::delete_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo build
```

Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/ src-tauri/src/lib.rs
git commit -m "feat: project CRUD commands with tests"
```

---

## Task 5: Issue Commands

**Files:**
- Create: `src-tauri/src/commands/issues.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write tests for issue CRUD**

Create `src-tauri/src/commands/issues.rs`:
```rust
use crate::models::issue::{CreateIssue, Issue, UpdateIssue};
use crate::state::AppState;
use tauri::State;
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

fn row_to_issue(row: &rusqlite::Row) -> Result<Issue, rusqlite::Error> {
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

const ISSUE_COLUMNS: &str = "id, project_id, number, title, body, state, status, sort_order, context, machine_id, milestone_id, locked, pinned, created_at, updated_at, closed_at";

#[tauri::command]
pub fn create_issue(state: State<AppState>, input: CreateIssue) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    let number = match &input.project_id {
        Some(pid) => Some(next_issue_number(&db, pid).map_err(|e| e.to_string())?),
        None => None,
    };
    let sort_order = next_sort_order(&db, input.project_id.as_deref()).map_err(|e| e.to_string())?;
    let status = input.status.as_deref().unwrap_or("idea");

    db.execute(
        "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, context, machine_id, milestone_id, locked, pinned, created_at, updated_at, closed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'open', ?6, ?7, NULL, ?8, ?9, 0, 0, ?10, ?11, NULL)",
        rusqlite::params![id, input.project_id, number, input.title, input.body, status, sort_order, input.machine_id, input.milestone_id, now, now],
    ).map_err(|e| e.to_string())?;

    // Log activity
    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, ?2, 'created', ?3, ?4)",
        rusqlite::params![id, input.project_id, serde_json::json!({"title": input.title}).to_string(), now],
    ).map_err(|e| e.to_string())?;

    Ok(Issue {
        id,
        project_id: input.project_id,
        number,
        title: input.title,
        body: input.body,
        state: "open".to_string(),
        status: Some(status.to_string()),
        sort_order,
        context: None,
        machine_id: input.machine_id,
        milestone_id: input.milestone_id,
        locked: false,
        pinned: false,
        created_at: now,
        updated_at: now,
        closed_at: None,
    })
}

#[tauri::command]
pub fn list_issues(state: State<AppState>, project_id: Option<String>, include_closed: Option<bool>) -> Result<Vec<Issue>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let include_closed = include_closed.unwrap_or(false);

    let sql = match (&project_id, include_closed) {
        (Some(_), false) => format!("SELECT {} FROM issues WHERE project_id = ?1 AND state = 'open' ORDER BY pinned DESC, sort_order ASC", ISSUE_COLUMNS),
        (Some(_), true) => format!("SELECT {} FROM issues WHERE project_id = ?1 ORDER BY pinned DESC, state ASC, sort_order ASC", ISSUE_COLUMNS),
        (None, false) => format!("SELECT {} FROM issues WHERE project_id IS NULL AND state = 'open' ORDER BY sort_order ASC", ISSUE_COLUMNS),
        (None, true) => format!("SELECT {} FROM issues WHERE project_id IS NULL ORDER BY state ASC, sort_order ASC", ISSUE_COLUMNS),
    };

    let mut stmt = db.prepare(&sql).map_err(|e| e.to_string())?;

    let issues = if project_id.is_some() {
        stmt.query_map([project_id.as_ref().unwrap()], |row| row_to_issue(row))
    } else {
        stmt.query_map([], |row| row_to_issue(row))
    }.map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(issues)
}

#[tauri::command]
pub fn get_issue(state: State<AppState>, id: String) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let sql = format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS);
    db.query_row(&sql, [&id], |row| row_to_issue(row))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_issue(state: State<AppState>, input: UpdateIssue) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    let sql = format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS);
    let existing = db.query_row(&sql, [&input.id], |row| row_to_issue(row)).map_err(|e| e.to_string())?;

    let title = input.title.unwrap_or(existing.title);
    let body = input.body.or(existing.body);
    let status = input.status.or(existing.status);
    let context = input.context.or(existing.context);
    let machine_id = input.machine_id.or(existing.machine_id);
    let milestone_id = input.milestone_id.or(existing.milestone_id);

    // Enforce: only one 'next' per project
    if status.as_deref() == Some("next") {
        if let Some(ref pid) = existing.project_id {
            db.execute(
                "UPDATE issues SET status = 'ready' WHERE project_id = ?1 AND status = 'next' AND id != ?2",
                rusqlite::params![pid, input.id],
            ).map_err(|e| e.to_string())?;
        }
    }

    db.execute(
        "UPDATE issues SET title=?1, body=?2, status=?3, context=?4, machine_id=?5, milestone_id=?6, updated_at=?7 WHERE id=?8",
        rusqlite::params![title, body, status, context, machine_id, milestone_id, now, input.id],
    ).map_err(|e| e.to_string())?;

    let sql = format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS);
    db.query_row(&sql, [&input.id], |row| row_to_issue(row)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_issue(state: State<AppState>, id: String) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    db.execute(
        "UPDATE issues SET state = 'closed', status = NULL, closed_at = ?1, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, id],
    ).map_err(|e| e.to_string())?;

    // Re-evaluate blocked issues that depended on this one
    let blocked_ids: Vec<String> = {
        let mut stmt = db.prepare("SELECT blocked_id FROM issue_deps WHERE blocker_id = ?1").map_err(|e| e.to_string())?;
        stmt.query_map([&id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect()
    };

    for blocked_id in blocked_ids {
        let still_blocked: i64 = db.query_row(
            "SELECT COUNT(*) FROM issue_deps d JOIN issues i ON d.blocker_id = i.id WHERE d.blocked_id = ?1 AND i.state = 'open'",
            [&blocked_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;

        if still_blocked == 0 {
            db.execute(
                "UPDATE issues SET status = 'ready' WHERE id = ?1 AND status = 'blocked'",
                [&blocked_id],
            ).map_err(|e| e.to_string())?;
        }
    }

    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, (SELECT project_id FROM issues WHERE id = ?1), 'closed', NULL, ?2)",
        rusqlite::params![id, now],
    ).map_err(|e| e.to_string())?;

    let sql = format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS);
    db.query_row(&sql, [&id], |row| row_to_issue(row)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reopen_issue(state: State<AppState>, id: String) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    db.execute(
        "UPDATE issues SET state = 'open', status = 'ready', closed_at = NULL, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, id],
    ).map_err(|e| e.to_string())?;

    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, (SELECT project_id FROM issues WHERE id = ?1), 'reopened', NULL, ?2)",
        rusqlite::params![id, now],
    ).map_err(|e| e.to_string())?;

    let sql = format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS);
    db.query_row(&sql, [&id], |row| row_to_issue(row)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_issue(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM issues WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn reorder_issue(state: State<AppState>, id: String, new_sort_order: f64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    db.execute(
        "UPDATE issues SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_sort_order, now, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn transfer_issue(state: State<AppState>, id: String, new_project_id: String) -> Result<Issue, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    let new_number = next_issue_number(&db, &new_project_id).map_err(|e| e.to_string())?;

    let old_project_id: Option<String> = db.query_row(
        "SELECT project_id FROM issues WHERE id = ?1", [&id], |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    db.execute(
        "UPDATE issues SET project_id = ?1, number = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![new_project_id, new_number, now, id],
    ).map_err(|e| e.to_string())?;

    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, ?2, 'transferred', ?3, ?4)",
        rusqlite::params![id, new_project_id, serde_json::json!({"from_project": old_project_id, "to_project": new_project_id}).to_string(), now],
    ).map_err(|e| e.to_string())?;

    let sql = format!("SELECT {} FROM issues WHERE id = ?1", ISSUE_COLUMNS);
    db.query_row(&sql, [&id], |row| row_to_issue(row)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn promote_idea(state: State<AppState>, issue_id: String) -> Result<crate::models::project::Project, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();

    let (title, body): (String, Option<String>) = db.query_row(
        "SELECT title, body FROM issues WHERE id = ?1", [&issue_id], |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|e| e.to_string())?;

    let project_id = Uuid::new_v4().to_string();
    db.execute(
        "INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES (?1, ?2, ?3, NULL, ?4, ?5)",
        rusqlite::params![project_id, title, body, now, now],
    ).map_err(|e| e.to_string())?;

    db.execute(
        "UPDATE issues SET state = 'closed', status = NULL, closed_at = ?1, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, issue_id],
    ).map_err(|e| e.to_string())?;

    db.execute(
        "INSERT INTO activity_log (issue_id, project_id, action, detail, created_at) VALUES (?1, ?2, 'promoted', ?3, ?4)",
        rusqlite::params![issue_id, project_id, serde_json::json!({"new_project": title}).to_string(), now],
    ).map_err(|e| e.to_string())?;

    Ok(crate::models::project::Project {
        id: project_id,
        name: title,
        description: body,
        notes: None,
        created_at: now,
        updated_at: now,
    })
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
    fn test_create_issue_with_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'Test', NULL, NULL, 1000, 1000)", []).unwrap();

        db.execute(
            "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, created_at, updated_at) VALUES ('i1', 'p1', 1, 'First', NULL, 'open', 'idea', 1.0, 1000, 1000)",
            [],
        ).unwrap();

        let number: i64 = db.query_row("SELECT number FROM issues WHERE id = 'i1'", [], |r| r.get(0)).unwrap();
        assert_eq!(number, 1);
    }

    #[test]
    fn test_create_issue_without_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        db.execute(
            "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, created_at, updated_at) VALUES ('i1', NULL, NULL, 'Idea', NULL, 'open', 'idea', 1.0, 1000, 1000)",
            [],
        ).unwrap();

        let project_id: Option<String> = db.query_row("SELECT project_id FROM issues WHERE id = 'i1'", [], |r| r.get(0)).unwrap();
        assert!(project_id.is_none());
    }

    #[test]
    fn test_close_issue_unblocks_dependents() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'Test', NULL, NULL, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issues (id, project_id, number, title, state, status, sort_order, created_at, updated_at) VALUES ('blocker', 'p1', 1, 'Blocker', 'open', 'next', 1.0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issues (id, project_id, number, title, state, status, sort_order, created_at, updated_at) VALUES ('blocked', 'p1', 2, 'Blocked', 'open', 'blocked', 2.0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issue_deps (blocker_id, blocked_id) VALUES ('blocker', 'blocked')", []).unwrap();

        // Close the blocker
        db.execute("UPDATE issues SET state = 'closed', status = NULL WHERE id = 'blocker'", []).unwrap();

        // Check if blocked still has open blockers
        let still_blocked: i64 = db.query_row(
            "SELECT COUNT(*) FROM issue_deps d JOIN issues i ON d.blocker_id = i.id WHERE d.blocked_id = 'blocked' AND i.state = 'open'",
            [], |r| r.get(0),
        ).unwrap();

        if still_blocked == 0 {
            db.execute("UPDATE issues SET status = 'ready' WHERE id = 'blocked' AND status = 'blocked'", []).unwrap();
        }

        let status: String = db.query_row("SELECT status FROM issues WHERE id = 'blocked'", [], |r| r.get(0)).unwrap();
        assert_eq!(status, "ready");
    }

    #[test]
    fn test_only_one_next_per_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('p1', 'Test', NULL, NULL, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issues (id, project_id, number, title, state, status, sort_order, created_at, updated_at) VALUES ('i1', 'p1', 1, 'First', 'open', 'next', 1.0, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO issues (id, project_id, number, title, state, status, sort_order, created_at, updated_at) VALUES ('i2', 'p1', 2, 'Second', 'open', 'ready', 2.0, 1000, 1000)", []).unwrap();

        // Set i2 to next, should clear i1
        db.execute("UPDATE issues SET status = 'ready' WHERE project_id = 'p1' AND status = 'next' AND id != 'i2'", []).unwrap();
        db.execute("UPDATE issues SET status = 'next' WHERE id = 'i2'", []).unwrap();

        let i1_status: String = db.query_row("SELECT status FROM issues WHERE id = 'i1'", [], |r| r.get(0)).unwrap();
        let i2_status: String = db.query_row("SELECT status FROM issues WHERE id = 'i2'", [], |r| r.get(0)).unwrap();
        assert_eq!(i1_status, "ready");
        assert_eq!(i2_status, "next");
    }

    #[test]
    fn test_promote_idea_to_project() {
        let state = test_state();
        let db = state.db.lock().unwrap();

        db.execute(
            "INSERT INTO issues (id, project_id, number, title, body, state, status, sort_order, created_at, updated_at) VALUES ('idea1', NULL, NULL, 'Cool App', 'Description of cool app', 'open', 'idea', 1.0, 1000, 1000)",
            [],
        ).unwrap();

        // Promote: create project, close issue
        db.execute("INSERT INTO projects (id, name, description, notes, created_at, updated_at) VALUES ('new_proj', 'Cool App', 'Description of cool app', NULL, 2000, 2000)", []).unwrap();
        db.execute("UPDATE issues SET state = 'closed', status = NULL, closed_at = 2000 WHERE id = 'idea1'", []).unwrap();

        let project_name: String = db.query_row("SELECT name FROM projects WHERE id = 'new_proj'", [], |r| r.get(0)).unwrap();
        let issue_state: String = db.query_row("SELECT state FROM issues WHERE id = 'idea1'", [], |r| r.get(0)).unwrap();
        assert_eq!(project_name, "Cool App");
        assert_eq!(issue_state, "closed");
    }
}
```

- [ ] **Step 2: Update commands/mod.rs**

```rust
pub mod projects;
pub mod issues;
```

- [ ] **Step 3: Run tests**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test commands::issues
```

Expected: All 5 tests PASS.

- [ ] **Step 4: Register issue commands in lib.rs**

Add to the `invoke_handler` macro in `lib.rs`:
```rust
commands::issues::create_issue,
commands::issues::list_issues,
commands::issues::get_issue,
commands::issues::update_issue,
commands::issues::close_issue,
commands::issues::reopen_issue,
commands::issues::delete_issue,
commands::issues::reorder_issue,
commands::issues::transfer_issue,
commands::issues::promote_idea,
```

- [ ] **Step 5: Verify it compiles**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo build
```

- [ ] **Step 6: Commit**

```bash
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/issues.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: issue CRUD, ordering, transfer, and promotion commands"
```

---

## Task 6: Comments + Reactions Commands

**Files:**
- Create: `src-tauri/src/commands/comments.rs`
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Implement comments and reactions commands**

Create `src-tauri/src/commands/comments.rs`:
```rust
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
```

- [ ] **Step 2: Update commands/mod.rs**

Add `pub mod comments;`

- [ ] **Step 3: Run tests**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test commands::comments
```

Expected: All 3 tests PASS.

- [ ] **Step 4: Register in lib.rs**

Add to `invoke_handler`:
```rust
commands::comments::create_comment,
commands::comments::list_comments,
commands::comments::update_comment,
commands::comments::delete_comment,
commands::comments::add_reaction,
commands::comments::remove_reaction,
commands::comments::list_reactions,
```

- [ ] **Step 5: Verify and commit**

```bash
cd /path/to/dev/ProjectOS/src-tauri && cargo build
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/comments.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: comments and reactions commands"
```

---

## Task 7: Labels Commands

**Files:**
- Create: `src-tauri/src/commands/labels.rs`
- Modify: `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Implement labels commands with tests**

Create `src-tauri/src/commands/labels.rs`:
```rust
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
```

- [ ] **Step 2: Update mod.rs, run tests, register, compile, commit**

Add `pub mod labels;` to `commands/mod.rs`. Register all label commands in `lib.rs`.

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test commands::labels && cargo build
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/labels.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: label commands"
```

---

## Task 8: Milestones Commands

**Files:**
- Create: `src-tauri/src/commands/milestones.rs`
- Modify: `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Implement milestone commands with tests**

Create `src-tauri/src/commands/milestones.rs`:
```rust
use crate::models::milestone::{CreateMilestone, Milestone};
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
}

fn row_to_milestone(row: &rusqlite::Row) -> Result<Milestone, rusqlite::Error> {
    Ok(Milestone {
        id: row.get(0)?,
        project_id: row.get(1)?,
        title: row.get(2)?,
        description: row.get(3)?,
        due_date: row.get(4)?,
        state: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        open_count: None,
        closed_count: None,
    })
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
```

- [ ] **Step 2: Update mod.rs, register, test, compile, commit**

Add `pub mod milestones;` to `commands/mod.rs`. Register all milestone commands.

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test commands::milestones && cargo build
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/milestones.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: milestone commands"
```

---

## Task 9: Dependencies + Relations + Assignees

**Files:**
- Modify: `src-tauri/src/commands/issues.rs` (add dep/relation/assignee commands)

- [ ] **Step 1: Add dependency, relation, and assignee commands to issues.rs**

Append to `src-tauri/src/commands/issues.rs`:
```rust
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
```

- [ ] **Step 2: Add tests for deps/relations/assignees**

Append to the `tests` module in `issues.rs`:
```rust
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
```

- [ ] **Step 3: Run tests, register commands, compile, commit**

Register new commands in `lib.rs`:
```rust
commands::issues::add_dependency,
commands::issues::remove_dependency,
commands::issues::add_relation,
commands::issues::remove_relation,
commands::issues::assign_issue,
commands::issues::unassign_issue,
commands::issues::get_issue_assignees,
```

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test commands::issues && cargo build
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/issues.rs src-tauri/src/lib.rs
git commit -m "feat: dependency, relation, and assignee commands"
```

---

## Task 10: Machine Commands

**Files:**
- Create: `src-tauri/src/commands/machines.rs`
- Modify: `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Implement machine + machine_docs commands with tests**

Create `src-tauri/src/commands/machines.rs`:
```rust
use crate::models::machine::{CreateMachine, CreateMachineDoc, Machine, MachineDoc, UpdateMachine};
use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
}

#[tauri::command]
pub fn create_machine(state: State<AppState>, input: CreateMachine) -> Result<Machine, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    db.execute(
        "INSERT INTO machines (id, name, hostname, ip, user, os, notes, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![id, input.name, input.hostname, input.ip, input.user, input.os, input.notes, now, now],
    ).map_err(|e| e.to_string())?;
    Ok(Machine { id, name: input.name, hostname: input.hostname, ip: input.ip, user: input.user, os: input.os, notes: input.notes, created_at: now, updated_at: now })
}

#[tauri::command]
pub fn list_machines(state: State<AppState>) -> Result<Vec<Machine>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare("SELECT id, name, hostname, ip, user, os, notes, created_at, updated_at FROM machines ORDER BY name").map_err(|e| e.to_string())?;
    let machines = stmt.query_map([], |row| {
        Ok(Machine { id: row.get(0)?, name: row.get(1)?, hostname: row.get(2)?, ip: row.get(3)?, user: row.get(4)?, os: row.get(5)?, notes: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)? })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(machines)
}

#[tauri::command]
pub fn update_machine(state: State<AppState>, input: UpdateMachine) -> Result<Machine, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = now_ms();
    let existing = db.query_row(
        "SELECT id, name, hostname, ip, user, os, notes, created_at, updated_at FROM machines WHERE id = ?1",
        [&input.id], |row| Ok(Machine { id: row.get(0)?, name: row.get(1)?, hostname: row.get(2)?, ip: row.get(3)?, user: row.get(4)?, os: row.get(5)?, notes: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)? }),
    ).map_err(|e| e.to_string())?;

    let name = input.name.unwrap_or(existing.name);
    let hostname = input.hostname.or(existing.hostname);
    let ip = input.ip.or(existing.ip);
    let user = input.user.or(existing.user);
    let os = input.os.or(existing.os);
    let notes = input.notes.or(existing.notes);

    db.execute(
        "UPDATE machines SET name=?1, hostname=?2, ip=?3, user=?4, os=?5, notes=?6, updated_at=?7 WHERE id=?8",
        rusqlite::params![name, hostname, ip, user, os, notes, now, input.id],
    ).map_err(|e| e.to_string())?;

    Ok(Machine { id: input.id, name, hostname, ip, user, os, notes, created_at: existing.created_at, updated_at: now })
}

#[tauri::command]
pub fn delete_machine(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM machines WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_current_machine(state: State<AppState>) -> Result<Machine, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let hostname = hostname::get().map(|h| h.to_string_lossy().to_string()).unwrap_or_default();

    match db.query_row(
        "SELECT id, name, hostname, ip, user, os, notes, created_at, updated_at FROM machines WHERE hostname = ?1",
        [&hostname], |row| Ok(Machine { id: row.get(0)?, name: row.get(1)?, hostname: row.get(2)?, ip: row.get(3)?, user: row.get(4)?, os: row.get(5)?, notes: row.get(6)?, created_at: row.get(7)?, updated_at: row.get(8)? }),
    ) {
        Ok(m) => Ok(m),
        Err(_) => {
            // Auto-create machine entry for current host
            let id = Uuid::new_v4().to_string();
            let now = now_ms();
            let name = hostname.clone();
            db.execute(
                "INSERT INTO machines (id, name, hostname, ip, user, os, notes, created_at, updated_at) VALUES (?1, ?2, ?3, NULL, NULL, ?4, NULL, ?5, ?6)",
                rusqlite::params![id, name, hostname, std::env::consts::OS, now, now],
            ).map_err(|e| e.to_string())?;
            Ok(Machine { id, name, hostname: Some(hostname), ip: None, user: None, os: Some(std::env::consts::OS.to_string()), notes: None, created_at: now, updated_at: now })
        }
    }
}

#[tauri::command]
pub fn create_machine_doc(state: State<AppState>, input: CreateMachineDoc) -> Result<MachineDoc, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = now_ms();
    db.execute(
        "INSERT INTO machine_docs (id, machine_id, title, content, url, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, input.machine_id, input.title, input.content, input.url, now],
    ).map_err(|e| e.to_string())?;
    Ok(MachineDoc { id, machine_id: input.machine_id, title: input.title, content: input.content, url: input.url, created_at: now })
}

#[tauri::command]
pub fn list_machine_docs(state: State<AppState>, machine_id: String) -> Result<Vec<MachineDoc>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare("SELECT id, machine_id, title, content, url, created_at FROM machine_docs WHERE machine_id = ?1 ORDER BY created_at DESC").map_err(|e| e.to_string())?;
    let docs = stmt.query_map([&machine_id], |row| {
        Ok(MachineDoc { id: row.get(0)?, machine_id: row.get(1)?, title: row.get(2)?, content: row.get(3)?, url: row.get(4)?, created_at: row.get(5)? })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    Ok(docs)
}

#[tauri::command]
pub fn update_machine_doc(state: State<AppState>, id: String, title: Option<String>, content: Option<String>, url: Option<String>) -> Result<MachineDoc, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let existing = db.query_row(
        "SELECT id, machine_id, title, content, url, created_at FROM machine_docs WHERE id = ?1",
        [&id], |row| Ok(MachineDoc { id: row.get(0)?, machine_id: row.get(1)?, title: row.get(2)?, content: row.get(3)?, url: row.get(4)?, created_at: row.get(5)? }),
    ).map_err(|e| e.to_string())?;
    let title = title.unwrap_or(existing.title);
    let content = content.or(existing.content);
    let url = url.or(existing.url);
    db.execute("UPDATE machine_docs SET title=?1, content=?2, url=?3 WHERE id=?4", rusqlite::params![title, content, url, id]).map_err(|e| e.to_string())?;
    Ok(MachineDoc { id, machine_id: existing.machine_id, title, content, url, created_at: existing.created_at })
}

#[tauri::command]
pub fn delete_machine_doc(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM machine_docs WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
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
    fn test_machine_crud() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO machines (id, name, hostname, ip, user, os, notes, created_at, updated_at) VALUES ('m1', 'homelab', 'server', '192.168.1.50', 'root', 'Ubuntu', NULL, 1000, 1000)", []).unwrap();

        let name: String = db.query_row("SELECT name FROM machines WHERE id = 'm1'", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "homelab");

        let ip: String = db.query_row("SELECT ip FROM machines WHERE id = 'm1'", [], |r| r.get(0)).unwrap();
        assert_eq!(ip, "192.168.1.50");
    }

    #[test]
    fn test_machine_docs() {
        let state = test_state();
        let db = state.db.lock().unwrap();
        db.execute("INSERT INTO machines (id, name, hostname, ip, user, os, notes, created_at, updated_at) VALUES ('m1', 'homelab', NULL, NULL, NULL, NULL, NULL, 1000, 1000)", []).unwrap();
        db.execute("INSERT INTO machine_docs (id, machine_id, title, content, url, created_at) VALUES ('d1', 'm1', 'SSH Config', 'ssh root@192.168.1.50', NULL, 1000)", []).unwrap();

        let title: String = db.query_row("SELECT title FROM machine_docs WHERE machine_id = 'm1'", [], |r| r.get(0)).unwrap();
        assert_eq!(title, "SSH Config");
    }
}
```

- [ ] **Step 2: Add `hostname` crate to Cargo.toml**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:
```toml
hostname = "0.4"
```

- [ ] **Step 3: Update mod.rs, register, test, compile, commit**

Add `pub mod machines;` to `commands/mod.rs`. Register all machine commands in `lib.rs`.

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test commands::machines && cargo build
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/machines.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: machine and machine docs commands with auto-detection"
```

---

## Task 11: Activity Log + Search Commands

**Files:**
- Create: `src-tauri/src/commands/activity.rs`
- Create: `src-tauri/src/commands/search.rs`
- Modify: `src-tauri/src/db/schema.rs` (add FTS5 table)
- Modify: `src-tauri/src/commands/mod.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: Add FTS5 virtual table to schema.rs**

Append to the `create_tables` function in `schema.rs`, after the `activity_log` index:
```sql
        CREATE VIRTUAL TABLE IF NOT EXISTS issues_fts USING fts5(
            title, body, content='issues', content_rowid='rowid'
        );

        CREATE TRIGGER IF NOT EXISTS issues_fts_insert AFTER INSERT ON issues BEGIN
            INSERT INTO issues_fts(rowid, title, body) VALUES (new.rowid, new.title, new.body);
        END;

        CREATE TRIGGER IF NOT EXISTS issues_fts_delete AFTER DELETE ON issues BEGIN
            INSERT INTO issues_fts(issues_fts, rowid, title, body) VALUES('delete', old.rowid, old.title, old.body);
        END;

        CREATE TRIGGER IF NOT EXISTS issues_fts_update AFTER UPDATE ON issues BEGIN
            INSERT INTO issues_fts(issues_fts, rowid, title, body) VALUES('delete', old.rowid, old.title, old.body);
            INSERT INTO issues_fts(rowid, title, body) VALUES (new.rowid, new.title, new.body);
        END;
```

- [ ] **Step 2: Implement activity log command**

Create `src-tauri/src/commands/activity.rs`:
```rust
use crate::models::activity::ActivityEntry;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub fn get_activity_log(state: State<AppState>, project_id: Option<String>, limit: Option<i64>) -> Result<Vec<ActivityEntry>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let limit = limit.unwrap_or(50);

    let (sql, param) = match &project_id {
        Some(pid) => (
            "SELECT id, issue_id, project_id, action, detail, created_at FROM activity_log WHERE project_id = ?1 ORDER BY created_at DESC LIMIT ?2",
            vec![pid.clone(), limit.to_string()],
        ),
        None => (
            "SELECT id, issue_id, project_id, action, detail, created_at FROM activity_log ORDER BY created_at DESC LIMIT ?1",
            vec![limit.to_string()],
        ),
    };

    let mut stmt = db.prepare(sql).map_err(|e| e.to_string())?;
    let entries = if project_id.is_some() {
        stmt.query_map(rusqlite::params![param[0], param[1].parse::<i64>().unwrap()], |row| {
            Ok(ActivityEntry { id: row.get(0)?, issue_id: row.get(1)?, project_id: row.get(2)?, action: row.get(3)?, detail: row.get(4)?, created_at: row.get(5)? })
        })
    } else {
        stmt.query_map(rusqlite::params![param[0].parse::<i64>().unwrap()], |row| {
            Ok(ActivityEntry { id: row.get(0)?, issue_id: row.get(1)?, project_id: row.get(2)?, action: row.get(3)?, detail: row.get(4)?, created_at: row.get(5)? })
        })
    }.map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    Ok(entries)
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
```

- [ ] **Step 3: Implement search command**

Create `src-tauri/src/commands/search.rs`:
```rust
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
        "SELECT i.id, i.project_id, i.number, i.title, i.body, i.state, i.status, i.sort_order, i.context, i.machine_id, i.milestone_id, i.locked, i.pinned, i.created_at, i.updated_at, i.closed_at
         FROM issues i JOIN issues_fts f ON i.rowid = f.rowid
         WHERE issues_fts MATCH ?1
         ORDER BY rank LIMIT 20"
    ).map_err(|e| e.to_string())?;

    let issues: Vec<Issue> = stmt.query_map([&fts_query], |row| {
        Ok(Issue {
            id: row.get(0)?, project_id: row.get(1)?, number: row.get(2)?, title: row.get(3)?,
            body: row.get(4)?, state: row.get(5)?, status: row.get(6)?, sort_order: row.get(7)?,
            context: row.get(8)?, machine_id: row.get(9)?, milestone_id: row.get(10)?,
            locked: row.get::<_, i64>(11).map(|v| v != 0)?, pinned: row.get::<_, i64>(12).map(|v| v != 0)?,
            created_at: row.get(13)?, updated_at: row.get(14)?, closed_at: row.get(15)?,
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // LIKE search on projects
    let mut stmt = db.prepare(
        "SELECT id, name, description, notes, created_at, updated_at FROM projects WHERE name LIKE ?1 LIMIT 10"
    ).map_err(|e| e.to_string())?;
    let projects = stmt.query_map([&like_query], |row| {
        Ok(crate::models::project::Project { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, notes: row.get(3)?, created_at: row.get(4)?, updated_at: row.get(5)? })
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

        let mut stmt = db.prepare("SELECT title FROM issues JOIN issues_fts ON issues.rowid = issues_fts.rowid WHERE issues_fts MATCH 'login*' LIMIT 5").unwrap();
        let titles: Vec<String> = stmt.query_map([], |r| r.get(0)).unwrap().filter_map(|r| r.ok()).collect();
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0], "Fix login bug");
    }
}
```

- [ ] **Step 4: Update mod.rs, register, test, compile, commit**

Add `pub mod activity;` and `pub mod search;` to `commands/mod.rs`. Register `get_activity_log` and `search_issues` in `lib.rs`.

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo test && cargo build
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/activity.rs src-tauri/src/commands/search.rs src-tauri/src/commands/mod.rs src-tauri/src/db/schema.rs src-tauri/src/lib.rs
git commit -m "feat: activity log, FTS5 search, and dashboard query"
```

---

## Task 12: Dashboard Command

**Files:**
- Modify: `src-tauri/src/commands/projects.rs`

- [ ] **Step 1: Add get_dashboard command**

Append to `src-tauri/src/commands/projects.rs`:
```rust
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
```

- [ ] **Step 2: Make `row_to_issue` and `ISSUE_COLUMNS` public in issues.rs**

Change in `src-tauri/src/commands/issues.rs`:
```rust
pub const ISSUE_COLUMNS: &str = ...
pub fn row_to_issue(row: &rusqlite::Row) -> Result<Issue, rusqlite::Error> {
```

- [ ] **Step 3: Register, compile, commit**

Add `commands::projects::get_dashboard` to the `invoke_handler`. 

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo build
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/projects.rs src-tauri/src/commands/issues.rs src-tauri/src/lib.rs
git commit -m "feat: dashboard query combining project nexts and activity"
```

---

## Task 13: Frontend Types + Command Wrappers + Event Listeners

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/commands.ts`
- Create: `src/lib/events.ts`

- [ ] **Step 1: Define all TypeScript types**

Create `src/lib/types.ts`:
```ts
export interface Project {
  id: string;
  name: string;
  description: string | null;
  notes: string | null;
  created_at: number;
  updated_at: number;
}

export interface Issue {
  id: string;
  project_id: string | null;
  number: number | null;
  title: string;
  body: string | null;
  state: "open" | "closed";
  status: "next" | "ready" | "blocked" | "idea" | null;
  sort_order: number;
  context: string | null;
  machine_id: string | null;
  milestone_id: string | null;
  locked: boolean;
  pinned: boolean;
  created_at: number;
  updated_at: number;
  closed_at: number | null;
}

export interface Comment {
  id: string;
  issue_id: string;
  body: string;
  created_at: number;
  updated_at: number;
}

export interface Label {
  id: string;
  name: string;
  color: string;
  project_id: string | null;
}

export interface Milestone {
  id: string;
  project_id: string;
  title: string;
  description: string | null;
  due_date: number | null;
  state: "open" | "closed";
  created_at: number;
  updated_at: number;
  open_count: number | null;
  closed_count: number | null;
}

export interface Machine {
  id: string;
  name: string;
  hostname: string | null;
  ip: string | null;
  user: string | null;
  os: string | null;
  notes: string | null;
  created_at: number;
  updated_at: number;
}

export interface MachineDoc {
  id: string;
  machine_id: string;
  title: string;
  content: string | null;
  url: string | null;
  created_at: number;
}

export interface ActivityEntry {
  id: number;
  issue_id: string | null;
  project_id: string | null;
  action: string;
  detail: string | null;
  created_at: number;
}

export interface ReactionGroup {
  emoji: string;
  count: number;
  ids: string[];
}

export interface DashboardProject {
  project: Project;
  next_issue: Issue | null;
  open_count: number;
}

export interface Dashboard {
  projects: DashboardProject[];
  recent_activity: ActivityEntry[];
}

export interface SearchResults {
  issues: Issue[];
  projects: Project[];
  machines: Machine[];
}

export type View =
  | { kind: "home" }
  | { kind: "project"; projectId: string }
  | { kind: "issue"; issueId: string }
  | { kind: "machine"; machineId: string };
```

- [ ] **Step 2: Create command wrappers**

Create `src/lib/commands.ts`:
```ts
import { invoke } from "@tauri-apps/api/core";
import type {
  Project, Issue, Comment, Label, Milestone, Machine, MachineDoc,
  ActivityEntry, ReactionGroup, Dashboard, SearchResults,
} from "./types";

// Projects
export const createProject = (name: string, description?: string) =>
  invoke<Project>("create_project", { input: { name, description } });
export const listProjects = () => invoke<Project[]>("list_projects");
export const updateProject = (id: string, name?: string, description?: string, notes?: string) =>
  invoke<Project>("update_project", { input: { id, name, description, notes } });
export const deleteProject = (id: string) => invoke<void>("delete_project", { id });
export const getDashboard = () => invoke<Dashboard>("get_dashboard");

// Issues
export const createIssue = (title: string, opts?: { project_id?: string; body?: string; status?: string; machine_id?: string; milestone_id?: string }) =>
  invoke<Issue>("create_issue", { input: { title, ...opts } });
export const listIssues = (projectId?: string, includeClosed?: boolean) =>
  invoke<Issue[]>("list_issues", { projectId, includeClosed });
export const getIssue = (id: string) => invoke<Issue>("get_issue", { id });
export const updateIssue = (id: string, updates: { title?: string; body?: string; status?: string; context?: string; machine_id?: string; milestone_id?: string }) =>
  invoke<Issue>("update_issue", { input: { id, ...updates } });
export const closeIssue = (id: string) => invoke<Issue>("close_issue", { id });
export const reopenIssue = (id: string) => invoke<Issue>("reopen_issue", { id });
export const deleteIssue = (id: string) => invoke<void>("delete_issue", { id });
export const reorderIssue = (id: string, newSortOrder: number) =>
  invoke<void>("reorder_issue", { id, newSortOrder });
export const transferIssue = (id: string, newProjectId: string) =>
  invoke<Issue>("transfer_issue", { id, newProjectId });
export const promoteIdea = (issueId: string) =>
  invoke<Project>("promote_idea", { issueId });

// Comments
export const createComment = (issueId: string, body: string) =>
  invoke<Comment>("create_comment", { input: { issue_id: issueId, body } });
export const listComments = (issueId: string) =>
  invoke<Comment[]>("list_comments", { issueId });
export const updateComment = (id: string, body: string) =>
  invoke<Comment>("update_comment", { input: { id, body } });
export const deleteComment = (id: string) => invoke<void>("delete_comment", { id });

// Reactions
export const addReaction = (emoji: string, issueId?: string, commentId?: string) =>
  invoke<void>("add_reaction", { issueId, commentId, emoji });
export const removeReaction = (id: string) => invoke<void>("remove_reaction", { id });
export const listReactions = (issueId?: string, commentId?: string) =>
  invoke<ReactionGroup[]>("list_reactions", { issueId, commentId });

// Labels
export const createLabel = (name: string, color: string, projectId?: string) =>
  invoke<Label>("create_label", { input: { name, color, project_id: projectId } });
export const listLabels = (projectId?: string) => invoke<Label[]>("list_labels", { projectId });
export const deleteLabel = (id: string) => invoke<void>("delete_label", { id });
export const addLabelToIssue = (issueId: string, labelId: string) =>
  invoke<void>("add_label_to_issue", { issueId, labelId });
export const removeLabelFromIssue = (issueId: string, labelId: string) =>
  invoke<void>("remove_label_from_issue", { issueId, labelId });
export const getIssueLabels = (issueId: string) => invoke<Label[]>("get_issue_labels", { issueId });

// Milestones
export const createMilestone = (projectId: string, title: string, description?: string, dueDate?: number) =>
  invoke<Milestone>("create_milestone", { input: { project_id: projectId, title, description, due_date: dueDate } });
export const listMilestones = (projectId: string) => invoke<Milestone[]>("list_milestones", { projectId });
export const closeMilestone = (id: string) => invoke<void>("close_milestone", { id });
export const setMilestone = (issueId: string, milestoneId?: string) =>
  invoke<void>("set_milestone", { issueId, milestoneId });

// Dependencies & Relations
export const addDependency = (blockerId: string, blockedId: string) =>
  invoke<void>("add_dependency", { blockerId, blockedId });
export const removeDependency = (blockerId: string, blockedId: string) =>
  invoke<void>("remove_dependency", { blockerId, blockedId });
export const addRelation = (issueAId: string, issueBId: string) =>
  invoke<void>("add_relation", { issueAId, issueBId });
export const removeRelation = (issueAId: string, issueBId: string) =>
  invoke<void>("remove_relation", { issueAId, issueBId });

// Assignees
export const assignIssue = (issueId: string, assigneeName: string) =>
  invoke<void>("assign_issue", { issueId, assigneeName });
export const unassignIssue = (issueId: string, assigneeName: string) =>
  invoke<void>("unassign_issue", { issueId, assigneeName });
export const getIssueAssignees = (issueId: string) => invoke<string[]>("get_issue_assignees", { issueId });

// Machines
export const createMachine = (input: { name: string; hostname?: string; ip?: string; user?: string; os?: string; notes?: string }) =>
  invoke<Machine>("create_machine", { input });
export const listMachines = () => invoke<Machine[]>("list_machines");
export const updateMachine = (id: string, updates: { name?: string; hostname?: string; ip?: string; user?: string; os?: string; notes?: string }) =>
  invoke<Machine>("update_machine", { input: { id, ...updates } });
export const deleteMachine = (id: string) => invoke<void>("delete_machine", { id });
export const getCurrentMachine = () => invoke<Machine>("get_current_machine");
export const createMachineDoc = (machineId: string, title: string, content?: string, url?: string) =>
  invoke<MachineDoc>("create_machine_doc", { input: { machine_id: machineId, title, content, url } });
export const listMachineDocs = (machineId: string) => invoke<MachineDoc[]>("list_machine_docs", { machineId });
export const updateMachineDoc = (id: string, title?: string, content?: string, url?: string) =>
  invoke<MachineDoc>("update_machine_doc", { id, title, content, url });
export const deleteMachineDoc = (id: string) => invoke<void>("delete_machine_doc", { id });

// Activity & Search
export const getActivityLog = (projectId?: string, limit?: number) =>
  invoke<ActivityEntry[]>("get_activity_log", { projectId, limit });
export const searchIssues = (query: string) => invoke<SearchResults>("search_issues", { query });
```

- [ ] **Step 3: Create event listeners**

Create `src/lib/events.ts`:
```ts
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function onIssuesChanged(callback: (projectId: string | null) => void): Promise<UnlistenFn> {
  return listen<{ project_id: string | null }>("issues-changed", (event) => {
    callback(event.payload.project_id);
  });
}

export function onProjectsChanged(callback: () => void): Promise<UnlistenFn> {
  return listen("projects-changed", () => callback());
}

export function onActivity(callback: (entry: { action: string; detail: string | null }) => void): Promise<UnlistenFn> {
  return listen("activity", (event) => callback(event.payload as any));
}
```

- [ ] **Step 4: Commit**

```bash
cd /path/to/dev/ProjectOS
git add src/lib/types.ts src/lib/commands.ts src/lib/events.ts
git commit -m "feat: TypeScript types, command wrappers, and event listeners"
```

---

## Task 14: Frontend App Shell (Sidebar + Navigation)

**Files:**
- Create: `src/lib/stores/navigation.svelte.ts`
- Create: `src/lib/stores/projects.svelte.ts`
- Create: `src/lib/stores/machines.svelte.ts`
- Create: `src/lib/components/Sidebar.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create navigation store**

Create `src/lib/stores/navigation.svelte.ts`:
```ts
import type { View } from "../types";

let current = $state<View>({ kind: "home" });

export function navigate(view: View) {
  current = view;
}

export function currentView() {
  return current;
}
```

- [ ] **Step 2: Create projects store**

Create `src/lib/stores/projects.svelte.ts`:
```ts
import type { Project } from "../types";
import { listProjects } from "../commands";

let projects = $state<Project[]>([]);
let loaded = $state(false);

export async function loadProjects() {
  projects = await listProjects();
  loaded = true;
}

export function getProjects() {
  return projects;
}

export function isLoaded() {
  return loaded;
}
```

- [ ] **Step 3: Create machines store**

Create `src/lib/stores/machines.svelte.ts`:
```ts
import type { Machine } from "../types";
import { listMachines } from "../commands";

let machines = $state<Machine[]>([]);

export async function loadMachines() {
  machines = await listMachines();
}

export function getMachines() {
  return machines;
}
```

- [ ] **Step 4: Build the Sidebar component**

Create `src/lib/components/Sidebar.svelte`. This is a Svelte file so the implementor should use the `svelte:svelte-file-editor` agent or `svelte:svelte-code-writer` skill for this. The sidebar should:

- Show a "Home" button at the top that navigates to `{ kind: "home" }`
- List all projects from the projects store, each clickable to navigate to `{ kind: "project", projectId }`
- Separate section for machines, each clickable to `{ kind: "machine", machineId }`
- Highlight the currently active view
- Import and use `navigate`, `currentView` from navigation store
- Import and use `getProjects` from projects store
- Import and use `getMachines` from machines store

- [ ] **Step 5: Build the App shell**

Modify `src/App.svelte` to:
- Import Sidebar and render it on the left
- Use `currentView()` to conditionally render the main content area
- For now, render placeholder text for each view kind ("Home", "Project: {id}", etc.)
- On mount, call `loadProjects()` and `loadMachines()`
- Set up event listeners from `events.ts` to refresh data on changes
- Apply a dark theme CSS baseline

- [ ] **Step 6: Verify frontend builds**

```bash
cd /path/to/dev/ProjectOS
pnpm build
```

- [ ] **Step 7: Commit**

```bash
cd /path/to/dev/ProjectOS
git add src/lib/stores/ src/lib/components/Sidebar.svelte src/App.svelte
git commit -m "feat: app shell with sidebar navigation and stores"
```

---

## Task 15: Dashboard View

**Files:**
- Create: `src/lib/stores/dashboard.svelte.ts`
- Create: `src/lib/components/Dashboard.svelte`
- Create: `src/lib/components/ActivityFeed.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create dashboard store**

Create `src/lib/stores/dashboard.svelte.ts`:
```ts
import type { Dashboard } from "../types";
import { getDashboard } from "../commands";

let dashboard = $state<Dashboard | null>(null);

export async function loadDashboard() {
  dashboard = await getDashboard();
}

export function getDashboardData() {
  return dashboard;
}
```

- [ ] **Step 2: Build Dashboard and ActivityFeed components**

These are Svelte files — the implementor should use the `svelte:svelte-file-editor` agent or `svelte:svelte-code-writer` skill.

**Dashboard.svelte** should:
- Show "What's Next" header
- Render a grid of cards, one per project with a `next` issue
- Each card shows: project name (with accent color), next issue title, machine indicator
- Clicking a card navigates to the project view
- Below the grid, render the `ActivityFeed` component

**ActivityFeed.svelte** should:
- Accept an `entries` prop of `ActivityEntry[]`
- Group by day ("Today", "Yesterday", date)
- Show action icons: checkmark for closed, plus for created, arrow for moved
- Show issue title from `detail` JSON when available
- Show relative time ("2h ago", "1d ago")

- [ ] **Step 3: Wire Dashboard into App.svelte**

Import Dashboard, render it when `currentView().kind === "home"`. Call `loadDashboard()` on mount and when `activity` events fire.

- [ ] **Step 4: Verify and commit**

```bash
cd /path/to/dev/ProjectOS
pnpm build
git add src/lib/stores/dashboard.svelte.ts src/lib/components/Dashboard.svelte src/lib/components/ActivityFeed.svelte src/App.svelte
git commit -m "feat: dashboard with What's Next cards and activity feed"
```

---

## Task 16: Project View + Issue List

**Files:**
- Create: `src/lib/stores/issues.svelte.ts`
- Create: `src/lib/components/ProjectView.svelte`
- Create: `src/lib/components/IssueList.svelte`
- Create: `src/lib/components/IssueRow.svelte`
- Create: `src/lib/components/StatusBadge.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create issues store**

Create `src/lib/stores/issues.svelte.ts`:
```ts
import type { Issue } from "../types";
import { listIssues } from "../commands";

let issues = $state<Issue[]>([]);
let currentProjectId = $state<string | null>(null);

export async function loadIssues(projectId: string, includeClosed = false) {
  currentProjectId = projectId;
  issues = await listIssues(projectId, includeClosed);
}

export function getIssues() {
  return issues;
}

export function getCurrentProjectId() {
  return currentProjectId;
}
```

- [ ] **Step 2: Build StatusBadge component**

Create `src/lib/components/StatusBadge.svelte` — renders the status as a colored badge:
- `next` → green
- `ready` → no badge (default)
- `blocked` → orange
- `idea` → subtle gray/italic

- [ ] **Step 3: Build IssueRow component**

Create `src/lib/components/IssueRow.svelte`:
- Shows issue title, number (`#N`), StatusBadge
- Shows machine indicator if set
- Shows "BLOCKED by #N" if status is blocked (need to fetch deps)
- Pinned issues get a pin icon
- Clicking navigates to `{ kind: "issue", issueId }`
- Dimmed style for blocked issues

- [ ] **Step 4: Build IssueList component**

Create `src/lib/components/IssueList.svelte`:
- Renders pinned issues section (if any), then regular issues
- Each issue rendered via IssueRow
- Drag-to-reorder support (implement in Task 22)

- [ ] **Step 5: Build ProjectView component**

Create `src/lib/components/ProjectView.svelte`:
- Header: project name, open/closed count, "+ New Issue" button
- Toggle to show/hide closed issues
- IssueList component
- Milestone progress at bottom (if milestone exists)
- On mount, call `loadIssues(projectId)`

- [ ] **Step 6: Wire into App.svelte**

Render ProjectView when `currentView().kind === "project"`.

- [ ] **Step 7: Verify and commit**

```bash
cd /path/to/dev/ProjectOS
pnpm build
git add src/lib/stores/issues.svelte.ts src/lib/components/ProjectView.svelte src/lib/components/IssueList.svelte src/lib/components/IssueRow.svelte src/lib/components/StatusBadge.svelte src/App.svelte
git commit -m "feat: project view with ordered issue list"
```

---

## Task 17: Issue Detail View

**Files:**
- Create: `src/lib/components/IssueDetail.svelte`
- Create: `src/lib/components/CommentThread.svelte`
- Create: `src/lib/components/LabelBadge.svelte`
- Create: `src/lib/components/MilestoneBar.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Build LabelBadge and MilestoneBar**

**LabelBadge.svelte**: renders a label as a small colored pill with the label name.

**MilestoneBar.svelte**: accepts `openCount` and `closedCount`, renders a progress bar with percentage.

- [ ] **Step 2: Build CommentThread**

Create `src/lib/components/CommentThread.svelte`:
- Accepts `issueId` prop
- On mount, fetches comments via `listComments(issueId)`
- Renders each comment with body (markdown), timestamp, reactions
- "Write a comment..." input at the bottom
- Submit calls `createComment`
- Reaction buttons on each comment

- [ ] **Step 3: Build IssueDetail**

Create `src/lib/components/IssueDetail.svelte`:
- Fetches full issue data via `getIssue(id)` on mount
- Left side: state badge (open/closed), status badge, title, `#N · opened X ago`
- Issue body (rendered markdown)
- "Where I left off" context block (editable)
- CommentThread
- Right sidebar: status, assignees, labels (LabelBadge), milestone (MilestoneBar), machine, blocking issues, related issues
- Actions: Close/Reopen, Transfer, Promote to project, Delete

- [ ] **Step 4: Wire into App.svelte**

Render IssueDetail when `currentView().kind === "issue"`.

- [ ] **Step 5: Verify and commit**

```bash
cd /path/to/dev/ProjectOS
pnpm build
git add src/lib/components/IssueDetail.svelte src/lib/components/CommentThread.svelte src/lib/components/LabelBadge.svelte src/lib/components/MilestoneBar.svelte src/App.svelte
git commit -m "feat: issue detail view with comments, labels, and metadata sidebar"
```

---

## Task 18: Machine View

**Files:**
- Create: `src/lib/components/MachineView.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Build MachineView**

Create `src/lib/components/MachineView.svelte`:
- Fetches machine data and docs on mount
- Shows: name, hostname, IP, user, OS, notes (editable markdown)
- Docs section: list of attached docs with title, content/url
- Add doc button
- Issues section: list of issues tagged to this machine (query by `machine_id`)
- Edit machine details inline

- [ ] **Step 2: Wire into App.svelte, verify, commit**

```bash
cd /path/to/dev/ProjectOS
pnpm build
git add src/lib/components/MachineView.svelte src/App.svelte
git commit -m "feat: machine view with docs and tagged issues"
```

---

## Task 19: Search Modal

**Files:**
- Create: `src/lib/stores/search.svelte.ts`
- Create: `src/lib/components/SearchModal.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Create search store**

Create `src/lib/stores/search.svelte.ts`:
```ts
import type { SearchResults } from "../types";
import { searchIssues } from "../commands";

let results = $state<SearchResults | null>(null);
let isOpen = $state(false);
let query = $state("");

export function openSearch() { isOpen = true; query = ""; results = null; }
export function closeSearch() { isOpen = false; }
export function isSearchOpen() { return isOpen; }
export function getQuery() { return query; }
export function getResults() { return results; }

export async function search(q: string) {
  query = q;
  if (q.length < 2) { results = null; return; }
  results = await searchIssues(q);
}
```

- [ ] **Step 2: Build SearchModal**

Create `src/lib/components/SearchModal.svelte`:
- Overlay modal triggered by Cmd+K
- Text input with auto-focus
- Debounced search (150ms)
- Results grouped: Issues, Projects, Machines
- Each result clickable → navigates to appropriate view and closes modal
- Esc closes
- Keyboard navigation (arrow keys + enter to select)

- [ ] **Step 3: Wire into App.svelte, verify, commit**

Mount SearchModal in App.svelte (always rendered, visibility controlled by store).

```bash
cd /path/to/dev/ProjectOS
pnpm build
git add src/lib/stores/search.svelte.ts src/lib/components/SearchModal.svelte src/App.svelte
git commit -m "feat: global search modal with FTS results"
```

---

## Task 20: Quick Capture

**Files:**
- Create: `src/lib/components/QuickCapture.svelte`
- Modify: `src/App.svelte`

- [ ] **Step 1: Build QuickCapture**

Create `src/lib/components/QuickCapture.svelte`:
- Small popup input triggered by Cmd+N
- Single text field, auto-focused
- Enter → calls `createIssue(title)` with no project, status "idea"
- Esc closes
- Shows brief "Created!" confirmation then auto-closes

- [ ] **Step 2: Wire into App.svelte, verify, commit**

```bash
cd /path/to/dev/ProjectOS
pnpm build
git add src/lib/components/QuickCapture.svelte src/App.svelte
git commit -m "feat: quick idea capture with Cmd+N"
```

---

## Task 21: Keyboard Shortcuts

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Add global keyboard listener**

In `src/App.svelte`, add a `svelte:window` keydown handler:

```ts
function handleKeydown(e: KeyboardEvent) {
  // Don't capture when typing in inputs
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;

  if (e.metaKey && e.key === "k") { e.preventDefault(); openSearch(); }
  if (e.metaKey && e.key === "n") { e.preventDefault(); openQuickCapture(); }
  if (e.key === "Escape") { /* close modals, go back */ }

  // Issue list navigation (only when in project view)
  if (currentView().kind === "project") {
    if (e.key === "j") { /* select next issue */ }
    if (e.key === "k") { /* select previous issue */ }
    if (e.key === "Enter") { /* open selected issue */ }
    if (e.key === "x") { /* close/reopen selected issue */ }
  }
}
```

- [ ] **Step 2: Verify and commit**

```bash
cd /path/to/dev/ProjectOS
pnpm build
git add src/App.svelte
git commit -m "feat: keyboard shortcuts for navigation and actions"
```

---

## Task 22: Drag-to-Reorder

**Files:**
- Modify: `src/lib/components/IssueList.svelte`

- [ ] **Step 1: Implement drag-to-reorder**

In `IssueList.svelte`, add HTML5 drag-and-drop:
- `draggable="true"` on each IssueRow
- On `dragstart`: store the dragged issue ID
- On `dragover`: highlight drop target, calculate position
- On `drop`: compute new `sort_order` as average of the two surrounding issues' sort orders (fractional indexing)
- Call `reorderIssue(id, newSortOrder)`
- Optimistically update the local list order

- [ ] **Step 2: Verify and commit**

```bash
cd /path/to/dev/ProjectOS
pnpm build
git add src/lib/components/IssueList.svelte
git commit -m "feat: drag-to-reorder with fractional indexing"
```

---

## Task 23: Emit Tauri Events from Rust

**Files:**
- Modify: `src-tauri/src/commands/projects.rs`
- Modify: `src-tauri/src/commands/issues.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add AppHandle to commands that need to emit events**

Update commands that mutate data to accept `app: tauri::AppHandle` and emit events after mutations. For example, in `create_project`:

```rust
use tauri::Emitter;

#[tauri::command]
pub fn create_project(state: State<AppState>, app: tauri::AppHandle, input: CreateProject) -> Result<Project, String> {
    // ... existing code ...
    app.emit("projects-changed", ()).map_err(|e| e.to_string())?;
    Ok(project)
}
```

Apply the same pattern to:
- All project mutations → emit `"projects-changed"`
- All issue mutations → emit `"issues-changed"` with `serde_json::json!({"project_id": project_id})`
- All mutations that log activity → emit `"activity"` with action details

- [ ] **Step 2: Verify and commit**

```bash
cd /path/to/dev/ProjectOS/src-tauri
cargo build
cd /path/to/dev/ProjectOS
git add src-tauri/src/commands/
git commit -m "feat: emit Tauri events on data mutations"
```

---

## Task 24: Styling + Polish

**Files:**
- Modify: `src/App.svelte` (or a global CSS file)
- Modify: various components for consistent styling

- [ ] **Step 1: Apply dark theme baseline**

Create or update the global styles to match the mockup aesthetic:
- Dark background (`#1a1a2e` or similar)
- Subtle borders (`#333`)
- Monospace font for the UI
- Accent colors per project
- Status colors: green (next), orange (blocked), gray (idea)
- Smooth transitions on navigation

- [ ] **Step 2: Polish component styles**

Ensure all components have consistent:
- Spacing and padding
- Hover states
- Focus indicators for keyboard navigation
- Transition animations

- [ ] **Step 3: Verify full app builds and runs**

```bash
cd /path/to/dev/ProjectOS
pnpm build
cd src-tauri && cargo build
```

- [ ] **Step 4: Commit**

```bash
cd /path/to/dev/ProjectOS
git add -A
git commit -m "feat: dark theme styling and UI polish"
```

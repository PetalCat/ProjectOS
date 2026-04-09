use rusqlite::Connection;

pub fn create_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            notes TEXT,
            github_repo TEXT,
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
            external_source TEXT,
            external_id TEXT,
            external_url TEXT,
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
        "
    )
}

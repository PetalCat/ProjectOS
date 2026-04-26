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
        // Fresh DB: schema.rs already creates the latest schema, so stamp
        // version to the current head and skip the per-version ALTERs (which
        // would otherwise fail with "duplicate column name" on a fresh DB
        // because the columns are already in CREATE TABLE).
        conn.execute("INSERT INTO schema_version (version) VALUES (3)", [])?;
        return Ok(());
    }

    if version < 2 {
        conn.execute_batch(
            "ALTER TABLE projects ADD COLUMN github_repo TEXT;
             ALTER TABLE issues ADD COLUMN external_source TEXT;
             ALTER TABLE issues ADD COLUMN external_id TEXT;
             ALTER TABLE issues ADD COLUMN external_url TEXT;
             UPDATE schema_version SET version = 2;"
        )?;
    }

    if version < 3 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS scan_folders (
                id TEXT PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                last_scanned_at INTEGER,
                created_at INTEGER NOT NULL
            );
            UPDATE schema_version SET version = 3;"
        )?;
    }

    Ok(())
}

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

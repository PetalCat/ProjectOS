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

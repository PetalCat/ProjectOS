mod state;
mod db;
mod models;
mod commands;

use state::AppState;
use std::sync::Mutex;
use tauri::Manager;

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

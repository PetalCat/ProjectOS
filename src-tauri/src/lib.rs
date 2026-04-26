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
            // PROJECTOS_DB_PATH env var wins so the desktop app + the MCP
            // server can be pointed at the same DB on Linux/Windows or for
            // testing against fixtures.
            let db_path = match std::env::var("PROJECTOS_DB_PATH") {
                Ok(custom) if !custom.is_empty() => {
                    let p = std::path::PathBuf::from(custom);
                    if let Some(parent) = p.parent() {
                        std::fs::create_dir_all(parent)
                            .expect("failed to create custom DB parent dir");
                    }
                    p
                }
                _ => {
                    let app_dir = app
                        .path()
                        .app_data_dir()
                        .expect("failed to get app data dir");
                    std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
                    app_dir.join("projectos.db")
                }
            };
            eprintln!("[projectos] db_path={}", db_path.display());
            let conn = db::open_db(&db_path).expect("failed to open database");
            db::migrations::run_migrations(&conn).expect("failed to run migrations");
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Projects
            commands::projects::create_project,
            commands::projects::list_projects,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::projects::get_dashboard,
            // Issues
            commands::issues::create_issue,
            commands::issues::list_issues,
            commands::issues::debug_issues,
            commands::issues::get_issue,
            commands::issues::update_issue,
            commands::issues::close_issue,
            commands::issues::reopen_issue,
            commands::issues::delete_issue,
            commands::issues::reorder_issue,
            commands::issues::transfer_issue,
            commands::issues::promote_idea,
            commands::issues::add_dependency,
            commands::issues::remove_dependency,
            commands::issues::add_relation,
            commands::issues::remove_relation,
            commands::issues::assign_issue,
            commands::issues::unassign_issue,
            commands::issues::get_issue_assignees,
            // Comments + Reactions
            commands::comments::create_comment,
            commands::comments::list_comments,
            commands::comments::update_comment,
            commands::comments::delete_comment,
            commands::comments::add_reaction,
            commands::comments::remove_reaction,
            commands::comments::list_reactions,
            // Labels
            commands::labels::create_label,
            commands::labels::list_labels,
            commands::labels::delete_label,
            commands::labels::add_label_to_issue,
            commands::labels::remove_label_from_issue,
            commands::labels::get_issue_labels,
            // Milestones
            commands::milestones::create_milestone,
            commands::milestones::list_milestones,
            commands::milestones::close_milestone,
            commands::milestones::set_milestone,
            // Machines
            commands::machines::create_machine,
            commands::machines::list_machines,
            commands::machines::update_machine,
            commands::machines::delete_machine,
            commands::machines::get_current_machine,
            commands::machines::create_machine_doc,
            commands::machines::list_machine_docs,
            commands::machines::update_machine_doc,
            commands::machines::delete_machine_doc,
            // Activity
            commands::activity::get_activity_log,
            // Projects (scan + rescan)
            commands::projects::scan_developer_folder,
            commands::projects::rescan_timestamps,
            commands::projects::list_scan_folders,
            commands::projects::add_scan_folder,
            commands::projects::remove_scan_folder,
            commands::projects::scan_folder,
            commands::projects::scan_all_folders,
            // Search
            commands::search::search_issues,
            // GitHub
            commands::github::sync_github_issues,
            commands::github::publish_issue_to_github,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

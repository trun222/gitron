pub mod commands;
pub mod git;
pub mod cache;
pub mod watcher;

use commands::{branch, diff, graph, repo, staging};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            repo::open_repo,
            repo::get_status,
            repo::get_repo_info,
            graph::get_commit_graph,
            graph::get_commit_detail,
            diff::get_workdir_diff,
            diff::get_file_diff,
            staging::stage_file,
            staging::unstage_file,
            staging::stage_all,
            staging::unstage_all,
            branch::list_branches,
            branch::create_branch,
            branch::checkout_branch,
            branch::delete_branch,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub mod commands;
pub mod tauri_impls;

use std::sync::Arc;

use commands::{ai as ai_cmd, branch, clone, commit, diff, github as github_cmd, graph, remote, repo, staging, stash, tag, terminal, worktree, AppState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::new())
        .setup(|app| {
            // Initialize credential store with Tauri implementation
            tauri_impls::TauriCredentialStore::init(app.handle().clone());
            gitron_core::credential::init(Arc::new(tauri_impls::TauriCredentialStore));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            repo::open_repo,
            repo::close_repo,
            repo::set_watcher_interval,
            repo::get_status,
            repo::get_repo_info,
            graph::get_commit_graph,
            graph::get_commit_detail,
            graph::search_commits,
            diff::get_workdir_diff,
            diff::get_file_diff,
            diff::get_staged_file_diff,
            diff::get_commit_diff,
            staging::stage_file,
            staging::unstage_file,
            staging::stage_files,
            staging::stage_all,
            staging::unstage_all,
            staging::discard_all_changes,
            staging::discard_files,
            staging::add_to_gitignore,
            branch::list_branches,
            branch::create_branch,
            branch::checkout_branch,
            branch::delete_branch,
            branch::reset_to_commit,
            branch::rebase_onto,
            branch::merge_into,
            commit::create_commit,
            stash::apply_stash,
            stash::pop_stash,
            stash::drop_stash,
            remote::list_remotes,
            remote::add_remote,
            remote::remove_remote,
            remote::get_tracking_status,
            remote::fetch_remote,
            remote::fetch_all_remotes,
            remote::push_to_remote,
            remote::pull_from_remote,
            remote::delete_remote_branch,
            remote::checkout_remote_branch,
            github_cmd::github_check_auth,
            github_cmd::github_start_device_flow,
            github_cmd::github_poll_device_flow,
            github_cmd::github_logout,
            github_cmd::github_get_user,
            github_cmd::github_list_repos,
            tag::create_tag,
            tag::delete_tag,
            tag::move_tag,
            tag::push_tag,
            tag::delete_remote_tag,
            tag::list_remote_tags,
            clone::clone_repo,
            worktree::list_worktrees,
            worktree::add_worktree,
            worktree::remove_worktree,
            worktree::lock_worktree,
            worktree::unlock_worktree,
            worktree::prune_worktrees,
            worktree::open_in_terminal,
            ai_cmd::ai_get_providers,
            ai_cmd::ai_save_key,
            ai_cmd::ai_delete_key,
            ai_cmd::ai_fetch_models,
            ai_cmd::ai_generate_commit_message,
            ai_cmd::ai_get_settings,
            ai_cmd::ai_save_settings,
            terminal::terminal_spawn,
            terminal::terminal_write,
            terminal::terminal_resize,
            terminal::terminal_kill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

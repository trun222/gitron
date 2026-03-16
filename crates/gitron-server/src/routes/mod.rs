mod repo;
mod graph;
mod diff;
mod staging;
mod branch;
mod commit;
mod stash;
mod remote;
mod tag;
mod clone;
mod worktree;
mod ai;
mod github;
mod fs;
mod settings;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use crate::ServerState;

pub fn api_router(state: Arc<ServerState>) -> Router {
    Router::new()
        // Repo
        .route("/repo/open", post(repo::open_repo))
        .route("/repo/close", post(repo::close_repo))
        .route("/repo/status", post(repo::get_status))
        .route("/repo/info", post(repo::get_repo_info))
        // Graph
        .route("/graph", post(graph::get_commit_graph))
        .route("/graph/detail", post(graph::get_commit_detail))
        .route("/graph/search", post(graph::search_commits))
        // Diff
        .route("/diff/workdir", post(diff::get_workdir_diff))
        .route("/diff/file", post(diff::get_file_diff))
        .route("/diff/staged", post(diff::get_staged_file_diff))
        .route("/diff/commit", post(diff::get_commit_diff))
        // Staging
        .route("/staging/stage", post(staging::stage_file))
        .route("/staging/unstage", post(staging::unstage_file))
        .route("/staging/stage-many", post(staging::stage_files))
        .route("/staging/stage-all", post(staging::stage_all))
        .route("/staging/unstage-all", post(staging::unstage_all))
        .route("/staging/discard-all", post(staging::discard_all))
        .route("/staging/discard-files", post(staging::discard_files))
        .route("/staging/gitignore", post(staging::add_to_gitignore))
        // Branch
        .route("/branch/list", post(branch::list_branches))
        .route("/branch/create", post(branch::create_branch))
        .route("/branch/checkout", post(branch::checkout_branch))
        .route("/branch/delete", post(branch::delete_branch))
        .route("/branch/reset", post(branch::reset_to_commit))
        .route("/branch/rebase", post(branch::rebase_onto))
        .route("/branch/merge", post(branch::merge_into))
        // Commit
        .route("/commit", post(commit::create_commit))
        // Stash
        .route("/stash/apply", post(stash::apply_stash))
        .route("/stash/pop", post(stash::pop_stash))
        .route("/stash/drop", post(stash::drop_stash))
        // Remote
        .route("/remote/list", post(remote::list_remotes))
        .route("/remote/add", post(remote::add_remote))
        .route("/remote/remove", post(remote::remove_remote))
        .route("/remote/tracking", post(remote::get_tracking_status))
        .route("/remote/fetch", post(remote::fetch_remote))
        .route("/remote/fetch-all", post(remote::fetch_all_remotes))
        .route("/remote/push", post(remote::push_to_remote))
        .route("/remote/pull", post(remote::pull_from_remote))
        .route("/remote/delete-branch", post(remote::delete_remote_branch))
        .route("/remote/checkout", post(remote::checkout_remote_branch))
        // Tag
        .route("/tag/create", post(tag::create_tag))
        .route("/tag/delete", post(tag::delete_tag))
        .route("/tag/move", post(tag::move_tag))
        .route("/tag/push", post(tag::push_tag))
        .route("/tag/delete-remote", post(tag::delete_remote_tag))
        .route("/tag/list-remote", post(tag::list_remote_tags))
        // Worktree
        .route("/worktree/list", post(worktree::list_worktrees))
        .route("/worktree/add", post(worktree::add_worktree))
        .route("/worktree/remove", post(worktree::remove_worktree))
        .route("/worktree/lock", post(worktree::lock_worktree))
        .route("/worktree/unlock", post(worktree::unlock_worktree))
        .route("/worktree/prune", post(worktree::prune_worktrees))
        // Clone
        .route("/clone", post(clone::clone_repo))
        // GitHub
        .route("/github/check-auth", post(github::check_auth))
        .route("/github/start-flow", post(github::start_device_flow))
        .route("/github/poll-flow", post(github::poll_device_flow))
        .route("/github/logout", post(github::logout))
        .route("/github/user", post(github::get_user))
        .route("/github/repos", post(github::list_repos))
        // AI
        .route("/ai/providers", post(ai::get_providers))
        .route("/ai/save-key", post(ai::save_key))
        .route("/ai/delete-key", post(ai::delete_key))
        .route("/ai/fetch-models", post(ai::fetch_models))
        .route("/ai/generate", post(ai::generate_commit_message))
        .route("/ai/settings", get(ai::get_settings).post(ai::save_settings))
        // Filesystem browser
        .route("/fs/list", post(fs::list_directory))
        // Settings
        .route("/settings", get(settings::get_settings).post(settings::save_settings))
        // SSE events
        .route("/events", get(events))
        .with_state(state)
}

async fn events(
    axum::extract::State(state): axum::extract::State<Arc<ServerState>>,
) -> impl axum::response::IntoResponse {
    state.broadcaster.subscribe()
}

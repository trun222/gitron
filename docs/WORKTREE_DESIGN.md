# Gitron — Worktree Support Design

## Overview

Git worktrees allow multiple working directories to share a single `.git` object store. Each worktree has its own checkout (branch, index, working tree) but shares commits, refs, and the object database with the main repository. This enables true parallel development — two branches checked out simultaneously, two build processes running, two AI agents working independently.

Gitron should be the first Git GUI with first-class worktree support, and the first to integrate worktrees with AI agent workflows.

## Why This Matters

### The Problem

Developers working with AI coding agents hit a workflow bottleneck: one branch at a time. If Claude Code is working on feature-A and you want to start feature-B, you either:

1. Wait for feature-A to finish
2. Stash/commit half-done work and switch branches
3. Clone the repo a second time (wasteful, disconnected histories)

### The Solution

Worktrees eliminate this entirely. Each worktree is a full, independent checkout that shares the same git history. You can have feature-A and feature-B progressing simultaneously with zero interference.

### Why Gitron Specifically

No mainstream Git GUI has meaningful worktree support:

- **GitKraken** — no worktree support
- **GitHub Desktop** — no worktree support
- **Fork** — shows worktrees in sidebar but no management
- **Lazygit** — basic worktree list, limited management
- **GitButler** — uses virtual branches (a different paradigm entirely)

Gitron can own this space, especially with the Agent Gateway (Phase 5) where worktrees become the isolation primitive for autonomous agents.

## Scope

**Phase**: 3 (Advanced Git Features) — add to the roadmap alongside stash, tags, cherry-pick.

**Milestone 1 (Core)**: List, create, remove, prune worktrees. UI panel for management.

**Milestone 2 (Integration)**: Open worktree in new Gitron window. Cross-worktree awareness in the commit graph.

**Milestone 3 (Agent-Native)**: Agent Gateway integration — agents get isolated worktrees with scoped permissions. "Spawn AI session on worktree" action.

This doc covers Milestones 1 and 2 in detail. Milestone 3 is sketched for future reference.

---

## Rust Types

Add to `crates/gitron-core/src/git/types.rs`:

```rust
/// A git worktree entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    /// Name of the worktree (directory basename for linked, "main" for the primary)
    pub name: String,
    /// Absolute path to the worktree's working directory
    pub path: String,
    /// Branch currently checked out (None if detached HEAD)
    pub branch: Option<String>,
    /// HEAD commit OID
    pub head_oid: Option<String>,
    /// Short HEAD commit OID
    pub head_short_oid: Option<String>,
    /// Whether this is the main (non-linked) worktree
    pub is_main: bool,
    /// Whether the worktree directory is locked (prevents pruning)
    pub is_locked: bool,
    /// Lock reason, if locked
    pub lock_reason: Option<String>,
    /// Whether the worktree path is valid (directory exists)
    pub is_valid: bool,
}

/// Result of creating a worktree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeCreateResult {
    pub worktree: WorktreeInfo,
    pub output: OperationOutput,
}

/// Result of removing a worktree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeRemoveResult {
    pub success: bool,
    pub output: OperationOutput,
}

/// Result of pruning worktrees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreePruneResult {
    pub pruned: Vec<String>,
    pub output: OperationOutput,
}
```

## Rust Core Logic

New file: `crates/gitron-core/src/git/worktree.rs`

### Backend Strategy: Git CLI

Worktree operations should use `git worktree` CLI commands rather than git2-rs. Rationale:

1. **git2-rs worktree API is limited** — `Repository::worktrees()` returns names only, `Worktree` has `name()`, `path()`, `is_locked()`, `is_prunable()` but no create/remove. Creating worktrees via git2-rs requires manual `.git` file setup, ref management, and lock file handling that `git worktree add` handles automatically.
2. **Consistent with Gitron's hybrid strategy** — git CLI is already used for merge, rebase, and remote operations. Worktrees are a "complex op" that fits this pattern.
3. **Porcelain output is stable** — `git worktree list --porcelain` has been stable since Git 2.15 (2017) and provides structured, parseable output.

### Functions

```rust
use super::cli::{run_git, run_git_raw};
use super::error::{GitError, GitResult};
use super::types::*;

/// List all worktrees for the repository at `workdir`.
///
/// Uses `git worktree list --porcelain` which outputs blocks like:
///   worktree /path/to/worktree
///   HEAD abc123...
///   branch refs/heads/feature-a
///   <blank line>
///
/// The main worktree is always listed first.
pub fn list_worktrees(workdir: &str) -> GitResult<Vec<WorktreeInfo>> {
    let output = run_git(workdir, &["worktree", "list", "--porcelain"])?;
    Ok(parse_worktree_list(&output.stdout))
}

/// Create a new worktree.
///
/// - `path`: Where to create the worktree (absolute or relative to workdir)
/// - `branch`: Branch to check out. If None, creates a detached HEAD at HEAD.
/// - `new_branch`: If true, creates a new branch with the given name.
///
/// Maps to: `git worktree add [-b <branch>] <path> [<commit-ish>]`
pub fn add_worktree(
    workdir: &str,
    path: &str,
    branch: Option<&str>,
    new_branch: bool,
) -> GitResult<WorktreeCreateResult> {
    let mut args = vec!["worktree", "add"];

    if let Some(b) = branch {
        if new_branch {
            args.extend(&["-b", b, path]);
        } else {
            args.extend(&[path, b]);
        }
    } else {
        args.extend(&["--detach", path]);
    }

    let output = run_git(workdir, &args)?;
    let worktrees = list_worktrees(workdir)?;

    // Find the newly created worktree by path
    let abs_path = std::path::Path::new(workdir).join(path);
    let canonical = abs_path.canonicalize()
        .unwrap_or_else(|_| abs_path.to_path_buf());
    let canonical_str = canonical.to_string_lossy().to_string();

    let worktree = worktrees
        .into_iter()
        .find(|w| w.path == canonical_str || w.path == path)
        .ok_or_else(|| GitError::Other(
            "Worktree created but not found in list".into()
        ))?;

    Ok(WorktreeCreateResult {
        worktree,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}

/// Remove a worktree.
///
/// - `force`: If true, removes even if there are uncommitted changes.
///
/// Maps to: `git worktree remove [--force] <worktree>`
pub fn remove_worktree(
    workdir: &str,
    worktree_path: &str,
    force: bool,
) -> GitResult<WorktreeRemoveResult> {
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(worktree_path);

    let output = run_git_raw(workdir, &args)?;
    Ok(WorktreeRemoveResult {
        success: output.exit_code == 0,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}

/// Lock a worktree to prevent pruning.
///
/// Maps to: `git worktree lock [--reason <reason>] <worktree>`
pub fn lock_worktree(
    workdir: &str,
    worktree_path: &str,
    reason: Option<&str>,
) -> GitResult<()> {
    let mut args = vec!["worktree", "lock"];
    if let Some(r) = reason {
        args.extend(&["--reason", r]);
    }
    args.push(worktree_path);
    run_git(workdir, &args)?;
    Ok(())
}

/// Unlock a worktree.
///
/// Maps to: `git worktree unlock <worktree>`
pub fn unlock_worktree(workdir: &str, worktree_path: &str) -> GitResult<()> {
    run_git(workdir, &["worktree", "unlock", worktree_path])?;
    Ok(())
}

/// Prune stale worktree references.
///
/// Maps to: `git worktree prune [--dry-run]`
pub fn prune_worktrees(workdir: &str, dry_run: bool) -> GitResult<WorktreePruneResult> {
    let mut args = vec!["worktree", "prune", "--verbose"];
    if dry_run {
        args.push("--dry-run");
    }
    let output = run_git(workdir, &args)?;
    let pruned = output.stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    Ok(WorktreePruneResult {
        pruned,
        output: OperationOutput {
            stdout: output.stdout,
            stderr: output.stderr,
        },
    })
}
```

### Parser for `git worktree list --porcelain`

```rust
/// Parse the porcelain output of `git worktree list --porcelain`.
///
/// Format (blocks separated by blank lines):
///   worktree /absolute/path
///   HEAD <oid>
///   branch refs/heads/<name>   (or "detached" for detached HEAD)
///   locked [<reason>]          (only if locked)
///   prunable                   (only if prunable)
fn parse_worktree_list(output: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut is_first = true;

    for block in output.split("\n\n") {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let mut path = String::new();
        let mut head_oid = None;
        let mut branch = None;
        let mut is_locked = false;
        let mut lock_reason = None;
        let mut is_bare = false;

        for line in block.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                path = p.to_string();
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                head_oid = Some(h.to_string());
            } else if let Some(b) = line.strip_prefix("branch ") {
                // Strip refs/heads/ prefix
                branch = Some(
                    b.strip_prefix("refs/heads/")
                        .unwrap_or(b)
                        .to_string()
                );
            } else if line == "detached" {
                branch = None;
            } else if line == "bare" {
                is_bare = true;
            } else if line == "locked" {
                is_locked = true;
            } else if let Some(reason) = line.strip_prefix("locked ") {
                is_locked = true;
                lock_reason = Some(reason.to_string());
            }
        }

        if path.is_empty() {
            continue;
        }

        let is_valid = std::path::Path::new(&path).exists();
        let head_short_oid = head_oid
            .as_ref()
            .map(|h| h.chars().take(7).collect::<String>());
        let name = if is_first {
            "main".to_string()
        } else {
            std::path::Path::new(&path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone())
        };

        worktrees.push(WorktreeInfo {
            name,
            path,
            branch,
            head_oid,
            head_short_oid,
            is_main: is_first,
            is_locked,
            lock_reason,
            is_valid,
        });

        is_first = false;
    }

    worktrees
}
```

### Module Registration

In `crates/gitron-core/src/git/mod.rs`:

```rust
pub mod worktree;  // add alongside existing modules
```

---

## Tauri Commands

New file: `src-tauri/src/commands/worktree.rs`

```rust
use gitron_core::git::{error::GitError, worktree, types::*};

#[tauri::command]
pub fn list_worktrees(path: String) -> Result<Vec<WorktreeInfo>, GitError> {
    worktree::list_worktrees(&path)
}

#[tauri::command]
pub fn add_worktree(
    path: String,
    worktree_path: String,
    branch: Option<String>,
    new_branch: bool,
) -> Result<WorktreeCreateResult, GitError> {
    worktree::add_worktree(
        &path,
        &worktree_path,
        branch.as_deref(),
        new_branch,
    )
}

#[tauri::command]
pub fn remove_worktree(
    path: String,
    worktree_path: String,
    force: bool,
) -> Result<WorktreeRemoveResult, GitError> {
    worktree::remove_worktree(&path, &worktree_path, force)
}

#[tauri::command]
pub fn lock_worktree(
    path: String,
    worktree_path: String,
    reason: Option<String>,
) -> Result<Vec<WorktreeInfo>, GitError> {
    worktree::lock_worktree(&path, &worktree_path, reason.as_deref())?;
    worktree::list_worktrees(&path)
}

#[tauri::command]
pub fn unlock_worktree(
    path: String,
    worktree_path: String,
) -> Result<Vec<WorktreeInfo>, GitError> {
    worktree::unlock_worktree(&path, &worktree_path)?;
    worktree::list_worktrees(&path)
}

#[tauri::command]
pub fn prune_worktrees(
    path: String,
    dry_run: bool,
) -> Result<WorktreePruneResult, GitError> {
    worktree::prune_worktrees(&path, dry_run)
}
```

Register in `src-tauri/src/lib.rs` `generate_handler![]`:

```rust
commands::worktree::list_worktrees,
commands::worktree::add_worktree,
commands::worktree::remove_worktree,
commands::worktree::lock_worktree,
commands::worktree::unlock_worktree,
commands::worktree::prune_worktrees,
```

---

## Server Routes

New file: `crates/gitron-server/src/routes/worktree.rs`

```rust
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use gitron_core::git::{worktree, types::*};

#[derive(Deserialize)]
pub struct ListRequest {
    path: String,
}

#[derive(Deserialize)]
pub struct AddRequest {
    path: String,
    #[serde(rename = "worktreePath")]
    worktree_path: String,
    branch: Option<String>,
    #[serde(rename = "newBranch")]
    new_branch: bool,
}

#[derive(Deserialize)]
pub struct RemoveRequest {
    path: String,
    #[serde(rename = "worktreePath")]
    worktree_path: String,
    force: bool,
}

#[derive(Deserialize)]
pub struct LockRequest {
    path: String,
    #[serde(rename = "worktreePath")]
    worktree_path: String,
    reason: Option<String>,
}

#[derive(Deserialize)]
pub struct UnlockRequest {
    path: String,
    #[serde(rename = "worktreePath")]
    worktree_path: String,
}

#[derive(Deserialize)]
pub struct PruneRequest {
    path: String,
    #[serde(rename = "dryRun")]
    dry_run: bool,
}

pub async fn list_worktrees(
    Json(req): Json<ListRequest>,
) -> Result<Json<Vec<WorktreeInfo>>, (StatusCode, String)> {
    let result = worktree::list_worktrees(&req.path).map_err(err)?;
    Ok(Json(result))
}

pub async fn add_worktree(
    Json(req): Json<AddRequest>,
) -> Result<Json<WorktreeCreateResult>, (StatusCode, String)> {
    let result = worktree::add_worktree(
        &req.path,
        &req.worktree_path,
        req.branch.as_deref(),
        req.new_branch,
    ).map_err(err)?;
    Ok(Json(result))
}

pub async fn remove_worktree(
    Json(req): Json<RemoveRequest>,
) -> Result<Json<WorktreeRemoveResult>, (StatusCode, String)> {
    let result = worktree::remove_worktree(
        &req.path,
        &req.worktree_path,
        req.force,
    ).map_err(err)?;
    Ok(Json(result))
}

pub async fn lock_worktree(
    Json(req): Json<LockRequest>,
) -> Result<Json<Vec<WorktreeInfo>>, (StatusCode, String)> {
    worktree::lock_worktree(
        &req.path,
        &req.worktree_path,
        req.reason.as_deref(),
    ).map_err(err)?;
    let list = worktree::list_worktrees(&req.path).map_err(err)?;
    Ok(Json(list))
}

pub async fn unlock_worktree(
    Json(req): Json<UnlockRequest>,
) -> Result<Json<Vec<WorktreeInfo>>, (StatusCode, String)> {
    worktree::unlock_worktree(&req.path, &req.worktree_path).map_err(err)?;
    let list = worktree::list_worktrees(&req.path).map_err(err)?;
    Ok(Json(list))
}

pub async fn prune_worktrees(
    Json(req): Json<PruneRequest>,
) -> Result<Json<WorktreePruneResult>, (StatusCode, String)> {
    let result = worktree::prune_worktrees(&req.path, req.dry_run).map_err(err)?;
    Ok(Json(result))
}

fn err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}
```

Register routes in `crates/gitron-server/src/routes/mod.rs`:

```rust
.route("/api/worktree/list", post(worktree::list_worktrees))
.route("/api/worktree/add", post(worktree::add_worktree))
.route("/api/worktree/remove", post(worktree::remove_worktree))
.route("/api/worktree/lock", post(worktree::lock_worktree))
.route("/api/worktree/unlock", post(worktree::unlock_worktree))
.route("/api/worktree/prune", post(worktree::prune_worktrees))
```

---

## TypeScript Types

Add to `src/lib/api/types.ts`:

```typescript
export interface WorktreeInfo {
  name: string;
  path: string;
  branch: string | null;
  head_oid: string | null;
  head_short_oid: string | null;
  is_main: boolean;
  is_locked: boolean;
  lock_reason: string | null;
  is_valid: boolean;
}

export interface WorktreeCreateResult {
  worktree: WorktreeInfo;
  output: OperationOutput;
}

export interface WorktreeRemoveResult {
  success: boolean;
  output: OperationOutput;
}

export interface WorktreePruneResult {
  pruned: string[];
  output: OperationOutput;
}
```

## Transport Mapping

Add to `COMMAND_MAP` in `src/lib/api/transport-http.ts`:

```typescript
// Worktree
list_worktrees: '/api/worktree/list',
add_worktree: '/api/worktree/add',
remove_worktree: '/api/worktree/remove',
lock_worktree: '/api/worktree/lock',
unlock_worktree: '/api/worktree/unlock',
prune_worktrees: '/api/worktree/prune',
```

## API Functions

New file: `src/lib/api/worktree.ts`

```typescript
import { getTransport } from '$lib/api';
import type {
  WorktreeInfo,
  WorktreeCreateResult,
  WorktreeRemoveResult,
  WorktreePruneResult,
} from '$lib/api/types';

export async function listWorktrees(path: string): Promise<WorktreeInfo[]> {
  return getTransport().invoke('list_worktrees', { path });
}

export async function addWorktree(
  path: string,
  worktreePath: string,
  branch: string | null,
  newBranch: boolean,
): Promise<WorktreeCreateResult> {
  return getTransport().invoke('add_worktree', {
    path,
    worktreePath,
    branch,
    newBranch,
  });
}

export async function removeWorktree(
  path: string,
  worktreePath: string,
  force: boolean,
): Promise<WorktreeRemoveResult> {
  return getTransport().invoke('remove_worktree', {
    path,
    worktreePath,
    force,
  });
}

export async function lockWorktree(
  path: string,
  worktreePath: string,
  reason?: string,
): Promise<WorktreeInfo[]> {
  return getTransport().invoke('lock_worktree', {
    path,
    worktreePath,
    reason: reason ?? null,
  });
}

export async function unlockWorktree(
  path: string,
  worktreePath: string,
): Promise<WorktreeInfo[]> {
  return getTransport().invoke('unlock_worktree', {
    path,
    worktreePath,
  });
}

export async function pruneWorktrees(
  path: string,
  dryRun: boolean,
): Promise<WorktreePruneResult> {
  return getTransport().invoke('prune_worktrees', {
    path,
    dryRun,
  });
}
```

---

## Svelte Store

New file: `src/lib/stores/worktree.ts`

```typescript
import { writable, derived, get } from 'svelte/store';
import { repoPath } from '$lib/stores/repo';
import { addOutput } from '$lib/stores/output';
import * as api from '$lib/api/worktree';
import type { WorktreeInfo } from '$lib/api/types';

// State
export const worktrees = writable<WorktreeInfo[]>([]);
export const worktreeLoading = writable(false);
export const worktreeError = writable<string | null>(null);

// Derived
export const linkedWorktrees = derived(worktrees, ($wt) =>
  $wt.filter((w) => !w.is_main)
);
export const worktreeCount = derived(worktrees, ($wt) => $wt.length);

// Actions
export async function refreshWorktrees(): Promise<void> {
  const path = get(repoPath);
  if (!path) return;

  worktreeLoading.set(true);
  worktreeError.set(null);

  try {
    const list = await api.listWorktrees(path);
    worktrees.set(list);
  } catch (e) {
    worktreeError.set(e instanceof Error ? e.message : String(e));
  } finally {
    worktreeLoading.set(false);
  }
}

export async function createWorktree(
  worktreePath: string,
  branch: string | null,
  newBranch: boolean,
): Promise<void> {
  const path = get(repoPath);
  if (!path) return;

  try {
    const result = await api.addWorktree(path, worktreePath, branch, newBranch);
    addOutput(`Created worktree: ${result.worktree.path} (${result.worktree.branch ?? 'detached'})`);
    await refreshWorktrees();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    worktreeError.set(msg);
    addOutput(`Failed to create worktree: ${msg}`);
    throw e;
  }
}

export async function deleteWorktree(
  worktreePath: string,
  force: boolean,
): Promise<void> {
  const path = get(repoPath);
  if (!path) return;

  try {
    const result = await api.removeWorktree(path, worktreePath, force);
    if (result.success) {
      addOutput(`Removed worktree: ${worktreePath}`);
    } else {
      addOutput(`Failed to remove worktree: ${result.output.stderr}`);
    }
    await refreshWorktrees();
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    worktreeError.set(msg);
    addOutput(`Failed to remove worktree: ${msg}`);
    throw e;
  }
}
```

---

## UI Design

### Worktree Panel Location

The worktree panel lives in the **Sidebar**, as a collapsible section below the existing Branches/Tags sections. It follows the same visual pattern: a header with a count badge and an expand/collapse chevron.

### Panel Layout

```
┌─────────────────────────────────┐
│ ▾ Worktrees (3)            [+]  │  ← header with add button
├─────────────────────────────────┤
│ ★ main                          │  ← main worktree (star icon)
│   main · abc123                  │  ← branch + short oid
│                                  │
│ ◆ feature-auth                   │  ← linked worktree
│   feat/auth · def456             │
│   ~/code/myrepo-auth             │  ← path (dimmed)
│                                  │
│ ◆ fix-perf  🔒                   │  ← locked worktree
│   fix/perf · 789abc              │
│   ~/code/myrepo-perf             │
└─────────────────────────────────┘
```

### Context Menu (Right-Click on Worktree)

For linked worktrees:

```
┌──────────────────────────┐
│ Open in New Window       │  ← Desktop: new Tauri window. Server: new browser tab.
│ Open in Terminal         │  ← Desktop only: opens OS terminal at worktree path
│ Copy Path               │
│ ─────────────────────── │
│ Lock Worktree...        │  ← or "Unlock" if already locked
│ ─────────────────────── │
│ Remove Worktree         │  ← confirmation dialog, warns about uncommitted changes
│ Force Remove            │  ← red/danger style
└──────────────────────────┘
```

For the main worktree: only "Copy Path" (cannot remove or lock the main worktree).

### Add Worktree Dialog

Triggered by the [+] button in the section header. A modal dialog with:

```
┌──────────────────────────────────────┐
│ Add Worktree                         │
│                                      │
│ Path                                 │
│ ┌──────────────────────────── [📁] ┐ │  ← text input + directory picker
│ │ ../myrepo-feature-auth           │ │
│ └──────────────────────────────────┘ │
│                                      │
│ Branch                               │
│ ○ Existing branch                    │
│   ┌────────────────────────────────┐ │  ← branch dropdown (filtered)
│   │ feat/auth                      │ │
│   └────────────────────────────────┘ │
│ ● Create new branch                 │
│   ┌────────────────────────────────┐ │  ← text input
│   │ feat/new-feature               │ │
│   └────────────────────────────────┘ │
│ ○ Detached HEAD                      │
│                                      │
│              [Cancel]  [Create]      │
└──────────────────────────────────────┘
```

### Commit Graph Integration (Milestone 2)

Worktree HEADs appear as distinct markers on the commit graph, similar to branch labels but with a different icon/color to indicate "this commit is checked out in another worktree." This gives you at-a-glance awareness of where all your parallel workstreams are.

In the `CommitGraph` rendering, for any commit whose OID matches a linked worktree's `head_oid`, render a small worktree badge:

```
● abc1234  feat/auth  [WT: feature-auth]  Add OAuth flow
● def5678  fix/perf   [WT: fix-perf]      Optimize query
● 789abcd  main       ★                    Release v1.2
```

This requires passing the worktree list to the graph component and matching on OIDs — no new backend calls needed.

---

## Open in New Window (Milestone 2)

### Desktop (Tauri)

Tauri v2 supports multi-window. When "Open in New Window" is clicked:

1. Create a new Tauri `WebviewWindow` with a URL parameter: `?repo=/path/to/worktree`
2. The new window's Svelte app reads the query param and auto-opens that path
3. Each window is fully independent — its own stores, its own state

This is straightforward with Tauri's `WebviewWindow::new()` API.

### Server (Axum)

For server mode, "Open in New Window" opens a new browser tab with `?repo=/path/to/worktree`. The frontend detects the query param and auto-opens the repo. Since the server manages repo state per-request (stateless), this works with no backend changes.

---

## Agent Gateway Integration (Milestone 3 — Sketch)

When the Agent Gateway (Phase 5) is built, worktrees become the natural isolation primitive:

1. **Agent spawns a worktree**: When an agent is granted `BranchScoped` permission, Gitron automatically creates a worktree for the agent's branch. The agent operates entirely within that worktree.

2. **Worktree lifecycle tied to agent session**: When the agent disconnects or the session ends, the worktree can be cleaned up (or left for review).

3. **UI integration**: The worktree panel shows which worktrees have active agent sessions. Clicking opens the agent's activity view.

4. **"Spawn AI Session" action**: A first-class action in the UI that:
   - Creates a new branch + worktree
   - Opens a new Gitron window for that worktree
   - Optionally launches an AI agent session (e.g., Claude Code) pointed at the worktree path
   - Shows agent activity in the original window's Agent Panel

This connects the worktree feature directly to Gitron's differentiator as an AI-native Git GUI.

---

## Implementation Checklist

Following Gitron's standard feature checklist:

### Milestone 1: Core (COMPLETE)

- [x] **Rust types** — `WorktreeInfo`, `WorktreeCreateResult`, `WorktreeRemoveResult`, `WorktreePruneResult` in `types.rs`
- [x] **Rust core logic** — `crates/gitron-core/src/git/worktree.rs` with list/add/remove/lock/unlock/prune (all via git CLI)
- [x] **Porcelain parser** — `parse_worktree_list()` with 5 unit tests (single, multiple, detached, locked, empty)
- [x] **Tauri commands** — `src-tauri/src/commands/worktree.rs` with 6 commands, registered in `lib.rs`
- [x] **Server routes** — `crates/gitron-server/src/routes/worktree.rs` with 6 Axum handlers, registered in `mod.rs`
- [x] **TS types** — 4 worktree interfaces in `src/lib/api/types.ts`
- [x] **COMMAND_MAP** — 6 worktree entries in `transport-http.ts`
- [x] **API functions** — `src/lib/api/worktree.ts` with 6 functions
- [x] **Store** — `src/lib/stores/worktree.ts` with state, derived stores, 5 action functions, and `showAddWorktreeDialog` writable for cross-component dialog control
- [x] **Sidebar section** — `WorktreeSection.svelte` component with collapsible list, star/diamond icons, lock/invalid indicators
- [x] **Context menu** — Right-click: Copy Path, Lock/Unlock, Remove, Force Remove
- [x] **Add dialog** — Modal with path input, new branch / existing branch / detached HEAD radio selection
- [x] **Remove confirmation** — Confirmation dialog with force option and warning text
- [x] **Auto-refresh** — Refreshes worktree list on repo open via `$effect`
- [x] **Command palette** — Worktrees group in Cmd+K: Add Worktree, linked worktree list with Remove, Prune Stale Worktrees

### Milestone 2: Integration (COMPLETE)

- [x] **Graph badges** — Worktree HEAD markers on the commit graph (diamond icon + worktree name, styled as `.worktree-pill` with primary color, in both search and normal rendering modes)
- [x] **Open in new window** — Tauri: `WebviewWindow` with `?repo=` param; Server: `window.open()` with `?repo=` param. Available from context menu and command palette.
- [x] **Open in new tab** — `+page.svelte` reads `?repo=` query parameter and auto-opens the repo, supporting both Tauri and server modes
- [x] **Open in terminal** — Desktop-only: `open_in_terminal` Tauri command spawns OS terminal at worktree path (macOS/Linux/Windows). Context menu item gated by `isTauri()`.

### Milestone 3: Agent-Native (Phase 5 prerequisite)

- [ ] **Agent worktree lifecycle** — Auto-create/cleanup worktrees for agent sessions
- [ ] **Spawn AI session** — UI action to create branch + worktree + launch agent
- [ ] **Agent indicators** — Show active agent sessions on worktree entries

---

## Git Version Requirements

- `git worktree` — Git 2.5+ (2015)
- `git worktree list --porcelain` — Git 2.7+ (2016)
- `git worktree lock/unlock` — Git 2.10+ (2016)
- `git worktree remove` — Git 2.17+ (2018)

Gitron already requires Git 2.30+ (documented in README), so all worktree features are available.

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| User deletes worktree directory manually (outside git) | `is_valid` field checks path existence; prune button cleans stale refs |
| Worktree branch gets deleted while worktree exists | Git prevents this by default (`branch is checked out` error) |
| Merge conflicts when recombining parallel work | Not a worktree-specific problem — standard merge/rebase tooling handles this |
| Disk space from multiple checkouts | Worktrees share the object store; only working tree files are duplicated. Show disk usage in UI as a nice-to-have. |
| `pnpm install` needed per worktree (monorepos) | Document this. Future: detect package manager and offer to run install on creation. |

---

## Effort Estimate

| Milestone | Scope | Complexity |
|-----------|-------|------------|
| 1 (Core) | Rust + commands + routes + types + UI panel | Medium — follows established patterns exactly. Stash was similar scope. |
| 2 (Integration) | Graph badges + multi-window + terminal | Medium — multi-window is the main new concept. |
| 3 (Agent-Native) | Depends on Agent Gateway (Phase 5) | Deferred — design only for now. |

Milestone 1 is a self-contained unit of work that delivers immediate value. Milestones 2 and 3 build on it incrementally.

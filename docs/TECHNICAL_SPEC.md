# Gitron — Technical Specification

This document is the definitive reference for how Gitron works. It covers the complete IPC contract, state management model, event system, data lifecycle, error handling, and runtime behavior of every major subsystem. Future contributors and AI assistants should consult this document before making changes.

---

## Table of Contents

1. [System Boundaries](#system-boundaries)
2. [Runtime Lifecycle](#runtime-lifecycle)
3. [IPC Command Reference](#ipc-command-reference)
4. [Event System](#event-system)
5. [State Management](#state-management)
6. [Data Types Contract](#data-types-contract)
7. [Git Operations Layer](#git-operations-layer)
8. [File Watcher System](#file-watcher-system)
9. [Repo State Cache](#repo-state-cache)
10. [Commit Graph Rendering](#commit-graph-rendering)
11. [Error Handling](#error-handling)
12. [Concurrency Model](#concurrency-model)
13. [Security Model](#security-model)

---

## 1. System Boundaries

Gitron is split into two processes that communicate via Tauri IPC:

```
┌─────────────────────┐         ┌─────────────────────┐
│   Webview Process   │  IPC    │    Rust Process      │
│   (Svelte App)      │◄───────►│    (Tauri Backend)   │
│                     │         │                      │
│ - UI rendering      │         │ - Git operations     │
│ - User interaction  │         │ - File system access │
│ - State stores      │         │ - IPC handlers       │
│ - Graph canvas      │         │ - File watcher       │
│                     │         │ - Cache management   │
└─────────────────────┘         └──────────────────────┘
```

**The frontend NEVER accesses the filesystem directly.** All file and git operations go through Tauri IPC commands to the Rust backend.

**The backend NEVER renders UI.** It only sends data and events to the frontend.

### Communication Patterns

1. **Request/Response** — Frontend calls `invoke('command', args)`, backend returns data or error. Used for all user-initiated operations.
2. **Push Events** — Backend emits events via Tauri's event system. Frontend subscribes and reacts. Used for file watcher notifications and background updates.

---

## 2. Runtime Lifecycle

### Application Startup

```
1. main.rs → gitron_lib::run()
2. Tauri Builder initializes:
   a. Registers plugins (tauri-plugin-opener, tauri-plugin-dialog, tauri-plugin-store)
   b. Registers IPC command handlers (repo, graph, diff, staging, branch, commit)
   c. Creates native window (1280x800, min 900x600)
3. Webview loads SvelteKit app
4. Frontend renders AppShell with empty state (no repo open)
5. Frontend loads persisted settings (lastActiveRepo, recentRepos) via tauri-plugin-store
6. If lastActiveRepo is set, auto-opens it; otherwise shows welcome screen with Cmd+K hint
```

### Repository Open Flow

```
1. User enters path in toolbar, presses Enter or clicks Open
2. Frontend: openRepo(path) action called
   a. Sets loading = true, error = null
   b. Calls invoke('open_repo', { path })
3. Backend: open_repo command handler
   a. Calls git2::Repository::discover(path)
   b. If valid repo found: extracts RepoInfo (path, workdir, head, etc.)
   c. If not a repo: returns GitError::NotARepository
4. Frontend receives RepoInfo
   a. Sets repoPath, repoInfo stores
   b. Calls refreshAll(path) which fires two parallel requests:
      - invoke('get_status', { path }) → RepoStatus
      - invoke('get_commit_graph', { path }) → CommitGraph
   c. Sets repoStatus, commitGraph stores
   d. Sets loading = false
5. UI reactively updates:
   - Sidebar shows staged/unstaged/untracked files
   - CommitGraph renders commit list with branch labels
   - Toolbar shows current branch badge
   - StatusBar shows branch name and change counts
```

### Repository Close / Switch

```
1. User enters a new path in toolbar
2. All stores are reset via openRepo() (overwrites previous state)
3. Previous repo state is discarded (no persistence yet)
```

---

## 3. IPC Command Reference

Every Tauri command is defined in `src-tauri/src/commands/`. Each command takes a `path: String` parameter identifying the repository. This path is used to open the repository on every call (stateless design — no persistent repo handle across calls).

### Repository Commands (`commands/repo.rs`)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `open_repo` | `path: String` | `RepoInfo` | Validates the path is a git repo and returns basic info |
| `get_status` | `path: String` | `RepoStatus` | Returns staged, unstaged, untracked, and conflicted files |
| `get_repo_info` | `path: String` | `RepoInfo` | Returns current repo info (head, branch, bare status) |

### Graph Commands (`commands/graph.rs`)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_commit_graph` | `path: String`, `max_commits: Option<usize>`, `include_remotes: Option<bool>` | `CommitGraph` | Returns the commit graph (commits, branches, tags) |
| `get_commit_detail` | `path: String`, `oid: String` | `Commit` | Returns detailed info for a single commit |

### Diff Commands (`commands/diff.rs`)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `get_workdir_diff` | `path: String` | `Vec<FileDiff>` | Returns diffs for all unstaged workdir changes |
| `get_file_diff` | `path: String`, `file_path: String` | `FileDiff` | Returns diff for a single unstaged file |
| `get_staged_file_diff` | `path: String`, `file_path: String` | `FileDiff` | Returns diff for a single staged file (tree-to-index) |

### Staging Commands (`commands/staging.rs`)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `stage_file` | `path: String`, `file_path: String` | `RepoStatus` | Stages a file and returns updated status |
| `unstage_file` | `path: String`, `file_path: String` | `RepoStatus` | Unstages a file and returns updated status |
| `stage_files` | `path: String`, `file_paths: Vec<String>` | `RepoStatus` | Stages multiple files and returns updated status |
| `stage_all` | `path: String` | `RepoStatus` | Stages all changes and returns updated status |
| `unstage_all` | `path: String` | `RepoStatus` | Unstages all changes and returns updated status |

### Branch Commands (`commands/branch.rs`)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_branches` | `path: String` | `Vec<Branch>` | Lists all local and remote branches |
| `create_branch` | `path: String`, `name: String`, `target: Option<String>` | `Branch` | Creates a new branch (defaults to HEAD) |
| `checkout_branch` | `path: String`, `name: String` | `RepoInfo` | Checks out a branch and returns updated repo info |
| `delete_branch` | `path: String`, `name: String` | `Vec<Branch>` | Deletes a local branch and returns updated list |

### Commit Commands (`commands/commit.rs`)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `create_commit` | `path: String`, `message: String` | `String` | Creates a commit with the given message. Returns the new commit OID. Uses the repo's git config for author/committer signature. Handles initial (parentless) commits. |

### AI Commands (`commands/ai.rs`)

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `ai_get_providers` | — | `Vec<AIProvider>` | Returns all configured providers with `has_key` status and fallback models |
| `ai_save_key` | `provider: String`, `key: String` | `()` | Stores an API key in the OS keychain |
| `ai_delete_key` | `provider: String` | `()` | Removes an API key from the OS keychain |
| `ai_fetch_models` | `provider: String`, `base_url: Option<String>` | `Vec<AIModel>` | Fetches available models from a provider's API (requires API key) |
| `ai_generate_commit_message` | `path: String`, `provider: String`, `model: String`, `base_url: Option<String>`, `max_tokens: Option<u32>` | `GenerateResult` | Generates a commit message from staged diffs using the specified AI provider/model |
| `ai_get_settings` | `app: AppHandle` | `AISettings` | Reads AI settings from `tauri-plugin-store` |
| `ai_save_settings` | `app: AppHandle`, `settings: AISettings` | `()` | Writes AI settings to `tauri-plugin-store` |

### Command Design Principles

1. **Stateless**: Each command opens the repository fresh. No shared mutable state between calls. This simplifies concurrency and error recovery at the cost of a small overhead per call (mitigated by OS filesystem caching).

2. **Return updated state**: Mutation commands (stage, unstage, branch operations) return the updated state so the frontend can immediately reflect changes without a separate refresh call.

3. **Thin handlers**: Commands contain zero business logic. They parse parameters, delegate to `git/` module functions, and return results. All git logic lives in the `git/` module.

4. **Serializable errors**: `GitError` implements `serde::Serialize` so errors can cross the IPC boundary as strings.

---

## 4. Event System

Tauri events flow from backend to frontend. The frontend subscribes to events and updates stores reactively.

### Event Types (Planned)

| Event Name | Payload | Trigger |
|-----------|---------|---------|
| `repo:status-changed` | `RepoStatus` | File watcher detects workdir/index change |
| `repo:head-changed` | `{ head_oid: String, head_branch: Option<String> }` | HEAD moved (commit, checkout, reset) |
| `repo:refs-changed` | `{ branches: Vec<Branch>, tags: Vec<Tag> }` | Branch/tag created, deleted, or updated |
| `repo:config-changed` | `{}` | Git config file modified |
| `agent:action-proposed` | `AgentAction` | Agent submits an action for review (Phase 5) |
| `agent:action-resolved` | `{ id: Uuid, status: ActionStatus }` | Action approved/rejected/executed (Phase 5) |

### Event Flow

```
File change on disk
  → notify-rs detects event
  → Debouncer batches events (200ms window)
  → Watcher handler classifies change:
      .git/* changes → RefsChanged or HeadChanged
      workdir changes → WorkdirChanged
  → Handler re-reads affected state from git2-rs
  → Handler emits Tauri event with new data
  → Frontend store subscription receives event
  → Svelte reactivity updates affected components
```

### Frontend Event Subscription Pattern

```typescript
// In a Svelte component or store initialization
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<RepoStatus>('repo:status-changed', (event) => {
  repoStatus.set(event.payload);
});

// Clean up on component destroy
onDestroy(() => unlisten());
```

---

## 5. State Management

### Frontend State Architecture

All frontend state lives in Svelte stores. Repository state is in `src/lib/stores/repo.ts` and settings/persistence state is in `src/lib/stores/settings.ts`. There is no additional state management library.

```
Repository Stores (stores/repo.ts):
├── repoPath: Writable<string | null>       — path to the open repository
├── repoInfo: Writable<RepoInfo | null>     — basic repo metadata
├── repoStatus: Writable<RepoStatus | null> — staged/unstaged/untracked files
├── commitGraph: Writable<CommitGraph | null> — commits, branches, tags
├── selectedCommit: Writable<Commit | null> — currently selected commit
├── selectedFileDiff: Writable<FileDiff | null> — currently viewed diff
├── selectedFile: Writable<SelectedFileInfo | null> — currently selected file (path + section)
├── loading: Writable<boolean>              — global loading state
├── error: Writable<string | null>          — last error message
│
Derived Stores:
├── hasRepo: Derived<boolean>               — whether a repo is open
├── isFileSelected: Derived<boolean>        — whether a file is selected for diff
├── currentBranch: Derived<string | null>   — current HEAD branch name
├── localBranches: Derived<Branch[]>        — non-remote branches
├── remoteBranches: Derived<Branch[]>       — remote branches
├── stagedCount: Derived<number>            — number of staged files
└── unstagedCount: Derived<number>          — unstaged + untracked count

Settings Stores (stores/settings.ts):
├── recentRepos: Writable<RecentRepo[]>     — recently opened repos (max 20)
├── lastActiveRepo: Writable<string | null> — last opened repo path (auto-opened on launch)
├── settingsLoaded: Writable<boolean>       — whether settings have loaded
├── graphColumnWidths: Writable<GraphColumnWidths> — persisted column widths
│
Derived:
└── sortedRecentRepos: Derived<RecentRepo[]> — pinned repos first, then sorted by lastOpened
```

Types used:
- `FileSection = 'staged' | 'unstaged' | 'untracked'`
- `SelectedFileInfo = { path: string, section: FileSection }`
- `RecentRepo = { path, name, lastOpened, pinned }`
- `GraphColumnWidths = { graph, author, date, sha }` (numbers in px)

### State Update Rules

1. **Single source of truth**: The Rust backend is the authority on git state. Frontend stores are projections of backend state.
2. **Optimistic updates are NOT used**: The frontend always waits for the backend to confirm before updating. This prevents state drift at the cost of slightly more latency (acceptable because git2-rs is fast).
3. **Batch updates**: When opening a repo, status and graph are fetched in parallel (`Promise.all`). Both arrive before the UI updates.
4. **Error isolation**: Errors are captured per-action and stored in the `error` store. They do not prevent other operations from proceeding.

### Backend State

The Rust backend is currently **stateless** — each command opens the repository fresh. This is intentional for Phase 1 simplicity.

In future phases, the `AppState` struct (defined in `commands/mod.rs`) will hold:
- A persistent `Repository` handle (avoids re-open overhead)
- The `RepoStateCache` (in-memory cached state)
- The file watcher handle (keeps watcher alive)
- Plugin registry
- Agent gateway

```rust
pub struct AppState {
    pub repo: Mutex<Option<Repository>>,
    pub repo_path: Mutex<Option<PathBuf>>,
    // Future: cache, watcher, plugins, agents
}
```

This will be registered as Tauri managed state:
```rust
tauri::Builder::default()
    .manage(AppState::new())
    // ...
```

Commands will then access it via `state: State<'_, AppState>` parameter.

---

## 6. Data Types Contract

Types are defined in two mirrored locations and MUST stay in sync:

- **Rust**: `src-tauri/src/git/types.rs`
- **TypeScript**: `src/lib/api/types.ts`

When a type is added or modified in Rust, the TypeScript mirror MUST be updated. There is no code generation — this is manual.

### Core Types

#### Commit
```
{
  oid: string              — full 40-char SHA
  short_oid: string        — first 7 chars of SHA
  message: string          — full commit message (may be multiline)
  summary: string          — first line of commit message
  author: Signature        — author name + email
  committer: Signature     — committer name + email
  parents: string[]        — OIDs of parent commits (0 = root, 1 = normal, 2+ = merge)
  timestamp: string        — ISO 8601 datetime (UTC)
}
```

#### CommitGraph
```
{
  commits: Commit[]        — ordered by topological + time sort
  branches: Branch[]       — all branches (local + remote)
  tags: Tag[]              — all tags
  head_oid: string | null  — OID that HEAD points to
  head_branch: string | null — branch name if HEAD is on a branch (null if detached)
}
```

#### RepoStatus
```
{
  staged: FileStatus[]     — files in the index (staged for commit)
  unstaged: FileStatus[]   — modified files in workdir not yet staged
  untracked: string[]      — new files not tracked by git (paths only)
  conflicted: string[]     — files with merge conflicts (paths only)
}
```

#### FileStatus
```
{
  path: string             — file path relative to repo root
  status: FileStatusType   — one of: Added, Modified, Deleted, Renamed, Copied, TypeChanged
}
```

#### FileDiff
```
{
  path: string             — file path
  old_path: string | null  — previous path (for renames)
  hunks: DiffHunk[]        — list of diff hunks
  is_binary: boolean       — true if binary file (no hunks)
  status: FileStatusType   — file change type
}
```

#### DiffHunk
```
{
  header: string           — hunk header (@@ line)
  old_start: number        — starting line in old file
  old_lines: number        — number of lines in old file
  new_start: number        — starting line in new file
  new_lines: number        — number of lines in new file
  lines: DiffLine[]        — individual diff lines
}
```

#### DiffLine
```
{
  origin: DiffLineType     — Context, Addition, Deletion, or Header
  content: string          — the actual line content
  old_lineno: number | null — line number in old file (null for additions)
  new_lineno: number | null — line number in new file (null for deletions)
}
```

#### Branch
```
{
  name: string             — branch name (e.g., "main", "origin/main")
  is_head: boolean         — true if this is the current checked-out branch
  is_remote: boolean       — true if this is a remote-tracking branch
  upstream: string | null  — name of upstream branch (if tracking)
  target_oid: string | null — OID the branch points to
}
```

#### RepoInfo
```
{
  path: string             — path to .git directory
  workdir: string          — path to working directory
  head_branch: string | null — current branch name (null if detached HEAD)
  head_oid: string | null  — OID of HEAD commit
  is_bare: boolean         — true if bare repository
  is_empty: boolean        — true if no commits
}
```

#### AI Types

AI types are defined in `src-tauri/src/ai/types.rs` and mirrored in `src/lib/api/types.ts`.

```
AIProvider {
  id: string              — provider identifier ("openai", "anthropic", "gemini", "openrouter")
  name: string            — display name
  has_key: boolean        — whether an API key exists in the keychain
  models: AIModel[]       — available models (fallback or fetched from API)
  base_url: string | null — default API endpoint URL
}

AIModel {
  id: string              — model identifier (e.g., "gpt-4.1-nano", "claude-haiku-4-5-20251001")
  name: string            — display name
}

GenerateResult {
  title: string           — commit title line (first line of response)
  body: string            — commit body (remaining lines after blank line separator)
}

AISettings {
  selected_provider: string | null  — active provider ID
  selected_model: string | null     — active model ID
  selected_models: Record<string, string>  — per-provider model memory (restores when switching)
  custom_base_urls: Record<string, string> — per-provider custom endpoint overrides
  max_tokens: number      — max output tokens for generation (default: 1500)
}
```

---

## 7. Git Operations Layer

### Module Structure

```
src-tauri/src/git/
├── mod.rs          — module declarations
├── types.rs        — all data types (Commit, Branch, FileDiff, etc.)
├── error.rs        — GitError enum and GitResult type alias
├── repository.rs   — repo open, status, staging, branch operations
├── graph.rs        — commit graph building and traversal
└── diff.rs         — diff computation (workdir, staged, per-file)
```

### Separation of Concerns

The `git/` module contains ALL git logic. The `commands/` module contains ZERO git logic — it only bridges IPC to `git/`.

```
commands/repo.rs     →  git/repository.rs  (open, status, stage, branch)
commands/graph.rs    →  git/graph.rs       (commit graph, commit detail)
commands/diff.rs     →  git/diff.rs        (workdir diff, file diff)
commands/staging.rs  →  git/repository.rs  (stage_file, unstage_file)
commands/branch.rs   →  git/repository.rs  (list, create, checkout, delete)
```

### git2-rs Usage Patterns

**Repository discovery**: `Repository::discover(path)` walks up from the given path to find a `.git` directory. This allows users to specify any path within a repo.

**Status reading**: Uses `repo.statuses()` with options to include untracked files, recurse directories, and exclude ignored files.

**Graph building**: Uses `repo.revwalk()` with topological + time sorting. Pushes HEAD and all reference tips, then walks up to `max_commits` (default 500).

**Diff computation**: Uses `repo.diff_index_to_workdir()` for unstaged changes and `repo.diff_tree_to_index()` for staged changes. Diffs are parsed via the print callback API.

**Staging**: Uses `repo.index()` to modify the index. `index.add_path()` for staging, `repo.reset_default()` for unstaging. Index is written after each operation.

**Branch operations**: Uses `repo.branches()` for listing, `repo.branch()` for creation, `branch.delete()` for deletion, `repo.checkout_tree()` + `repo.set_head()` for checkout.

### Git CLI Bridge (Future)

For operations not supported by git2-rs, a CLI bridge will shell out to the `git` binary:

```rust
// Future: src-tauri/src/git/cli_backend.rs
use std::process::Command;

fn interactive_rebase(repo_path: &Path, onto: &str) -> GitResult<()> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["rebase", "-i", onto])
        .output()?;
    // Parse output...
}
```

Operations designated for CLI bridge:
- Interactive rebase
- Advanced merge strategies
- Submodule operations
- Git-flow commands
- Bisect
- Reflog manipulation

---

## 8. File Watcher System

### Architecture

```
src-tauri/src/watcher/
├── mod.rs          — module declaration
└── handler.rs      — watcher setup and event classification
```

### How It Works

1. When a repository is opened, `watch_repo()` is called with the repo path
2. `notify-rs` creates an OS-native file watcher (FSEvents on macOS, ReadDirectoryChangesW on Windows)
3. Events are debounced with a 200ms window using `notify-debouncer-mini`
4. The debounced callback classifies events:
   - Paths containing `.git` → `RepoChangeEvent::RefsChanged`
   - All other paths → `RepoChangeEvent::WorkdirChanged`
5. Events are sent through an `mpsc` channel to the handler
6. The handler triggers appropriate cache updates and Tauri events

### Event Classification Logic

```
File changed: src/main.rs          → WorkdirChanged (workdir file modified)
File changed: .git/refs/heads/main → RefsChanged (branch ref updated)
File changed: .git/HEAD            → RefsChanged (HEAD moved)
File changed: .git/index           → RefsChanged (index changed externally)
File changed: .git/config          → RefsChanged (config changed)
```

### Lifecycle

```
Repo opened → watcher starts → events flow → repo closed → watcher dropped
```

The `Debouncer` is owned by whoever starts the watcher. When it's dropped, the watcher stops automatically (RAII pattern).

---

## 9. Repo State Cache

### Architecture

```
src-tauri/src/cache/
├── mod.rs          — module declaration
└── repo_state.rs   — RepoStateCache struct
```

### Purpose

The cache holds a computed snapshot of the repository state in memory so the frontend can receive instant responses without re-reading git objects. The file watcher triggers incremental updates to the cache.

### Cache Contents

```rust
struct CachedState {
    repo_info: RepoInfo,     // head, branch, paths
    status: RepoStatus,      // staged, unstaged, untracked, conflicted
    graph: CommitGraph,      // commits, branches, tags
    last_updated: Instant,   // when last refreshed
}
```

### Thread Safety

`RepoStateCache` uses `Arc<RwLock<Option<CachedState>>>`:
- Multiple readers can read concurrently (UI queries)
- Writers acquire exclusive access (watcher updates)
- `Option` wraps the state so it can be `None` when no repo is open

### Update Strategy (Planned)

1. **Full rebuild**: On repo open, the entire cache is populated from scratch
2. **Status-only update**: When `WorkdirChanged` event fires, only re-read status (fast operation)
3. **Graph update**: When `RefsChanged` event fires, re-build the commit graph (more expensive but still fast for typical repos)
4. **Invalidation**: On repo close or path change, cache is cleared

---

## 10. Commit Graph Rendering

### Current Implementation

The commit graph is rendered as a styled HTML grid with one row per commit. Each row shows:

```
[Graph dot] [Branch labels] [Commit summary] [Author] [Date] [SHA]
```

Branch labels are color-coded using a cycling palette of 10 colors:
- Current branch (HEAD): solid background with white text, white circle stroke on graph dot
- Local branches: outlined with colored border
- Remote branches: dashed outline, reduced opacity

Branch colors cycle through: aqua, green, orange, red, purple, teal, lime, coral, pink, indigo.

### Column Layout

Columns are **resizable** via drag handles between them. Widths are persisted to disk via `tauri-plugin-store` through the `graphColumnWidths` settings store.

| Column | Default Width | Content |
|--------|--------------|---------|
| Graph | 40px | SVG circle (colored by branch index) |
| Message | flex: 1 | Branch labels + commit summary |
| Author | 140px | Author name |
| Date | 80px | Relative time (e.g., "2h ago") |
| SHA | 70px | Short OID (7 chars, monospace) |

### Interaction

- **Click a commit**: Sets `selectedCommit` store, shows `CommitDetail` panel below the graph
- **Keyboard navigation**: `ArrowUp`/`ArrowDown` moves through commits (scrolls into view)
- **CommitDetail panel**: Collapsible — click to toggle expand/collapse
  - Collapsed: single row with chevron, summary, author, date, short OID
  - Expanded: full message (monospace pre block), metadata table (Author, Date, SHA, Parents)
  - Parent SHAs are displayed as 7-char links

### Date Formatting

Relative time display: "just now" (< 1 min), "Nm ago" (< 1 hr), "Nh ago" (< 1 day), "Nd ago" (< 30 days), locale date string beyond.

### Diff Viewer (FilePreview)

When a file is selected in the sidebar, the graph view is replaced by the `FilePreview` component — a full-area inline diff viewer with syntax highlighting.

Features:
- **Syntax highlighting** via `shiki` (Catppuccin Mocha theme, singleton pattern with lazy init)
- Preloaded languages: js, ts, jsx, tsx, rust, json, toml, yaml, html, css, svelte, markdown, bash, python
- Per-line rendering: gutter (line number + origin char), syntax-highlighted content
- Hunk separators with skipped-line count
- Binary file and empty diff states
- Keyboard shortcuts: `ArrowDown`/`ArrowUp` navigate files, `Escape` closes, `s` stages, `u` unstages

### Settings Persistence

Application settings are persisted via `tauri-plugin-store` (`settings.json`).

Persisted data:
- `lastActiveRepo` — auto-opened on app launch
- `recentRepos` — up to 20 recently opened repos (with pinned flag, sorted pinned-first)
- `graphColumnWidths` — commit graph column width preferences

Settings API (`src/lib/api/settings.ts`):
- `getSettings()`, `addRecentRepo(path)`, `removeRecentRepo(path)`, `togglePinRepo(path)`
- `getColumnWidths()`, `saveColumnWidths(widths)`

### Command Palette (CommandBar)

The `CommandBar` component (built on `bits-ui` Command primitive) provides a searchable command palette:
- Opened via `Cmd+K` / `Ctrl+K` or clicking the branch badge in the toolbar
- Sections: Recent Repositories (up to 20, pinned-first), "Open Repository..." (native folder dialog), Local Branches, Remote Branches
- Selecting a branch triggers checkout; selecting a repo opens it

### Future: Canvas-Based Rendering

The current HTML table approach works for moderate repos but will not scale to repos with thousands of commits. The planned upgrade:

1. Replace the HTML table with a `<canvas>` element
2. Compute graph lane assignments in Rust (which commit goes in which column)
3. Send lane data alongside commits to the frontend
4. Render lines, nodes, and labels on canvas with virtualized scrolling (only draw visible rows)
5. Use GPU-accelerated rendering for smooth scrolling

Graph lane assignment algorithm:
```
For each commit in topological order:
  1. Assign to an existing lane if a parent is in that lane
  2. Otherwise, open a new lane
  3. Draw merge lines when a commit has multiple parents in different lanes
  4. Close lanes when a branch has been fully merged
```

---

## 11. Error Handling

### Backend Errors

All git operations return `GitResult<T>` which is `Result<T, GitError>`.

```rust
pub enum GitError {
    NotARepository(String),   // Path is not inside a git repo
    RepoNotOpen,              // Operation requires an open repo
    BranchNotFound(String),   // Named branch doesn't exist
    CommitNotFound(String),   // OID doesn't match any commit
    ReferenceError(String),   // Invalid reference
    StagingError(String),     // Staging operation failed
    Git2(git2::Error),        // Wrapped git2-rs error
    Io(std::io::Error),       // File system error
    Other(String),            // Catch-all
}
```

`GitError` implements `serde::Serialize` by converting to its `Display` string. This is how errors cross the IPC boundary.

### Frontend Error Handling

```typescript
// Actions catch errors and store them
try {
  const result = await api.someOperation(path);
  someStore.set(result);
} catch (e) {
  error.set(String(e));  // GitError becomes a string via Tauri
}
```

Errors are stored in the global `error` store. The UI can display them. Errors don't crash the app or prevent other operations.

### Error Recovery

- **Repo not found**: User sees error, can enter a different path
- **Git operation fails**: Error is displayed, repo state is not corrupted (git2-rs operations are atomic)
- **File watcher fails**: Watcher is dropped, live updates stop. User can manually refresh.

---

## 12. Concurrency Model

### Rust Backend

- **Tauri commands run on a thread pool**: Tauri dispatches IPC commands to a thread pool. Multiple commands can execute concurrently.
- **git2::Repository is NOT thread-safe**: Currently, each command opens its own `Repository` instance, so there's no shared mutable state. When we move to a persistent `Repository` in `AppState`, it will be wrapped in a `Mutex`.
- **File watcher runs on its own thread**: `notify-rs` creates a background thread for filesystem events.
- **Cache access is lock-based**: `RwLock` allows concurrent reads, exclusive writes.

### Frontend

- **Single-threaded**: The Svelte app runs on the webview's main thread.
- **Async IPC**: All `invoke()` calls are async and non-blocking. The webview remains responsive while waiting for backend responses.
- **Parallel fetches**: Independent data fetches (status + graph) are done with `Promise.all`.

### Future: Tokio

The backend includes `tokio` as a dependency for future async operations:
- Streaming large diffs
- Background git fetch/push operations
- Agent gateway server
- MCP server

---

## 13. Security Model

### Filesystem Access

- The app has full filesystem access via the Rust backend (required for git operations)
- Tauri's CSP is set to `null` (no restrictions) — this is intentional for a desktop git tool
- No remote code execution — the frontend is bundled static files, not loaded from the internet

### User Input

- Repository paths come from user input in the toolbar — they are passed directly to `Repository::discover()` which validates them
- File paths in staging/diff commands are relative to the repo root and validated by git2-rs
- Branch names are validated by git2-rs
- No SQL, no eval, no template injection surfaces

### Plugin Security (Future)

- Backend plugins run in-process (Rust) — they are trusted code
- Frontend plugins run in the webview — they have the same permissions as the core app
- Plugin sandboxing will restrict filesystem and network access based on declared permissions
- Plugins will be loaded from a configured directory, not from the internet at runtime

### Agent Security (Future)

- Agents connect via MCP with explicit authentication
- Each agent has a permission level (ReadOnly, BranchScoped, FullAccess)
- Each agent has an approval mode (HumanInTheLoop, AutoApprove, FullyAutonomous)
- Agent actions go through an action queue — even FullyAutonomous agents have their actions logged
- Agent permissions are configured per-repo, per-agent

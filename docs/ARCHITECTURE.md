# Gitron — Architecture

## System Overview

```
┌──────────────────────────────────────────────────┐
│                Svelte Frontend                    │
│  ┌───────────┐  ┌───────────┐  ┌──────────────┐  │
│  │ Core UI   │  │ Plugin    │  │ Agent        │  │
│  │ (graph,   │  │ Panels    │  │ Activity     │  │
│  │  diff,    │  │ (dynamic  │  │ View         │  │
│  │  stage)   │  │  slots)   │  │              │  │
│  └───────────┘  └───────────┘  └──────────────┘  │
│  ┌─────────────────────────────────────────────┐  │
│  │       Frontend Plugin API (TypeScript)      │  │
│  └───────────────────┬─────────────────────────┘  │
└──────────────────────┼────────────────────────────┘
                       │ Tauri IPC (commands + events)
┌──────────────────────▼────────────────────────────┐
│                Rust Backend                        │
│  ┌─────────────────────────────────────────────┐   │
│  │            Core API Layer                   │   │
│  │   (all access goes through here)            │   │
│  └──┬──────────┬───────────┬──────────┬────────┘   │
│     │          │           │          │            │
│  ┌──▼────┐  ┌──▼─────┐  ┌─▼──────┐  ┌▼─────────┐  │
│  │git2   │  │git CLI │  │File    │  │Repo      │  │
│  │-rs    │  │bridge  │  │Watcher │  │State     │  │
│  │       │  │        │  │(notify)│  │Cache     │  │
│  └───────┘  └────────┘  └────────┘  └──────────┘  │
│                                                    │
│  ┌─────────────────────────────────────────────┐   │
│  │          Plugin Host (Rust)                 │   │
│  │  load, lifecycle, sandboxing                │   │
│  └─────────────────────────────────────────────┘   │
│                                                    │
│  ┌─────────────────────────────────────────────┐   │
│  │          Agent Gateway                      │   │
│  │  ┌──────────┐  ┌───────────────────────┐    │   │
│  │  │ MCP      │  │ Agent Auth &          │    │   │
│  │  │ Server   │  │ Permissions           │    │   │
│  │  └──────────┘  └───────────────────────┘    │   │
│  │  ┌──────────┐  ┌───────────────────────┐    │   │
│  │  │ Event    │  │ Action Queue          │    │   │
│  │  │ Stream   │  │ (approve/reject)      │    │   │
│  │  └──────────┘  └───────────────────────┘    │   │
│  └─────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────┘
```

## Project Structure

```
gitron/
├── docs/                        # Project documentation
│   ├── README.md                # Doc index and reading order
│   ├── VISION.md
│   ├── ARCHITECTURE.md          # (this file)
│   ├── TECHNICAL_SPEC.md
│   ├── DEVELOPER_GUIDE.md
│   ├── SPECIALISTS.md
│   ├── ROADMAP.md
│   ├── PLUGIN_SYSTEM.md
│   └── AGENT_GATEWAY.md
├── src-tauri/                   # Rust backend (Tauri app)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json # Tauri capability grants (core, opener, dialog, store)
│   ├── icons/
│   └── src/
│       ├── main.rs              # Entry point (calls gitron_lib::run())
│       ├── lib.rs               # Tauri builder, plugin + command registration
│       ├── commands/            # Tauri IPC command handlers (thin, no git logic)
│       │   ├── mod.rs           # Module declarations, AppState struct (not yet wired)
│       │   ├── repo.rs          # open_repo, get_status, get_repo_info
│       │   ├── graph.rs         # get_commit_graph, get_commit_detail
│       │   ├── diff.rs          # get_workdir_diff, get_file_diff, get_staged_file_diff
│       │   ├── staging.rs       # stage_file, unstage_file, stage_files, stage_all, unstage_all
│       │   ├── branch.rs        # list_branches, create_branch, checkout_branch, delete_branch
│       │   ├── commit.rs        # create_commit
│       │   └── ai.rs            # AI commands: get_providers, save/delete key, fetch_models, generate, settings
│       ├── git/                 # Core git logic (all git2-rs calls live here)
│       │   ├── mod.rs           # Re-exports: types, error, repository, graph, diff
│       │   ├── types.rs         # All data types (Commit, Branch, FileDiff, RepoStatus, etc.)
│       │   ├── error.rs         # GitError enum + GitResult type alias
│       │   ├── repository.rs    # Repo open, status, staging, branch CRUD, checkout, commit
│       │   ├── graph.rs         # Commit graph building (revwalk, branch/tag collection)
│       │   └── diff.rs          # Diff computation (workdir, staged, per-file)
│       ├── ai/                  # AI commit message generation
│       │   ├── mod.rs           # Module declarations
│       │   ├── error.rs         # AIError enum + AIResult type alias
│       │   ├── credential.rs    # API key storage via credential_store (OS keychain)
│       │   ├── types.rs         # AIProvider, AIModel, GenerateResult, AISettings
│       │   ├── providers.rs     # Provider defs, default models, dynamic model fetching
│       │   └── generate.rs      # Prompt building, provider-specific API calls, response parsing
│       ├── cache/               # Repo state cache (implemented, not yet wired)
│       │   ├── mod.rs
│       │   └── repo_state.rs    # RepoStateCache with Arc<RwLock<Option<CachedState>>>
│       └── watcher/             # File system watcher (implemented, not yet wired to events)
│           ├── mod.rs
│           └── handler.rs       # watch_repo(), event classification, 200ms debounce
├── src/                         # Svelte 5 frontend (SPA mode)
│   ├── app.html                 # HTML shell
│   ├── app.css                  # Design system (TailwindCSS v4, oklch colors, shadcn vars)
│   ├── lib/
│   │   ├── utils.ts             # cn() helper (clsx + tailwind-merge), bits-ui re-exports
│   │   ├── highlight.ts         # Shiki syntax highlighter (Catppuccin Mocha, singleton)
│   │   ├── api/                 # Tauri IPC bindings
│   │   │   ├── types.ts         # TypeScript mirrors of all Rust types + frontend-only types
│   │   │   ├── repo.ts          # Git invoke() calls
│   │   │   ├── ai.ts            # AI invoke() calls (providers, keys, generation, settings)
│   │   │   └── settings.ts      # Persistent settings via tauri-plugin-store
│   │   ├── stores/              # Svelte stores (state management)
│   │   │   ├── repo.ts          # All repo state (writable + derived) and actions
│   │   │   ├── ai.ts            # AI state (providers, settings, generation) and actions
│   │   │   └── settings.ts      # Settings state (recent repos, column widths) and actions
│   │   └── components/          # UI components
│   │       ├── layout/          # App layout shell
│   │       │   ├── AppShell.svelte    # Outer frame: toolbar + sidebar + main + status bar
│   │       │   ├── Toolbar.svelte     # Top bar: brand, CommandBar, branch button
│   │       │   ├── Sidebar.svelte     # Left panel: file sections + commit panel
│   │       │   └── StatusBar.svelte   # Bottom bar: branch, staged/changed counts
│   │       ├── graph/           # Commit graph
│   │       │   ├── CommitGraph.svelte  # Resizable 5-column commit list with keyboard nav
│   │       │   └── CommitDetail.svelte # Collapsible commit detail panel
│   │       ├── diff/            # Diff viewer
│   │       │   └── FilePreview.svelte  # Full-area inline diff with syntax highlighting
│   │       └── ui/              # Headless UI primitives (shadcn-svelte via bits-ui)
│   │           ├── button/      # Button component (variants: default, destructive, outline, etc.)
│   │           ├── badge/       # Badge component (variants: default, secondary, etc.)
│   │           ├── command/     # CommandBar — searchable command palette (repos, branches)
│   │           ├── scroll-area/ # ScrollArea wrapper (vertical/horizontal)
│   │           ├── separator/   # Separator (horizontal/vertical)
│   │           ├── tabs/        # Tabs primitive
│   │           └── tooltip/     # Tooltip primitive
│   └── routes/
│       ├── +layout.ts           # SSR disabled (ssr = false)
│       ├── +layout.svelte       # Root layout (imports app.css)
│       └── +page.svelte         # Main page (settings load, conditional graph/diff/welcome)
├── static/                      # Static assets
├── components.json              # shadcn-svelte configuration
├── package.json
├── svelte.config.js             # SvelteKit + adapter-static (SPA fallback)
├── vite.config.js               # Vite + SvelteKit + TailwindCSS
├── tsconfig.json
└── CLAUDE.md                    # Claude Code instructions
```

**Note:** Plugin system (`plugins/`, `agents/`) directories listed in the diagram at the top are planned for Phase 4-5 and do not exist yet.

## Module Details

### Core Git API Layer (`src-tauri/src/git/`)

The git layer provides all git operations as free functions that take a `&Repository` reference. All access — from Tauri commands, and eventually plugins and agents — goes through this layer.

#### Module Organization

```
git/
├── mod.rs          — Re-exports: types, error, repository, graph, diff
├── types.rs        — All data types (Commit, Branch, FileDiff, RepoStatus, etc.)
├── error.rs        — GitError enum (thiserror) + GitResult<T> type alias
├── repository.rs   — Core operations: open, status, staging, branch CRUD, checkout, commit
├── graph.rs        — Commit graph: build_commit_graph, get_commit_detail
└── diff.rs         — Diffs: diff_workdir, diff_staged, diff_file, diff_file_staged
```

#### Key Functions

**`repository.rs`** — Core repository operations:
- `open(path) → GitResult<Repository>` — uses `Repository::discover()` for parent traversal
- `get_repo_info(repo) → GitResult<RepoInfo>`
- `get_status(repo) → GitResult<RepoStatus>` — staged, unstaged, untracked, conflicted
- `stage_file / unstage_file / stage_files / stage_all / unstage_all`
- `list_branches / create_branch / checkout_branch / delete_branch`
- `create_commit(repo, message) → GitResult<String>` — returns OID, handles initial commits

**`graph.rs`** — Commit graph building:
- `build_commit_graph(repo, options) → GitResult<CommitGraph>` — revwalk with TOPOLOGICAL|TIME sort, caps at max_commits (default 500)
- `get_commit_detail(repo, oid) → GitResult<Commit>`

**`diff.rs`** — Diff computation:
- `diff_workdir(repo) → GitResult<Vec<FileDiff>>` — index-to-workdir (includes untracked content)
- `diff_staged(repo) → GitResult<Vec<FileDiff>>` — tree-to-index (HEAD vs index)
- `diff_file(repo, path) → GitResult<FileDiff>` — single unstaged file
- `diff_file_staged(repo, path) → GitResult<FileDiff>` — single staged file

#### Hybrid Backend Strategy (Current + Future)

- **git2-rs** (current): Used for all implemented operations — graph traversal, diffs, status, staging, commits, branch CRUD. This is the hot path.
- **git CLI** (future): Will be added for operations not well-supported by git2-rs — interactive rebase, advanced merge strategies, submodule operations, git-flow commands.

### AI Module (`src-tauri/src/ai/`)

Provides AI-powered commit message generation using external LLM providers. API keys are stored securely in the OS keychain; non-sensitive settings are persisted via `tauri-plugin-store`.

```
ai/
├── mod.rs          — Module declarations
├── error.rs        — AIError enum (Http, Keychain, ApiError, NoApiKey, InvalidResponse, NoStagedFiles)
├── credential.rs   — API key CRUD via crate::credential_store (OS keychain)
├── types.rs        — AIProvider, AIModel, GenerateResult, AISettings
├── providers.rs    — Provider registry, default models, dynamic model fetching from APIs
└── generate.rs     — Staged diff → prompt → provider API call → title + body parsing
```

**Supported providers**: OpenAI, Anthropic, Gemini, OpenRouter (plus any OpenAI-compatible endpoint via custom base URL).

**Key flows**:
- `generate_commit_message()`: opens repo → reads staged diffs → builds prompt (truncated at 8k chars) → calls provider API → parses response into title + body
- `fetch_models()`: calls provider's model listing API, filters to relevant models, sorts by cost tier
- Provider-specific API differences: OpenAI uses `max_completion_tokens`, Anthropic/OpenRouter use `max_tokens`, Gemini uses `maxOutputTokens` and `systemInstruction`

### Repo State Cache (`src-tauri/src/cache/`)

Maintains an in-memory representation of the repository state. Implemented but not yet wired to command handlers or the file watcher.

```rust
pub struct RepoStateCache {
    state: Arc<RwLock<Option<CachedState>>>,
}

struct CachedState {
    repo_info: RepoInfo,
    status: RepoStatus,
    graph: CommitGraph,
    last_updated: Instant,
}
```

Methods: `new()`, `initialize()`, `update_status()`, `update_graph()`, `get_status()`, `get_graph()`, `clear()`

**Status:** Implemented, not yet connected. Commands currently re-open the repo on every call (stateless pattern). Cache integration is planned for when the file watcher is wired to Tauri events.

### File Watcher (`src-tauri/src/watcher/`)

Uses `notify-rs` to watch the repository directory for changes. Debounces events and triggers incremental cache updates. Pushes update events to the frontend via Tauri's event system.

Events emitted:
- `repo:status-changed` — working directory or index changed
- `repo:head-changed` — HEAD moved (checkout, commit, reset)
- `repo:refs-changed` — branches or tags created/deleted/updated
- `repo:config-changed` — git config modified

### Tauri Commands (`src-tauri/src/commands/`)

Thin handlers that bridge Tauri IPC to the git module. Each command:
1. Receives typed parameters from the frontend
2. Opens the repo fresh via `repository::open(path)`
3. Calls the appropriate `git/` module function
4. Returns serialized results

Commands are grouped by domain (repo, graph, diff, staging, branch, commit) to keep files focused. There are currently 19 registered commands — see `TECHNICAL_SPEC.md` Section 3 for the full reference.

**`AppState`** is defined in `commands/mod.rs` with `Mutex<Option<Repository>>` but is NOT yet registered as Tauri managed state. All commands currently operate statelessly.

### Plugin System

#### Backend Plugins (`src-tauri/src/plugins/`)

```rust
/// Plugin trait — all backend plugins implement this
pub trait GitronPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self, api: &dyn PluginApi) -> Result<()>;
    fn on_unload(&mut self) -> Result<()>;
}

/// API surface available to plugins
pub trait PluginApi: Send + Sync {
    fn git(&self) -> &dyn GitRepository;
    fn graph(&self) -> &dyn GraphOps;
    fn diff(&self) -> &dyn DiffOps;
    fn staging(&self) -> &dyn StagingOps;
    fn branches(&self) -> &dyn BranchOps;
    fn register_command(&self, name: &str, handler: CommandHandler);
    fn subscribe_event(&self, event: &str, handler: EventHandler);
    fn emit_event(&self, event: &str, payload: Value);
}
```

#### Frontend Plugins (`src/lib/plugins/`)

```typescript
interface GitronPluginFrontend {
  name: string;
  version: string;
  activate(api: PluginApiClient): void;
  deactivate(): void;
}

interface PluginApiClient {
  // UI registration
  registerPanel(location: PanelLocation, component: SvelteComponent): void;
  registerContextMenu(target: MenuTarget, items: MenuItem[]): void;
  registerToolbarAction(action: ToolbarAction): void;

  // State access
  onGraphSelectionChange(callback: (commit: Commit) => void): Unsubscribe;
  onRepoStatusChange(callback: (status: RepoStatus) => void): Unsubscribe;

  // Tauri IPC (for plugin-registered backend commands)
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;

  // UI utilities
  showNotification(options: NotificationOptions): void;
  showDialog(options: DialogOptions): Promise<DialogResult>;
}

type PanelLocation =
  | 'sidebar'
  | 'bottom-panel'
  | 'right-panel'
  | 'toolbar'
  | 'status-bar'
  | 'commit-detail';
```

### Agent Gateway (`src-tauri/src/agents/`)

The Agent Gateway makes Gitron an AI-native application.

#### MCP Server (`mcp.rs`)

Exposes repository state as MCP resources and tools:

**Resources:**
- `repo://status` — current repo status
- `repo://graph` — commit graph data
- `repo://branches` — branch list
- `repo://diff/{path}` — file diff
- `repo://file/{path}` — file content at HEAD

**Tools:**
- `stage_files` — stage files for commit
- `create_commit` — create a commit with message
- `create_branch` — create a new branch
- `checkout` — switch branches
- `propose_action` — submit an action for human review

#### Permission System (`permissions.rs`)

```rust
pub enum AgentPermission {
    ReadOnly,           // Can read repo state only
    BranchScoped(Vec<String>),  // Can write to specific branches
    FullAccess,         // Can perform any git operation
}

pub enum ApprovalMode {
    HumanInTheLoop,     // All actions require approval
    AutoApprove(Vec<ActionType>), // Auto-approve specific action types
    FullyAutonomous,    // No approval required
}
```

#### Action Queue (`queue.rs`)

Agent-proposed actions land in a queue. The frontend displays them for human review. Actions can be approved, rejected, or modified before execution.

```rust
pub struct AgentAction {
    pub id: Uuid,
    pub agent_id: String,
    pub action_type: ActionType,
    pub description: String,
    pub payload: Value,
    pub status: ActionStatus,  // Pending, Approved, Rejected, Executed
    pub created_at: DateTime<Utc>,
}
```

## Data Flow

### Reading the Commit Graph (Hot Path)

```
User opens repo
  → Rust: git2-rs reads commit objects, builds graph topology
  → Rust: Cache stores computed CommitGraph
  → Tauri IPC: Serialized graph sent to frontend
  → Svelte: Canvas renders nodes, edges, branch labels
  → File watcher: detects changes → incremental cache update → event push
```

### Staging and Committing

```
User clicks file to stage
  → Svelte: calls invoke('stage_file', { path })
  → Rust: git2-rs adds file to index
  → Rust: Cache updates status
  → Tauri event: repo:status-changed
  → Svelte: staging panel updates reactively

User writes commit message, clicks commit
  → Svelte: calls invoke('commit', { message })
  → Rust: git2-rs creates commit object
  → Rust: Cache updates graph, HEAD, status
  → Tauri event: repo:head-changed
  → Svelte: graph re-renders with new commit
```

### Agent Flow

```
Agent connects via MCP
  → Agent Gateway: authenticates, assigns permissions
  → Agent reads repo://graph (MCP resource)
  → Agent calls propose_action({ type: 'commit', message: '...' })
  → Action Queue: action added with Pending status
  → Tauri event: agent:action-proposed
  → Svelte: AgentPanel shows pending action
  → User approves
  → Rust: executes the commit
  → Cache updated, graph refreshed
```

## Performance Considerations

1. **Graph rendering**: Canvas/WebGL, not DOM. Virtualize — only render visible nodes.
2. **Large repos**: Paginate commit loading. Load recent commits first, lazy-load history.
3. **Diffs**: Stream large diffs, don't load entire file into memory.
4. **File watcher**: Debounce events (100ms window). Batch cache updates.
5. **IPC serialization**: Use `serde` with efficient formats. Avoid sending full graph on every update — send deltas.
6. **Startup**: Open repo asynchronously. Show UI shell immediately, populate data progressively.

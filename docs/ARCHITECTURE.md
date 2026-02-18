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
│   ├── VISION.md
│   ├── ARCHITECTURE.md
│   └── ROADMAP.md
├── src-tauri/                   # Rust backend (Tauri app)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs              # Tauri app entry point
│   │   ├── lib.rs               # Library root
│   │   ├── commands/            # Tauri IPC command handlers
│   │   │   ├── mod.rs
│   │   │   ├── repo.rs          # Repository operations
│   │   │   ├── graph.rs         # Commit graph queries
│   │   │   ├── diff.rs          # Diff operations
│   │   │   ├── staging.rs       # Stage/unstage operations
│   │   │   └── branch.rs        # Branch management
│   │   ├── git/                 # Core git abstraction layer
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs        # Core traits (GitRepository, etc.)
│   │   │   ├── git2_backend.rs  # git2-rs implementation
│   │   │   ├── cli_backend.rs   # git CLI fallback implementation
│   │   │   ├── graph.rs         # Commit graph computation
│   │   │   ├── diff.rs          # Diff computation
│   │   │   └── types.rs         # Shared types (Commit, Branch, etc.)
│   │   ├── watcher/             # File system watcher
│   │   │   ├── mod.rs
│   │   │   └── handler.rs       # Change event processing
│   │   ├── cache/               # Repo state cache
│   │   │   ├── mod.rs
│   │   │   └── repo_state.rs    # In-memory repo state
│   │   ├── plugins/             # Plugin host system
│   │   │   ├── mod.rs
│   │   │   ├── traits.rs        # Plugin trait definitions
│   │   │   ├── loader.rs        # Plugin discovery and loading
│   │   │   └── registry.rs      # Plugin registry
│   │   └── agents/              # Agent gateway
│   │       ├── mod.rs
│   │       ├── gateway.rs       # Main agent gateway
│   │       ├── mcp.rs           # MCP server implementation
│   │       ├── permissions.rs   # Agent permission system
│   │       ├── events.rs        # Event stream for agents
│   │       └── queue.rs         # Action queue (approve/reject)
│   ├── tauri.conf.json
│   └── icons/
├── src/                         # Svelte frontend
│   ├── app.html
│   ├── app.css
│   ├── lib/
│   │   ├── components/          # UI components
│   │   │   ├── graph/           # Commit graph renderer
│   │   │   │   ├── CommitGraph.svelte
│   │   │   │   ├── GraphCanvas.svelte
│   │   │   │   └── GraphNode.svelte
│   │   │   ├── diff/            # Diff viewer
│   │   │   │   ├── DiffView.svelte
│   │   │   │   ├── DiffLine.svelte
│   │   │   │   └── DiffHeader.svelte
│   │   │   ├── staging/         # Staging area
│   │   │   │   ├── StagingPanel.svelte
│   │   │   │   └── FileEntry.svelte
│   │   │   ├── branches/        # Branch panel
│   │   │   │   ├── BranchList.svelte
│   │   │   │   └── BranchItem.svelte
│   │   │   ├── commit/          # Commit authoring
│   │   │   │   └── CommitPanel.svelte
│   │   │   ├── agents/          # Agent activity view
│   │   │   │   ├── AgentPanel.svelte
│   │   │   │   └── AgentAction.svelte
│   │   │   └── layout/          # App layout shell
│   │   │       ├── AppShell.svelte
│   │   │       ├── Sidebar.svelte
│   │   │       ├── Toolbar.svelte
│   │   │       └── StatusBar.svelte
│   │   ├── stores/              # Svelte stores (state management)
│   │   │   ├── repo.ts          # Repository state store
│   │   │   ├── graph.ts         # Graph data store
│   │   │   ├── ui.ts            # UI state (selected commit, panels, etc.)
│   │   │   └── agents.ts        # Agent state store
│   │   ├── api/                 # Tauri IPC bindings
│   │   │   ├── repo.ts          # Repository commands
│   │   │   ├── graph.ts         # Graph commands
│   │   │   ├── diff.ts          # Diff commands
│   │   │   ├── staging.ts       # Staging commands
│   │   │   └── branch.ts        # Branch commands
│   │   ├── plugins/             # Frontend plugin system
│   │   │   ├── api.ts           # Plugin API definition
│   │   │   ├── registry.ts      # Plugin registry
│   │   │   └── slots.ts         # UI extension point definitions
│   │   └── utils/               # Shared utilities
│   │       ├── colors.ts        # Branch color assignment
│   │       └── format.ts        # Date/time formatting
│   └── routes/
│       ├── +layout.svelte       # Root layout
│       └── +page.svelte         # Main page
├── static/                      # Static assets
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
├── CONTRIBUTING.md
├── LICENSE
└── README.md
```

## Module Details

### Core Git API Layer (`src-tauri/src/git/`)

The git abstraction layer provides a unified interface over git operations. All access — from Tauri commands, plugins, and agents — goes through this layer.

#### Key Traits

```rust
/// Core repository operations
pub trait GitRepository: Send + Sync {
    fn open(path: &Path) -> Result<Self> where Self: Sized;
    fn head(&self) -> Result<Reference>;
    fn status(&self) -> Result<RepoStatus>;
    fn is_valid_repo(path: &Path) -> bool;
}

/// Commit graph operations (hot path — git2-rs)
pub trait GraphOps: Send + Sync {
    fn get_commits(&self, range: CommitRange) -> Result<Vec<Commit>>;
    fn get_graph(&self, options: GraphOptions) -> Result<CommitGraph>;
    fn get_commit_detail(&self, oid: &str) -> Result<CommitDetail>;
}

/// Diff operations
pub trait DiffOps: Send + Sync {
    fn diff_workdir(&self) -> Result<Diff>;
    fn diff_index(&self) -> Result<Diff>;
    fn diff_commits(&self, from: &str, to: &str) -> Result<Diff>;
    fn diff_file(&self, path: &str) -> Result<FileDiff>;
}

/// Staging operations
pub trait StagingOps: Send + Sync {
    fn stage_file(&self, path: &str) -> Result<()>;
    fn unstage_file(&self, path: &str) -> Result<()>;
    fn stage_hunk(&self, path: &str, hunk: &HunkRange) -> Result<()>;
    fn stage_lines(&self, path: &str, lines: &[usize]) -> Result<()>;
    fn discard_file(&self, path: &str) -> Result<()>;
}

/// Branch operations
pub trait BranchOps: Send + Sync {
    fn list_branches(&self) -> Result<Vec<Branch>>;
    fn create_branch(&self, name: &str, target: &str) -> Result<Branch>;
    fn delete_branch(&self, name: &str) -> Result<()>;
    fn rename_branch(&self, old: &str, new: &str) -> Result<()>;
    fn checkout(&self, refname: &str) -> Result<()>;
    fn merge(&self, source: &str) -> Result<MergeResult>;
}

/// Commit authoring
pub trait CommitOps: Send + Sync {
    fn commit(&self, message: &str) -> Result<String>;
    fn amend(&self, message: &str) -> Result<String>;
}

/// Remote operations
pub trait RemoteOps: Send + Sync {
    fn fetch(&self, remote: &str) -> Result<()>;
    fn pull(&self, remote: &str, branch: &str) -> Result<MergeResult>;
    fn push(&self, remote: &str, branch: &str) -> Result<()>;
    fn list_remotes(&self) -> Result<Vec<Remote>>;
}
```

#### Hybrid Backend Strategy

- **git2-rs** (`git2_backend.rs`): Used for all read operations and simple writes — graph traversal, diffs, status, staging, basic commits. This is the hot path and must be fast.
- **git CLI** (`cli_backend.rs`): Used for complex operations that git2-rs doesn't fully support — interactive rebase, advanced merge strategies, submodule operations, git-flow commands.

### Repo State Cache (`src-tauri/src/cache/`)

Maintains an in-memory representation of the repository state. Updated incrementally via the file watcher. The frontend never waits for a full re-read.

```rust
pub struct RepoState {
    pub head: Reference,
    pub branches: Vec<Branch>,
    pub tags: Vec<Tag>,
    pub status: RepoStatus,
    pub graph: CommitGraph,
    pub remotes: Vec<Remote>,
    pub stashes: Vec<Stash>,
    pub last_updated: Instant,
}
```

### File Watcher (`src-tauri/src/watcher/`)

Uses `notify-rs` to watch the repository directory for changes. Debounces events and triggers incremental cache updates. Pushes update events to the frontend via Tauri's event system.

Events emitted:
- `repo:status-changed` — working directory or index changed
- `repo:head-changed` — HEAD moved (checkout, commit, reset)
- `repo:refs-changed` — branches or tags created/deleted/updated
- `repo:config-changed` — git config modified

### Tauri Commands (`src-tauri/src/commands/`)

Thin handlers that bridge Tauri IPC to the Core API. Each command:
1. Receives typed parameters from the frontend
2. Calls the appropriate Core API method
3. Returns serialized results

Commands are grouped by domain (repo, graph, diff, staging, branch) to keep files focused.

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

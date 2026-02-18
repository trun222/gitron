# Gitron — Plugin System Design

This document defines how the Gitron plugin system works. It covers the full lifecycle of a plugin, the API surface available to plugins, frontend extension points, distribution, and examples. This system is designed in Phase 1 and implemented in Phase 4.

---

## Table of Contents

1. [Overview](#overview)
2. [Design Principles](#design-principles)
3. [Plugin Anatomy](#plugin-anatomy)
4. [Backend Plugin System](#backend-plugin-system)
5. [Frontend Plugin System](#frontend-plugin-system)
6. [Extension Points](#extension-points)
7. [Plugin Communication](#plugin-communication)
8. [Plugin Distribution](#plugin-distribution)
9. [Security & Sandboxing](#security--sandboxing)
10. [Example Plugins](#example-plugins)

---

## 1. Overview

Gitron's plugin system allows third-party developers to extend both the Rust backend and Svelte frontend without modifying core code. Plugins can:

- Add new git operation logic in Rust
- Register new Tauri IPC commands
- Inject UI panels, toolbar buttons, context menu items, and status bar widgets
- Hook into git events (commit, checkout, push, etc.)
- Access the full git state (graph, status, diffs, branches)

The core application is itself built on the same APIs plugins use. If the built-in diff viewer uses an internal API that plugins can't access, the plugin system is incomplete.

---

## 2. Design Principles

1. **Plugins are first-class**: Core features should be buildable as plugins. If they can't, the plugin API needs expanding.

2. **Controlled extension points**: Plugins inject UI into defined slots, not arbitrary DOM. This prevents layout breakage and maintains visual consistency.

3. **Type-safe backend**: Rust plugins implement traits. The compiler enforces correctness at build time. No dynamic dispatch surprises.

4. **Accessible frontend**: Frontend plugins use TypeScript/JavaScript. Any web developer can write one without learning Rust.

5. **Dual-layer**: A plugin can have only a backend component, only a frontend component, or both. They are independent but can communicate.

---

## 3. Plugin Anatomy

A full plugin has two parts:

```
gitron-plugin-github/
├── rust/                        # Backend plugin (Rust crate)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs               # Implements GitronPlugin trait
├── frontend/                    # Frontend plugin (npm package)
│   ├── package.json
│   └── src/
│       ├── index.ts             # Plugin entry, implements GitronPluginFrontend
│       └── PullRequestPanel.svelte  # UI component
└── plugin.toml                  # Plugin manifest
```

### Plugin Manifest (`plugin.toml`)

```toml
[plugin]
name = "github-integration"
version = "0.1.0"
description = "GitHub PR, issue, and actions integration for Gitron"
author = "Gitron Contributors"
license = "MIT"

[backend]
crate = "gitron-plugin-github"

[frontend]
package = "@gitron/plugin-github"
entry = "src/index.ts"

[permissions]
network = true          # Needs network access for GitHub API
filesystem = false      # No additional filesystem access needed
git_write = false       # Read-only git access
```

---

## 4. Backend Plugin System

### Plugin Trait

Every backend plugin implements this trait:

```rust
pub trait GitronPlugin: Send + Sync {
    /// Unique plugin identifier
    fn name(&self) -> &str;

    /// Semantic version
    fn version(&self) -> &str;

    /// Called when the plugin is loaded. Receives the API for registration.
    fn on_load(&mut self, api: &dyn PluginApi) -> Result<()>;

    /// Called when the plugin is unloaded. Clean up resources.
    fn on_unload(&mut self) -> Result<()>;

    /// Called when a repository is opened. Optional.
    fn on_repo_open(&mut self, _repo_info: &RepoInfo) -> Result<()> {
        Ok(())
    }

    /// Called when a repository is closed. Optional.
    fn on_repo_close(&mut self) -> Result<()> {
        Ok(())
    }
}
```

### Plugin API

The API surface available to plugins:

```rust
pub trait PluginApi: Send + Sync {
    // --- Git State Access ---

    /// Access current repository (if open)
    fn git(&self) -> Option<&Repository>;

    /// Read the commit graph
    fn graph(&self) -> Option<&CommitGraph>;

    /// Read the repo status
    fn status(&self) -> Option<&RepoStatus>;

    /// Read branches
    fn branches(&self) -> Option<&[Branch]>;

    // --- IPC Registration ---

    /// Register a new Tauri command that the frontend can invoke
    fn register_command(&self, name: &str, handler: Box<dyn CommandHandler>);

    // --- Event System ---

    /// Subscribe to a repo event
    fn subscribe_event(&self, event: &str, handler: Box<dyn EventHandler>);

    /// Emit a custom event to the frontend
    fn emit_event(&self, event: &str, payload: serde_json::Value);

    // --- Configuration ---

    /// Read plugin-specific configuration
    fn get_config(&self, key: &str) -> Option<serde_json::Value>;

    /// Write plugin-specific configuration
    fn set_config(&self, key: &str, value: serde_json::Value);
}
```

### Plugin Lifecycle

```
App starts
  → Plugin loader scans plugin directory
  → For each plugin:
      1. Load the crate (dynamic linking or compiled-in)
      2. Call plugin.on_load(api)
      3. Plugin registers commands, subscribes to events
      4. Plugin is now active

Repo opened
  → For each active plugin:
      1. Call plugin.on_repo_open(repo_info)
      2. Plugin can start repo-specific work

Repo closed
  → For each active plugin:
      1. Call plugin.on_repo_close()
      2. Plugin cleans up repo-specific state

App closes
  → For each active plugin:
      1. Call plugin.on_unload()
      2. Plugin cleans up all resources
```

### Plugin Registry

```rust
pub struct PluginRegistry {
    plugins: Vec<Box<dyn GitronPlugin>>,
    commands: HashMap<String, Box<dyn CommandHandler>>,
    event_handlers: HashMap<String, Vec<Box<dyn EventHandler>>>,
}
```

The registry:
- Maintains the list of loaded plugins
- Routes custom commands to plugin handlers
- Dispatches events to subscribed plugin handlers
- Manages plugin lifecycle (load, unload, repo open/close)

---

## 5. Frontend Plugin System

### Plugin Interface

```typescript
interface GitronPluginFrontend {
  /** Unique plugin identifier (must match backend if paired) */
  name: string;

  /** Semantic version */
  version: string;

  /** Called when the plugin is activated. Register UI and subscriptions. */
  activate(api: PluginApiClient): void;

  /** Called when the plugin is deactivated. Clean up. */
  deactivate(): void;
}
```

### Frontend Plugin API

```typescript
interface PluginApiClient {
  // --- UI Registration ---

  /** Register a panel at a specific location */
  registerPanel(location: PanelLocation, config: PanelConfig): void;

  /** Register context menu items */
  registerContextMenu(target: MenuTarget, items: MenuItem[]): void;

  /** Register a toolbar button */
  registerToolbarAction(action: ToolbarAction): void;

  /** Register a status bar widget */
  registerStatusBarItem(item: StatusBarItem): void;

  // --- State Access ---

  /** Subscribe to commit selection changes */
  onGraphSelectionChange(callback: (commit: Commit | null) => void): Unsubscribe;

  /** Subscribe to repo status changes */
  onRepoStatusChange(callback: (status: RepoStatus) => void): Unsubscribe;

  /** Subscribe to custom events from backend plugin */
  onEvent(event: string, callback: (payload: unknown) => void): Unsubscribe;

  /** Get current repo info */
  getRepoInfo(): RepoInfo | null;

  /** Get current repo status */
  getRepoStatus(): RepoStatus | null;

  /** Get current commit graph */
  getCommitGraph(): CommitGraph | null;

  // --- Backend Communication ---

  /** Call a Tauri command (core or plugin-registered) */
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;

  // --- UI Utilities ---

  /** Show a notification toast */
  showNotification(options: NotificationOptions): void;

  /** Show a modal dialog */
  showDialog(options: DialogOptions): Promise<DialogResult>;

  /** Show a confirmation dialog */
  confirm(message: string, title?: string): Promise<boolean>;
}
```

---

## 6. Extension Points

The frontend defines explicit slots where plugins can inject UI.

### Panel Locations

```typescript
type PanelLocation =
  | 'sidebar'          // Tab in the left sidebar (alongside Changes, Branches)
  | 'bottom-panel'     // Panel below the commit graph (alongside Commit Detail)
  | 'right-panel'      // Panel to the right of the main content area
  | 'toolbar'          // Widget in the toolbar area
  | 'status-bar'       // Widget in the bottom status bar
  | 'commit-detail'    // Tab/section within the commit detail panel
  | 'diff-header'      // Widget in the diff viewer header area
  | 'graph-overlay';   // Overlay on the commit graph (for agent visualization)
```

### Panel Configuration

```typescript
interface PanelConfig {
  id: string;                          // Unique panel ID
  title: string;                       // Display title (shown in tab)
  icon?: string;                       // SVG icon string
  component: typeof SvelteComponent;   // The Svelte component to render
  props?: Record<string, unknown>;     // Initial props for the component
  order?: number;                      // Sort order within the location
  badge?: Writable<string | number>;   // Reactive badge (e.g., PR count)
}
```

### Context Menu Targets

```typescript
type MenuTarget =
  | 'commit'           // Right-click on a commit in the graph
  | 'branch'           // Right-click on a branch in the sidebar
  | 'file-staged'      // Right-click on a staged file
  | 'file-unstaged'    // Right-click on an unstaged file
  | 'file-untracked'   // Right-click on an untracked file
  | 'diff-hunk'        // Right-click on a diff hunk
  | 'graph-background'; // Right-click on empty graph area
```

### Menu Item

```typescript
interface MenuItem {
  label: string;
  icon?: string;
  shortcut?: string;
  action: (context: MenuContext) => void | Promise<void>;
  enabled?: (context: MenuContext) => boolean;
  separator?: boolean;  // If true, renders as a divider
}

interface MenuContext {
  commit?: Commit;
  branch?: Branch;
  filePath?: string;
  hunk?: DiffHunk;
}
```

### Toolbar Action

```typescript
interface ToolbarAction {
  id: string;
  label: string;
  icon: string;           // SVG string
  tooltip?: string;
  action: () => void | Promise<void>;
  enabled?: Writable<boolean>;
  position?: 'left' | 'center' | 'right';
}
```

---

## 7. Plugin Communication

### Backend ↔ Frontend

Plugins that have both a backend and frontend component communicate through:

1. **Custom Tauri commands**: Backend registers a command via `api.register_command()`. Frontend calls it via `api.invoke()`.
2. **Custom events**: Backend emits events via `api.emit_event()`. Frontend subscribes via `api.onEvent()`.

### Plugin ↔ Plugin

Plugins can communicate with each other through the event system:

```rust
// Plugin A emits
api.emit_event("github:pr-merged", json!({ "pr_number": 42 }));

// Plugin B subscribes
api.subscribe_event("github:pr-merged", handler);
```

Event naming convention: `plugin-name:event-name`

### Plugin ↔ Core

Plugins access core state through the `PluginApi` / `PluginApiClient` interfaces. They cannot modify core internals directly.

---

## 8. Plugin Distribution

### Backend Plugins

- Distributed as Rust crates on crates.io
- Naming convention: `gitron-plugin-{name}`
- Added to `Cargo.toml` as dependencies
- Registered in a plugin config file or at compile time

### Frontend Plugins

- Distributed as npm packages
- Naming convention: `@gitron/plugin-{name}` or `gitron-plugin-{name}`
- Installed via npm/pnpm
- Registered in a frontend plugin config

### Combined Distribution (Future)

- A single repository containing both Rust and frontend code
- Published to both crates.io and npm
- Plugin manifest (`plugin.toml`) ties them together

---

## 9. Security & Sandboxing

### Permission Model

Plugins declare required permissions in `plugin.toml`:

```toml
[permissions]
network = true/false     # Can make network requests
filesystem = true/false  # Can access filesystem beyond the repo
git_write = true/false   # Can modify git state (stage, commit, etc.)
git_read = true          # Always granted (read repo state)
```

### Sandboxing Strategy

**Backend plugins (Rust)**: Run in-process. Trusted by default since they're compiled into the binary. Permissions are enforced at the API level — the `PluginApi` methods check permissions before executing.

**Frontend plugins (JS/TS)**: Run in the webview. They cannot access Node.js APIs or the filesystem directly. All system access goes through `api.invoke()` which routes through the Tauri permission system.

### Review & Trust

- First-party plugins (maintained by the Gitron team) are bundled with the app
- Third-party plugins are installed by the user — they accept the risk
- A plugin marketplace (future) would include a review process

---

## 10. Example Plugins

### GitHub Integration

```
Purpose: Show PRs, issues, and GitHub Actions status in Gitron

Backend:
  - On repo open, detect GitHub remote URL
  - Register commands: get_pull_requests, get_issues, get_actions_status
  - Periodically poll GitHub API for updates
  - Emit events: github:pr-updated, github:check-completed

Frontend:
  - Register sidebar tab "GitHub" showing PRs and issues
  - Register commit context menu: "Create PR from this branch"
  - Register status bar item showing CI status
  - Register graph overlay showing PR merge points
```

### Conventional Commits Helper

```
Purpose: Enforce conventional commit message format

Backend:
  - Hook into commit event
  - Validate message matches: type(scope): description
  - If invalid, return error with suggestion

Frontend:
  - Replace default commit message input with structured form
  - Dropdowns for type (feat, fix, chore, etc.) and scope
  - Auto-format the commit message
```

### Git Hooks Manager

```
Purpose: Visual editor for git hooks

Backend:
  - Read/write .git/hooks/ directory
  - Register commands: list_hooks, save_hook, delete_hook

Frontend:
  - Register sidebar tab "Hooks"
  - List all hooks with enabled/disabled toggle
  - Code editor for hook scripts
  - Templates for common hooks (pre-commit, pre-push)
```

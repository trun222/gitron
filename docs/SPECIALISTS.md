# Gitron — Specialist Reference

This file maps topics, prompts, and areas of work to the specific files and documentation that should be consulted. When a question or task falls into one of these domains, read the listed files before proceeding.

---

## Specialist: Git Backend

**Trigger keywords**: git operations, git2, libgit2, commit, branch, staging, diff, status, merge, rebase, stash, tag, remote, fetch, push, pull, checkout, blame, reflog

**Read first**:
- `docs/TECHNICAL_SPEC.md` — Section 7 (Git Operations Layer)
- `docs/DEVELOPER_GUIDE.md` — Section 6 (Adding a New Feature)

**Core files**:
| File | Responsibility |
|------|---------------|
| `src-tauri/src/git/types.rs` | All data types (Commit, Branch, FileDiff, RepoStatus, etc.) |
| `src-tauri/src/git/error.rs` | GitError enum and GitResult type alias |
| `src-tauri/src/git/repository.rs` | Repo open, status, staging, branch CRUD, checkout |
| `src-tauri/src/git/graph.rs` | Commit graph building (revwalk, branch/tag collection) |
| `src-tauri/src/git/diff.rs` | Diff computation (workdir, staged, per-file) |
| `src-tauri/src/git/mod.rs` | Module declarations |

**Key dependencies**: `git2` crate (v0.19), `chrono`, `serde`

**Rules**:
- All git logic lives in `src-tauri/src/git/`. Never in `commands/`.
- Functions return `GitResult<T>`.
- Use `Repository::discover()` to open repos (walks up to find `.git`).
- Hot path (graph, diff, status) uses git2-rs. Complex ops (rebase, advanced merge) will use git CLI.

---

## Specialist: Tauri IPC / Commands

**Trigger keywords**: command, invoke, IPC, Tauri command, handler, register command, generate_handler

**Read first**:
- `docs/TECHNICAL_SPEC.md` — Section 3 (IPC Command Reference)
- `docs/DEVELOPER_GUIDE.md` — Section 7 (Adding a New IPC Command)

**Core files**:
| File | Responsibility |
|------|---------------|
| `src-tauri/src/lib.rs` | Command registration in `generate_handler![]` |
| `src-tauri/src/commands/mod.rs` | Module declarations, AppState struct |
| `src-tauri/src/commands/repo.rs` | open_repo, get_status, get_repo_info |
| `src-tauri/src/commands/graph.rs` | get_commit_graph, get_commit_detail |
| `src-tauri/src/commands/diff.rs` | get_workdir_diff, get_file_diff |
| `src-tauri/src/commands/staging.rs` | stage_file, unstage_file, stage_all, unstage_all |
| `src-tauri/src/commands/branch.rs` | list_branches, create_branch, checkout_branch, delete_branch |

**Rules**:
- Commands are thin — delegate ALL logic to `git/` module.
- Every command takes `path: String` as first param (stateless, opens repo per call).
- Mutation commands return updated state (e.g., stage_file returns RepoStatus).
- New commands MUST be added to `generate_handler![]` in `lib.rs`.
- Rust `snake_case` params auto-convert to frontend `camelCase`.

---

## Specialist: Frontend API & Types

**Trigger keywords**: TypeScript types, invoke binding, API call, frontend types, type mirror, data contract

**Read first**:
- `docs/TECHNICAL_SPEC.md` — Section 6 (Data Types Contract)
- `docs/DEVELOPER_GUIDE.md` — Section 7 (Adding a New IPC Command), Step 5-6

**Core files**:
| File | Responsibility |
|------|---------------|
| `src/lib/api/types.ts` | TypeScript mirrors of all Rust types |
| `src/lib/api/repo.ts` | All `invoke()` calls — the ONLY file that imports from `@tauri-apps/api/core` |

**Rules**:
- Types in `types.ts` MUST match `src-tauri/src/git/types.rs`. Manual sync.
- Rust `Option<T>` → TypeScript `T | null`
- Rust enums → TypeScript string literal unions (e.g., `'Added' | 'Modified' | ...`)
- Only `repo.ts` calls `invoke()`. Components and stores NEVER import from `@tauri-apps/api/core`.

---

## Specialist: State Management / Stores

**Trigger keywords**: store, state, writable, derived, reactive, action, refresh, loading, error state

**Read first**:
- `docs/TECHNICAL_SPEC.md` — Section 5 (State Management)
- `docs/DEVELOPER_GUIDE.md` — Section 8 (Adding a Frontend Component)

**Core files**:
| File | Responsibility |
|------|---------------|
| `src/lib/stores/repo.ts` | All repo state (writable + derived stores), all actions |

**Stores**:
| Store | Type | Purpose |
|-------|------|---------|
| `repoPath` | `Writable<string \| null>` | Path to open repo |
| `repoInfo` | `Writable<RepoInfo \| null>` | Repo metadata |
| `repoStatus` | `Writable<RepoStatus \| null>` | Staged/unstaged/untracked |
| `commitGraph` | `Writable<CommitGraph \| null>` | Commits, branches, tags |
| `selectedCommit` | `Writable<Commit \| null>` | Currently selected commit |
| `selectedFileDiff` | `Writable<FileDiff \| null>` | Currently viewed diff |
| `loading` | `Writable<boolean>` | Global loading flag |
| `error` | `Writable<string \| null>` | Last error message |
| `hasRepo` | `Derived<boolean>` | Whether a repo is open |
| `currentBranch` | `Derived<string \| null>` | Current HEAD branch |
| `localBranches` | `Derived<Branch[]>` | Non-remote branches |
| `remoteBranches` | `Derived<Branch[]>` | Remote branches |
| `stagedCount` | `Derived<number>` | Number of staged files |
| `unstagedCount` | `Derived<number>` | Unstaged + untracked count |

**Actions**: `openRepo`, `refreshAll`, `refreshStatus`, `stageFile`, `unstageFile`, `stageAllFiles`, `unstageAllFiles`, `selectCommit`, `viewFileDiff`

**Rules**:
- No optimistic updates — always wait for backend confirmation.
- Actions catch errors and write to the `error` store.
- Parallel fetches use `Promise.all` (e.g., status + graph on repo open).

---

## Specialist: Frontend UI / Components

**Trigger keywords**: component, Svelte, layout, sidebar, toolbar, graph, diff viewer, panel, styling, CSS, theme, design token

**Read first**:
- `docs/DEVELOPER_GUIDE.md` — Section 8 (Adding a Frontend Component)
- `docs/TECHNICAL_SPEC.md` — Section 10 (Commit Graph Rendering)

**Core files**:
| File | Responsibility |
|------|---------------|
| `src/app.css` | Design system: all CSS custom properties (colors, spacing) |
| `src/app.html` | HTML shell |
| `src/routes/+page.svelte` | Main page (renders AppShell + graph + detail) |
| `src/routes/+layout.svelte` | Root layout (imports app.css) |
| `src/lib/components/layout/AppShell.svelte` | App shell (toolbar + sidebar + main + status bar) |
| `src/lib/components/layout/Toolbar.svelte` | Top bar (repo input, branch badge) |
| `src/lib/components/layout/Sidebar.svelte` | Left panel (changes tab, branches tab) |
| `src/lib/components/layout/StatusBar.svelte` | Bottom bar (branch, staged/changed counts) |
| `src/lib/components/graph/CommitGraph.svelte` | Commit list with branch labels |
| `src/lib/components/graph/CommitDetail.svelte` | Commit detail panel (message, author, parents) |

**Design tokens** (from `app.css`):
```
Backgrounds: --bg-primary, --bg-secondary, --bg-hover, --bg-selected, --bg-input, --bg-badge
Text: --text-primary, --text-secondary, --text-muted, --text-accent
Borders: --border-color, --border-subtle
Accent: --accent-color, --accent-hover
Status: --color-added, --color-modified, --color-deleted (+ -bg variants)
```

**Rules**:
- Use Svelte 5 runes (`$state`, `$derived`, `$props`), NOT Svelte 4 patterns.
- Scoped styles only (in `<style>` block). Never global styles in components.
- Use design tokens for all colors. No hard-coded hex values.
- Components read stores, call store actions. Never call `invoke()` directly.

---

## Specialist: File Watcher / Live Updates

**Trigger keywords**: file watcher, notify, live update, real-time, auto-refresh, filesystem events, debounce

**Read first**:
- `docs/TECHNICAL_SPEC.md` — Section 8 (File Watcher System)
- `docs/TECHNICAL_SPEC.md` — Section 4 (Event System)

**Core files**:
| File | Responsibility |
|------|---------------|
| `src-tauri/src/watcher/handler.rs` | Watcher setup, event debouncing, event classification |
| `src-tauri/src/watcher/mod.rs` | Module declaration |
| `src-tauri/src/cache/repo_state.rs` | In-memory cache updated by watcher |
| `src-tauri/src/cache/mod.rs` | Module declaration |

**How it works**:
1. `notify-rs` watches the repo directory recursively
2. Events are debounced (200ms window via `notify-debouncer-mini`)
3. Events are classified: `.git/*` paths → RefsChanged, other paths → WorkdirChanged
4. Handler triggers cache update and Tauri event emission
5. Frontend listens for events and updates stores

**Status**: Stub implemented. Not yet wired to Tauri events or cache updates.

---

## Specialist: Plugin System

**Trigger keywords**: plugin, extension, extensible, addon, third-party, panel registration, context menu, toolbar action, extension point, slot

**Read first**:
- `docs/PLUGIN_SYSTEM.md` — Full design document
- `docs/ARCHITECTURE.md` — Plugin System section

**Planned files** (not yet implemented):
| File | Responsibility |
|------|---------------|
| `src-tauri/src/plugins/mod.rs` | Module declarations |
| `src-tauri/src/plugins/traits.rs` | `GitronPlugin` and `PluginApi` traits |
| `src-tauri/src/plugins/loader.rs` | Plugin discovery and loading |
| `src-tauri/src/plugins/registry.rs` | Plugin registry, command routing, event dispatch |
| `src/lib/plugins/api.ts` | `GitronPluginFrontend` and `PluginApiClient` interfaces |
| `src/lib/plugins/registry.ts` | Frontend plugin registry |
| `src/lib/plugins/slots.ts` | UI extension point definitions |

**Status**: Designed, not implemented. Scheduled for Phase 4.

---

## Specialist: Agent Gateway / AI Integration

**Trigger keywords**: agent, AI, MCP, Model Context Protocol, autonomous, action queue, approve, reject, permission, agent visualization, smart commit, AI workflow

**Read first**:
- `docs/AGENT_GATEWAY.md` — Full design document
- `docs/ARCHITECTURE.md` — Agent Gateway section

**Planned files** (not yet implemented):
| File | Responsibility |
|------|---------------|
| `src-tauri/src/agents/mod.rs` | Module declarations |
| `src-tauri/src/agents/gateway.rs` | Main gateway logic, request routing |
| `src-tauri/src/agents/mcp.rs` | MCP server (resources + tools) |
| `src-tauri/src/agents/permissions.rs` | Permission types and enforcement |
| `src-tauri/src/agents/events.rs` | Event stream management |
| `src-tauri/src/agents/queue.rs` | Action queue (propose, approve, reject, execute) |
| `src/lib/components/agents/AgentPanel.svelte` | Agent activity UI |
| `src/lib/components/agents/AgentAction.svelte` | Individual action card |
| `src/lib/stores/agents.ts` | Agent state store |

**Status**: Designed, not implemented. Scheduled for Phase 5.

---

## Specialist: Build / Configuration / Deployment

**Trigger keywords**: build, compile, Cargo.toml, package.json, tauri.conf.json, release, bundle, icon, install, deploy, CI/CD

**Core files**:
| File | Responsibility |
|------|---------------|
| `src-tauri/Cargo.toml` | Rust dependencies and crate config |
| `src-tauri/tauri.conf.json` | Tauri app config (window, build commands, bundle) |
| `package.json` | Frontend dependencies and scripts |
| `svelte.config.js` | SvelteKit config (static adapter) |
| `vite.config.js` | Vite config (dev server, Tauri integration) |
| `tsconfig.json` | TypeScript config |
| `.gitignore` | Ignored files |

**Commands**:
```bash
npm run tauri dev     # Development mode (hot reload frontend, recompile Rust)
npm run tauri build   # Production build
cargo check           # Type-check Rust only
npm run check         # Type-check frontend only
```

---

## Cross-Cutting Concerns

### When adding a new domain (e.g., stash, remote, tag management):
1. Read `docs/DEVELOPER_GUIDE.md` Section 6 — full walkthrough
2. Touch files in this order: `git/types.rs` → `git/*.rs` → `commands/*.rs` → `lib.rs` → `api/types.ts` → `api/repo.ts` → `stores/repo.ts` → `components/*.svelte`

### When debugging IPC issues:
1. Read `docs/TECHNICAL_SPEC.md` Section 3 — command reference
2. Check `lib.rs` — is the command registered?
3. Check parameter naming — Rust `snake_case` ↔ frontend `camelCase`
4. Check error serialization — `GitError` implements `Serialize`

### When modifying the UI layout:
1. Read `docs/DEVELOPER_GUIDE.md` Section 8 — component conventions
2. Read `docs/TECHNICAL_SPEC.md` Section 10 — graph rendering approach
3. Use design tokens from `app.css`. Never hard-code colors.

### When working on performance:
1. Read `docs/TECHNICAL_SPEC.md` Section 12 (Concurrency) and the Performance Considerations in `docs/ARCHITECTURE.md`
2. Hot paths: graph building, diff computation, status reading — these must use git2-rs, not CLI
3. Frontend: virtualize long lists, use Canvas for the graph, debounce events

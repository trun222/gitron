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
| `crates/gitron-core/src/git/types.rs` | All data types (Commit, Branch, FileDiff, RepoStatus, etc.) |
| `crates/gitron-core/src/git/error.rs` | GitError enum and GitResult type alias |
| `crates/gitron-core/src/git/repository.rs` | Repo open, status, staging, branch CRUD, checkout, rebase, merge |
| `crates/gitron-core/src/git/graph.rs` | Commit graph building (revwalk, branch/tag collection) |
| `crates/gitron-core/src/git/diff.rs` | Diff computation (workdir, staged, per-file) |
| `crates/gitron-core/src/git/cli.rs` | Git CLI bridge: `run_git_raw`, `run_git`, `run_git_async`, `run_git_async_with_github_auth` |
| `crates/gitron-core/src/git/remote.rs` | Remote operations: fetch, push, pull, tracking status |
| `crates/gitron-core/src/git/mod.rs` | Module declarations |

**Key dependencies**: `git2` crate (v0.19), `chrono`, `serde`

**Rules**:
- All git logic lives in `crates/gitron-core/src/git/`. Never in `commands/` or `routes/`.
- Functions return `GitResult<T>`.
- Use `Repository::discover()` to open repos (walks up to find `.git`).
- Hot path (graph, diff, status) uses git2-rs. Complex ops (rebase, merge) use git CLI via `cli::run_git_raw`.

---

## Specialist: Tauri Desktop App

**Trigger keywords**: Tauri, desktop, native, IPC, invoke, command, handler, register command, generate_handler, tauri_impls, TauriCredentialStore, TauriEventEmitter

**Read first**:
- `docs/TECHNICAL_SPEC.md` — Section 3 (IPC Command Reference)
- `docs/DEVELOPER_GUIDE.md` — Section 7 (Adding a New IPC Command)

**Core files**:
| File | Responsibility |
|------|---------------|
| `src-tauri/src/lib.rs` | Command registration in `generate_handler![]`, credential store init |
| `src-tauri/src/tauri_impls.rs` | `TauriCredentialStore` (tauri-plugin-store), `TauriEventEmitter` (AppHandle::emit) |
| `src-tauri/src/commands/mod.rs` | Module declarations, AppState struct |
| `src-tauri/src/commands/repo.rs` | open_repo, close_repo, get_status, get_repo_info, set_watcher_interval |
| `src-tauri/src/commands/graph.rs` | get_commit_graph, get_commit_detail |
| `src-tauri/src/commands/diff.rs` | get_workdir_diff, get_file_diff, get_staged_file_diff |
| `src-tauri/src/commands/staging.rs` | stage_file, unstage_file, stage_files, stage_all, unstage_all, discard_all_changes |
| `src-tauri/src/commands/branch.rs` | list_branches, create_branch, checkout_branch, delete_branch, reset_to_commit, rebase_onto, merge_into |
| `src-tauri/src/commands/commit.rs` | create_commit |
| `src-tauri/src/commands/stash.rs` | apply_stash, pop_stash, drop_stash |
| `src-tauri/src/commands/remote.rs` | list_remotes, add/remove_remote, fetch, push, pull, tracking, delete_remote_branch, checkout_remote_branch |
| `src-tauri/src/commands/tag.rs` | create_tag, delete_tag, push_tag, delete_remote_tag, list_remote_tags |
| `src-tauri/src/commands/clone.rs` | clone_repo |
| `src-tauri/src/commands/ai.rs` | AI commands: get_providers, save/delete key, fetch_models, generate, settings |
| `src-tauri/src/commands/github.rs` | GitHub: check_auth, device flow, logout, user, repos |
| `src-tauri/tauri.conf.json` | App config (window, build commands, bundle) |

**Rules**:
- Commands are thin — delegate ALL logic to `gitron_core::` modules.
- Every command takes `path: String` as first param (stateless, opens repo per call).
- Mutation commands return updated state (e.g., stage_file returns RepoStatus).
- New commands MUST be added to `generate_handler![]` in `lib.rs`.
- Rust `snake_case` params auto-convert to frontend `camelCase`.
- **PARITY**: Every new Tauri command must also be added as a server route in `crates/gitron-server/src/routes/`.

---

## Specialist: Axum Web Server

**Trigger keywords**: server, web, HTTP, Axum, REST, API endpoint, SSE, events, FileCredentialStore, SseBroadcaster, bearer token, self-hosted, Docker

**Read first**:
- `CLAUDE.md` — Dual-Mode Parity rules

**Core files**:
| File | Responsibility |
|------|---------------|
| `crates/gitron-server/src/main.rs` | CLI (--port, --host, --token, --frontend-dir, --repo), ServerState, middleware, static file serving |
| `crates/gitron-server/src/routes/mod.rs` | All API routes registered (50+ POST + GET endpoints) |
| `crates/gitron-server/src/routes/repo.rs` | open_repo, close_repo, get_status, get_repo_info |
| `crates/gitron-server/src/routes/graph.rs` | get_commit_graph, get_commit_detail |
| `crates/gitron-server/src/routes/diff.rs` | get_workdir_diff, get_file_diff, get_staged_file_diff |
| `crates/gitron-server/src/routes/staging.rs` | stage, unstage, stage-many, stage-all, unstage-all, discard-all |
| `crates/gitron-server/src/routes/branch.rs` | list, create, checkout, delete, reset, rebase, merge |
| `crates/gitron-server/src/routes/commit.rs` | create_commit |
| `crates/gitron-server/src/routes/stash.rs` | apply, pop, drop |
| `crates/gitron-server/src/routes/remote.rs` | list, add, remove, tracking, fetch, fetch-all, push, pull, delete-branch, checkout |
| `crates/gitron-server/src/routes/tag.rs` | create, delete, push, delete-remote, list-remote |
| `crates/gitron-server/src/routes/clone.rs` | clone_repo |
| `crates/gitron-server/src/routes/ai.rs` | providers, save/delete key, fetch models, generate, settings |
| `crates/gitron-server/src/routes/github.rs` | check-auth, start-flow, poll-flow, logout, user, repos |
| `crates/gitron-server/src/routes/fs.rs` | Directory browser for web mode |
| `crates/gitron-server/src/routes/settings.rs` | Server-side app settings (GET/POST) |
| `crates/gitron-server/src/sse.rs` | `SseBroadcaster` — implements `EventEmitter`, uses `tokio::sync::broadcast` for SSE |
| `crates/gitron-server/src/file_store.rs` | `FileCredentialStore` — file-based credentials in `~/.config/gitron/credentials.json` |
| `crates/gitron-server/src/auth.rs` | Bearer token auth middleware for `/api/*` routes |

**Rules**:
- Route handlers are thin — delegate ALL logic to `gitron_core::` modules.
- Request structs must use `#[serde(rename = "camelCase")]` for multi-word fields (no Tauri auto-conversion in HTTP mode).
- When the frontend sends `{ wrapper: data }`, the handler must accept a struct with a `wrapper` field — not the raw inner type.
- AI settings persist to `~/.config/gitron/ai_settings.json`. App settings to `~/.config/gitron/settings.json`.
- **PARITY**: Every new server route must also be added as a Tauri command in `src-tauri/src/commands/`.

---

## Specialist: Frontend Transport & API

**Trigger keywords**: transport, invoke, API call, HTTP, SSE, EventSource, isTauri, COMMAND_MAP, TypeScript types, type mirror, data contract, settings, persistence, localStorage

**Read first**:
- `CLAUDE.md` — Dual-Mode Parity rules and Frontend Conventions
- `docs/TECHNICAL_SPEC.md` — Section 6 (Data Types Contract)

**Core files**:
| File | Responsibility |
|------|---------------|
| `src/lib/api/transport.ts` | `Transport` interface + `isTauri()` runtime detection |
| `src/lib/api/transport-tauri.ts` | `TauriTransport` — dynamic imports of `@tauri-apps/*` for invoke, listen, openUrl, pickDirectory |
| `src/lib/api/transport-http.ts` | `HttpTransport` — fetch + COMMAND_MAP (command→endpoint), EventSource for SSE, window.open for URLs |
| `src/lib/api/index.ts` | `getTransport()` singleton — returns TauriTransport or HttpTransport based on runtime |
| `src/lib/api/types.ts` | TypeScript mirrors of all Rust types + frontend-only types (RecentRepo, GraphColumnWidths, AppSettings, AI types) |
| `src/lib/api/repo.ts` | Git API calls via `getTransport().invoke()` |
| `src/lib/api/ai.ts` | AI API calls via `getTransport().invoke()` |
| `src/lib/api/github.ts` | GitHub API calls via `getTransport().invoke()` |
| `src/lib/api/settings.ts` | `TauriSettingsStore` (tauri-plugin-store) / `WebSettingsStore` (localStorage with `gitron:` prefix) |
| `src/lib/components/ui/dialog/DirectoryBrowser.svelte` | Web-mode directory picker (calls `/api/fs/list`, dispatched via `gitron:pick-directory` custom event) |

**Rules**:
- Types in `types.ts` MUST match their Rust counterparts (`gitron-core` types). Manual sync.
- Rust `Option<T>` → TypeScript `T | null`
- Rust enums → TypeScript string literal unions (e.g., `'Added' | 'Modified' | ...`)
- **NEVER** import `@tauri-apps/*` in components, stores, or API files. The only allowed locations are `transport-tauri.ts` and `TauriSettingsStore` in `settings.ts`.
- All API files use `getTransport().invoke('command_name', { args })`.
- Settings persistence: Tauri mode uses `tauri-plugin-store` (LazyStore on `settings.json`). Web mode uses `localStorage` with `gitron:` key prefix.
- **PARITY**: When adding a new command, you must add it to COMMAND_MAP in `transport-http.ts` AND to the server routes. If it's a GET command, add it to the `GET_COMMANDS` set.

---

## Specialist: State Management / Stores

**Trigger keywords**: store, state, writable, derived, reactive, action, refresh, loading, error state, settings, recent repos

**Read first**:
- `docs/TECHNICAL_SPEC.md` — Section 5 (State Management)
- `docs/DEVELOPER_GUIDE.md` — Section 8 (Adding a Frontend Component)

**Core files**:
| File | Responsibility |
|------|---------------|
| `src/lib/stores/repo.ts` | All repo state (writable + derived stores), all repo actions |
| `src/lib/stores/ai.ts` | AI state (providers, settings, generating, fetching models), AI actions |
| `src/lib/stores/settings.ts` | Settings state (recent repos, column widths), persistence actions |

**Repository Stores** (`stores/repo.ts`):
| Store | Type | Purpose |
|-------|------|---------|
| `repoPath` | `Writable<string \| null>` | Path to open repo |
| `repoInfo` | `Writable<RepoInfo \| null>` | Repo metadata |
| `repoStatus` | `Writable<RepoStatus \| null>` | Staged/unstaged/untracked |
| `commitGraph` | `Writable<CommitGraph \| null>` | Commits, branches, tags |
| `selectedCommit` | `Writable<Commit \| null>` | Currently selected commit |
| `selectedFileDiff` | `Writable<FileDiff \| null>` | Currently viewed diff |
| `selectedFile` | `Writable<SelectedFileInfo \| null>` | Currently selected file (path + section) |
| `loading` | `Writable<boolean>` | Global loading flag |
| `error` | `Writable<string \| null>` | Last error message |
| `hasRepo` | `Derived<boolean>` | Whether a repo is open |
| `isFileSelected` | `Derived<boolean>` | Whether a file is selected for diff |
| `currentBranch` | `Derived<string \| null>` | Current HEAD branch |
| `localBranches` | `Derived<Branch[]>` | Non-remote branches |
| `remoteBranches` | `Derived<Branch[]>` | Remote branches |
| `stagedCount` | `Derived<number>` | Number of staged files |
| `unstagedCount` | `Derived<number>` | Unstaged + untracked count |

**Settings Stores** (`stores/settings.ts`):
| Store | Type | Purpose |
|-------|------|---------|
| `recentRepos` | `Writable<RecentRepo[]>` | Recently opened repos (max 20) |
| `lastActiveRepo` | `Writable<string \| null>` | Auto-opened on launch |
| `settingsLoaded` | `Writable<boolean>` | Whether settings have loaded |
| `graphColumnWidths` | `Writable<GraphColumnWidths>` | Persisted column widths |
| `sortedRecentRepos` | `Derived<RecentRepo[]>` | Pinned-first, then sorted by lastOpened |

**Repository Actions**: `openRepo`, `refreshAll`, `refreshStatus`, `stageFile`, `unstageFile`, `stageFiles`, `stageAllFiles`, `unstageAllFiles`, `stageAllAndClear`, `stageUnstagedAndClear`, `stageUntrackedAndClear`, `unstageAllAndClear`, `selectFile`, `clearFileSelection`, `selectNextFile`, `selectPrevFile`, `stageSelectedFile`, `unstageSelectedFile`, `viewFileDiff`, `viewStagedFileDiff`, `selectCommit`, `commitAndRefresh`, `checkoutBranch`, `rebaseOnto`, `mergeInto`

**AI Stores** (`stores/ai.ts`):
| Store | Type | Purpose |
|-------|------|---------|
| `aiProviders` | `Writable<AIProvider[]>` | All configured providers with has_key status |
| `aiSettings` | `Writable<AISettings>` | Selected provider, model, custom URLs, max tokens |
| `aiGenerating` | `Writable<boolean>` | Whether generation is in progress |
| `aiError` | `Writable<string \| null>` | Config pre-check errors (shown inline in sidebar) |
| `aiFetchingModels` | `Writable<boolean>` | Whether model list is being fetched |
| `hasConfiguredProvider` | `Derived<boolean>` | Whether a provider with an API key is selected |
| `selectedProviderModels` | `Derived<AIModel[]>` | Models for the selected provider |

**AI Actions**: `initAI`, `loadAIProviders`, `loadAISettings`, `fetchModelsForProvider`, `saveAIKey`, `deleteAIKey`, `setSelectedProvider`, `setSelectedModel`, `setCustomBaseUrl`, `setMaxTokens`, `generateCommitMessage`

**Settings Actions**: `loadSettings`, `trackRepoOpen`, `removeRepo`, `togglePin`, `saveGraphColumnWidths`

**Rules**:
- No optimistic updates — always wait for backend confirmation.
- Actions catch errors and write to the `error` store.
- Parallel fetches use `Promise.all` (e.g., status + graph on repo open).
- Selection-aware actions (e.g., `stageSelectedFile`) auto-advance to the next file.

---

## Specialist: Frontend UI / Components

**Trigger keywords**: component, Svelte, layout, sidebar, toolbar, graph, diff viewer, panel, styling, CSS, theme, design token, command palette, syntax highlighting

**Read first**:
- `docs/DEVELOPER_GUIDE.md` — Section 8 (Adding a Frontend Component)
- `docs/TECHNICAL_SPEC.md` — Section 10 (Commit Graph Rendering)

**Core files**:
| File | Responsibility |
|------|---------------|
| `src/app.css` | Design system: TailwindCSS v4, shadcn vars, oklch colors, git status colors |
| `src/app.html` | HTML shell |
| `src/lib/utils.ts` | `cn()` class merge helper (clsx + tailwind-merge) |
| `src/lib/highlight.ts` | Shiki syntax highlighter (Catppuccin Mocha, singleton, 14 preloaded languages) |
| `src/routes/+page.svelte` | Main page (settings load, conditional graph/diff/welcome) |
| `src/routes/+layout.svelte` | Root layout (imports app.css) |

**Layout components** (`components/layout/`):
| File | Responsibility |
|------|---------------|
| `AppShell.svelte` | Outer frame: toolbar (top) + sidebar (left) + main area + status bar (bottom) |
| `Toolbar.svelte` | 48px header: "Gitron" brand, CommandBar (center), branch button (right); Cmd+K global shortcut |
| `Sidebar.svelte` | 260px left panel: staged/unstaged/untracked file sections with per-file stage/unstage buttons, bulk actions, commit panel (textarea + Cmd+Enter submit) |
| `StatusBar.svelte` | 24px footer: branch name, staged/changed counts |

**Graph components** (`components/graph/`):
| File | Responsibility |
|------|---------------|
| `CommitGraph.svelte` | Resizable 5-column grid (Graph/Message/Author/Date/SHA), persisted column widths, 10-color branch palette, ArrowUp/Down keyboard nav |
| `CommitDetail.svelte` | Collapsible detail panel: summary row (collapsed) or full message + metadata table (expanded) |

**Diff components** (`components/diff/`):
| File | Responsibility |
|------|---------------|
| `FilePreview.svelte` | Full-area inline diff viewer with shiki syntax highlighting, hunk separators, keyboard shortcuts (s/u/Escape/arrows), binary + empty states |

**UI primitives** (`components/ui/` — shadcn-svelte via bits-ui):
| Directory | Responsibility |
|-----------|---------------|
| `button/` | Button (variants: default, destructive, outline, secondary, ghost, link; sizes: default, sm, lg, icon) |
| `badge/` | Badge (variants: default, secondary, destructive, outline) |
| `command/` | CommandBar: searchable palette (recent repos, folder dialog, local/remote branches) |
| `scroll-area/` | ScrollArea wrapper (vertical/horizontal) |
| `separator/` | Separator (horizontal/vertical) |
| `tabs/` | Tabs primitive |
| `tooltip/` | Tooltip primitive |

**Design tokens** (from `app.css` — shadcn/Tailwind convention):
```
Core: --background, --foreground, --card, --popover, --primary, --secondary, --muted, --accent, --destructive
Structural: --border, --input, --ring
Sidebar: --sidebar-background, --sidebar-foreground, --sidebar-primary, etc.
Git status: --color-git-added, --color-git-modified, --color-git-deleted (+ -bg, -foreground variants)
```

**Rules**:
- Use Svelte 5 runes (`$state`, `$derived`, `$props`), NOT Svelte 4 patterns.
- Use Tailwind classes and shadcn CSS variables for all colors. No hard-coded hex values.
- UI primitives use `bits-ui` for headless behavior + `tailwind-variants` for styling.
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
| `crates/gitron-core/src/watcher/handler.rs` | Watcher setup, event debouncing, event classification |
| `crates/gitron-core/src/watcher/manager.rs` | WatcherManager lifecycle, uses `Arc<dyn EventEmitter>` |
| `crates/gitron-core/src/watcher/mod.rs` | Module declaration |
| `crates/gitron-core/src/cache/repo_state.rs` | In-memory cache updated by watcher |
| `crates/gitron-core/src/cache/mod.rs` | Module declaration |
| `crates/gitron-core/src/event.rs` | `EventEmitter` trait (emit_status_changed, emit_refs_changed) |
| `src-tauri/src/tauri_impls.rs` | `TauriEventEmitter` — emits Tauri events via AppHandle |
| `crates/gitron-server/src/sse.rs` | `SseBroadcaster` — emits SSE events via broadcast channel |

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

## Specialist: AI Commit Generation

**Trigger keywords**: AI, commit message, generate, LLM, provider, API key, keychain, OpenAI, Anthropic, Gemini, OpenRouter, model selection, max tokens

**Read first**:
- This section (file map and rules below)
- `docs/TECHNICAL_SPEC.md` — Section 3 (AI Commands in IPC Reference)

**Core files**:
| File | Responsibility |
|------|---------------|
| `crates/gitron-core/src/ai/mod.rs` | Module declarations |
| `crates/gitron-core/src/ai/error.rs` | `AIError` enum (thiserror, Serialize) and `AIResult<T>` type alias |
| `crates/gitron-core/src/ai/credential.rs` | API key storage via `crate::credential` trait. Functions: `store_key`, `get_key`, `delete_key`, `has_key` |
| `crates/gitron-core/src/ai/types.rs` | `AIProvider`, `AIModel`, `GenerateResult`, `AISettings` structs |
| `crates/gitron-core/src/ai/providers.rs` | Provider definitions (OpenAI, Anthropic, Gemini, OpenRouter), fallback models, default base URLs, dynamic model fetching from provider APIs |
| `crates/gitron-core/src/ai/generate.rs` | Builds prompt from staged diffs, calls provider APIs (OpenAI, Anthropic, Gemini, OpenRouter), parses response into title + body |
| `src-tauri/src/commands/ai.rs` | 7 Tauri commands (thin wrappers around gitron-core) |
| `crates/gitron-server/src/routes/ai.rs` | 7 HTTP endpoints (mirrors Tauri commands) |
| `src/lib/api/ai.ts` | Tauri invoke wrappers for all AI commands |
| `src/lib/api/types.ts` | TypeScript mirrors: `AIProvider`, `AIModel`, `GenerateResult`, `AISettings` |
| `src/lib/stores/ai.ts` | AI state stores + actions: providers, settings, generating state, model fetching, commit generation |
| `src/lib/components/ui/settings/AISettings.svelte` | Settings UI: provider cards, API key management, model dropdown, custom base URL, max tokens |
| `src/lib/components/layout/Sidebar.svelte` | Sparkle button next to commit title for AI generation |

**Key dependencies**: `reqwest` (HTTP client), `keyring` (via `credential_store`), `serde_json`

**Architecture**:
- API keys stored in OS keychain via `credential_store` module (same pattern as GitHub OAuth)
- Non-sensitive config (`AISettings`: selected provider/model, custom URLs, max tokens) persisted in `tauri-plugin-store`
- Provider-specific API calls: OpenAI uses `max_completion_tokens`, OpenRouter uses `max_tokens`, Anthropic uses `max_tokens`, Gemini uses `maxOutputTokens`
- Dynamic model fetching from each provider's API with filtering (OpenAI: chat models only; OpenRouter: affordable models; Gemini: text generation models)
- Per-provider model selection remembered via `selected_models` map in `AISettings`
- Generation errors display in the top-level error banner (same as git push/pull failures)

**Rules**:
- AI logic lives in `src-tauri/src/ai/`. Never in `commands/`.
- Commands are thin — delegate to `ai/` module functions.
- Provider API key format: `ai-{provider_id}` in keychain (e.g., `ai-openai`).
- Adding a new provider: add to `PROVIDERS` array in `providers.rs`, add `fetch_*_models()` function, add match arm in `generate.rs`, add match arm in `providers::fetch_models()`.

---

## Specialist: Agent Gateway / MCP (Future)

**Trigger keywords**: agent, MCP, Model Context Protocol, autonomous, action queue, approve, reject, permission, agent visualization, AI workflow

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

**Trigger keywords**: build, compile, Cargo.toml, package.json, tauri.conf.json, release, bundle, icon, install, deploy, CI/CD, Docker, server binary

**Core files**:
| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Workspace root — members, shared dependency versions |
| `crates/gitron-core/Cargo.toml` | Core crate dependencies |
| `src-tauri/Cargo.toml` | Tauri app dependencies (depends on gitron-core) |
| `crates/gitron-server/Cargo.toml` | Server dependencies (depends on gitron-core) |
| `src-tauri/tauri.conf.json` | Tauri app config (window, build commands, bundle) |
| `src-tauri/capabilities/default.json` | Tauri capability grants (core, opener, dialog, store) |
| `package.json` | Frontend dependencies and scripts |
| `svelte.config.js` | SvelteKit config (adapter-static, SPA fallback) |
| `vite.config.js` | Vite config (dev server port 1420, TailwindCSS plugin, Tauri HMR) |
| `Dockerfile` | Multi-stage build (Node frontend + Rust server + Debian runtime) |
| `.github/workflows/ci.yml` | CI: frontend check, Tauri build (4 targets), server build |
| `.github/workflows/release.yml` | Release: Tauri installers + server binaries (4 targets) |

**Commands**:
```bash
pnpm dev              # Desktop mode (Tauri + Vite dev server)
pnpm server:dev       # Web mode (build frontend + Axum server at :9417)
pnpm build            # Build frontend only
pnpm build:server     # Build server binary only
cargo check           # Type-check Rust workspace
pnpm check            # Type-check frontend only
```

---

## Cross-Cutting Concerns

### When adding a new feature or command (PARITY CHECKLIST):

Every new backend feature must work in BOTH Tauri and web mode. Follow this order:

1. **Core logic** → `crates/gitron-core/src/` (types, git logic, AI logic, etc.)
2. **Tauri command** → `src-tauri/src/commands/*.rs` + register in `lib.rs`
3. **Server route** → `crates/gitron-server/src/routes/*.rs` + register in `routes/mod.rs`
4. **TypeScript types** → `src/lib/api/types.ts`
5. **API function** → `src/lib/api/repo.ts` (or `ai.ts` / `github.ts`) using `getTransport().invoke()`
6. **COMMAND_MAP** → Add entry in `src/lib/api/transport-http.ts` mapping command name to `/api/*` endpoint
7. **Store** → `src/lib/stores/*.ts`
8. **Component** → `src/lib/components/`
9. **Test both** → `pnpm dev` (Tauri) AND `pnpm server:dev` (web)

### When adding a new real-time event:
1. Define payload type in `gitron-core` types
2. Add method to `EventEmitter` trait in `crates/gitron-core/src/event.rs`
3. Implement in `TauriEventEmitter` (`src-tauri/src/tauri_impls.rs`)
4. Implement in `SseBroadcaster` (`crates/gitron-server/src/sse.rs`)
5. Add listener in `src/lib/stores/watcher.ts` using `getTransport().listen()`

### When working on AI / commit generation:
1. Read the "Specialist: AI Commit Generation" section above
2. Provider-specific logic lives in `ai/providers.rs` (model fetching) and `ai/generate.rs` (API calls)
3. Adding a new provider: `providers.rs` PROVIDERS array → `providers.rs` fetch function → `generate.rs` match arm → frontend will auto-discover via `ai_get_providers`

### When debugging IPC / API issues:
1. **Tauri mode**: Check `lib.rs` — is the command registered in `generate_handler![]`?
2. **Web mode**: Check `routes/mod.rs` — is the route registered? Check `transport-http.ts` — is the command in COMMAND_MAP?
3. Check parameter naming — Tauri auto-converts camelCase↔snake_case, but HTTP does NOT. Server structs need `#[serde(rename = "camelCase")]`.
4. Check wrapper structs — if frontend sends `{ settings: data }`, server handler must accept a struct with a `settings` field.
5. Check error serialization — `GitError` implements `Serialize`

### When modifying the UI layout:
1. Read `docs/DEVELOPER_GUIDE.md` Section 8 — component conventions
2. Read `docs/TECHNICAL_SPEC.md` Section 10 — graph rendering approach
3. Use design tokens from `app.css`. Never hard-code colors.
4. Use `isTauri()` from `$lib/api` for platform-specific UI (e.g., zoom uses Tauri API vs CSS transform)

### When working on performance:
1. Read `docs/TECHNICAL_SPEC.md` Section 12 (Concurrency) and the Performance Considerations in `docs/ARCHITECTURE.md`
2. Hot paths: graph building, diff computation, status reading — these must use git2-rs, not CLI
3. Frontend: virtualize long lists, use Canvas for the graph, debounce events

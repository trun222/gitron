# Gitron — Roadmap

## Phase 1: Foundation (Current)

**Goal:** Prove the Rust + Tauri + Svelte pipeline works end-to-end. Establish the core git abstraction and render a commit graph.

- [x] Document vision, architecture, and roadmap
- [x] Document technical spec, plugin system, agent gateway, developer guide
- [x] Scaffold Tauri v2 + Svelte project
- [x] Define core data types (Rust + TypeScript mirrors)
- [x] Implement git2-rs backend for graph reading
- [x] Implement git2-rs backend for repo status
- [x] Implement git2-rs backend for diff computation
- [x] Implement git2-rs backend for staging operations
- [x] Implement git2-rs backend for branch operations
- [x] Wire up Tauri IPC commands (repo, graph, diff, staging, branch)
- [x] Build app shell layout (sidebar, toolbar, status bar)
- [x] Build commit graph list view with branch labels
- [x] Build commit detail panel
- [x] Build sidebar (changes tab, branches tab)
- [x] Create Svelte stores and API bindings
- [x] Create file watcher stub (notify-rs)
- [x] Create repo state cache stub
- [ ] Test end-to-end with `npm run tauri dev`
- [ ] Wire file watcher to Tauri events for live updates
- [ ] Implement Canvas-based graph renderer (replace HTML table)

**Exit criteria:** Can open a git repo, see the commit graph, and see it update live when changes happen.

## Phase 2: Core Git Workflow

**Goal:** Make Gitron usable as a daily Git GUI for basic operations.

- [x] Staging panel — interactive stage/unstage per file (in Sidebar with per-file buttons + bulk actions)
- [ ] Hunk-level and line-level staging
- [x] Commit authoring panel (message input + Cmd/Ctrl+Enter commit in Sidebar)
- [x] Diff viewer (inline mode with shiki syntax highlighting — FilePreview component)
- [ ] Diff viewer (side-by-side mode)
- [ ] Branch panel — create, delete, rename with UI
- [x] Checkout branches via UI (CommandBar with local/remote branch sections)
- [ ] Fetch / pull / push commands
- [ ] Remote management
- [x] Keyboard shortcuts for common operations (partial: Cmd+K command palette, ArrowUp/Down graph nav, s/u/Escape in diff, Cmd+Enter commit)
- [ ] Multiple repo tabs / switching
- [ ] File tree view for navigating repo

**Exit criteria:** Can perform a full daily git workflow (status -> stage -> commit -> push) without touching the CLI.

## Phase 3: Advanced Git Features

**Goal:** Reach GitKraken feature parity for power users.

- [ ] Merge with conflict detection
- [ ] Merge conflict editor (3-way)
- [ ] Interactive rebase (via git CLI bridge)
- [ ] Cherry-pick
- [ ] Stash management (create, pop, apply, drop)
- [ ] Tag management
- [ ] Git blame integration
- [ ] File history view
- [ ] Submodule support
- [ ] Git-flow support
- [ ] Gitignore editor
- [ ] Commit search and filtering
- [ ] Commit amend / fixup
- [ ] Drag-and-drop branch operations (merge, rebase, cherry-pick)

**Exit criteria:** Power users can do everything they do in GitKraken.

## Phase 4: Plugin System

**Goal:** Open the platform for community extensions.

See [PLUGIN_SYSTEM.md](./PLUGIN_SYSTEM.md) for full design.

- [ ] Backend plugin trait and lifecycle
- [ ] Plugin loader and registry
- [ ] Plugin sandboxing and permissions
- [ ] Frontend plugin API (panel registration, context menus, toolbar)
- [ ] UI extension points (slots) implemented
- [ ] Plugin configuration UI
- [ ] Plugin distribution format (crate + npm package)
- [ ] Documentation: plugin development guide
- [ ] Example plugins:
  - [ ] GitHub integration (PRs, issues, actions status)
  - [ ] GitLab integration
  - [ ] Bitbucket integration
  - [ ] Conventional commits helper
  - [ ] Git hooks manager

**Exit criteria:** A third-party developer can write, package, and distribute a plugin that adds UI and backend functionality.

## Phase 5: Agent Gateway & AI Integration

**Goal:** Make Gitron an AI-native git interface.

See [AGENT_GATEWAY.md](./AGENT_GATEWAY.md) for full design.

- [ ] Agent Gateway core
- [ ] MCP server exposing repo state as resources
- [ ] MCP tools for git operations
- [ ] Agent permission system (read-only, branch-scoped, full)
- [ ] Agent action queue with approval UI
- [ ] Event stream for agent subscriptions
- [ ] Agent activity visualization on commit graph
- [ ] Human vs agent commit distinction in graph
- [ ] Built-in AI workflows:
  - [ ] Smart commit message generation
  - [ ] PR description generation
  - [ ] Conflict resolution suggestions
  - [ ] Code review in diff viewer
  - [ ] "Explain this history" narrative
- [ ] Autonomous agent mode:
  - [ ] Auto-branching
  - [ ] CI-aware agent responses
  - [ ] Multi-repo orchestration
  - [ ] Release management agent

**Exit criteria:** An AI agent can connect to Gitron via MCP, read repo state, propose changes, and have them reviewed/executed — all visible in the UI.

## Phase 6: Polish & Distribution

**Goal:** Production-ready release.

- [ ] Theming system (light/dark + custom)
- [ ] Settings / preferences panel
- [ ] Auto-update system (Tauri updater)
- [ ] Code signing (macOS, Windows)
- [ ] Installer builds (DMG, MSI/NSIS)
- [ ] Performance profiling and optimization
- [ ] Accessibility audit
- [ ] Onboarding / first-run experience
- [ ] Website and documentation site
- [ ] CI/CD pipeline for releases
- [ ] Community contribution pipeline

**Exit criteria:** Users can download, install, and use Gitron with a polished, production-quality experience.

---

## Current Status

**Phase:** 1 (Foundation) — nearing completion; several Phase 2 items already done
**Completed:** Scaffolding, core git backend (19 IPC commands), frontend UI shell (sidebar with staging + commit, command palette, inline diff viewer with syntax highlighting, resizable commit graph), settings persistence (recent repos, column widths), documentation
**Remaining in Phase 1:** Test end-to-end with `npm run tauri dev`, wire file watcher to Tauri events for live updates, Canvas-based graph renderer
**Phase 2 progress:** Staging panel, commit authoring, inline diff, branch checkout, partial keyboard shortcuts already implemented
**Documentation:** See [docs/README.md](./README.md) for a full index

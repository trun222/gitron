# Gitron — Roadmap

## Phase 1: Foundation (Current)

**Goal:** Prove the Rust + Tauri + Svelte pipeline works end-to-end. Establish the core git abstraction and render a commit graph.

- [x] Document vision, architecture, and roadmap
- [ ] Scaffold Tauri v2 + Svelte project
- [ ] Implement core git trait layer (`git/traits.rs`, `git/types.rs`)
- [ ] Implement git2-rs backend for graph reading
- [ ] Implement git2-rs backend for repo status
- [ ] Wire up Tauri IPC commands (repo open, graph query, status)
- [ ] Build commit graph canvas renderer (Svelte)
- [ ] Build basic app shell layout (sidebar, toolbar, main area)
- [ ] File watcher integration (notify-rs)
- [ ] Repo state cache with incremental updates

**Exit criteria:** Can open a git repo, see the commit graph, and see it update live when changes happen.

## Phase 2: Core Git Workflow

**Goal:** Make Gitron usable as a daily Git GUI for basic operations.

- [ ] Staging panel — stage/unstage files
- [ ] Hunk-level and line-level staging
- [ ] Commit authoring panel
- [ ] Diff viewer (inline mode)
- [ ] Diff viewer (side-by-side mode)
- [ ] Branch panel — list, create, delete, rename
- [ ] Checkout branches
- [ ] Fetch / pull / push
- [ ] Remote management
- [ ] Status bar with repo info
- [ ] Keyboard shortcuts for common operations
- [ ] Multiple repo tabs / switching

**Exit criteria:** Can perform a full daily git workflow (status → stage → commit → push) without touching the CLI.

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

**Phase:** 1 — Foundation
**Next step:** Scaffold Tauri v2 + Svelte project, implement core git traits
**Documentation:** See [VISION.md](./VISION.md) and [ARCHITECTURE.md](./ARCHITECTURE.md)

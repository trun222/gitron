# Gitron — Vision

## What is Gitron?

Gitron is an open-source, high-performance Git GUI built with Rust (Tauri v2) and Svelte. It aims to deliver the polish and feature depth of GitKraken while being free, extensible, and AI-native.

## Why Gitron?

The Git GUI landscape has a gap:

- **GitKraken** — polished but Electron-heavy, partially paywalled, closed-source
- **GitHub Desktop** — limited features, GitHub-centric, Electron
- **GitButler** — innovative but opinionated (virtual branches), not a traditional Git GUI
- **Lazygit** — excellent but terminal-only, no graphical UI
- **Gittyup** — small community, limited momentum

Gitron fills the gap: a **fast, full-featured, cross-platform, open-source Git GUI** with a plugin architecture and first-class AI agent integration.

## Core Principles

### 1. Performance First
Every interaction must feel instant. The commit graph, diffs, staging — all hot paths use in-process Rust (git2-rs) with an in-memory repo state cache. No Electron. No unnecessary overhead. The UI compiles away framework abstractions (Svelte) and renders the commit graph via Canvas/WebGL.

### 2. Open Source and Extensible
Gitron is built plugin-first. The core application uses the same APIs available to plugins. Contributors can add features without touching core code. The plugin system spans both the Rust backend (trait-based) and Svelte frontend (slot-based UI injection).

### 3. AI-Native
Gitron treats AI agents as first-class participants in the git workflow. The Agent Gateway exposes repo state via MCP (Model Context Protocol), allowing external agents to read, analyze, and propose changes to repositories. Agents operate within a permission system with human-in-the-loop or fully autonomous modes.

### 4. Cross-Platform
macOS and Windows are first-class targets. Linux support is a natural follow-on given the Tauri + Svelte stack.

## Differentiators

### Plugin Architecture
- Rust-side plugins via traits and Tauri's plugin system
- Frontend plugins via defined UI extension points (panels, sidebars, context menus, toolbar)
- Plugins distributed as crates (backend) and npm packages (frontend)
- The core app eats its own dog food — built on the same plugin APIs

### Agent Gateway
- **MCP Server** — exposes structured repo state (graph, diffs, branches, conflicts) to AI agents
- **Agent Permissions** — scoped access control (read-only, branch-only, full access)
- **Event Stream** — agents subscribe to real-time repo events
- **Action Queue** — agent-proposed actions go through human review or auto-approve policies
- **Agent Visualization** — see agent activity on the commit graph, distinguish human vs agent commits

### AI Workflows (Built-in and Extensible)
- Smart commit message generation from staged changes
- PR description generation from branch diffs
- Conflict resolution suggestions
- Inline code review within the diff viewer
- "Explain this history" — narrative summaries of commit ranges
- Autonomous agent flows: auto-branching, CI-aware responses, release management

## Target Users

1. **Individual developers** who want a fast, free Git GUI
2. **Open-source contributors** who want to extend and customize their Git workflow
3. **AI-augmented development teams** who want agents integrated into their git operations
4. **Developers frustrated with GitKraken's paywall** who want equivalent features without cost

## Technology Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Backend language | Rust | Performance, safety, git2-rs bindings, Tauri ecosystem |
| Desktop framework | Tauri v2 | Lightweight (~5-10MB), native webview, no Electron overhead |
| Frontend framework | Svelte | Compiles away, minimal runtime, less boilerplate, proven with Tauri |
| Git library | git2-rs + git CLI (hybrid) | git2-rs for hot paths, CLI for full feature coverage |
| Graph rendering | Canvas/WebGL | DOM nodes cannot handle large repos performantly |
| Agent protocol | MCP | Standard protocol, works with Claude and other AI systems |
| Plugin backend | Rust traits + Tauri plugins | Type-safe, composable, distribution via crates |
| Plugin frontend | JS/TS API with UI slots | Accessible to web developers, controlled extension points |

# Gitron — Claude Code Instructions

## Project Overview

Gitron is an open-source, AI-native Git GUI built with Rust (Tauri v2) and Svelte 5. It aims to replace GitKraken with a fast, extensible, cross-platform desktop app that has first-class plugin and AI agent support.

## Documentation

**Always consult the relevant documentation before making changes.** The docs are the source of truth for architecture, conventions, and design decisions.

| Document | When to read |
|----------|-------------|
| `docs/ROADMAP.md` | First — check current status and what phase we're in |
| `docs/TECHNICAL_SPEC.md` | Before implementing any feature — IPC commands, data types, state management, event system |
| `docs/DEVELOPER_GUIDE.md` | Before writing code — module boundaries, conventions, step-by-step guides |
| `docs/ARCHITECTURE.md` | When understanding how subsystems connect — system diagram, module structure |
| `docs/VISION.md` | When making strategic/directional decisions |
| `docs/PLUGIN_SYSTEM.md` | When working on the plugin system (Phase 4) |
| `docs/AGENT_GATEWAY.md` | When working on the agent gateway (Phase 5) |
| `docs/SPECIALISTS.md` | When routing work — maps topics to files and docs |

## Critical Rules

### Module Boundaries (STRICT)

1. **`src-tauri/src/commands/`** contains ZERO git logic. Commands only parse params, call `git/` functions, return results.
2. **`src-tauri/src/git/`** contains ALL git logic. Every git2-rs call lives here.
3. **Frontend components** NEVER call `invoke()` directly. They use stores (`$lib/stores/`) which call API functions (`$lib/api/`).
4. **Types** are defined in Rust (`git/types.rs`) and mirrored in TypeScript (`lib/api/types.ts`). Keep them in sync manually.

### Adding Features

Follow the step-by-step checklist in `docs/DEVELOPER_GUIDE.md` Section 6 ("Adding a New Feature"). The pattern is always:

```
Rust types → Rust git logic → Rust command → Register in lib.rs → TS types → TS API → Store → Component
```

### Frontend Conventions

- **Svelte 5 runes**: Use `$state()`, `$derived()`, `$props()`. Do NOT use Svelte 4 patterns (`$:`, `export let`).
- **Styling**: Use CSS custom properties from `app.css`. Never hard-code colors.
- **SSR disabled**: SvelteKit runs in SPA mode (`ssr = false`). Do not enable SSR.

### Rust Conventions

- Return `GitResult<T>` from git functions
- No `unwrap()` in production code
- Use `thiserror` for error types
- Command handlers are functions, not methods

### Tauri IPC

- Rust uses `snake_case` for command parameters
- Frontend passes `camelCase` — Tauri auto-converts
- Every mutation command returns updated state (so frontend doesn't need a separate refresh)

## Tech Stack

- **Backend**: Rust, Tauri v2, git2-rs, notify-rs, tokio, serde
- **Frontend**: Svelte 5, SvelteKit, TypeScript, Vite
- **Crate name**: `gitron`, lib name: `gitron_lib`
- **Git strategy**: git2-rs for hot paths (graph, diff, status, staging), git CLI for complex ops (rebase, advanced merge)

## Project Structure Quick Reference

```
src-tauri/src/
  lib.rs           — Tauri builder, command registration
  commands/        — IPC handlers (repo, graph, diff, staging, branch)
  git/             — All git logic (types, error, repository, graph, diff)
  cache/           — Repo state cache
  watcher/         — File system watcher

src/
  app.css          — Design system / theme tokens
  lib/api/         — Tauri invoke bindings + TypeScript types
  lib/stores/      — Svelte stores + actions
  lib/components/  — UI components (layout/, graph/)
  routes/          — SvelteKit pages
```

## Commit Convention

```
type(scope): description

Types: feat, fix, refactor, docs, chore, test, style
Scopes: git, ui, graph, diff, staging, branch, plugin, agent, docs
```

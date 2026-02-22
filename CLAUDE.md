# Gitron — Claude Code Instructions

## Project Overview

Gitron is an open-source, AI-native Git GUI built with Rust and Svelte 5. It runs in two modes from the same codebase:

1. **Desktop mode** — Tauri v2 native app (default: `pnpm dev`)
2. **Web/Server mode** — Axum HTTP server serving the Svelte frontend (`pnpm server:dev`)

Both modes share the same Svelte frontend and the same Rust core logic (`gitron-core`). The only differences are the transport layer (Tauri IPC vs HTTP/SSE) and platform-specific adapters (credential storage, event emission, directory picker).

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

### Dual-Mode Parity (STRICT)

**The Tauri desktop app and the Axum web server MUST stay in sync.** When making changes:

1. **New backend feature** → Add to `gitron-core` first, then wire into BOTH `src-tauri/src/commands/` AND `crates/gitron-server/src/routes/`. Never add logic to only one mode.
2. **New API endpoint** → Add the Tauri command, the server route handler, AND the command-to-endpoint mapping in `src/lib/api/transport-http.ts` `COMMAND_MAP`.
3. **New frontend feature** → Use `getTransport()` for all backend calls. Never import directly from `@tauri-apps/*` outside of `transport-tauri.ts` and `settings.ts` (TauriSettingsStore).
4. **New real-time event** → Emit via `EventEmitter` trait in core, handle in BOTH `TauriEventEmitter` (desktop) and `SseBroadcaster` (server).
5. **Test both modes** after any change: `pnpm dev` (Tauri) and `pnpm server:dev` (web).

### Module Boundaries (STRICT)

1. **`crates/gitron-core/`** contains ALL shared logic: git, ai, github, cache, watcher, credential/event traits.
2. **`src-tauri/src/commands/`** contains ZERO git logic. Commands only parse params, call `gitron_core::` functions, return results.
3. **`crates/gitron-server/src/routes/`** mirrors Tauri commands as HTTP endpoints. Same rule: ZERO git logic.
4. **Frontend components** NEVER call `invoke()` or `fetch()` directly. They use stores (`$lib/stores/`) which call API functions (`$lib/api/`), which use `getTransport()`.
5. **Types** are defined in Rust (`gitron-core` types) and mirrored in TypeScript (`lib/api/types.ts`). Keep them in sync manually.

### Adding Features

Follow the step-by-step checklist in `docs/DEVELOPER_GUIDE.md` Section 6 ("Adding a New Feature"). The pattern is always:

```
Rust types (gitron-core) → Rust git logic (gitron-core) → Tauri command + Server route → TS types → TS API → COMMAND_MAP entry → Store → Component
```

### Frontend Conventions

- **Transport abstraction**: Always use `getTransport()` from `$lib/api`. Never import `@tauri-apps/*` in components or stores (except `transport-tauri.ts` and `TauriSettingsStore` in `settings.ts`).
- **Svelte 5 runes**: Use `$state()`, `$derived()`, `$props()`. Do NOT use Svelte 4 patterns (`$:`, `export let`).
- **Styling**: Use CSS custom properties from `app.css`. Never hard-code colors.
- **SSR disabled**: SvelteKit runs in SPA mode (`ssr = false`). Do not enable SSR.

### Rust Conventions

- Return `GitResult<T>` from git functions
- No `unwrap()` in production code
- Use `thiserror` for error types
- Command handlers are functions, not methods

### Tauri IPC / HTTP Parity

- Tauri uses `snake_case` command names. The HTTP transport maps them to `/api/*` endpoints via `COMMAND_MAP` in `transport-http.ts`.
- Tauri auto-converts `camelCase` frontend params to `snake_case`. Server route structs must use `#[serde(rename = "camelCase")]` for multi-word fields.
- When the frontend sends `invoke('cmd', { wrapper: data })`, the server handler must accept a struct with a `wrapper` field — not the raw inner type.
- Every mutation command returns updated state (so frontend doesn't need a separate refresh).

## Tech Stack

- **Shared core**: Rust (`gitron-core`), git2-rs, notify-rs, tokio, serde, reqwest
- **Desktop**: Tauri v2, tauri-plugin-store, tauri-plugin-dialog, tauri-plugin-opener
- **Web server**: Axum, tower-http (fs, cors, trace), clap, tokio-stream (SSE)
- **Frontend**: Svelte 5, SvelteKit, TypeScript, Vite
- **Workspace**: Cargo workspace with 3 members: `src-tauri`, `crates/gitron-core`, `crates/gitron-server`
- **Git strategy**: git2-rs for hot paths (graph, diff, status, staging), git CLI for complex ops (rebase, advanced merge)

## Project Structure Quick Reference

```
Cargo.toml               — Workspace root (3 members)
crates/gitron-core/src/  — Shared Rust logic (git, ai, github, cache, watcher, traits)
  credential.rs          — CredentialStore trait + global OnceLock
  event.rs               — EventEmitter trait (status/refs changed)
  git/                   — All git logic (types, error, repository, graph, diff, cli, remote)
  ai/                    — AI generation (providers, credentials, generate)
  github/                — GitHub OAuth + API (auth, api, credentials, types)
  cache/                 — Repo state cache
  watcher/               — File system watcher (manager + handler)

src-tauri/src/           — Tauri desktop app (thin wrapper)
  lib.rs                 — Tauri builder, command registration
  tauri_impls.rs         — TauriCredentialStore, TauriEventEmitter
  commands/              — IPC handlers (delegates to gitron-core)

crates/gitron-server/src/ — Axum web server (thin wrapper)
  main.rs                — CLI (--port, --host, --token, --frontend-dir)
  routes/                — HTTP handlers (mirrors Tauri commands)
  sse.rs                 — SseBroadcaster (implements EventEmitter)
  file_store.rs          — FileCredentialStore (~/.config/gitron/)
  auth.rs                — Bearer token middleware

src/                     — Svelte frontend (shared between both modes)
  lib/api/
    transport.ts         — Transport interface + isTauri() detection
    transport-tauri.ts   — Tauri transport (dynamic imports)
    transport-http.ts    — HTTP transport (fetch + EventSource + COMMAND_MAP)
    index.ts             — getTransport() singleton
    repo.ts, ai.ts, github.ts — API functions using getTransport()
    settings.ts          — TauriSettingsStore / WebSettingsStore abstraction
  lib/stores/            — Svelte stores + actions
  lib/components/        — UI components (layout/, graph/, diff/, ui/)
  routes/                — SvelteKit pages
```

## Running

```bash
pnpm dev                 # Desktop mode (Tauri + Vite dev server)
pnpm server:dev          # Web mode (build frontend + start Axum server at :9417)
pnpm build               # Build frontend only
pnpm build:server        # Build server binary only
```

## Version Bumps

When asked to update the version, **all four files must be updated together**:

1. `src-tauri/tauri.conf.json` — `"version"` field (Tauri uses this for installers)
2. `package.json` — `"version"` field
3. `src-tauri/Cargo.toml` — `version` under `[package]`
4. `crates/gitron-server/Cargo.toml` — `version` under `[package]`

Tag format: `v{VERSION}` (e.g. `v0.2.0`). The tag must match the version in the files.

## Commit Convention

```
type(scope): description

Types: feat, fix, refactor, docs, chore, test, style
Scopes: git, ui, graph, diff, staging, branch, plugin, agent, docs, server, transport
```

# Gitron — Developer Guide

This document covers how to work on Gitron: environment setup, project conventions, module boundaries, how to add features, testing strategy, and contribution patterns. Anyone working on the codebase — including AI assistants — should read this first.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Getting Started](#getting-started)
3. [Project Structure](#project-structure)
4. [Development Workflow](#development-workflow)
5. [Module Boundaries](#module-boundaries)
6. [Adding a New Feature](#adding-a-new-feature)
7. [Adding a New IPC Command](#adding-a-new-ipc-command)
8. [Adding a Frontend Component](#adding-a-frontend-component)
9. [Coding Conventions](#coding-conventions)
10. [Testing Strategy](#testing-strategy)
11. [Common Pitfalls](#common-pitfalls)

---

## 1. Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Backend language |
| Cargo | 1.75+ | Rust package manager |
| Node.js | 20+ | Frontend toolchain |
| npm | 9+ | Frontend package manager |
| Git | 2.30+ | Required for git CLI bridge |

macOS additionally requires Xcode Command Line Tools:
```bash
xcode-select --install
```

Windows additionally requires:
- Visual Studio C++ Build Tools
- WebView2 (usually pre-installed on Windows 11)

---

## 2. Getting Started

```bash
# Clone the repository
git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron

# Install frontend dependencies
npm install

# Run in development mode (compiles Rust + starts Vite dev server)
npm run tauri dev

# Build for production
npm run tauri build
```

`npm run tauri dev` will:
1. Start the Vite dev server for the Svelte frontend (hot reload)
2. Compile the Rust backend
3. Open the Tauri window pointing at the dev server
4. Watch for Rust changes and recompile

---

## 3. Project Structure

```
gitron/
├── docs/                    # All documentation (you are here)
├── src/                     # Frontend (Svelte 5 + TypeScript)
│   ├── app.html             # HTML shell
│   ├── app.css              # Design system (TailwindCSS v4, shadcn vars, oklch colors)
│   ├── lib/
│   │   ├── utils.ts         # cn() helper (clsx + tailwind-merge)
│   │   ├── highlight.ts     # Shiki syntax highlighter (Catppuccin Mocha)
│   │   ├── api/             # Tauri IPC bindings
│   │   │   ├── types.ts     # TypeScript types mirroring Rust types + frontend-only types
│   │   │   ├── repo.ts      # Git invoke() calls
│   │   │   ├── ai.ts        # AI invoke() calls (providers, keys, generation, settings)
│   │   │   └── settings.ts  # Persistent settings via tauri-plugin-store
│   │   ├── stores/          # Svelte stores (state management)
│   │   │   ├── repo.ts      # All repo state (writable + derived) and actions
│   │   │   ├── ai.ts        # AI state (providers, settings, generation) and actions
│   │   │   └── settings.ts  # Settings state (recent repos, column widths) and actions
│   │   └── components/      # UI components
│   │       ├── layout/      # App shell (toolbar, sidebar, status bar)
│   │       ├── graph/       # Commit graph and detail views
│   │       ├── diff/        # Diff viewer (FilePreview with syntax highlighting)
│   │       └── ui/          # Headless UI primitives (shadcn-svelte via bits-ui)
│   └── routes/
│       ├── +layout.svelte   # Root layout
│       ├── +layout.ts       # SSR disabled (ssr = false)
│       └── +page.svelte     # Main page
├── src-tauri/               # Backend (Rust + Tauri v2)
│   ├── Cargo.toml           # Rust dependencies
│   ├── tauri.conf.json      # Tauri configuration
│   ├── capabilities/        # Tauri capability grants
│   ├── src/
│   │   ├── main.rs          # Entry point (calls lib::run)
│   │   ├── lib.rs           # Tauri builder, plugin + command registration
│   │   ├── commands/        # IPC command handlers (thin, no git/ai logic)
│   │   │   ├── repo.rs, graph.rs, diff.rs, staging.rs, branch.rs, commit.rs, ai.rs
│   │   │   └── mod.rs       # Module declarations + AppState struct
│   │   ├── git/             # Core git operations (ALL git logic here)
│   │   │   ├── types.rs, error.rs, repository.rs, graph.rs, diff.rs, cli.rs
│   │   │   └── mod.rs       # Re-exports
│   │   ├── ai/              # AI commit generation (providers, credentials, generation)
│   │   │   ├── error.rs, credential.rs, types.rs, providers.rs, generate.rs
│   │   │   └── mod.rs
│   │   ├── cache/           # In-memory repo state cache (implemented, not wired)
│   │   └── watcher/         # File system change watcher (implemented, not wired)
│   └── icons/               # App icons for all platforms
├── static/                  # Static assets
├── components.json          # shadcn-svelte configuration
├── package.json
├── svelte.config.js
├── vite.config.js
└── tsconfig.json
```

---

## 4. Development Workflow

### Frontend-only changes

Svelte components and stores hot-reload instantly via Vite. Edit a `.svelte` file, save, and the change appears in the window immediately. No restart needed.

### Backend changes

Rust changes trigger a recompile. Tauri watches `src-tauri/src/` and recompiles on save. The window restarts after recompile. Compile times after the first build are typically 1-5 seconds (incremental).

### Checking Rust without full build

```bash
cd src-tauri
cargo check        # Type-check only (fastest)
cargo clippy       # Lint check
cargo test         # Run tests
```

### Checking frontend without Tauri

```bash
npm run check      # Svelte type checking
```

---

## 5. Module Boundaries

These boundaries are strict. Violating them creates maintenance problems.

### Rule 1: `commands/` contains ZERO git logic

Command handlers parse IPC parameters, call `git/` functions, and return results. If you find yourself writing git2-rs code in a command handler, move it to `git/`.

```rust
// CORRECT — command handler delegates to git module
#[tauri::command]
pub fn get_status(path: String) -> Result<RepoStatus, GitError> {
    let repo = repository::open(&path)?;
    repository::get_status(&repo)
}

// WRONG — git logic in command handler
#[tauri::command]
pub fn get_status(path: String) -> Result<RepoStatus, GitError> {
    let repo = git2::Repository::discover(&path)?;
    let statuses = repo.statuses(Some(/* ... */))?;
    // ... 50 lines of parsing ...
}
```

### Rule 2: Frontend components don't call `invoke()` directly

Components use stores and actions. Only `src/lib/api/repo.ts` calls `invoke()`. Only `src/lib/stores/repo.ts` calls the API functions. Components read stores and call store actions.

```
Component → Store action → API function → invoke()
Component ← Store subscription ← Store update ← API response
```

```svelte
<!-- CORRECT — component uses store -->
<script>
  import { repoStatus, stageFile, repoPath } from '$lib/stores/repo';
  const handleStage = (path: string) => stageFile($repoPath!, path);
</script>

<!-- WRONG — component calls invoke directly -->
<script>
  import { invoke } from '@tauri-apps/api/core';
  const handleStage = (path: string) => invoke('stage_file', { path });
</script>
```

### Rule 3: Types are defined in Rust and mirrored in TypeScript

The Rust `git/types.rs` is the source of truth. When you change a type in Rust, update `src/lib/api/types.ts` to match. There is no codegen — this is manual and intentional (keeps things simple).

### Rule 4: One file per domain in `commands/`

Each file in `commands/` corresponds to one domain: `repo.rs`, `graph.rs`, `diff.rs`, `staging.rs`, `branch.rs`. When adding a new domain (e.g., `remote.rs`, `stash.rs`), create a new file. Don't stuff unrelated commands into existing files.

---

## 6. Adding a New Feature

Example: adding stash support.

### Step 1: Add types in Rust

```rust
// src-tauri/src/git/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stash {
    pub index: usize,
    pub message: String,
    pub oid: String,
}
```

### Step 2: Add git logic

```rust
// src-tauri/src/git/repository.rs (or a new stash.rs file)
pub fn list_stashes(repo: &Repository) -> GitResult<Vec<Stash>> { ... }
pub fn create_stash(repo: &Repository, message: &str) -> GitResult<Stash> { ... }
pub fn apply_stash(repo: &Repository, index: usize) -> GitResult<()> { ... }
pub fn drop_stash(repo: &Repository, index: usize) -> GitResult<()> { ... }
```

### Step 3: Add Tauri commands

```rust
// src-tauri/src/commands/stash.rs
#[tauri::command]
pub fn list_stashes(path: String) -> Result<Vec<Stash>, GitError> { ... }

#[tauri::command]
pub fn create_stash(path: String, message: String) -> Result<Stash, GitError> { ... }
```

### Step 4: Register commands

```rust
// src-tauri/src/lib.rs — add to invoke_handler
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    stash::list_stashes,
    stash::create_stash,
    stash::apply_stash,
    stash::drop_stash,
])
```

### Step 5: Add TypeScript types

```typescript
// src/lib/api/types.ts
export interface Stash {
  index: number;
  message: string;
  oid: string;
}
```

### Step 6: Add API bindings

```typescript
// src/lib/api/repo.ts
export async function listStashes(path: string): Promise<Stash[]> {
  return invoke('list_stashes', { path });
}
```

### Step 7: Add store state and actions

```typescript
// src/lib/stores/repo.ts
export const stashes = writable<Stash[]>([]);

export async function refreshStashes(path: string) {
  const result = await api.listStashes(path);
  stashes.set(result);
}
```

### Step 8: Add UI component

```svelte
<!-- src/lib/components/stash/StashPanel.svelte -->
<script>
  import { stashes } from '$lib/stores/repo';
</script>
```

### Step 9: Wire into the layout

Add the new component to the appropriate location (sidebar tab, bottom panel, etc.).

---

## 7. Adding a New IPC Command

Checklist for adding a new Tauri command:

1. [ ] Add any new types to `git/types.rs`
2. [ ] Implement the git logic in the appropriate `git/*.rs` file
3. [ ] Create the command handler in `commands/*.rs`
4. [ ] Add the command to `lib.rs` in the `generate_handler![]` macro
5. [ ] Mirror new types in `src/lib/api/types.ts`
6. [ ] Add the `invoke()` call in `src/lib/api/repo.ts`
7. [ ] Add store state/actions in `src/lib/stores/repo.ts` if needed
8. [ ] Test: `cargo check` (backend compiles), `npm run check` (frontend compiles)

---

## 8. Adding a Frontend Component

### Component conventions

- Components live in `src/lib/components/{domain}/`
- One component per file
- Use Svelte 5 runes: `$state()`, `$derived()`, `$props()`, `$effect()`
- Use TypeScript (`<script lang="ts">`)
- Styles are scoped (in `<style>` block, not global)
- Use CSS custom properties from `app.css` for theming

### Svelte 5 patterns used in this project

```svelte
<script lang="ts">
  // Props (Svelte 5)
  let { someProp, anotherProp = 'default' }: {
    someProp: string;
    anotherProp?: string;
  } = $props();

  // Local state (Svelte 5)
  let count = $state(0);

  // Derived values (Svelte 5)
  let doubled = $derived(count * 2);

  // Store subscriptions use $ prefix (unchanged from Svelte 4)
  import { repoStatus } from '$lib/stores/repo';
  // Use $repoStatus in template
</script>
```

### Styling conventions

- The project uses **TailwindCSS v4** with **shadcn-svelte** conventions
- Colors use oklch color space with CSS custom properties mapped to Tailwind utilities
- Never use hard-coded colors in components — use Tailwind classes or CSS variables
- UI primitives (button, badge, command, etc.) are in `src/lib/components/ui/` via `bits-ui`
- Component variants use `tailwind-variants`; class merging uses `clsx` + `tailwind-merge` via `cn()`

### Design tokens reference

The design system uses shadcn/Tailwind conventions. Variables are defined in `app.css` and mapped to Tailwind via `@theme inline`.

```css
/* Core backgrounds & text (shadcn convention) */
--background / --foreground        /* main app background/text */
--card / --card-foreground         /* card surfaces */
--popover / --popover-foreground   /* popover/dropdown surfaces */
--primary / --primary-foreground   /* primary actions */
--secondary / --secondary-foreground /* secondary elements */
--muted / --muted-foreground       /* muted/deemphasized text */
--accent / --accent-foreground     /* accent highlights */
--destructive / --destructive-foreground /* destructive actions */

/* Structural */
--border         /* standard borders */
--input          /* input field borders */
--ring           /* focus rings */

/* Sidebar (extends shadcn) */
--sidebar-* variants for sidebar-specific theming

/* Git status colors (Gitron-specific) */
--color-git-added / --color-git-added-bg / --color-git-added-foreground
--color-git-modified / --color-git-modified-bg / --color-git-modified-foreground
--color-git-deleted / --color-git-deleted-bg / --color-git-deleted-foreground
```

Dark theme uses Catppuccin Mocha-inspired oklch values. Light theme uses neutral grays. Theme is dark-first (`:root` = dark, `.light` variant available).

---

## 9. Coding Conventions

### Rust

- Use `thiserror` for error types, not manual `impl Display`
- Use `anyhow` for quick prototyping, `GitError` for the git API boundary
- Functions return `GitResult<T>` (alias for `Result<T, GitError>`)
- Command handlers are `#[tauri::command]` functions, not methods on structs
- Prefer explicit types over `impl Trait` in function signatures
- Clone sparingly — use references where possible, clone for IPC serialization
- No `unwrap()` in production code — use `?` or handle the error

### TypeScript

- Strict mode enabled
- Use `interface` for data shapes, `type` for unions and aliases
- Async functions for all IPC calls
- Store actions are `async function` exports, not methods on an object
- Use `$state()`, `$derived()`, `$props()` (Svelte 5 runes), NOT `let x;` reactive declarations (Svelte 4)

### Git

- Conventional commits: `type(scope): description`
- Types: `feat`, `fix`, `refactor`, `docs`, `chore`, `test`, `style`
- Scopes: `git`, `ui`, `graph`, `diff`, `staging`, `branch`, `plugin`, `agent`
- Example: `feat(graph): add branch color assignment`

---

## 10. Testing Strategy

### Backend Testing

```bash
cd src-tauri
cargo test
```

Test categories:
- **Unit tests**: Test individual git functions with temporary repos (`tempfile` + `git2::Repository::init`)
- **Integration tests**: Test Tauri command handlers end-to-end
- **Property tests**: Test graph building with randomly generated commit histories (future)

### Frontend Testing

```bash
npm run check        # Type checking
npm run test         # Unit tests (future, vitest)
```

Test categories:
- **Component tests**: Render components with mock stores, verify output
- **Store tests**: Test store actions with mocked API calls
- **E2E tests**: Use Tauri's WebDriver support for full-app tests (future)

### Manual Testing

For development, the fastest feedback loop is:
1. `npm run tauri dev`
2. Open a real git repository in the app
3. Make changes in the repository via terminal
4. Verify the app reflects the changes

---

## 11. Common Pitfalls

### 1. Forgetting to register a command

If you add a `#[tauri::command]` function but forget to add it to `generate_handler![]` in `lib.rs`, the command silently doesn't exist. The frontend will get an error when calling `invoke()`.

### 2. Type mismatches between Rust and TypeScript

Rust `Option<T>` serializes to `null` in JSON, which maps to `T | null` in TypeScript. Make sure TypeScript types use `| null` for optional Rust fields.

Rust enums serialize as strings by default with serde: `FileStatusType::Added` → `"Added"`. TypeScript should use string literal types: `'Added' | 'Modified' | ...`

### 3. git2::Repository is not thread-safe

Don't share a `Repository` across threads without a `Mutex`. The current stateless design avoids this by opening a fresh repo per command, but when moving to persistent state, wrap it.

### 4. Svelte 5 runes vs Svelte 4 reactivity

This project uses Svelte 5. Do NOT use:
- `$:` reactive declarations (use `$derived()` instead)
- `export let prop` (use `$props()` instead)
- `{#key}` for state (use `$state()` instead)

### 5. Tauri IPC parameter naming

Tauri commands use snake_case in Rust but the frontend must pass camelCase parameters. Tauri automatically converts between them. So `max_commits` in Rust becomes `maxCommits` in the `invoke()` call.

### 6. SvelteKit SSR must be disabled

Tauri apps don't have a server. `src/routes/+layout.ts` must export `export const ssr = false;`. If you add new route files, this is inherited from the layout — don't re-enable SSR.

<p align="center">
  <h1 align="center">Gitron</h1>
  <p align="center">
    Open-source, AI-native Git GUI built with Rust and Svelte
  </p>
  <p align="center">
    <a href="#installation">Install</a> &middot;
    <a href="#features">Features</a> &middot;
    <a href="#development">Development</a> &middot;
    <a href="#roadmap">Roadmap</a> &middot;
    <a href="#contributing">Contributing</a>
  </p>
</p>

---

Gitron is a fast, cross-platform Git GUI that aims to deliver the polish and depth of GitKraken — for free, as open source. Built on Rust (Tauri v2) for performance and Svelte 5 for a responsive UI, Gitron is designed from the ground up with a plugin architecture and first-class AI agent integration.

> **Status:** Early development (v0.1.0). Core git operations work. Not yet recommended for production use.

## Why Gitron?

The Git GUI landscape has a gap:

- **GitKraken** — polished but Electron-heavy, partially paywalled, closed-source
- **GitHub Desktop** — limited features, GitHub-centric, Electron
- **GitButler** — innovative but opinionated (virtual branches), not a traditional Git GUI
- **Lazygit** — excellent but terminal-only
- **Gittyup** — small community, limited momentum

Gitron fills the gap: a **fast, full-featured, cross-platform, open-source Git GUI** with a plugin architecture and first-class AI agent integration.

## Features

### What works today

- **Commit graph** — resizable, multi-column commit list with keyboard navigation
- **Staging panel** — interactive stage/unstage per file with bulk actions
- **Commit authoring** — message input with Cmd/Ctrl+Enter to commit
- **Inline diff viewer** — syntax-highlighted diffs powered by Shiki (Catppuccin Mocha theme)
- **Branch management** — create, checkout, and delete branches
- **Command palette** — Cmd/Ctrl+K to search repos, branches, and actions
- **Settings persistence** — recent repositories, column widths saved between sessions
- **Keyboard shortcuts** — graph navigation, staging shortcuts, command palette

### Planned

- Side-by-side diffs, hunk-level staging
- Fetch, pull, push
- Merge with conflict resolution, interactive rebase, cherry-pick
- Stash management, tags, git blame, file history
- **Plugin system** — extend Gitron with Rust backend plugins and Svelte frontend plugins
- **Agent Gateway** — MCP server exposing repo state to AI agents with permissioned access

See the full [Roadmap](#roadmap) below.

## Installation

### Download (coming soon)

Pre-built binaries for macOS, Windows, and Linux will be available on the [Releases](https://github.com/thomasunderwoodii/gitron/releases) page once the project reaches a stable milestone.

### Build from source

Building from source works on macOS, Windows, and Linux.

#### Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| [Rust](https://www.rust-lang.org/tools/install) | 1.75+ | Install via `rustup` |
| [Node.js](https://nodejs.org/) | 20+ | LTS recommended |
| npm | 9+ | Comes with Node.js |
| [Git](https://git-scm.com/) | 2.30+ | Required at runtime |

#### macOS

Install Xcode Command Line Tools (required for Rust compilation and system libraries):

```bash
xcode-select --install
```

Then build Gitron:

```bash
git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron
npm install
npm run tauri build
```

The built `.app` and `.dmg` will be in `src-tauri/target/release/bundle/`.

#### Windows

1. Install [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (select "Desktop development with C++")
2. WebView2 is required — it's pre-installed on Windows 10 (late builds) and Windows 11. If missing, install it from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

Then build Gitron:

```powershell
git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron
npm install
npm run tauri build
```

The installer (`.msi` or `.exe`) will be in `src-tauri\target\release\bundle\`.

#### Linux

Install system dependencies. The exact packages depend on your distribution:

**Debian / Ubuntu:**

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Fedora:**

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libxdo-devel librsvg2-devel
sudo dnf group install "C Development Tools and Libraries"
```

**Arch Linux:**

```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl \
  xdotool librsvg
```

Then build Gitron:

```bash
git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron
npm install
npm run tauri build
```

The built packages (`.deb`, `.AppImage`, `.rpm`) will be in `src-tauri/target/release/bundle/`.

## Development

### Running in dev mode

```bash
git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron
npm install
npm run tauri dev
```

This will:
1. Start the Vite dev server for the Svelte frontend (with hot reload)
2. Compile the Rust backend
3. Open the Tauri window pointing at the dev server
4. Watch for Rust changes and recompile automatically

### Project structure

```
gitron/
├── src-tauri/               # Rust backend (Tauri v2)
│   └── src/
│       ├── lib.rs           # Tauri builder, command registration
│       ├── commands/        # IPC handlers (thin wrappers, no git logic)
│       ├── git/             # All git logic (git2-rs)
│       ├── cache/           # In-memory repo state cache
│       └── watcher/         # File system watcher (notify-rs)
├── src/                     # Svelte 5 frontend
│   └── lib/
│       ├── api/             # Tauri IPC bindings + TypeScript types
│       ├── stores/          # Svelte stores (state management)
│       └── components/      # UI components
├── docs/                    # Project documentation
└── package.json
```

### Architecture

```
┌──────────────────────────────────┐
│         Svelte 5 Frontend        │
│  (graph, diff, staging, UI)      │
└──────────────┬───────────────────┘
               │ Tauri IPC
┌──────────────▼───────────────────┐
│          Rust Backend            │
│  ┌─────────┐  ┌──────────────┐  │
│  │ git2-rs │  │ File Watcher │  │
│  │ (core)  │  │  (notify-rs) │  │
│  └─────────┘  └──────────────┘  │
│  ┌─────────┐  ┌──────────────┐  │
│  │ git CLI │  │ Repo State   │  │
│  │ (future)│  │ Cache        │  │
│  └─────────┘  └──────────────┘  │
└──────────────────────────────────┘
```

- **git2-rs** handles all hot-path operations (graph traversal, diffs, status, staging, commits, branches) in-process for maximum performance
- **git CLI** will be used for complex operations (interactive rebase, advanced merge) in later phases
- **Tauri IPC** bridges the Rust backend and Svelte frontend with typed, serialized commands
- **Strict module boundaries** — `commands/` contains zero git logic; `git/` contains all git logic; frontend components never call `invoke()` directly (they go through stores and API layers)

### Tech stack

| Layer | Technology | Why |
|-------|-----------|-----|
| Backend | Rust + Tauri v2 | Native performance, ~5-10MB bundle, no Electron |
| Git operations | git2-rs | In-process, zero-overhead git operations |
| Frontend | Svelte 5 + SvelteKit | Compiles away, minimal runtime, great DX |
| Styling | TailwindCSS v4 | Utility-first, design tokens via CSS custom properties |
| UI components | shadcn-svelte (bits-ui) | Accessible, headless primitives |
| Syntax highlighting | Shiki | Accurate, TextMate grammar-based highlighting |

### Useful commands

```bash
# Development
npm run tauri dev         # Run in dev mode (frontend + backend)
npm run dev               # Run frontend only (Vite dev server)
npm run check             # TypeScript / Svelte type checking

# Production
npm run tauri build       # Build release binary + installer
npm run build             # Build frontend only
```

## Roadmap

Gitron is developed in phases. Each phase builds on the previous one.

| Phase | Focus | Status |
|-------|-------|--------|
| **1. Foundation** | Core git backend, commit graph, app shell | Nearly complete |
| **2. Core Workflow** | Stage, commit, push, pull, branch UI, diffs | In progress |
| **3. Advanced Git** | Merge, rebase, cherry-pick, stash, blame, tags | Planned |
| **4. Plugin System** | Backend (Rust traits) + frontend (UI slots) plugins | Planned |
| **5. Agent Gateway** | MCP server, AI agent permissions, action queue | Planned |
| **6. Polish & Release** | Theming, auto-update, code signing, installers | Planned |

See [docs/ROADMAP.md](docs/ROADMAP.md) for detailed status of every item.

## Documentation

| Document | Description |
|----------|-------------|
| [ROADMAP](docs/ROADMAP.md) | Current status and what's next |
| [ARCHITECTURE](docs/ARCHITECTURE.md) | System design, module structure, data flow |
| [TECHNICAL_SPEC](docs/TECHNICAL_SPEC.md) | IPC commands, data types, state management |
| [DEVELOPER_GUIDE](docs/DEVELOPER_GUIDE.md) | How to add features, conventions, pitfalls |
| [VISION](docs/VISION.md) | Project direction and principles |
| [PLUGIN_SYSTEM](docs/PLUGIN_SYSTEM.md) | Plugin architecture design |
| [AGENT_GATEWAY](docs/AGENT_GATEWAY.md) | AI agent gateway design |

## Contributing

Contributions are welcome! Whether it's bug reports, feature requests, documentation improvements, or code contributions — all help is appreciated.

### Getting started

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Read the [Developer Guide](docs/DEVELOPER_GUIDE.md) for conventions and module boundaries
4. Make your changes following the feature checklist: Rust types → Rust git logic → Rust command → register in lib.rs → TS types → TS API → Store → Component
5. Commit using [conventional commits](https://www.conventionalcommits.org/): `type(scope): description`
6. Open a pull request

### Commit convention

```
type(scope): description

Types: feat, fix, refactor, docs, chore, test, style
Scopes: git, ui, graph, diff, staging, branch, plugin, agent, docs
```

### Key guidelines

- Follow the strict module boundaries (see [CLAUDE.md](CLAUDE.md) or [Developer Guide](docs/DEVELOPER_GUIDE.md))
- Use Svelte 5 runes (`$state`, `$derived`, `$props`) — no Svelte 4 patterns
- No `unwrap()` in Rust production code — use `GitResult<T>` and proper error handling
- Keep command handlers thin — all git logic belongs in `src-tauri/src/git/`

## License

[PolyForm Noncommercial 1.0.0](LICENSE) — free for personal, academic, and non-commercial use. Commercial use requires a separate agreement. See the [LICENSE](LICENSE) file for full terms.

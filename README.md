<p align="center">
  <h1 align="center">Gitron</h1>
  <p align="center">
    Open-source, AI-native Git GUI built with Rust and Svelte
  </p>
  <p align="center">
    <a href="#installation">Install</a> &middot;
    <a href="#features">Features</a> &middot;
    <a href="#ai-commit-messages">AI</a> &middot;
    <a href="#server-mode">Server Mode</a> &middot;
    <a href="#development">Development</a> &middot;
    <a href="#contributing">Contributing</a>
  </p>
</p>

<p align="center">
  <img src="static/screenshot-1.png" alt="Gitron screenshot" width="800" />
</p>

---

Gitron is a fast, cross-platform Git GUI built on Rust (Tauri v2) and Svelte 5. It runs in two modes from the same codebase: as a **native desktop app** or as a **self-hosted web server** accessible from any browser.

## Features

### Commit Graph

- Visual lane-based branch graph with colored paths and merge lines
- Multi-column layout: graph, message, author, date, SHA — all resizable with persistent widths
- Column visibility toggles — show or hide individual columns
- Author filtering to exclude noisy commits (bots, CI)
- Gravatar avatars for commit authors
- Worktree indicators on commits
- Keyboard navigation (arrow keys to move, Enter to select)

### Commit Search

- Full-text search across commit messages from the command bar (`/`)
- Optional diff content search — find commits that changed specific code
- Live results with debounced highlighting
- Navigate between matches with keyboard

### Staging & Commits

- Stage and unstage individual files, directories, or all at once
- Flat file list or collapsible tree view for working changes
- Discard changes per file, per directory, or all at once via context menu
- Add files or extensions to `.gitignore` from the right-click menu
- Commit with title and optional multi-line body
- `Cmd+Enter` / `Ctrl+Enter` to commit
- Git hooks are executed on commit
- Commits are disabled during rebase and merge conflicts to prevent mistakes

### Diff Viewer

- Syntax-highlighted inline diffs powered by Shiki (Catppuccin Mocha theme)
- View diffs for unstaged changes, staged changes, or any commit's files
- Click a commit to see its changed files in the sidebar, then click a file to view the diff
- Arrow key navigation between files
- Hunk grouping with skip indicators
- File status badges (added, modified, deleted, renamed, conflicted)

### Branches

- Create, checkout, delete, merge, and rebase branches
- Protected branches — configurable list (main, master, develop by default) prevents accidental deletion
- Branch conflict detection — prompts when a local and remote branch share a name
- Find and clean up merged branches with an interactive selection dialog
- Bulk delete all non-protected local branches
- Reset to any commit (soft, mixed, or hard)
- Rebase continue/abort and merge abort from the UI
- Cherry-pick abort support

### Conflict Resolution

- Automatic detection of merge, rebase, and cherry-pick conflicts
- Conflict banner with operation progress and continue/abort controls
- Conflicted files listed in their own sidebar section
- Line-level ours/theirs selection in the diff viewer
- Quick "accept ours" / "accept theirs" for full sections
- Step-by-step progress tracking during multi-commit rebases

### Remotes

- List, add, and remove remotes
- Fetch from a specific remote or all at once
- Pull and push with tracking status (ahead/behind counts)
- Force push with confirmation dialog
- Delete remote branches
- Checkout remote branches as local tracking branches
- Auto-fetch at configurable intervals (off, 15s, 30s, 1m, 5m, 15m)

### Tags

- Create annotated or lightweight tags at any commit
- Delete local and remote tags
- Move tags to a different commit
- Push tags to remotes (single or force)
- View remote-only tags with context menu actions
- Tags sorted by commit position in the graph
- Click a tag to jump to its commit

### Stashes

- Save stashes with an optional message
- Apply, pop, or drop individual stashes
- Full stash list in the sidebar
- Quick stash access from the command palette

### Worktrees

- List, add, and remove linked worktrees
- Lock and unlock worktrees
- Prune stale worktree entries
- Worktree indicators shown on commits in the graph
- Sidebar visibility toggle

### Checkpoints

- Detect refs created by AI coding tools (Claude, Copilot, Cursor, etc.)
- Purge checkpoint refs and run garbage collection to reclaim disk space

### GitHub Integration

- Device flow OAuth login — no app registration required
- User profile display in the toolbar
- Browse and search your GitHub repositories
- Clone repositories directly from the GitHub repo list

### AI Commit Messages

Generate conventional commit messages from your staged diffs using LLM providers. Stage your files, click the sparkle button next to the commit title, and Gitron fills in both the title and description.

**Supported providers:**

| Provider | Models | Notes |
|----------|--------|-------|
| **OpenAI** | GPT-4.1 Nano, GPT-4.1 Mini, GPT-4o Mini, and more | Filtered to chat-capable models |
| **Anthropic** | Claude 4.5 Haiku, Claude Sonnet, etc. | Sorted by cost tier |
| **Gemini** | Gemini 2.0 Flash Lite, Gemini 2.0 Flash, and more | Filtered to text generation |
| **OpenRouter** | Auto (best available) + affordable models | Filtered to models under $2/M input tokens |

**Setup:** Open Settings (gear icon) > AI tab > choose a provider > paste your API key > select a model > activate the provider.

**Advanced options:** custom base URL for proxies or self-hosted models, configurable max output tokens (500–4,000).

**Privacy:** API keys are stored in your OS keychain (not plain text). Diffs go directly to your chosen provider — Gitron does not proxy or store your code. Only staged diffs are sent, truncated to ~8,000 characters for large changesets.

### Integrated Terminal

- Built-in PTY terminal panel powered by xterm.js
- Opens in the current repository directory
- Configurable shell, font, cursor style, and scrollback buffer
- Clickable URLs
- Open external terminal from context menu (iTerm, Warp, Alacritty, Kitty, Ghostty, Windows Terminal, and more)
- Toggle with `Ctrl+``

### Output Panel

- View stdout/stderr from git operations
- Color-coded success and error output
- Toggle with `Cmd+``

### Command Palette

Open with **Cmd+K** (macOS) or **Ctrl+K** (Windows/Linux) to quickly access:

- Open, switch, or clone repositories
- Create, checkout, delete, merge, or rebase branches
- Stage, unstage, or discard files
- Save, apply, pop, or drop stashes
- Add or remove remotes
- Fetch, pull, push
- Create or delete worktrees
- Search commits
- Clean up merged branches, purge checkpoint refs
- Open repository in terminal

### Settings

| Category | Options |
|----------|---------|
| **Appearance** | Theme (Tron, Tron Enhanced, Dark, Light, System), zoom (80–150%), high contrast mode, editor font size, monospace font family |
| **Git** | Auto-fetch interval, file watcher interval, verbose error display, protected branches list, excluded authors |
| **Terminal** | Shell, font size, font family, cursor style (block/underline/bar), scrollback buffer |
| **AI** | Provider, API key, model selection, custom base URL, max output tokens |
| **GitHub** | Authentication status, OAuth token |

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+K` / `Ctrl+K` | Open command palette |
| `Cmd+R` | Refresh repository |
| `/` | Search commits |
| `?` | Show keyboard shortcuts |
| `Cmd+,` | Open settings |
| `Cmd+`` ` | Toggle output panel |
| `` Ctrl+` `` | Toggle terminal |
| `Escape` | Close panel / clear selection |
| `Up / Down` | Navigate commits or files |
| `S` | Stage selected file |
| `U` | Unstage selected file |
| `Cmd+Shift+A` | Stage all |
| `Cmd+Shift+U` | Unstage all |
| `Cmd+Shift+D` | Discard all changes |
| `Cmd+Enter` | Create commit |
| `Cmd+Shift+P` | Push |
| `Cmd+Shift+L` | Pull |
| `Cmd+Shift+F` | Fetch |

## Server Mode

Run Gitron as a self-hosted web server for headless machines, remote development, or browser-based access.

### Quick Start with Docker

```bash
docker run -d \
  -p 9417:9417 \
  -v /path/to/your/repos:/repos \
  --name gitron \
  gitron-server
```

Then open `http://localhost:9417` in your browser.

### Build the Docker Image

```bash
git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron
docker build -t gitron-server .
```

### Run the Server Binary Directly

```bash
# Build
cargo build -p gitron-server --release
pnpm build

# Run (serves the frontend and API on port 9417)
./target/release/gitron-server --frontend-dir build
```

### Server Options

```
gitron-server [OPTIONS]

Options:
  -p, --port <PORT>                Port to listen on [default: 9417]
      --host <HOST>                Host to bind to [default: 127.0.0.1]
      --token <TOKEN>              Authentication token (required when host != localhost)
      --frontend-dir <PATH>        Path to built frontend files
      --repo <PATH>                Auto-open a repository on startup
```

### Authentication

When binding to a non-localhost address, the server requires a bearer token:

```bash
gitron-server --host 0.0.0.0 --token my-secret-token
```

The frontend will prompt for the token on first load.

### Credentials and Settings

In server mode, credentials and settings are stored in `~/.config/gitron/`:

- `credentials.json` — API keys for AI providers
- `ai_settings.json` — AI provider configuration
- `settings.json` — general settings

## Installation

### Download

Pre-built binaries for macOS (ARM and Intel), Windows, and Linux are available on the [Releases](https://github.com/thomasunderwoodii/gitron/releases) page.

### Build from Source

#### Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| [Rust](https://www.rust-lang.org/tools/install) | 1.75+ | Install via `rustup` |
| [Node.js](https://nodejs.org/) | 20+ | LTS recommended |
| [pnpm](https://pnpm.io/) | 9+ | Install via `npm install -g pnpm` |
| [Git](https://git-scm.com/) | 2.30+ | Required at runtime |

#### macOS

```bash
xcode-select --install

git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron
pnpm install
pnpm tauri build
```

The `.app` and `.dmg` will be in `src-tauri/target/release/bundle/`.

#### Windows

1. Install [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (select "Desktop development with C++")
2. WebView2 is required — pre-installed on Windows 10 (late builds) and Windows 11

```powershell
git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron
pnpm install
pnpm tauri build
```

The installer will be in `src-tauri\target\release\bundle\`.

#### Linux

Install system dependencies for your distribution:

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

Then:
```bash
git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron
pnpm install
pnpm tauri build
```

The built packages (`.deb`, `.AppImage`, `.rpm`) will be in `src-tauri/target/release/bundle/`.

## Development

### Running in Dev Mode

**Desktop (Tauri):**

```bash
git clone https://github.com/thomasunderwoodii/gitron.git
cd gitron
pnpm install
pnpm dev
```

**Server (Axum):**

```bash
pnpm server:dev
```

Serves everything at `http://localhost:9417`.

### Architecture

```
┌──────────────────────────────────┐
│         Svelte 5 Frontend        │
│  (graph, diff, staging, UI)      │
└──────┬───────────────┬───────────┘
       │ Tauri IPC     │ HTTP/SSE
┌──────▼──────┐  ┌─────▼──────────┐
│  Tauri App  │  │  Axum Server   │
│  (desktop)  │  │  (self-hosted) │
└──────┬──────┘  └─────┬──────────┘
       └───────┬───────┘
┌──────────────▼───────────────────┐
│        gitron-core (shared)      │
│  ┌─────────┐  ┌──────────────┐  │
│  │ git2-rs │  │ File Watcher │  │
│  │         │  │  (notify-rs) │  │
│  └─────────┘  └──────────────┘  │
│  ┌─────────┐  ┌──────────────┐  │
│  │ git CLI │  │ Repo State   │  │
│  │         │  │ Cache        │  │
│  └─────────┘  └──────────────┘  │
└──────────────────────────────────┘
```

- **gitron-core** contains all shared logic — both the Tauri app and the Axum server are thin wrappers
- **git2-rs** handles hot-path operations (graph, diffs, status, staging, commits, branches) in-process
- **git CLI** is used for complex operations (rebase, merge, cherry-pick)
- **Transport abstraction** — the frontend auto-detects Tauri IPC or HTTP and uses the same API surface

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Shared core | Rust (gitron-core) |
| Desktop | Tauri v2 |
| Web server | Axum |
| Git operations | git2-rs |
| Frontend | Svelte 5 + SvelteKit |
| Styling | TailwindCSS v4 |
| UI components | shadcn-svelte (bits-ui) |
| Syntax highlighting | Shiki |

### Project Structure

```
gitron/
├── crates/
│   ├── gitron-core/         # Shared Rust logic (git, AI, GitHub, cache, watcher)
│   │   └── src/
│   │       ├── git/         # All git operations (git2-rs)
│   │       ├── ai/          # AI commit message generation
│   │       ├── github/      # GitHub OAuth + API
│   │       ├── cache/       # Repo state cache
│   │       └── watcher/     # File system watcher (notify-rs)
│   └── gitron-server/       # Axum web server
│       └── src/
│           ├── main.rs      # CLI, server setup
│           └── routes/      # HTTP handlers (mirrors Tauri commands)
├── src-tauri/               # Tauri desktop app
│   └── src/
│       ├── lib.rs           # Tauri builder, command registration
│       └── commands/        # IPC handlers
├── src/                     # Svelte 5 frontend (shared between both modes)
│   └── lib/
│       ├── api/             # Transport abstraction (Tauri IPC or HTTP)
│       ├── stores/          # Svelte stores (state management)
│       └── components/      # UI components
├── Dockerfile               # Multi-stage build for server mode
└── docs/                    # Project documentation
```

### Commands

```bash
pnpm dev                  # Desktop mode (Tauri + Vite dev server)
pnpm server:dev           # Server mode (build frontend + Axum on :9417)
pnpm check                # TypeScript / Svelte type checking
pnpm tauri build          # Build desktop app + installer
pnpm build                # Build frontend only
pnpm build:server         # Build server binary only
```

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE](docs/ARCHITECTURE.md) | System design, module structure, data flow |
| [TECHNICAL_SPEC](docs/TECHNICAL_SPEC.md) | IPC commands, data types, state management |
| [DEVELOPER_GUIDE](docs/DEVELOPER_GUIDE.md) | How to add features, conventions, pitfalls |
| [VISION](docs/VISION.md) | Project direction and principles |

## Contributing

Contributions are welcome — bug reports, feature requests, documentation improvements, and code contributions.

### Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-feature`
3. Read the [Developer Guide](docs/DEVELOPER_GUIDE.md) for conventions and module boundaries
4. Follow the feature pipeline: Rust types → Rust logic → Tauri command + Server route → TS types → TS API → Store → Component
5. Commit using [conventional commits](https://www.conventionalcommits.org/): `type(scope): description`
6. Open a pull request

### Key Guidelines

- **Dual-mode parity** — every feature must work in both desktop (Tauri) and server (Axum) modes
- Follow strict module boundaries — all git logic in `gitron-core`, command handlers are thin wrappers
- Svelte 5 runes only (`$state`, `$derived`, `$props`)
- No `unwrap()` in Rust production code
- Use `getTransport()` for all backend calls in the frontend

## License

[PolyForm Noncommercial 1.0.0](LICENSE) — free for personal, academic, and non-commercial use. Commercial use requires a separate agreement. See the [LICENSE](LICENSE) file for full terms.

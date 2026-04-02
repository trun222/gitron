# Open Terminal Feature - Implementation Plan

## Overview
Add the ability to open a terminal from the command palette, via a keyboard shortcut (`Ctrl+``), using the system default terminal shell with existing configuration options.

## Current State
- **Tauri command** `open_in_terminal` already exists in `src-tauri/src/commands/worktree.rs:58-202` (cross-platform: macOS, Linux, Windows)
- **Settings** for `terminalApp` already exist across `types.ts`, `api/settings.ts`, `stores/settings.ts`
- **Settings UI** for choosing a terminal app already exists in `GitSettings.svelte:85-120`
- **Usage** is limited to worktree context menus only (`WorktreeSection.svelte:105-108`)
- No Rust changes needed. This is frontend-only.

## Changes Required

### 1. `src/lib/stores/repo.ts` — Add shared `openInTerminal()` helper

**Add imports** (line 3, after existing imports):
```ts
import { isTauri } from '$lib/api';
import { trackRepoOpen, excludedAuthors, terminalApp } from '$lib/stores/settings';
```
(Note: `terminalApp` added to existing settings import)

**Add function** at end of file (after line 941):
```ts
export async function openInTerminal(path?: string) {
  if (!isTauri()) return;
  const targetPath = path ?? get(repoPath);
  if (!targetPath) return;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('open_in_terminal', {
      path: targetPath,
      terminalApp: get(terminalApp) || null,
    });
  } catch (e) {
    addToast(`Failed to open terminal: ${e}`, 'error');
  }
}
```

### 2. `src/lib/components/ui/command/CommandBar.svelte` — Add command item

**Add import** (line 17, add `repoPath` and `openInTerminal` to the repo store import):
```ts
import {
  openRepo, hasRepo, localBranches, remoteBranches, currentBranch,
  checkoutBranch, createAndCheckoutBranch,
  remotes, networkOperation,
  fetchFromRemote, pushToRemote, pullFromRemote,
  addRemote, removeRemote,
  discardConfirmOpen,
  forcePushConfirmOpen,
  commitSearchActive, commitSearchQuery, commitSearchLoading, commitSearchDiffs,
  searchCommitsAction, clearCommitSearch,
  repoPath, openInTerminal,
} from '$lib/stores/repo';
```

**Add Command.Item** after the "Discard All Changes" item (after line 550, before the closing `</Command.GroupItems>`):
```svelte
{#if isTauri()}
  <Command.Item
    value="open-terminal"
    keywords={['terminal', 'shell', 'console', 'command line', 'bash', 'zsh', 'cmd']}
    onSelect={() => { openInTerminal(); close(); }}
    class="flex items-center gap-2 rounded-md px-2 py-1.5 text-sm cursor-pointer outline-none data-[selected]:bg-accent"
  >
    <svg class="shrink-0 text-muted-foreground" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="4 17 10 11 4 5"></polyline><line x1="12" y1="19" x2="20" y2="19"></line></svg>
    <span>Open Terminal</span>
    <kbd class="ml-auto text-xs text-muted-foreground border border-border rounded px-1 py-0.5 font-mono">Ctrl+`</kbd>
  </Command.Item>
{/if}
```

### 3. `src/lib/components/layout/AppShell.svelte` — Add keyboard shortcut

**Add imports** (line 12-13, add `openInTerminal` to repo import, add `isTauri`):
```ts
import {
  error, repoPath, hasRepo,
  pullFromRemote, pushToRemote, fetchFromRemote,
  stageAllAndClear, unstageAllAndClear, discardAllChanges,
  refreshAll, openInTerminal,
} from '$lib/stores/repo';
import { toggleOutputPanel } from '$lib/stores/output';
import { isTauri } from '$lib/api';
```

**Replace** the existing `Cmd+`` handler (lines 96-101):
```ts
// Cmd+` — Toggle output panel (metaKey only, not ctrlKey)
if (e.metaKey && !e.ctrlKey && e.key === '`') {
  e.preventDefault();
  toggleOutputPanel();
  return;
}

// Ctrl+` — Open terminal (Tauri desktop only)
if (e.ctrlKey && !e.metaKey && e.key === '`') {
  e.preventDefault();
  openInTerminal();
  return;
}
```

### 4. `src/lib/components/ui/shortcuts/ShortcutsModal.svelte` — Add display entry

**Add** after line 10 (`Toggle output panel` entry):
```ts
{ keys: ['Ctrl', '`'], description: 'Open terminal' },
```

### 5. `src/lib/components/layout/WorktreeSection.svelte` — Refactor to use shared helper

**Replace import** (line 14):
```ts
import { repoPath, openInTerminal } from '$lib/stores/repo';
```

**Replace function** (lines 105-108):
```ts
async function openWorktreeInTerminal(wtPath: string) {
  await openInTerminal(wtPath);
}
```

## Files Modified Summary

| File | Change |
|------|--------|
| `src/lib/stores/repo.ts` | Add `openInTerminal()` function + imports |
| `src/lib/components/ui/command/CommandBar.svelte` | Add "Open Terminal" command item |
| `src/lib/components/layout/AppShell.svelte` | Add `Ctrl+`` shortcut, refine `Cmd+`` detection |
| `src/lib/components/ui/shortcuts/ShortcutsModal.svelte` | Add shortcut display entry |
| `src/lib/components/layout/WorktreeSection.svelte` | Refactor to use shared helper |

## No Rust/Server Changes Needed
- The existing `open_in_terminal` Tauri command handles all platforms
- Opening a terminal is desktop-only — hidden in web mode via `isTauri()` guards

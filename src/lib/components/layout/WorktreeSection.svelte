<script lang="ts">
  import {
    worktrees,
    linkedWorktrees,
    worktreeLoading,
    worktreeError,
    showAddWorktreeDialog,
    refreshWorktrees,
    createWorktree,
    deleteWorktree,
    toggleWorktreeLock,
    pruneStaleWorktrees,
  } from '$lib/stores/worktree';
  import { repoPath } from '$lib/stores/repo';
  import { terminalApp, worktreesExpanded, setWorktreesExpanded } from '$lib/stores/settings';
  import { isTauri } from '$lib/api';
  import type { WorktreeInfo } from '$lib/api/types';


  // Add dialog state
  let addPath = $state('');
  let addBranchMode = $state<'existing' | 'new' | 'detached'>('new');
  let addBranchName = $state('');
  let addCreating = $state(false);
  let addError = $state<string | null>(null);

  // Remove confirmation
  let removeConfirm = $state<{ path: string; name: string; force: boolean } | null>(null);

  // Context menu
  interface ContextMenuItem {
    id: string;
    label: string;
    danger?: boolean;
  }
  interface ContextMenuState {
    x: number;
    y: number;
    items: (ContextMenuItem | 'separator')[];
    worktree: WorktreeInfo;
  }
  let contextMenu: ContextMenuState | null = $state(null);

  function adjustMenuPosition(node: HTMLElement) {
    const rect = node.getBoundingClientRect();
    if (rect.bottom > window.innerHeight) {
      node.style.top = `${Math.max(4, rect.top - rect.height)}px`;
    }
    if (rect.right > window.innerWidth) {
      node.style.left = `${Math.max(4, window.innerWidth - rect.width)}px`;
    }
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function handleContextMenu(e: MouseEvent, wt: WorktreeInfo) {
    e.preventDefault();
    e.stopPropagation();

    const items: (ContextMenuItem | 'separator')[] = [];

    if (!wt.is_main) {
      items.push({ id: 'open-window', label: 'Open in New Window' });
      if (isTauri()) {
        items.push({ id: 'open-terminal', label: 'Open in Terminal' });
      }
      items.push('separator');
    }

    items.push({ id: 'copy-path', label: 'Copy Path' });

    if (!wt.is_main) {
      items.push('separator');
      items.push({
        id: 'toggle-lock',
        label: wt.is_locked ? 'Unlock Worktree' : 'Lock Worktree',
      });
      items.push('separator');
      items.push({ id: 'remove', label: 'Remove Worktree', danger: true });
      items.push({ id: 'force-remove', label: 'Force Remove', danger: true });
    }

    contextMenu = { x: e.clientX, y: e.clientY, items, worktree: wt };
  }

  async function openWorktreeInNewWindow(wtPath: string) {
    if (isTauri()) {
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');
      const label = `worktree-${Date.now()}`;
      new WebviewWindow(label, {
        url: `/?repo=${encodeURIComponent(wtPath)}`,
        title: `Gitron — ${wtPath.split('/').pop() ?? 'Worktree'}`,
        width: 1280,
        height: 800,
        minWidth: 900,
        minHeight: 600,
      });
    } else {
      window.open(`/?repo=${encodeURIComponent(wtPath)}`, '_blank');
    }
  }

  async function openWorktreeInTerminal(wtPath: string) {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('open_in_terminal', { path: wtPath, terminalApp: $terminalApp || null });
  }

  async function executeMenuAction(actionId: string) {
    const wt = contextMenu?.worktree;
    closeContextMenu();
    if (!wt) return;

    if (actionId === 'open-window') {
      await openWorktreeInNewWindow(wt.path);
    } else if (actionId === 'open-terminal') {
      await openWorktreeInTerminal(wt.path);
    } else if (actionId === 'copy-path') {
      await navigator.clipboard.writeText(wt.path);
    } else if (actionId === 'toggle-lock') {
      await toggleWorktreeLock(wt.path, wt.is_locked);
    } else if (actionId === 'remove') {
      removeConfirm = { path: wt.path, name: wt.name, force: false };
    } else if (actionId === 'force-remove') {
      removeConfirm = { path: wt.path, name: wt.name, force: true };
    }
  }

  async function confirmRemove() {
    if (!removeConfirm) return;
    await deleteWorktree(removeConfirm.path, removeConfirm.force);
    removeConfirm = null;
  }

  function openAddDialog() {
    const rp = $repoPath ?? '';
    const parentDir = rp.split('/').slice(0, -1).join('/');
    const repoName = rp.split('/').pop() ?? 'repo';
    addPath = `${parentDir}/${repoName}-worktree`;
    addBranchMode = 'new';
    addBranchName = '';
    addError = null;
    addCreating = false;
    showAddWorktreeDialog.set(true);
  }

  function closeAddDialog() {
    showAddWorktreeDialog.set(false);
  }

  // React to external open requests (e.g. from command palette)
  $effect(() => {
    if ($showAddWorktreeDialog && !addPath) {
      const rp = $repoPath ?? '';
      const parentDir = rp.split('/').slice(0, -1).join('/');
      const repoName = rp.split('/').pop() ?? 'repo';
      addPath = `${parentDir}/${repoName}-worktree`;
      addBranchMode = 'new';
      addBranchName = '';
      addError = null;
      addCreating = false;
    }
  });

  async function handleAdd() {
    if (!addPath.trim()) {
      addError = 'Path is required';
      return;
    }

    addCreating = true;
    addError = null;

    const branch = addBranchMode === 'detached' ? null : addBranchName.trim() || null;
    const newBranch = addBranchMode === 'new';

    if ((addBranchMode === 'new' || addBranchMode === 'existing') && !branch) {
      addError = 'Branch name is required';
      addCreating = false;
      return;
    }

    const success = await createWorktree(addPath.trim(), branch, newBranch);
    addCreating = false;

    if (success) {
      closeAddDialog();
    } else {
      addError = $worktreeError;
    }
  }

  // Refresh on mount when repo is open
  $effect(() => {
    if ($repoPath) {
      refreshWorktrees();
    }
  });
</script>

<!-- Worktrees Section -->
{#if $worktrees.length > 0}
  <div class="border-t border-border">
    <div class="flex items-center justify-between w-full px-3 py-1.5">
      <button
        class="flex items-center gap-1 cursor-pointer hover:bg-accent/50 transition-colors rounded px-1 -ml-1"
        onclick={() => setWorktreesExpanded(!$worktreesExpanded)}
        aria-expanded={$worktreesExpanded}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="text-muted-foreground transition-transform duration-150"
          style="transform: rotate({$worktreesExpanded ? '90deg' : '0deg'})"
        >
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
        <span class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
          Worktrees ({$worktrees.length})
        </span>
      </button>
      <button
        class="w-5 h-5 flex items-center justify-center rounded text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
        onclick={openAddDialog}
        aria-label="Add worktree"
        title="Add worktree"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
      </button>
    </div>
    {#if $worktreesExpanded}
      <ul class="list-none overflow-y-auto max-h-[30vh]">
        {#each $worktrees as wt (wt.path)}
          <li>
            <button
              class="flex items-center gap-2 w-full px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors text-left"
              oncontextmenu={(e) => handleContextMenu(e, wt)}
              title={wt.path}
            >
              <!-- Icon: star for main, diamond for linked -->
              {#if wt.is_main}
                <svg class="w-3 h-3 shrink-0 text-primary" viewBox="0 0 16 16" fill="currentColor"><path d="M8 .25a.75.75 0 0 1 .673.418l1.882 3.815 4.21.612a.75.75 0 0 1 .416 1.279l-3.046 2.97.719 4.192a.75.75 0 0 1-1.088.791L8 12.347l-3.766 1.98a.75.75 0 0 1-1.088-.79l.72-4.194L.818 6.374a.75.75 0 0 1 .416-1.28l4.21-.611L7.327.668A.75.75 0 0 1 8 .25Z"/></svg>
              {:else}
                <svg class="w-3 h-3 shrink-0 text-muted-foreground" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1.5L12 8h-2l4 5H9v2H7v-2H2l4-5H4z"/></svg>
              {/if}
              <div class="flex flex-col flex-1 min-w-0">
                <div class="flex items-center gap-1.5">
                  <span class="truncate text-foreground font-medium">{wt.name}</span>
                  {#if wt.is_locked}
                    <span title={wt.lock_reason ?? 'Locked'}>
                      <svg class="w-3 h-3 shrink-0 text-warning" viewBox="0 0 16 16" fill="currentColor"><path d="M4 4a4 4 0 0 1 8 0v2h.25c.966 0 1.75.784 1.75 1.75v5.5A1.75 1.75 0 0 1 12.25 15h-8.5A1.75 1.75 0 0 1 2 13.25v-5.5C2 6.784 2.784 6 3.75 6H4Zm8.25 3.5h-8.5a.25.25 0 0 0-.25.25v5.5c0 .138.112.25.25.25h8.5a.25.25 0 0 0 .25-.25v-5.5a.25.25 0 0 0-.25-.25ZM10.5 6V4a2.5 2.5 0 1 0-5 0v2Z"/></svg>
                    </span>
                  {/if}
                  {#if !wt.is_valid}
                    <span title="Path does not exist">
                      <svg class="w-3 h-3 shrink-0 text-destructive" viewBox="0 0 16 16" fill="currentColor"><path d="M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575ZM8 5a.75.75 0 0 0-.75.75v2.5a.75.75 0 0 0 1.5 0v-2.5A.75.75 0 0 0 8 5Zm1 6a1 1 0 1 0-2 0 1 1 0 0 0 2 0Z"/></svg>
                    </span>
                  {/if}
                </div>
                <span class="text-[10px] text-muted-foreground truncate">
                  {wt.branch ?? 'detached'}{wt.head_short_oid ? ` · ${wt.head_short_oid}` : ''}
                </span>
              </div>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<!-- Add Worktree Dialog -->
{#if $showAddWorktreeDialog}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40 bg-black/50" onclick={closeAddDialog}></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed z-50 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[380px] rounded-lg border border-border bg-popover shadow-xl p-4"
    onclick={(e) => e.stopPropagation()}
  >
    <h3 class="text-sm font-semibold text-foreground mb-3">Add Worktree</h3>

    <!-- Path -->
    <label class="block text-[11px] font-medium text-muted-foreground mb-1">Path</label>
    <input
      type="text"
      class="w-full bg-input text-foreground text-xs rounded-md border border-border px-2 py-1.5 mb-3 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
      placeholder="/path/to/worktree"
      bind:value={addPath}
    />

    <!-- Branch mode -->
    <label class="block text-[11px] font-medium text-muted-foreground mb-1.5">Branch</label>
    <div class="flex flex-col gap-2 mb-3">
      <label class="flex items-center gap-2 text-xs text-foreground cursor-pointer">
        <input type="radio" bind:group={addBranchMode} value="new" class="accent-primary" />
        Create new branch
      </label>
      {#if addBranchMode === 'new'}
        <input
          type="text"
          class="w-full bg-input text-foreground text-xs rounded-md border border-border px-2 py-1.5 ml-5 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
          placeholder="feat/my-feature"
          bind:value={addBranchName}
        />
      {/if}

      <label class="flex items-center gap-2 text-xs text-foreground cursor-pointer">
        <input type="radio" bind:group={addBranchMode} value="existing" class="accent-primary" />
        Existing branch
      </label>
      {#if addBranchMode === 'existing'}
        <input
          type="text"
          class="w-full bg-input text-foreground text-xs rounded-md border border-border px-2 py-1.5 ml-5 placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary"
          placeholder="branch-name"
          bind:value={addBranchName}
        />
      {/if}

      <label class="flex items-center gap-2 text-xs text-foreground cursor-pointer">
        <input type="radio" bind:group={addBranchMode} value="detached" class="accent-primary" />
        Detached HEAD
      </label>
    </div>

    {#if addError}
      <p class="text-[11px] text-destructive mb-2">{addError}</p>
    {/if}

    <!-- Actions -->
    <div class="flex justify-end gap-2">
      <button
        class="text-xs px-3 py-1.5 rounded-md border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
        onclick={closeAddDialog}
      >
        Cancel
      </button>
      <button
        class="text-xs px-3 py-1.5 rounded-md transition-colors cursor-pointer {addCreating ? 'bg-muted text-muted-foreground cursor-not-allowed' : 'bg-primary text-primary-foreground hover:bg-primary/90'}"
        onclick={handleAdd}
        disabled={addCreating}
      >
        {addCreating ? 'Creating...' : 'Create'}
      </button>
    </div>
  </div>
{/if}

<!-- Remove Confirmation Dialog -->
{#if removeConfirm}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40 bg-black/50" onclick={() => removeConfirm = null}></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed z-50 top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[340px] rounded-lg border border-border bg-popover shadow-xl p-4"
    onclick={(e) => e.stopPropagation()}
  >
    <h3 class="text-sm font-semibold text-foreground mb-2">Remove Worktree</h3>
    <p class="text-xs text-muted-foreground mb-1">
      {removeConfirm.force ? 'Force remove' : 'Remove'} worktree <strong class="text-foreground">{removeConfirm.name}</strong>?
    </p>
    <p class="text-xs text-muted-foreground mb-3">
      {removeConfirm.path}
    </p>
    {#if removeConfirm.force}
      <p class="text-[11px] text-destructive mb-3">This will discard any uncommitted changes in the worktree.</p>
    {/if}
    <div class="flex justify-end gap-2">
      <button
        class="text-xs px-3 py-1.5 rounded-md border border-border text-muted-foreground hover:text-foreground hover:bg-accent transition-colors cursor-pointer"
        onclick={() => removeConfirm = null}
      >
        Cancel
      </button>
      <button
        class="text-xs px-3 py-1.5 rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors cursor-pointer"
        onclick={confirmRemove}
      >
        {removeConfirm.force ? 'Force Remove' : 'Remove'}
      </button>
    </div>
  </div>
{/if}

<!-- Context menu overlay -->
{#if contextMenu}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fixed inset-0 z-40" onclick={closeContextMenu} oncontextmenu={(e) => { e.preventDefault(); closeContextMenu(); }}></div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed z-50 min-w-[180px] rounded-lg border border-border bg-popover shadow-lg py-1"
    style="left: {contextMenu.x}px; top: {contextMenu.y}px"
    use:adjustMenuPosition
    onclick={(e) => e.stopPropagation()}
  >
    {#each contextMenu.items as item}
      {#if item === 'separator'}
        <div class="my-1 h-px bg-border"></div>
      {:else}
        <button
          type="button"
          class="w-full text-left px-3 py-1.5 text-sm outline-none transition-colors cursor-pointer
            {item.danger ? 'text-destructive hover:bg-destructive/10' : 'text-popover-foreground hover:bg-accent'}"
          onclick={() => executeMenuAction(item.id)}
        >
          {item.label}
        </button>
      {/if}
    {/each}
  </div>
{/if}

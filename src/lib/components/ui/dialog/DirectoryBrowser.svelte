<script lang="ts">
  import { Dialog } from 'bits-ui';
  import { onMount, onDestroy } from 'svelte';
  import { isTauri } from '$lib/api';
  import { base } from '$app/paths';

  interface DirEntry {
    name: string;
    path: string;
    isDir: boolean;
    isGitRepo: boolean;
  }

  let isOpen = $state(false);
  let entries = $state<DirEntry[]>([]);
  let currentPath = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let resolver: ((value: string | null) => void) | null = null;

  async function loadDirectory(path?: string) {
    loading = true;
    error = null;
    try {
      const response = await fetch(`${base}/api/fs/list`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: path || null }),
      });
      if (!response.ok) {
        throw new Error(await response.text());
      }
      entries = await response.json();
      if (path) {
        currentPath = path;
      } else if (entries.length > 0) {
        // Derive current path from parent entry
        const parentEntry = entries.find((e) => e.name === '..');
        if (parentEntry) {
          // currentPath is the child of parent
          currentPath = parentEntry.path;
          // Actually, the parent path's child is our directory - just use the first non-parent entry
          const firstChild = entries.find((e) => e.name !== '..');
          if (firstChild) {
            const parts = firstChild.path.split('/');
            parts.pop();
            currentPath = parts.join('/') || '/';
          }
        }
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function handleSelect(entry: DirEntry) {
    if (entry.isDir) {
      loadDirectory(entry.path);
    }
  }

  function handlePick(path: string) {
    isOpen = false;
    resolver?.(path);
    resolver = null;
  }

  function handleCancel() {
    isOpen = false;
    resolver?.(null);
    resolver = null;
  }

  function handlePickDirectoryEvent(e: Event) {
    const detail = (e as CustomEvent).detail;
    resolver = detail.resolve;
    isOpen = true;
    loadDirectory();
  }

  onMount(() => {
    if (!isTauri()) {
      window.addEventListener('gitron:pick-directory', handlePickDirectoryEvent);
    }
  });

  onDestroy(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('gitron:pick-directory', handlePickDirectoryEvent);
    }
  });
</script>

{#if !isTauri()}
  <Dialog.Root
    open={isOpen}
    onOpenChange={(open) => { if (!open) handleCancel(); }}
  >
    <Dialog.Portal>
      <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
      <Dialog.Content
        class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-lg rounded-lg border border-border bg-card shadow-lg flex flex-col max-h-[70vh]"
      >
        <div class="px-5 pt-5 pb-3">
          <Dialog.Title class="text-sm font-semibold text-foreground">
            Browse Directories
          </Dialog.Title>
          {#if currentPath}
            <p class="text-xs text-muted-foreground mt-1 truncate">{currentPath}</p>
          {/if}
        </div>

        <div class="flex-1 overflow-y-auto min-h-0 border-y border-border">
          {#if loading}
            <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
              <svg class="animate-spin mr-2 h-4 w-4" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                <path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="3" stroke-linecap="round" class="opacity-75" />
              </svg>
              Loading...
            </div>
          {:else if error}
            <div class="flex items-center justify-center py-8 text-sm text-destructive px-4">
              {error}
            </div>
          {:else}
            <ul class="list-none">
              {#each entries as entry (entry.path)}
                <li>
                  {#if entry.isGitRepo}
                    <div
                      class="w-full px-4 py-2 hover:bg-accent transition-colors cursor-pointer flex items-center gap-2 text-sm"
                      role="button"
                      tabindex="0"
                      onclick={() => handleSelect(entry)}
                      onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') handleSelect(entry); }}
                    >
                      <svg class="shrink-0 text-primary" viewBox="0 0 16 16" width="14" height="14">
                        <path fill="currentColor" d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8ZM5 12.25a.25.25 0 0 1 .25-.25h3.5a.25.25 0 0 1 .25.25v3.25a.25.25 0 0 1-.4.2l-1.45-1.087a.25.25 0 0 0-.3 0L5.4 15.7a.25.25 0 0 1-.4-.2Z" />
                      </svg>
                      <span class="font-medium text-primary">{entry.name}</span>
                      <button
                        class="ml-auto px-2 py-0.5 text-xs rounded border border-primary text-primary hover:bg-primary hover:text-primary-foreground transition-colors"
                        onclick={(e: MouseEvent) => { e.stopPropagation(); handlePick(entry.path); }}
                      >
                        Select
                      </button>
                    </div>
                  {:else}
                    <button
                      class="w-full text-left px-4 py-2 hover:bg-accent transition-colors cursor-pointer flex items-center gap-2 text-sm"
                      onclick={() => handleSelect(entry)}
                    >
                      {#if entry.name === '..'}
                        <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                          <path fill="currentColor" d="M8 4a.75.75 0 0 1 .75.75v6.69l2.72-2.72a.75.75 0 1 1 1.06 1.06l-4 4a.75.75 0 0 1-1.06 0l-4-4a.75.75 0 0 1 1.06-1.06l2.72 2.72V4.75A.75.75 0 0 1 8 4Z" />
                        </svg>
                        <span class="text-muted-foreground">..</span>
                      {:else}
                        <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="14" height="14">
                          <path fill="currentColor" d="M.513 1.513A1.75 1.75 0 0 1 1.75 1h3.5c.55 0 1.07.26 1.4.7l.9 1.2a.25.25 0 0 0 .2.1H13a1 1 0 0 1 1 1v.5H2.75a.75.75 0 0 0 0 1.5h11.978a1 1 0 0 1 .994 1.117L15 13.25A1.75 1.75 0 0 1 13.25 15H1.75A1.75 1.75 0 0 1 0 13.25V2.75c0-.464.184-.91.513-1.237Z" />
                        </svg>
                        <span>{entry.name}</span>
                      {/if}
                    </button>
                  {/if}
                </li>
              {/each}
            </ul>
          {/if}
        </div>

        <div class="px-5 py-4 flex items-center gap-2 justify-between">
          <button
            class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors cursor-pointer"
            onclick={() => handlePick(currentPath)}
            disabled={!currentPath}
          >
            Select Current Directory
          </button>
          <button
            class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors cursor-pointer"
            onclick={handleCancel}
          >
            Cancel
          </button>
        </div>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
{/if}

<script lang="ts">
  import { Dialog } from 'bits-ui';
  import type { MergedBranch } from '$lib/api/types';
  import {
    cleanupBranchesOpen,
    cleanupBranchesList,
    cleanupBranchesLoading,
    confirmCleanupBranches,
  } from '$lib/stores/repo';

  let selected = $state<Set<string>>(new Set());

  // Reset selection when the dialog opens with new data
  $effect(() => {
    if ($cleanupBranchesOpen) {
      selected = new Set($cleanupBranchesList.map((b) => b.name));
    }
  });

  let localBranches = $derived($cleanupBranchesList.filter((b) => !b.is_remote));
  let remoteBranches = $derived($cleanupBranchesList.filter((b) => b.is_remote));
  let selectedCount = $derived(selected.size);
  let allSelected = $derived(selected.size === $cleanupBranchesList.length && $cleanupBranchesList.length > 0);

  function toggleBranch(name: string) {
    const next = new Set(selected);
    if (next.has(name)) {
      next.delete(name);
    } else {
      next.add(name);
    }
    selected = next;
  }

  function toggleAll() {
    if (allSelected) {
      selected = new Set();
    } else {
      selected = new Set($cleanupBranchesList.map((b) => b.name));
    }
  }

  function handleConfirm() {
    const branches = $cleanupBranchesList.filter((b) => selected.has(b.name));
    confirmCleanupBranches(branches);
  }
</script>

<Dialog.Root
  open={$cleanupBranchesOpen}
  onOpenChange={(open) => cleanupBranchesOpen.set(open)}
>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg"
    >
      <Dialog.Title class="text-sm font-semibold text-foreground mb-1">
        Clean Up Merged Branches
      </Dialog.Title>
      <Dialog.Description class="text-sm text-muted-foreground mb-4">
        These branches are fully merged into the current HEAD and can be safely deleted.
      </Dialog.Description>

      {#if $cleanupBranchesLoading}
        <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
          <svg class="animate-spin mr-2" viewBox="0 0 16 16" width="14" height="14">
            <circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="28" stroke-dashoffset="8" stroke-linecap="round"/>
          </svg>
          Scanning for merged branches...
        </div>
      {:else if $cleanupBranchesList.length === 0}
        <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
          No merged branches found. Everything is clean!
        </div>
      {:else}
        <div class="mb-3">
          <label class="flex items-center gap-2 text-xs text-muted-foreground cursor-pointer select-none">
            <input
              type="checkbox"
              checked={allSelected}
              onchange={toggleAll}
              class="accent-primary"
            />
            Select all ({$cleanupBranchesList.length})
          </label>
        </div>

        <div class="max-h-[280px] overflow-y-auto border border-border rounded-md divide-y divide-border">
          {#if localBranches.length > 0}
            <div class="px-3 py-1.5 bg-secondary/50 text-xs font-medium text-muted-foreground sticky top-0">
              Local ({localBranches.length})
            </div>
            {#each localBranches as branch (branch.name)}
              <label class="flex items-center gap-2 px-3 py-2 text-sm cursor-pointer hover:bg-accent/50 transition-colors">
                <input
                  type="checkbox"
                  checked={selected.has(branch.name)}
                  onchange={() => toggleBranch(branch.name)}
                  class="accent-primary shrink-0"
                />
                <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="12" height="12">
                  <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
                </svg>
                <span class="truncate font-mono text-xs">{branch.name}</span>
              </label>
            {/each}
          {/if}
          {#if remoteBranches.length > 0}
            <div class="px-3 py-1.5 bg-secondary/50 text-xs font-medium text-muted-foreground sticky top-0">
              Remote ({remoteBranches.length})
            </div>
            {#each remoteBranches as branch (branch.name)}
              <label class="flex items-center gap-2 px-3 py-2 text-sm cursor-pointer hover:bg-accent/50 transition-colors">
                <input
                  type="checkbox"
                  checked={selected.has(branch.name)}
                  onchange={() => toggleBranch(branch.name)}
                  class="accent-primary shrink-0"
                />
                <svg class="shrink-0 text-muted-foreground" viewBox="0 0 16 16" width="12" height="12">
                  <path fill="currentColor" d="M2 2.5A2.5 2.5 0 0 1 4.5 0h8.75a.75.75 0 0 1 .75.75v12.5a.75.75 0 0 1-.75.75h-2.5a.75.75 0 0 1 0-1.5h1.75v-2h-8a1 1 0 0 0-.714 1.7.75.75 0 1 1-1.072 1.05A2.495 2.495 0 0 1 2 11.5Zm10.5-1h-8a1 1 0 0 0-1 1v6.708A2.486 2.486 0 0 1 4.5 9h8Z" />
                </svg>
                <span class="truncate font-mono text-xs">{branch.name}</span>
              </label>
            {/each}
          {/if}
        </div>

        {#if remoteBranches.length > 0}
          <p class="mt-2 text-xs text-destructive">
            Deleting remote branches will affect all collaborators.
          </p>
        {/if}
      {/if}

      <div class="flex items-center gap-2 justify-end mt-5">
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
          onclick={() => cleanupBranchesOpen.set(false)}
        >
          Cancel
        </button>
        {#if $cleanupBranchesList.length > 0}
          <button
            type="button"
            class="px-3 py-1.5 text-sm rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors font-medium disabled:opacity-50"
            disabled={selectedCount === 0}
            onclick={handleConfirm}
          >
            Delete {selectedCount} branch{selectedCount === 1 ? '' : 'es'}
          </button>
        {/if}
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<script lang="ts">
  import { Dialog } from 'bits-ui';
  import type { CheckpointRef } from '$lib/api/types';
  import {
    purgeCheckpointsOpen,
    purgeCheckpointsList,
    purgeCheckpointsLoading,
    confirmPurgeCheckpoints,
  } from '$lib/stores/repo';

  let selected = $state<Set<string>>(new Set());

  $effect(() => {
    if ($purgeCheckpointsOpen) {
      selected = new Set($purgeCheckpointsList.map((r) => r.refname));
    }
  });

  // Group refs by source tool
  let groupedRefs = $derived.by(() => {
    const groups = new Map<string, CheckpointRef[]>();
    for (const ref of $purgeCheckpointsList) {
      const list = groups.get(ref.source) ?? [];
      list.push(ref);
      groups.set(ref.source, list);
    }
    return groups;
  });

  let selectedCount = $derived(selected.size);
  let allSelected = $derived(selected.size === $purgeCheckpointsList.length && $purgeCheckpointsList.length > 0);

  function toggleRef(refname: string) {
    const next = new Set(selected);
    if (next.has(refname)) {
      next.delete(refname);
    } else {
      next.add(refname);
    }
    selected = next;
  }

  function toggleAll() {
    if (allSelected) {
      selected = new Set();
    } else {
      selected = new Set($purgeCheckpointsList.map((r) => r.refname));
    }
  }

  function handleConfirm() {
    const refs = $purgeCheckpointsList.filter((r) => selected.has(r.refname));
    confirmPurgeCheckpoints(refs);
  }

  function shortRef(refname: string): string {
    // Show last 2 path segments for readability
    const parts = refname.split('/');
    return parts.length > 3 ? '.../' + parts.slice(-2).join('/') : refname;
  }
</script>

<Dialog.Root
  open={$purgeCheckpointsOpen}
  onOpenChange={(open) => purgeCheckpointsOpen.set(open)}
>
  <Dialog.Portal>
    <Dialog.Overlay class="fixed inset-0 bg-black/50 z-50" />
    <Dialog.Content
      class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-50 w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-lg"
    >
      <Dialog.Title class="text-sm font-semibold text-foreground mb-1">
        Purge Checkpoint Refs
      </Dialog.Title>
      <Dialog.Description class="text-sm text-muted-foreground mb-4">
        These refs were created by AI coding tools. Deleting them removes checkpoint commits from the graph.
      </Dialog.Description>

      {#if $purgeCheckpointsLoading}
        <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
          <svg class="animate-spin mr-2" viewBox="0 0 16 16" width="14" height="14">
            <circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="28" stroke-dashoffset="8" stroke-linecap="round"/>
          </svg>
          Scanning for checkpoint refs...
        </div>
      {:else if $purgeCheckpointsList.length === 0}
        <div class="flex items-center justify-center py-8 text-sm text-muted-foreground">
          No checkpoint refs found. Graph is clean!
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
            Select all ({$purgeCheckpointsList.length} ref{$purgeCheckpointsList.length === 1 ? '' : 's'})
          </label>
        </div>

        <div class="max-h-[280px] overflow-y-auto border border-border rounded-md divide-y divide-border">
          {#each [...groupedRefs] as [source, refs] (source)}
            <div class="px-3 py-1.5 bg-secondary/50 text-xs font-medium text-muted-foreground sticky top-0">
              {source} ({refs.length})
            </div>
            {#each refs as ref (ref.refname)}
              <label class="flex items-center gap-2 px-3 py-2 text-sm cursor-pointer hover:bg-accent/50 transition-colors">
                <input
                  type="checkbox"
                  checked={selected.has(ref.refname)}
                  onchange={() => toggleRef(ref.refname)}
                  class="accent-primary shrink-0"
                />
                <span class="truncate font-mono text-xs text-muted-foreground">{shortRef(ref.refname)}</span>
                <span class="ml-auto shrink-0 text-xs text-muted-foreground font-mono">{ref.short_oid}</span>
              </label>
            {/each}
          {/each}
        </div>

        <p class="mt-2 text-xs text-destructive">
          This will permanently delete the selected refs and run garbage collection.
        </p>
      {/if}

      <div class="flex items-center gap-2 justify-end mt-5">
        <button
          type="button"
          class="px-3 py-1.5 text-sm rounded-md border border-border bg-background text-foreground hover:bg-accent transition-colors"
          onclick={() => purgeCheckpointsOpen.set(false)}
        >
          Cancel
        </button>
        {#if $purgeCheckpointsList.length > 0}
          <button
            type="button"
            class="px-3 py-1.5 text-sm rounded-md bg-destructive text-destructive-foreground hover:bg-destructive/90 transition-colors font-medium disabled:opacity-50"
            disabled={selectedCount === 0}
            onclick={handleConfirm}
          >
            Purge {selectedCount} ref{selectedCount === 1 ? '' : 's'}
          </button>
        {/if}
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

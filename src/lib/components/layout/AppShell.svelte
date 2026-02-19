<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import Toolbar from './Toolbar.svelte';
  import StatusBar from './StatusBar.svelte';
  import { error } from '$lib/stores/repo';
  import { BranchConflictDialog, DiscardChangesDialog } from '$lib/components/ui/dialog';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  let errorExpanded = $state(false);

  let errorFirstLine = $derived($error?.split('\n')[0] ?? '');
  let errorIsMultiline = $derived(($error?.split('\n').length ?? 0) > 1);
</script>

<div class="flex flex-col h-screen overflow-hidden bg-background text-foreground">
  <Toolbar />
  {#if $error}
    <div class="flex flex-col border-b border-destructive/30 bg-destructive/15 text-destructive text-sm">
      <div class="flex items-center gap-2 px-4 py-2">
        {#if errorIsMultiline}
          <button
            type="button"
            class="shrink-0 text-xs transition-transform"
            class:rotate-90={errorExpanded}
            onclick={() => errorExpanded = !errorExpanded}
            title="Toggle error details"
          >
            <svg viewBox="0 0 16 16" width="12" height="12">
              <path fill="currentColor" d="M6.22 3.22a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06L9.94 8 6.22 4.28a.75.75 0 0 1 0-1.06Z" />
            </svg>
          </button>
        {/if}
        <span class="flex-1 truncate">{errorFirstLine}</span>
        {#if errorIsMultiline && !errorExpanded}
          <button
            type="button"
            class="shrink-0 text-xs px-2 py-0.5 rounded hover:bg-destructive/20 transition-colors"
            onclick={() => errorExpanded = true}
          >
            Details
          </button>
        {/if}
        <button
          type="button"
          class="shrink-0 text-xs px-2 py-0.5 rounded hover:bg-destructive/20 transition-colors"
          onclick={() => { error.set(null); errorExpanded = false; }}
        >
          Dismiss
        </button>
      </div>
      {#if errorExpanded && errorIsMultiline}
        <pre class="px-4 pb-3 text-xs whitespace-pre-wrap font-mono overflow-auto max-h-[200px]">{$error}</pre>
      {/if}
    </div>
  {/if}
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 overflow-auto flex flex-col">
      {@render children()}
    </main>
  </div>
  <StatusBar />
</div>

<BranchConflictDialog />
<DiscardChangesDialog />

<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import Toolbar from './Toolbar.svelte';
  import StatusBar from './StatusBar.svelte';
  import { error } from '$lib/stores/repo';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<div class="flex flex-col h-screen overflow-hidden bg-background text-foreground">
  <Toolbar />
  {#if $error}
    <div class="flex items-center gap-2 px-4 py-2 bg-destructive/15 border-b border-destructive/30 text-destructive text-sm">
      <span class="flex-1">{$error}</span>
      <button
        type="button"
        class="shrink-0 text-xs px-2 py-0.5 rounded hover:bg-destructive/20 transition-colors"
        onclick={() => error.set(null)}
      >
        Dismiss
      </button>
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

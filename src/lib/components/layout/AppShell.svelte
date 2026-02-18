<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import Toolbar from './Toolbar.svelte';
  import StatusBar from './StatusBar.svelte';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();
</script>

<div class="flex flex-col h-screen overflow-hidden bg-background text-foreground">
  <Toolbar />
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />
    <main class="flex-1 overflow-auto flex flex-col">
      {@render children()}
    </main>
  </div>
  <StatusBar />
</div>

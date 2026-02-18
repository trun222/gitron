<script lang="ts">
  import AppShell from '$lib/components/layout/AppShell.svelte';
  import CommitGraph from '$lib/components/graph/CommitGraph.svelte';
  import CommitDetail from '$lib/components/graph/CommitDetail.svelte';
  import { hasRepo, selectedCommit } from '$lib/stores/repo';
</script>

<AppShell>
  {#if $hasRepo}
    <CommitGraph />
    {#if $selectedCommit}
      <CommitDetail />
    {/if}
  {:else}
    <div class="flex items-center justify-center flex-1">
      <div class="text-center">
        <h1 class="text-5xl font-extrabold text-primary mb-2 tracking-tight">Gitron</h1>
        <p class="text-lg text-muted-foreground mb-6">Open-source, AI-native Git GUI</p>
        <p class="text-sm text-muted-foreground/70">Enter a repository path in the toolbar to get started</p>
      </div>
    </div>
  {/if}
</AppShell>

<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/layout/AppShell.svelte';
  import CommitGraph from '$lib/components/graph/CommitGraph.svelte';
  import CommitDetail from '$lib/components/graph/CommitDetail.svelte';
  import FilePreview from '$lib/components/diff/FilePreview.svelte';
  import { hasRepo, selectedCommit, isFileSelected, openRepo } from '$lib/stores/repo';
  import { loadSettings, lastActiveRepo } from '$lib/stores/settings';
  import { get } from 'svelte/store';

  onMount(async () => {
    await loadSettings();
    const lastRepo = get(lastActiveRepo);
    if (lastRepo) {
      await openRepo(lastRepo);
    }
  });
</script>

<AppShell>
  {#if $hasRepo}
    {#if $isFileSelected}
      <FilePreview />
    {:else}
      <CommitGraph />
      {#if $selectedCommit}
        <CommitDetail />
      {/if}
    {/if}
  {:else}
    <div class="flex items-center justify-center flex-1">
      <div class="text-center">
        <h1 class="text-5xl font-extrabold text-primary mb-2 tracking-tight">Gitron</h1>
        <p class="text-lg text-muted-foreground mb-6">Open-source, AI-native Git GUI</p>
        <p class="text-sm text-muted-foreground/70">
          Press <kbd class="px-1.5 py-0.5 rounded bg-secondary text-secondary-foreground font-mono text-xs">Cmd+K</kbd> to search repositories or open a new one
        </p>
      </div>
    </div>
  {/if}
</AppShell>

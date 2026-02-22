<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '$lib/components/layout/AppShell.svelte';
  import CommitGraph from '$lib/components/graph/CommitGraph.svelte';
  import CommitDetail from '$lib/components/graph/CommitDetail.svelte';
  import FilePreview from '$lib/components/diff/FilePreview.svelte';
  import { hasRepo, selectedCommit, isFileSelected, isCommitFileSelected, openRepo, clearCommitFileSelection } from '$lib/stores/repo';
  import { loadSettings, lastActiveRepo } from '$lib/stores/settings';
  import { openCloneDialog } from '$lib/stores/clone';
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
    {:else if $isCommitFileSelected}
      <FilePreview onClose={clearCommitFileSelection} />
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
        <button
          class="px-4 py-2 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors cursor-pointer font-medium mb-4"
          onclick={openCloneDialog}
        >
          Clone Repository
        </button>
        <p class="text-sm text-muted-foreground/70">
          Press <kbd class="px-1.5 py-0.5 rounded bg-secondary text-secondary-foreground font-mono text-xs">Cmd+K</kbd> to search repositories or open a new one
        </p>
      </div>
    </div>
  {/if}
</AppShell>

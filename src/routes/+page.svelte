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
    <div class="welcome">
      <div class="welcome-content">
        <h1 class="welcome-title">Gitron</h1>
        <p class="welcome-subtitle">Open-source, AI-native Git GUI</p>
        <p class="welcome-hint">Enter a repository path in the toolbar to get started</p>
      </div>
    </div>
  {/if}
</AppShell>

<style>
  .welcome {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
  }

  .welcome-content {
    text-align: center;
  }

  .welcome-title {
    font-size: 48px;
    font-weight: 800;
    color: var(--text-accent);
    margin-bottom: 8px;
    letter-spacing: -1px;
  }

  .welcome-subtitle {
    font-size: 18px;
    color: var(--text-secondary);
    margin-bottom: 24px;
  }

  .welcome-hint {
    font-size: 14px;
    color: var(--text-muted);
  }
</style>

<script lang="ts">
  import { repoInfo, currentBranch, stagedCount, unstagedCount, hasRepo } from '$lib/stores/repo';
</script>

<footer class="status-bar">
  {#if $hasRepo}
    <div class="status-left">
      <span class="status-item">
        <svg class="icon" viewBox="0 0 16 16" width="12" height="12">
          <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
        </svg>
        {$currentBranch ?? 'HEAD detached'}
      </span>
    </div>
    <div class="status-right">
      {#if $stagedCount > 0}
        <span class="status-item staged">{$stagedCount} staged</span>
      {/if}
      {#if $unstagedCount > 0}
        <span class="status-item unstaged">{$unstagedCount} changed</span>
      {/if}
    </div>
  {:else}
    <div class="status-left">
      <span class="status-item muted">No repository open</span>
    </div>
  {/if}
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 24px;
    padding: 0 12px;
    background: var(--accent-color);
    color: white;
    font-size: 11px;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .status-item.muted {
    opacity: 0.7;
  }

  .icon {
    flex-shrink: 0;
  }
</style>

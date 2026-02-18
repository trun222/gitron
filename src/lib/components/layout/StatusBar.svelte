<script lang="ts">
  import { repoInfo, currentBranch, stagedCount, unstagedCount, hasRepo } from '$lib/stores/repo';
</script>

<footer class="flex items-center justify-between h-6 px-3 bg-card border-t border-border text-muted-foreground text-[11px]">
  {#if $hasRepo}
    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1">
        <svg class="shrink-0" viewBox="0 0 16 16" width="12" height="12">
          <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
        </svg>
        {$currentBranch ?? 'HEAD detached'}
      </span>
    </div>
    <div class="flex items-center gap-3">
      {#if $stagedCount > 0}
        <span>{$stagedCount} staged</span>
      {/if}
      {#if $unstagedCount > 0}
        <span>{$unstagedCount} changed</span>
      {/if}
    </div>
  {:else}
    <div class="flex items-center">
      <span class="opacity-70">No repository open</span>
    </div>
  {/if}
</footer>

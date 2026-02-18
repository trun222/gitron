<script lang="ts">
  import { repoInfo, currentBranch, hasRepo } from '$lib/stores/repo';
  import { openRepo } from '$lib/stores/repo';

  let folderPath = $state('');

  async function handleOpenRepo() {
    if (folderPath.trim()) {
      await openRepo(folderPath.trim());
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleOpenRepo();
    }
  }
</script>

<header class="flex items-center justify-between h-12 px-4 bg-card border-b border-border select-none" style="-webkit-app-region: drag;">
  <div class="flex items-center min-w-[200px]">
    <span class="font-bold text-sm tracking-wide text-primary">Gitron</span>
  </div>

  <div class="flex-1 flex justify-center" style="-webkit-app-region: no-drag;">
    <div class="flex gap-2 max-w-[500px] w-full">
      <input
        type="text"
        placeholder="Open repository path..."
        bind:value={folderPath}
        onkeydown={handleKeydown}
        class="flex-1 px-3 py-1.5 rounded-md border border-input bg-background text-foreground text-sm outline-none focus:border-primary transition-colors"
      />
      <button
        onclick={handleOpenRepo}
        class="px-4 py-1.5 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors cursor-pointer"
      >
        Open
      </button>
    </div>
  </div>

  <div class="flex items-center justify-end min-w-[200px]">
    {#if $hasRepo && $currentBranch}
      <span class="flex items-center gap-1.5 px-2.5 py-1 rounded bg-secondary text-xs font-medium text-muted-foreground">
        <svg class="shrink-0" viewBox="0 0 16 16" width="14" height="14">
          <path fill="currentColor" d="M9.5 3.25a2.25 2.25 0 1 1 3 2.122V6A2.5 2.5 0 0 1 10 8.5H6a1 1 0 0 0-1 1v1.128a2.251 2.251 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.5 0v1.836A2.5 2.5 0 0 1 6 7h4a1 1 0 0 0 1-1v-.628A2.25 2.25 0 0 1 9.5 3.25Z" />
        </svg>
        {$currentBranch}
      </span>
    {/if}
  </div>
</header>

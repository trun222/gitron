<script lang="ts">
  import {
    hasRepo,
    repoStatus,
    commitGraph,
    localBranches,
    remoteBranches,
    stagedCount,
    unstagedCount,
    repoPath,
  } from '$lib/stores/repo';

  let activeTab: 'changes' | 'branches' = $state('changes');
</script>

<aside class="w-[260px] min-w-[200px] bg-card border-r border-border flex flex-col overflow-hidden">
  {#if $hasRepo}
    <div class="flex border-b border-border">
      <button
        class="flex-1 py-2 text-xs font-medium flex items-center justify-center gap-1.5 border-b-2 transition-all cursor-pointer {activeTab === 'changes' ? 'text-primary border-primary' : 'text-muted-foreground border-transparent hover:text-foreground hover:bg-accent'}"
        onclick={() => (activeTab = 'changes')}
      >
        Changes
        {#if $stagedCount + $unstagedCount > 0}
          <span class="bg-git-modified text-git-modified-foreground text-[10px] px-1.5 rounded-full min-w-[18px] text-center">
            {$stagedCount + $unstagedCount}
          </span>
        {/if}
      </button>
      <button
        class="flex-1 py-2 text-xs font-medium flex items-center justify-center gap-1.5 border-b-2 transition-all cursor-pointer {activeTab === 'branches' ? 'text-primary border-primary' : 'text-muted-foreground border-transparent hover:text-foreground hover:bg-accent'}"
        onclick={() => (activeTab = 'branches')}
      >
        Branches
      </button>
    </div>

    <div class="flex-1 overflow-y-auto py-2">
      {#if activeTab === 'changes'}
        {#if $repoStatus}
          {#if $repoStatus.staged.length > 0}
            <div class="px-3 py-1.5 text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
              Staged ({$repoStatus.staged.length})
            </div>
            <ul class="list-none">
              {#each $repoStatus.staged as file}
                <li class="flex items-center gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors">
                  <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm text-[var(--color-git-added)] bg-[var(--color-git-added-bg)]">
                    {file.status[0]}
                  </span>
                  <span class="truncate text-foreground">{file.path}</span>
                </li>
              {/each}
            </ul>
          {/if}

          {#if $repoStatus.unstaged.length > 0}
            <div class="px-3 py-1.5 text-[11px] font-semibold text-git-modified uppercase tracking-wide">
              Unstaged ({$repoStatus.unstaged.length})
            </div>
            <ul class="list-none">
              {#each $repoStatus.unstaged as file}
                <li class="flex items-center gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors">
                  <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm text-[var(--color-git-modified)] bg-[var(--color-git-modified-bg)]">
                    {file.status[0]}
                  </span>
                  <span class="truncate text-foreground">{file.path}</span>
                </li>
              {/each}
            </ul>
          {/if}

          {#if $repoStatus.untracked.length > 0}
            <div class="px-3 py-1.5 text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
              Untracked ({$repoStatus.untracked.length})
            </div>
            <ul class="list-none">
              {#each $repoStatus.untracked as file}
                <li class="flex items-center gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors">
                  <span class="text-[10px] font-bold w-4 h-4 flex items-center justify-center rounded-sm text-muted-foreground bg-accent">
                    ?
                  </span>
                  <span class="truncate text-foreground">{file}</span>
                </li>
              {/each}
            </ul>
          {/if}

          {#if $repoStatus.staged.length === 0 && $repoStatus.unstaged.length === 0 && $repoStatus.untracked.length === 0}
            <p class="text-muted-foreground text-sm text-center p-4">Working tree clean</p>
          {/if}
        {/if}

      {:else if activeTab === 'branches'}
        <div class="px-3 py-1.5 text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">Local</div>
        <ul class="list-none">
          {#each $localBranches as branch}
            <li class="flex items-center gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors {branch.is_head ? 'text-primary font-medium' : ''}">
              {#if branch.is_head}
                <span class="text-primary font-bold">*</span>
              {/if}
              <span class="truncate">{branch.name}</span>
            </li>
          {/each}
        </ul>

        {#if $remoteBranches.length > 0}
          <div class="px-3 py-1.5 text-[11px] font-semibold text-muted-foreground uppercase tracking-wide mt-2">Remote</div>
          <ul class="list-none">
            {#each $remoteBranches as branch}
              <li class="flex items-center gap-2 px-3 py-1 text-xs cursor-pointer hover:bg-accent transition-colors opacity-70">
                <span class="truncate">{branch.name}</span>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  {:else}
    <div class="flex items-center justify-center h-full">
      <p class="text-muted-foreground text-sm text-center p-4">Open a repository to get started</p>
    </div>
  {/if}
</aside>

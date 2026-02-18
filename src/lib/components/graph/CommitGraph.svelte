<script lang="ts">
  import { commitGraph, selectedCommit, selectCommit } from '$lib/stores/repo';
  import type { Commit, Branch } from '$lib/api/types';

  const BRANCH_COLORS = [
    '#4fc3f7', '#81c784', '#ffb74d', '#e57373',
    '#ba68c8', '#4dd0e1', '#aed581', '#ff8a65',
    '#f06292', '#7986cb',
  ];

  let listEl: HTMLDivElement | undefined = $state();

  function getBranchColor(index: number): string {
    return BRANCH_COLORS[index % BRANCH_COLORS.length];
  }

  function formatDate(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 30) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  }

  function getBranchesForCommit(oid: string): Branch[] {
    if (!$commitGraph) return [];
    return $commitGraph.branches.filter((b) => b.target_oid === oid);
  }

  function isSelected(commit: Commit): boolean {
    return $selectedCommit?.oid === commit.oid;
  }

  function selectedIndex(): number {
    if (!$commitGraph || !$selectedCommit) return -1;
    return $commitGraph.commits.findIndex((c) => c.oid === $selectedCommit!.oid);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!$commitGraph || $commitGraph.commits.length === 0) return;
    const commits = $commitGraph.commits;
    const idx = selectedIndex();

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      const next = idx < commits.length - 1 ? idx + 1 : idx;
      selectCommit(commits[next]);
      scrollToIndex(next);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      const prev = idx > 0 ? idx - 1 : 0;
      selectCommit(commits[prev]);
      scrollToIndex(prev);
    }
  }

  function scrollToIndex(index: number) {
    if (!listEl) return;
    const row = listEl.children[index] as HTMLElement | undefined;
    row?.scrollIntoView({ block: 'nearest' });
  }
</script>

<div class="flex flex-col flex-1 overflow-hidden text-[13px]">
  {#if $commitGraph && $commitGraph.commits.length > 0}
    <div class="commit-row px-2 py-1.5 bg-card border-b border-border text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
      <span class="text-center">Graph</span>
      <span>Message</span>
      <span>Author</span>
      <span>Date</span>
      <span class="text-right">SHA</span>
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      class="flex-1 overflow-y-auto outline-none"
      tabindex="0"
      role="listbox"
      aria-label="Commit list"
      bind:this={listEl}
      onkeydown={handleKeydown}
    >
      {#each $commitGraph.commits as commit, i}
        {@const branches = getBranchesForCommit(commit.oid)}
        {@const isHead = commit.oid === $commitGraph?.head_oid}
        <button
          role="option"
          aria-selected={isSelected(commit)}
          class="commit-row px-2 py-1 border-b border-border/50 w-full text-left cursor-pointer transition-colors font-inherit text-inherit {isSelected(commit) ? 'bg-accent' : 'hover:bg-accent/50'} {isHead ? 'font-medium' : ''}"
          onclick={() => selectCommit(commit)}
        >
          <span class="flex items-center justify-center">
            <svg width="24" height="24" viewBox="0 0 24 24">
              <circle
                cx="12"
                cy="12"
                r="4"
                fill={getBranchColor(i % BRANCH_COLORS.length)}
                stroke={isHead ? '#fff' : 'none'}
                stroke-width={isHead ? 2 : 0}
              />
            </svg>
          </span>

          <span class="flex items-center gap-1.5 min-w-0 overflow-hidden">
            {#each branches as branch}
              <span
                class="inline-flex px-1.5 py-px rounded-sm text-[11px] font-semibold shrink-0 border {branch.is_head ? 'bg-primary text-primary-foreground border-primary' : branch.is_remote ? 'bg-transparent text-primary border-primary/50 border-dashed opacity-70' : 'bg-primary/10 text-primary border-primary'}"
              >
                {branch.name}
              </span>
            {/each}
            <span class="truncate">{commit.summary}</span>
          </span>

          <span class="text-muted-foreground truncate">{commit.author.name}</span>
          <span class="text-muted-foreground text-xs">{formatDate(commit.timestamp)}</span>
          <span class="font-mono text-[11px] text-muted-foreground text-right">{commit.short_oid}</span>
        </button>
      {/each}
    </div>
  {:else}
    <div class="flex items-center justify-center h-full text-muted-foreground">
      <p>No commits to display</p>
    </div>
  {/if}
</div>

<style>
  .commit-row {
    display: grid;
    grid-template-columns: 40px 1fr 140px 80px 70px;
    align-items: center;
    gap: 4px;
  }
</style>

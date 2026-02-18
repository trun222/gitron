<script lang="ts">
  import { commitGraph, selectedCommit, selectCommit } from '$lib/stores/repo';
  import type { Commit, Branch } from '$lib/api/types';

  const BRANCH_COLORS = [
    '#4fc3f7', '#81c784', '#ffb74d', '#e57373',
    '#ba68c8', '#4dd0e1', '#aed581', '#ff8a65',
    '#f06292', '#7986cb',
  ];

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
</script>

<div class="flex flex-col flex-1 overflow-hidden text-[13px]">
  {#if $commitGraph && $commitGraph.commits.length > 0}
    <div class="flex items-center px-2 py-1.5 bg-card border-b border-border text-[11px] font-semibold text-muted-foreground uppercase tracking-wide">
      <span class="w-8 shrink-0">Graph</span>
      <span class="flex-1">Message</span>
      <span class="w-[140px] shrink-0 px-2">Author</span>
      <span class="w-[80px] shrink-0 px-2">Date</span>
      <span class="w-[70px] shrink-0">SHA</span>
    </div>

    <div class="flex-1 overflow-y-auto">
      {#each $commitGraph.commits as commit, i}
        {@const branches = getBranchesForCommit(commit.oid)}
        {@const isHead = commit.oid === $commitGraph?.head_oid}
        <button
          class="flex items-center px-2 py-1 border-b border-border/50 w-full text-left cursor-pointer transition-colors font-inherit text-inherit {isSelected(commit) ? 'bg-accent' : 'hover:bg-accent/50'} {isHead ? 'font-medium' : ''}"
          onclick={() => selectCommit(commit)}
        >
          <span class="w-8 shrink-0 flex items-center">
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

          <span class="flex-1 flex items-center gap-1.5 min-w-0 overflow-hidden">
            {#each branches as branch}
              <span
                class="inline-flex px-1.5 py-px rounded-sm text-[11px] font-semibold shrink-0 border {branch.is_head ? 'bg-primary text-primary-foreground border-primary' : branch.is_remote ? 'bg-transparent text-primary border-primary/50 border-dashed opacity-70' : 'bg-primary/10 text-primary border-primary'}"
              >
                {branch.name}
              </span>
            {/each}
            <span class="truncate">{commit.summary}</span>
          </span>

          <span class="w-[140px] shrink-0 px-2 text-muted-foreground truncate">{commit.author.name}</span>
          <span class="w-[80px] shrink-0 px-2 text-muted-foreground text-xs">{formatDate(commit.timestamp)}</span>
          <span class="w-[70px] shrink-0 font-mono text-[11px] text-muted-foreground">{commit.short_oid}</span>
        </button>
      {/each}
    </div>
  {:else}
    <div class="flex items-center justify-center h-full text-muted-foreground">
      <p>No commits to display</p>
    </div>
  {/if}
</div>

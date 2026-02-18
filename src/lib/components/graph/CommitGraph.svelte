<script lang="ts">
  import { commitGraph, selectedCommit, selectCommit } from '$lib/stores/repo';
  import type { Commit, Branch } from '$lib/api/types';

  // Branch colors for the graph
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

<div class="commit-graph">
  {#if $commitGraph && $commitGraph.commits.length > 0}
    <div class="graph-header">
      <span class="col col-graph">Graph</span>
      <span class="col col-message">Message</span>
      <span class="col col-author">Author</span>
      <span class="col col-date">Date</span>
      <span class="col col-sha">SHA</span>
    </div>

    <div class="graph-body">
      {#each $commitGraph.commits as commit, i}
        {@const branches = getBranchesForCommit(commit.oid)}
        {@const isHead = commit.oid === $commitGraph?.head_oid}
        <button
          class="commit-row"
          class:selected={isSelected(commit)}
          class:is-head={isHead}
          onclick={() => selectCommit(commit)}
        >
          <span class="col col-graph">
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

          <span class="col col-message">
            {#each branches as branch}
              <span
                class="branch-label"
                class:head={branch.is_head}
                class:remote={branch.is_remote}
              >
                {branch.name}
              </span>
            {/each}
            <span class="commit-summary">{commit.summary}</span>
          </span>

          <span class="col col-author">{commit.author.name}</span>
          <span class="col col-date">{formatDate(commit.timestamp)}</span>
          <span class="col col-sha">{commit.short_oid}</span>
        </button>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <p>No commits to display</p>
    </div>
  {/if}
</div>

<style>
  .commit-graph {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    font-size: 13px;
  }

  .graph-header {
    display: flex;
    align-items: center;
    padding: 6px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .graph-body {
    flex: 1;
    overflow-y: auto;
  }

  .commit-row {
    display: flex;
    align-items: center;
    padding: 4px 8px;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    background: none;
    width: 100%;
    text-align: left;
    cursor: pointer;
    transition: background-color 0.1s;
    color: var(--text-primary);
    font-family: inherit;
    font-size: inherit;
  }

  .commit-row:hover {
    background: var(--bg-hover);
  }

  .commit-row.selected {
    background: var(--bg-selected);
  }

  .commit-row.is-head {
    font-weight: 500;
  }

  .col {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .col-graph {
    width: 32px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
  }

  .col-message {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .col-author {
    width: 140px;
    flex-shrink: 0;
    color: var(--text-secondary);
    padding: 0 8px;
  }

  .col-date {
    width: 80px;
    flex-shrink: 0;
    color: var(--text-muted);
    font-size: 12px;
    padding: 0 8px;
  }

  .col-sha {
    width: 70px;
    flex-shrink: 0;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 11px;
    color: var(--text-muted);
  }

  .commit-summary {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .branch-label {
    display: inline-flex;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 11px;
    font-weight: 600;
    flex-shrink: 0;
    background: var(--bg-badge);
    color: var(--text-accent);
    border: 1px solid var(--accent-color);
  }

  .branch-label.head {
    background: var(--accent-color);
    color: white;
    border-color: var(--accent-color);
  }

  .branch-label.remote {
    opacity: 0.7;
    border-style: dashed;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-muted);
  }
</style>

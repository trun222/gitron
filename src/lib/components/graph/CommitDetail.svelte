<script lang="ts">
  import { selectedCommit } from '$lib/stores/repo';

  function formatFullDate(timestamp: string): string {
    return new Date(timestamp).toLocaleString();
  }
</script>

{#if $selectedCommit}
  <div class="commit-detail">
    <div class="detail-header">
      <h3 class="commit-summary">{$selectedCommit.summary}</h3>
      <span class="commit-sha">{$selectedCommit.oid}</span>
    </div>

    {#if $selectedCommit.message !== $selectedCommit.summary}
      <pre class="commit-body">{$selectedCommit.message}</pre>
    {/if}

    <div class="detail-meta">
      <div class="meta-row">
        <span class="meta-label">Author</span>
        <span class="meta-value">
          {$selectedCommit.author.name}
          &lt;{$selectedCommit.author.email}&gt;
        </span>
      </div>
      <div class="meta-row">
        <span class="meta-label">Date</span>
        <span class="meta-value">{formatFullDate($selectedCommit.timestamp)}</span>
      </div>
      {#if $selectedCommit.parents.length > 0}
        <div class="meta-row">
          <span class="meta-label">Parents</span>
          <span class="meta-value">
            {#each $selectedCommit.parents as parent, i}
              <span class="parent-sha">{parent.substring(0, 7)}</span>
              {#if i < $selectedCommit.parents.length - 1},&nbsp;{/if}
            {/each}
          </span>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .commit-detail {
    padding: 12px 16px;
    border-top: 1px solid var(--border-color);
    background: var(--bg-secondary);
    max-height: 200px;
    overflow-y: auto;
  }

  .detail-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 8px;
  }

  .commit-summary {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .commit-sha {
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 11px;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .commit-body {
    margin: 0 0 8px;
    padding: 8px;
    background: var(--bg-primary);
    border-radius: 4px;
    font-size: 12px;
    color: var(--text-secondary);
    white-space: pre-wrap;
    font-family: 'SF Mono', 'Fira Code', monospace;
  }

  .detail-meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .meta-row {
    display: flex;
    gap: 8px;
    font-size: 12px;
  }

  .meta-label {
    color: var(--text-muted);
    min-width: 60px;
    font-weight: 500;
  }

  .meta-value {
    color: var(--text-secondary);
  }

  .parent-sha {
    font-family: 'SF Mono', 'Fira Code', monospace;
    color: var(--accent-color);
  }
</style>
